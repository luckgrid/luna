use crate::runner;
use miette::{miette, IntoDiagnostic, Result};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Walk up from the current directory until both `.prototools` and `package.json`
/// exist, marking the Luna monorepo root.
pub fn find_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().into_diagnostic()?;
    let mut dir = cwd.as_path();

    loop {
        if dir.join(".prototools").is_file() && dir.join("package.json").is_file() {
            return Ok(dir.to_path_buf());
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => {
                return Err(miette!(
                    "Could not find the Luna workspace root (no `.prototools` + `package.json` \
                     in any parent of {}). Run `luna` from inside the repository.",
                    cwd.display()
                ))
            }
        }
    }
}

/// Pinned version for a tool in `.prototools` (e.g. `go = "1.26.4"` → `1.26.4`).
pub fn prototools_pin(root: &Path, tool: &str) -> Option<String> {
    let path = root.join(".prototools");
    let text = std::fs::read_to_string(&path).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() == tool {
            let version = value.trim().trim_matches('"').trim_matches('\'');
            if !version.is_empty() {
                return Some(version.to_string());
            }
        }
    }
    None
}

/// `~/.proto/tools/<tool>/<version>/bin/<tool>` from `.prototools` pins.
pub fn proto_tool_binary(root: &Path, tool: &str) -> Option<PathBuf> {
    let version = prototools_pin(root, tool)?;
    let bin = home::home_dir()?
        .join(".proto")
        .join("tools")
        .join(tool)
        .join(&version)
        .join("bin")
        .join(tool);
    if bin.is_file() {
        Some(bin)
    } else {
        None
    }
}

/// Module paths listed in `go.work` (`use (` … `)`).
pub fn go_work_use_paths(root: &Path) -> Vec<PathBuf> {
    let path = root.join("go.work");
    let Ok(text) = std::fs::read_to_string(&path) else {
        return Vec::new();
    };
    let mut in_use = false;
    let mut modules = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("use (") || trimmed == "use (" {
            in_use = true;
            continue;
        }
        if in_use {
            if trimmed == ")" {
                break;
            }
            if let Some(rest) = trimmed.strip_prefix("./") {
                let rel = rest.trim_end_matches(',');
                modules.push(root.join(rel));
            }
        }
    }
    modules
}

/// Align `go.work` and workspace `go.mod` `go` directives with `.prototools`.
pub fn sync_go_toolchain(root: &Path, quiet: bool) -> Result<()> {
    let Some(version) = prototools_pin(root, "go") else {
        return Ok(());
    };
    if !root.join("go.work").is_file() {
        return Ok(());
    }

    runner::ensure_installed("proto", root)?;

    let edit_go = format!("-go={version}");
    let code = runner::run_proto(
        "go",
        &["work".to_string(), "edit".to_string(), edit_go.clone()],
        root,
        quiet,
    )?;
    if code != 0 {
        return Err(miette!(
            "`proto run go -- work edit -go={version}` failed with exit code {code}"
        ));
    }

    for module in go_work_use_paths(root) {
        if !module.join("go.mod").is_file() {
            continue;
        }
        let code = runner::run_proto(
            "go",
            &["mod".to_string(), "edit".to_string(), edit_go.clone()],
            &module,
            quiet,
        )?;
        if code != 0 {
            return Err(miette!(
                "`proto run go -- mod edit -go={version}` failed in {} (exit {code})",
                module.display()
            ));
        }
    }
    Ok(())
}

/// Root of the uv workspace when `pyproject.toml` defines `[tool.uv.workspace]`.
pub fn uv_workspace_root(root: &Path) -> Option<PathBuf> {
    let pyproject = root.join("pyproject.toml");
    if !pyproject.is_file() {
        return None;
    }
    let Ok(text) = std::fs::read_to_string(&pyproject) else {
        return None;
    };
    if text.contains("[tool.uv.workspace]") {
        Some(root.to_path_buf())
    } else {
        None
    }
}

/// Discover project roots for a language via `moon query projects --json`,
/// falling back to scanning `apps/*` and `packages/*` for `moon.yml`.
pub fn project_roots(root: &Path, language: &str, manifest: &str) -> Vec<PathBuf> {
    let from_moon = moon_query_roots(root, language, manifest);
    if !from_moon.is_empty() {
        return from_moon;
    }
    scan_roots(root, language, manifest)
}

