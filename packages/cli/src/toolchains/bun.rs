use crate::systems::model::{DependencyRow, ToolchainKind};
use crate::systems::runner;
use crate::toolchains::{run_blocking, ProbeOutcome, ToolchainAdapter, UpdateOpts, UpdateOutcome};
use async_trait::async_trait;
use serde_json::Value;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub struct BunAdapter;

#[async_trait]
impl ToolchainAdapter for BunAdapter {
    fn kind(&self) -> ToolchainKind {
        ToolchainKind::Bun
    }

    async fn probe(&self, root: &Path) -> ProbeOutcome {
        let root = root.to_path_buf();
        run_blocking(move || probe(&root)).await
    }

    async fn update(&self, root: &Path, opts: UpdateOpts) -> UpdateOutcome {
        let root = root.to_path_buf();
        run_blocking(move || update(&root, opts.major)).await
    }
}

/// Probe Bun workspace dependencies via `bun outdated --recursive`.
fn probe(root: &Path) -> ProbeOutcome {
    if runner::ensure_installed("bun", root).is_err() {
        return ProbeOutcome::failed("bun is not installed");
    }
    let out = match runner::capture(
        "bun",
        &[
            "outdated".to_string(),
            "--recursive".to_string(),
            "--no-cache".to_string(),
        ],
        root,
    ) {
        Ok(o) => o,
        Err(err) => return ProbeOutcome::failed(format!("bun outdated failed: {err}")),
    };
    let combined = format!("{}{}", out.stdout, out.stderr);
    let mut table = parse_bun_outdated_table(&combined);
    let mut rows: Vec<DependencyRow> = table.drain(..).map(bun_row_to_dependency).collect();
    merge_bun_blocked_rows(root, &combined, &mut rows);
    merge_bun_missing_manifest_rows(root, &mut rows);

    if rows.is_empty() {
        return ProbeOutcome::up_to_date();
    }
    ProbeOutcome::outdated(rows)
}

fn merge_bun_blocked_rows(root: &Path, combined: &str, rows: &mut Vec<DependencyRow>) {
    let existing: BTreeSet<String> = rows
        .iter()
        .map(|r| package_base_name(&r.dependency))
        .collect();
    for (pkg, specifier) in parse_bun_blocked_errors(combined) {
        if existing.contains(&pkg) {
            continue;
        }
        let current = find_bun_package_version(root, &pkg).unwrap_or_else(|| specifier.clone());
        let mut dep = DependencyRow::outdated(ToolchainKind::Bun, pkg.clone(), current, None, None);
        dep.blocked_reason = Some("minimum-release-age".to_string());
        dep.result = Some(format!("blocked at {specifier}"));
        rows.push(dep);
    }
}

fn merge_bun_missing_manifest_rows(root: &Path, rows: &mut Vec<DependencyRow>) {
    let existing: BTreeSet<String> = rows
        .iter()
        .map(|r| package_base_name(&r.dependency))
        .collect();
    let missing: Vec<String> = all_manifest_package_names(root)
        .into_iter()
        .filter(|name| !existing.contains(name))
        .collect();
    if missing.is_empty() {
        return;
    }

    let mut args = vec![
        "outdated".to_string(),
        "--recursive".to_string(),
        "--no-cache".to_string(),
    ];
    args.extend(missing);

    let Ok(out) = runner::capture("bun", &args, root) else {
        return;
    };
    let combined = format!("{}{}", out.stdout, out.stderr);
    for row in parse_bun_outdated_table(&combined) {
        if existing.contains(&row.package) {
            continue;
        }
        rows.push(bun_row_to_dependency(row));
    }
    merge_bun_blocked_rows(root, &combined, rows);
}

fn package_base_name(dependency: &str) -> String {
    dependency
        .split_whitespace()
        .next()
        .unwrap_or(dependency)
        .to_string()
}

fn all_manifest_package_names(root: &Path) -> Vec<String> {
    let mut names = BTreeSet::new();
    for path in bun_package_json_paths(root) {
        if let Ok(text) = std::fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                collect_manifest_deps(&json, &mut names);
            }
        }
    }
    names.into_iter().collect()
}

