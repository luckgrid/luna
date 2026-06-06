use crate::deps::model::{DependencyRow, ToolchainKind};
use crate::deps::probes::ProbeOutcome;
use crate::runner;
use std::path::Path;

/// Probe Bun workspace dependencies via `bun outdated --recursive`.
pub fn probe(root: &Path) -> ProbeOutcome {
    if runner::ensure_installed("bun", root).is_err() {
        return ProbeOutcome::failed("bun is not installed");
    }
    let out = match runner::capture(
        "bun",
        &["outdated".to_string(), "--recursive".to_string()],
        root,
    ) {
        Ok(o) => o,
        Err(err) => return ProbeOutcome::failed(format!("bun outdated failed: {err}")),
    };
    let table = parse_bun_outdated_table(&format!("{}{}", out.stdout, out.stderr));
    if table.is_empty() {
        return ProbeOutcome::up_to_date();
    }
    let rows = table.into_iter().map(bun_row_to_dependency).collect();
    ProbeOutcome::outdated(rows)
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
    !combined.lines().any(|line| {
        let lower = line.to_lowercase();
        lower.contains("error:")
            && !lower.contains("blocked by minimum-release-age")
            && !lower.contains("no version matching")
    })
}

/// Packages Bun could not resolve because of `minimum-release-age` (from update stderr).
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
}
