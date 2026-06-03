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

/// True for Hugo-style modules: `tool` directives but no local Go packages.
pub fn is_go_tool_only(module_root: &Path) -> bool {
    if go_tool_paths(module_root).is_empty() {
        return false;
    }
    !go_has_local_packages(module_root)
}

fn go_has_local_packages(module_root: &Path) -> bool {
    match runner::capture(
        "go",
        &["list".to_string(), "./...".to_string()],
        module_root,
    ) {
        Ok(out) => out.code == 0 && out.stdout.split_whitespace().next().is_some(),
        Err(_) => false,
    }
}