fn collect_manifest_deps(json: &Value, out: &mut BTreeSet<String>) {
    for section in [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ] {
        if let Some(obj) = json.get(section).and_then(Value::as_object) {
            for key in obj.keys() {
                out.insert(key.clone());
            }
        }
    }
}

fn bun_package_json_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths = vec![root.join("package.json")];
    for sub in ["apps", "packages"] {
        let dir = root.join(sub);
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let pkg = entry.path().join("package.json");
                if pkg.is_file() {
                    paths.push(pkg);
                }
            }
        }
    }
    paths
}

fn find_bun_package_version(root: &Path, name: &str) -> Option<String> {
    for path in bun_package_json_paths(root) {
        if let Some(version) = read_package_version(&path, name) {
            return Some(version);
        }
    }
    None
}

fn read_package_version(path: &Path, name: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    let json: Value = serde_json::from_str(&text).ok()?;
    for section in [
        "dependencies",
        "devDependencies",
        "optionalDependencies",
        "peerDependencies",
    ] {
        if let Some(v) = json
            .get(section)
            .and_then(|d| d.get(name))
            .and_then(Value::as_str)
        {
            return Some(v.to_string());
        }
    }
    json.get("overrides")
        .and_then(|o| o.get(name))
        .and_then(Value::as_str)
        .map(str::to_string)
}

/// Update Bun workspaces via `bun update --recursive` (release-age aware).
fn update(root: &Path, major: bool) -> UpdateOutcome {
    let mut args = vec!["update".to_string(), "--recursive".to_string()];
    if major {
        args.push("--latest".to_string());
    }
    match runner::capture("bun", &args, root) {
        Ok(out) if out.code == 0 => UpdateOutcome::Done,
        Ok(out) => {
            if bun_failure_is_age_only(&out.stdout, &out.stderr) {
                UpdateOutcome::Blocked
            } else {
                UpdateOutcome::Failed(format!("{}{}", out.stdout, out.stderr))
            }
        }
        Err(err) => UpdateOutcome::Failed(err.to_string()),
    }
}

fn bun_row_to_dependency(row: BunOutdatedRow) -> DependencyRow {
    let newest = (row.update != row.current).then(|| row.update.clone());
    let latest = (!row.latest.is_empty()).then(|| row.latest.clone());
    let mut dep =
        DependencyRow::outdated(ToolchainKind::Bun, row.package, row.current, newest, latest);
    if let Some(ws) = row.workspace {
        if !ws.is_empty() {
            dep.workspaces = vec![ws];
        }
    }
    if row.latest_blocked_by_age {
        dep.blocked_reason = Some("minimum-release-age".to_string());
    }
    dep
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BunOutdatedRow {
    pub package: String,
    pub current: String,
    pub update: String,
    pub latest: String,
    pub latest_blocked_by_age: bool,
    pub workspace: Option<String>,
}

/// True when Bun failed only because of `minimum-release-age` blocks (no other errors).
pub fn bun_failure_is_age_only(stdout: &str, stderr: &str) -> bool {
    let combined = format!("{stdout}{stderr}");
    let blocked = parse_bun_blocked_errors(&combined);
    if blocked.is_empty() {
        return false;
    }
    let blocked_names: BTreeSet<String> = blocked.into_iter().map(|(pkg, _)| pkg).collect();
    !combined.lines().any(|line| {
        let lower = line.to_lowercase();
        if !lower.contains("error:") {
            return false;
        }
        if lower.contains("blocked by minimum-release-age") {
            return false;
        }
        if lower.contains("no version matching") {
            return false;
        }
        if lower.contains("failed to resolve") {
            return !blocked_names.iter().any(|pkg| line.contains(pkg.as_str()));
        }
        true
    })
}

/// Packages Bun could not resolve because of `minimum-release-age` (from stderr).
pub fn parse_bun_blocked_errors(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        if !line.contains("No version matching") || !line.contains("blocked by minimum-release-age")
        {
            continue;
        }
        let Some(pkg) = extract_quoted_field(line, "No version matching \"") else {
            continue;
        };
        let Some(spec) = extract_quoted_field(line, "found for specifier \"") else {
            continue;
        };
        if !out.iter().any(|(p, _)| p == &pkg) {
            out.push((pkg, spec));
        }
    }
    out
}

