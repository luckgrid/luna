use crate::systems::model::{DependencyRow, ToolchainKind};
use crate::systems::{runner, workspace};
use crate::toolchains::{run_blocking, ProbeOutcome, ToolchainAdapter, UpdateOpts, UpdateOutcome};
use async_trait::async_trait;
use std::collections::BTreeSet;
use std::path::Path;

pub struct GoAdapter;

#[async_trait]
impl ToolchainAdapter for GoAdapter {
    fn kind(&self) -> ToolchainKind {
        ToolchainKind::Go
    }

    async fn probe(&self, root: &Path) -> ProbeOutcome {
        let root = root.to_path_buf();
        run_blocking(move || probe(&root)).await
    }

    async fn update(&self, root: &Path, _opts: UpdateOpts) -> UpdateOutcome {
        let root = root.to_path_buf();
        run_blocking(move || update(&root)).await
    }
}

/// Short display name for a Go module path (last two segments, or last one).
pub fn short_go_dep_name(module_path: &str) -> String {
    let segments: Vec<&str> = module_path.split('/').filter(|s| !s.is_empty()).collect();
    match segments.len() {
        0 => module_path.to_string(),
        1 => segments[0].to_string(),
        _ => format!(
            "{}/{}",
            segments[segments.len() - 2],
            segments[segments.len() - 1]
        ),
    }
}

fn go_row(
    module_path: &str,
    current: String,
    newest: Option<String>,
    module_root: &Path,
) -> DependencyRow {
    let display = short_go_dep_name(module_path);
    let mut dep = DependencyRow::outdated(ToolchainKind::Go, display, current, newest, None);
    dep.registry_name = Some(module_path.to_string());
    dep.source_path = Some(module_root.display().to_string());
    dep
}

/// Probe Go modules via `go list -m -u` on tool paths + direct requires only.
fn probe(root: &Path) -> ProbeOutcome {
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
        let targets = workspace::go_list_targets(module);
        if targets.is_empty() {
            continue;
        }
        let args = go_list_args(&targets);
        let out = match runner::capture("go", &args, module) {
            Ok(o) => o,
            Err(err) => {
                diagnostics.push(format!("{}: {err}", rel(root, module)));
                continue;
            }
        };
        for (path, current, newest) in parse_go_list_upgrades(&out.stdout) {
            rows.push(go_row(&path, current, Some(newest), module));
        }
    }

    let mut outcome = ProbeOutcome::outdated(rows);
    outcome.diagnostics.extend(diagnostics);
    outcome
}

/// Update Go modules: tools via `go get -tool …@latest`, direct deps via `go get -u`.
fn update(root: &Path) -> UpdateOutcome {
    let modules = workspace::project_roots(root, "go", "go.mod");
    for module in &modules {
        let tools: BTreeSet<String> = workspace::go_tool_paths(module).into_iter().collect();
        let directs: BTreeSet<String> = workspace::go_mod_direct_requires(module)
            .into_iter()
            .collect();

        for tool in &tools {
            let args = vec![
                "get".to_string(),
                "-tool".to_string(),
                format!("{tool}@latest"),
            ];
            if let Ok(out) = runner::capture("go", &args, module) {
                if out.code != 0 {
                    return UpdateOutcome::Failed(format!("{}{}", out.stdout, out.stderr));
                }
            }
        }

        let direct_only: Vec<String> = directs
            .into_iter()
            .filter(|path| !tools.contains(path))
            .collect();
        if !direct_only.is_empty() {
            let args = go_list_args(&direct_only);
            if let Ok(out) = runner::capture("go", &args, module) {
                for (path, _current, newest) in parse_go_list_upgrades(&out.stdout) {
                    let args = vec!["get".to_string(), "-u".to_string(), path];
                    if let Ok(get_out) = runner::capture("go", &args, module) {
                        if get_out.code != 0 {
                            return UpdateOutcome::Failed(format!(
                                "{}{}",
                                get_out.stdout, get_out.stderr
                            ));
                        }
                    }
                    let _ = newest;
                }
            }
        }

        match runner::capture("go", &["mod".to_string(), "tidy".to_string()], module) {
            Ok(out) if out.code == 0 => {}
            Ok(out) => return UpdateOutcome::Failed(format!("{}{}", out.stdout, out.stderr)),
            Err(err) => return UpdateOutcome::Failed(err.to_string()),
        }
    }
    UpdateOutcome::Done
}

fn go_list_args(targets: &[String]) -> Vec<String> {
    let mut args = vec!["list".to_string(), "-m".to_string(), "-u".to_string()];
    args.extend(targets.iter().cloned());
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

    #[test]
    fn short_go_dep_name_uses_last_two_segments() {
        assert_eq!(
            short_go_dep_name("github.com/gohugoio/hugo"),
            "gohugoio/hugo"
        );
        assert_eq!(
            short_go_dep_name("github.com/luckgrid/luna/packages/go-demo"),
            "packages/go-demo"
        );
        assert_eq!(short_go_dep_name("rsc.io/sampler"), "rsc.io/sampler");
        assert_eq!(short_go_dep_name("cel.dev/expr"), "cel.dev/expr");
    }

    #[test]
    fn go_list_args_includes_targets() {
        let targets = vec!["github.com/a/b".into(), "github.com/c/d".into()];
        let args = go_list_args(&targets);
        assert_eq!(args[0..3], ["list", "-m", "-u"]);
        assert_eq!(args[3..], ["github.com/a/b", "github.com/c/d"]);
    }
}
