use crate::systems::runner;
use miette::{miette, IntoDiagnostic, Result};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Walk up from the current directory until a recognized root marker is found.
///
/// Detection strategy (in priority order):
/// 1. `luna.toml` present — canonical Luna workspace root
/// 2. `.prototools` + `package.json` both present — legacy root detection
pub fn find_root() -> Result<PathBuf> {
    let cwd = std::env::current_dir().into_diagnostic()?;
    let mut dir = cwd.as_path();

    loop {
        if dir.join("luna.toml").is_file() {
            return Ok(dir.to_path_buf());
        }
        if dir.join(".prototools").is_file() && dir.join("package.json").is_file() {
            return Ok(dir.to_path_buf());
        }

        match dir.parent() {
            Some(parent) => dir = parent,
            None => {
                return Err(miette!(
                    "Could not find the Luna workspace root (no `luna.toml` or \
                     `.prototools` + `package.json` in any parent of {}). Run `luna` \
                     from inside the repository.",
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

/// A discovered Moon project: its name (Moon id) and absolute root path.
#[derive(Debug, Clone)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
}

/// Discover all Moon projects (name + path) via `moon query projects`,
/// falling back to scanning `apps/*` and `packages/*` for `moon.yml`.
pub fn discover_projects(root: &Path) -> Vec<Project> {
    let from_moon = moon_query_projects(root);
    if !from_moon.is_empty() {
        return from_moon;
    }
    scan_projects(root)
}

fn moon_query_projects(root: &Path) -> Vec<Project> {
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

    let mut out = Vec::new();
    for project in projects {
        let name = project
            .get("id")
            .or_else(|| project.get("name"))
            .and_then(|v| v.as_str());
        let rel = project.get("root").and_then(|r| r.as_str());
        if let (Some(name), Some(rel)) = (name, rel) {
            out.push(Project {
                name: name.to_string(),
                path: absolutize(root, rel),
            });
        }
    }
    out
}

fn scan_projects(root: &Path) -> Vec<Project> {
    let mut out = Vec::new();
    for top in ["apps", "packages"] {
        let base = root.join(top);
        let Ok(entries) = std::fs::read_dir(&base) else {
            continue;
        };
        for entry in entries.flatten() {
            let dir = entry.path();
            if !dir.is_dir() || !dir.join("moon.yml").is_file() {
                continue;
            }
            if let Some(name) = dir.file_name().and_then(|n| n.to_str()) {
                out.push(Project {
                    name: name.to_string(),
                    path: dir.clone(),
                });
            }
        }
    }
    out
}

/// Moon project name(s) owning a path (a manifest dir maps to its containing project).
pub fn project_names_for_path(projects: &[Project], path: &Path) -> Vec<String> {
    let mut names: Vec<String> = projects
        .iter()
        .filter(|p| path.starts_with(&p.path))
        .map(|p| p.name.clone())
        .collect();
    names.sort();
    names.dedup();
    names
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

/// Direct `require` entries from `go.mod` (excludes `// indirect` lines).
pub fn go_mod_direct_requires(module_root: &Path) -> Vec<String> {
    let Ok(text) = std::fs::read_to_string(module_root.join("go.mod")) else {
        return Vec::new();
    };
    let mut seen = BTreeSet::new();
    let mut requires = Vec::new();
    let mut in_require_block = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("require (") {
            in_require_block = true;
            continue;
        }
        if in_require_block && trimmed == ")" {
            in_require_block = false;
            continue;
        }

        if trimmed.starts_with("require ") && !trimmed.starts_with("require (") {
            let rest = trimmed.strip_prefix("require ").unwrap_or("");
            if let Some((path, _)) = parse_go_require_line(rest) {
                if !go_require_line_is_indirect(trimmed) && seen.insert(path.clone()) {
                    requires.push(path);
                }
            }
            continue;
        }

        if in_require_block && !go_require_line_is_indirect(trimmed) {
            if let Some((path, _)) = parse_go_require_line(trimmed) {
                if seen.insert(path.clone()) {
                    requires.push(path);
                }
            }
        }
    }
    requires
}

fn go_require_line_is_indirect(line: &str) -> bool {
    line.contains("// indirect")
}

fn parse_go_require_line(line: &str) -> Option<(String, String)> {
    let line = line.split("//").next()?.trim();
    if line.is_empty() {
        return None;
    }
    let mut parts = line.split_whitespace();
    let path = parts.next()?.to_string();
    let version = parts.next().unwrap_or("").to_string();
    Some((path, version))
}

/// Probe/update targets: deduped union of `tool` paths and direct `require` modules.
pub fn go_list_targets(module_root: &Path) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut targets = Vec::new();
    for path in go_tool_paths(module_root)
        .into_iter()
        .chain(go_mod_direct_requires(module_root))
    {
        if seen.insert(path.clone()) {
            targets.push(path);
        }
    }
    targets
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

    #[test]
    fn go_mod_direct_requires_skips_indirect_on_repo_web() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let web = root.join("apps/web");
        if !web.join("go.mod").is_file() {
            return;
        }
        let requires = go_mod_direct_requires(&web);
        assert!(
            requires.iter().any(|r| r.contains("go-demo")),
            "expected direct go-demo require: {requires:?}"
        );
        assert!(
            !requires.iter().any(|r| r.starts_with("cloud.google.com")),
            "indirect deps must not appear: {requires:?}"
        );
    }

    #[test]
    fn go_list_targets_unions_tools_and_directs_on_repo_web() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let web = root.join("apps/web");
        if !web.join("go.mod").is_file() {
            return;
        }
        let targets = go_list_targets(&web);
        assert!(targets.iter().any(|t| t.contains("hugo")));
        assert!(targets.iter().any(|t| t.contains("go-demo")));
        assert!(targets.len() <= 3, "expected few targets, got {targets:?}");
    }
}