fn extract_quoted_field(line: &str, prefix: &str) -> Option<String> {
    let rest = line.split_once(prefix)?.1;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub fn parse_bun_outdated_table(stdout: &str) -> Vec<BunOutdatedRow> {
    let mut rows = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        if trimmed.contains("Package") || trimmed.contains("---") {
            continue;
        }
        let cells: Vec<&str> = trimmed
            .split('|')
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .collect();
        if cells.len() < 4 {
            continue;
        }
        let latest_raw = cells[3];
        let latest_blocked = latest_raw.contains('*');
        let latest = latest_raw
            .trim_end_matches(|c: char| c == '*' || c.is_whitespace())
            .to_string();
        rows.push(BunOutdatedRow {
            package: cells[0].to_string(),
            current: cells[1].to_string(),
            update: cells[2].to_string(),
            latest,
            latest_blocked_by_age: latest_blocked,
            workspace: cells.get(4).map(|s| s.to_string()),
        });
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_bun_outdated_table_rows() {
        let text = r#"
| Package    | Current | Update | Latest   | Workspace |
|------------|---------|--------|----------|-----------|
| vite       | 7.3.4   | 7.3.5  | 8.0.14 * | web       |
"#;
        let rows = parse_bun_outdated_table(text);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].latest, "8.0.14");
        assert!(rows[0].latest_blocked_by_age);
        assert_eq!(rows[0].workspace.as_deref(), Some("web"));
    }

    #[test]
    fn bun_row_maps_to_dependency() {
        let row = BunOutdatedRow {
            package: "vite".into(),
            current: "7.3.4".into(),
            update: "7.3.5".into(),
            latest: "8.0.14".into(),
            latest_blocked_by_age: true,
            workspace: Some("web".into()),
        };
        let dep = bun_row_to_dependency(row);
        assert_eq!(dep.newest.as_deref(), Some("7.3.5"));
        assert_eq!(dep.latest.as_deref(), Some("8.0.14"));
        assert_eq!(dep.workspaces, vec!["web".to_string()]);
        assert!(dep.latest_one_major_ahead);
        assert_eq!(dep.blocked_reason.as_deref(), Some("minimum-release-age"));
    }

    #[test]
    fn parse_bun_blocked_error_line() {
        let text = r#"error: No version matching "vite" found for specifier "7.3.5" (blocked by minimum-release-age: 1209600 seconds)"#;
        let blocked = parse_bun_blocked_errors(text);
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].0, "vite");
        assert_eq!(blocked[0].1, "7.3.5");
    }

    #[test]
    fn bun_failure_is_age_only_true_for_cooldown_errors() {
        let stderr = r#"error: No version matching "vite" found for specifier "7.3.5" (blocked by minimum-release-age: 1209600 seconds)"#;
        assert!(bun_failure_is_age_only("", stderr));
    }

    #[test]
    fn bun_failure_is_age_only_true_with_failed_to_resolve_companion() {
        let stderr = r#"error: No version matching "oxfmt" found for specifier "^0.53.0" (blocked by minimum-release-age: 1209600 seconds)
error: oxfmt@^0.53.0 failed to resolve"#;
        assert!(bun_failure_is_age_only("", stderr));
    }

    #[test]
    fn bun_failure_is_age_only_false_for_other_errors() {
        let stderr = r#"error: Connection refused while resolving vite"#;
        assert!(!bun_failure_is_age_only("", stderr));
    }

    #[test]
    fn bun_failure_is_age_only_false_when_mixed_errors() {
        let stderr = r#"error: No version matching "vite" found for specifier "7.3.5" (blocked by minimum-release-age: 1209600 seconds)
error: Connection refused while resolving react"#;
        assert!(!bun_failure_is_age_only("", stderr));
    }

    #[test]
    fn package_base_name_strips_dev_suffix() {
        assert_eq!(package_base_name("vite (dev)"), "vite");
    }
}
