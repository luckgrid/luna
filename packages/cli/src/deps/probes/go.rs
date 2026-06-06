use crate::deps::model::{DependencyRow, ToolchainKind};
use crate::deps::probes::ProbeOutcome;
use crate::runner;
use crate::workspace;
use std::path::Path;

/// Probe Go modules via `go list -m -u` across discovered modules.
pub fn probe(root: &Path) -> ProbeOutcome {
    let modules = workspace::project_roots(root, "go", "go.mod");
    if modules.is_empty() {
        return ProbeOutcome::up_to_date();
    }
    if runner::ensure_installed("go", root).is_err() {
        return ProbeOutcome::failed("go is not installed");
    }

    let mut rows = Vec::new();
    let mut diagnostics = Vec::new();
    for module in &modules {
        let args = go_list_args(module);
        let out = match runner::capture("go", &args, module) {
            Ok(o) => o,
            Err(err) => {
                diagnostics.push(format!("{}: {err}", rel(root, module)));
                continue;
            }
        };
        for (path, current, newest) in parse_go_list_upgrades(&out.stdout) {
            let mut dep =
                DependencyRow::outdated(ToolchainKind::Go, path, current, Some(newest), None);
            dep.source_path = Some(module.display().to_string());
            rows.push(dep);
        }
    }

    let mut outcome = ProbeOutcome::outdated(rows);
    outcome.diagnostics.extend(diagnostics);
    outcome
}

fn go_list_args(module: &Path) -> Vec<String> {
    let mut args = vec!["list".to_string(), "-m".to_string(), "-u".to_string()];
    if workspace::go_uses_tool_fast_path(module) {
        args.extend(workspace::go_tool_paths(module));
    } else if workspace::full_graph_enabled() {
        args.push("all".to_string());
    }
    args
}

/// Parse `go list -m -u` lines `path current [newest]` into upgrade tuples.
pub fn parse_go_list_upgrades(stdout: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        let mut parts = line.split_whitespace();
        let (Some(path), Some(current), Some(third)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        if !third.starts_with('[') {
            continue;
        }
        let newest = third.trim_start_matches('[').trim_end_matches(']');
        out.push((path.to_string(), current.to_string(), newest.to_string()));
    }
    out
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_bracketed_upgrades_only() {
        let stdout = "github.com/a/b v1.0.0 [v1.2.0]\ngithub.com/c/d v2.0.0\n";
        let ups = parse_go_list_upgrades(stdout);
        assert_eq!(ups.len(), 1);
        assert_eq!(ups[0].0, "github.com/a/b");
        assert_eq!(ups[0].1, "v1.0.0");
        assert_eq!(ups[0].2, "v1.2.0");
    }
}
