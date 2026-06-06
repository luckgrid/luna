use crate::deps::model::{DependencyRow, ToolchainKind};
use crate::deps::probes::ProbeOutcome;
use crate::runner;
use crate::security;
use crate::workspace;
use std::path::{Path, PathBuf};

/// Probe Python dependencies via `uv lock --upgrade --dry-run` (with cooldown).
pub fn probe(root: &Path) -> ProbeOutcome {
    let projects = uv_projects(root);
    if projects.is_empty() {
        return ProbeOutcome::up_to_date();
    }
    if runner::ensure_installed("uv", root).is_err() {
        return ProbeOutcome::failed("uv is not installed");
    }

    let mut rows = Vec::new();
    for project in &projects {
        let mut args = vec![
            "lock".to_string(),
            "--upgrade".to_string(),
            "--dry-run".to_string(),
        ];
        args.extend(security::uv_exclude_newer_args());
        let out = match runner::capture("uv", &args, project) {
            Ok(o) => o,
            Err(err) => return ProbeOutcome::failed(format!("uv lock failed: {err}")),
        };
        let combined = format!("{}{}", out.stdout, out.stderr);
        for (package, from_ver, to_ver) in parse_uv_dry_run_updates(&combined) {
            let mut dep = DependencyRow::outdated(
                ToolchainKind::Uv,
                package,
                from_ver,
                Some(to_ver.clone()),
                Some(to_ver),
            );
            dep.source_path = Some(project.display().to_string());
            rows.push(dep);
        }
    }
    ProbeOutcome::outdated(rows)
}

/// uv workspace root when configured, otherwise discovered per-project roots.
pub fn uv_projects(root: &Path) -> Vec<PathBuf> {
    if let Some(uv_root) = workspace::uv_workspace_root(root) {
        vec![uv_root]
    } else {
        workspace::project_roots(root, "python", "pyproject.toml")
    }
}

/// `uv lock --upgrade --dry-run` lines (`Update pkg vFROM -> vTO`).
pub fn parse_uv_dry_run_updates(text: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("Update ") {
            continue;
        }
        let rest = line.strip_prefix("Update ").unwrap_or(line);
        let Some((left, right)) = rest.split_once(" -> ") else {
            continue;
        };
        let from_ver = left
            .rsplit_once(' ')
            .map(|(_, v)| strip_v_prefix(v))
            .unwrap_or_else(|| strip_v_prefix(left));
        let to_ver = strip_v_prefix(right);
        let package = left
            .rsplit_once(' ')
            .map(|(p, _)| p.to_string())
            .unwrap_or_else(|| left.to_string());
        out.push((package, from_ver, to_ver));
    }
    out
}

fn strip_v_prefix(version: &str) -> String {
    version.trim().trim_start_matches('v').to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uv_dry_run_update_line() {
        let ups = parse_uv_dry_run_updates("Update fastapi v0.136.1 -> v0.136.3\n");
        assert_eq!(ups.len(), 1);
        assert_eq!(ups[0].0, "fastapi");
        assert_eq!(ups[0].1, "0.136.1");
        assert_eq!(ups[0].2, "0.136.3");
    }
}