fn moon_query_roots(root: &Path, language: &str, manifest: &str) -> Vec<PathBuf> {
    let Ok(out) = runner::capture("moon", &["query".to_string(), "projects".to_string()], root)
    else {
        return Vec::new();
    };
    if out.code != 0 {
        return Vec::new();
    }

    let Ok(value) = serde_json::from_str::<serde_json::Value>(out.stdout.trim()) else {
        return Vec::new();
    };
    let Some(projects) = value.get("projects").and_then(|p| p.as_array()) else {
        return Vec::new();
    };

    let mut roots = BTreeSet::new();
    for project in projects {
        let lang = project
            .get("language")
            .or_else(|| project.get("config").and_then(|c| c.get("language")))
            .and_then(|l| l.as_str())
            .unwrap_or("")
            .to_lowercase();
        if lang != language {
            continue;
        }
        let Some(rel) = project.get("root").and_then(|r| r.as_str()) else {
            continue;
        };
        let abs = absolutize(root, rel);
        if abs.join(manifest).is_file() {
            roots.insert(abs);
        }
    }
    roots.into_iter().collect()
}

fn scan_roots(root: &Path, language: &str, manifest: &str) -> Vec<PathBuf> {
    let needle = format!("language: {language}");
    let mut roots = BTreeSet::new();

    for top in ["apps", "packages"] {
        let base = root.join(top);
        let Ok(entries) = std::fs::read_dir(&base) else {
            continue;
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() {
                continue;
            }
            let moon_yml = dir.join("moon.yml");
            if !moon_yml.is_file() || !dir.join(manifest).is_file() {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(&moon_yml) {
                if text.lines().any(|l| l.trim() == needle) {
                    roots.insert(dir);
                }
            }
        }
    }
    roots.into_iter().collect()
}

fn absolutize(root: &Path, rel: &str) -> PathBuf {
    let path = Path::new(rel);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

/// `tool <path>` directives declared in a module's `go.mod`.
pub fn go_tool_paths(module_root: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(module_root.join("go.mod")) else {
        return Vec::new();
    };
    let mut seen = BTreeSet::new();
    let mut tools = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("tool ") {
            let path = rest.trim();
            if !path.is_empty() && seen.insert(path.to_string()) {
                tools.push(path.to_string());
            }
        }
    }
    tools
}

/// Fast outdated/update path: `tool` modules without real local packages (Hugo + workspace glue).
pub fn go_uses_tool_fast_path(module_root: &Path) -> bool {
    if go_tool_paths(module_root).is_empty() || full_graph_enabled() {
        return false;
    }
    !go_has_non_workspace_local_packages(module_root)
}

/// When set, Go outdated/update scans the full module graph (`all`), not just tools or direct deps.
pub fn full_graph_enabled() -> bool {
    match std::env::var_os("LUNA_FULL_GRAPH") {
        Some(v) => {
            let s = v.to_string_lossy();
            !s.is_empty() && s != "0" && s != "false"
        }
        None => false,
    }
}

/// Local packages outside `workspace/` (e.g. `packages/go-demo` at module root).
fn go_has_non_workspace_local_packages(module_root: &Path) -> bool {
    match runner::capture(
        "go",
        &[
            "list".to_string(),
            "-f".to_string(),
            "{{.ImportPath}}".to_string(),
            "./...".to_string(),
        ],
        module_root,
    ) {
        Ok(out) if out.code == 0 => out.stdout.lines().any(|line| {
            let import = line.trim();
            !import.is_empty() && !import.ends_with("/workspace")
        }),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prototools_pin_reads_repo_root() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let pin = prototools_pin(&root, "go");
        assert!(pin.is_some(), "expected go pin in repo .prototools");
        assert!(
            pin.as_deref() == Some("1.26.4"),
            "go pin should match .prototools: {:?}",
            pin
        );
    }

    #[test]
    fn go_work_use_paths_reads_repo_root() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let paths = go_work_use_paths(&root);
        assert!(paths.iter().any(|p| p.ends_with("apps/web")));
        assert!(paths.iter().any(|p| p.ends_with("packages/go-demo")));
    }
}
