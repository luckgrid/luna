pub mod model;
pub mod probes;
pub mod registry;
pub mod snapshot;
pub mod ui;

use crate::security;
use crate::workspace::{self, Project};
use model::{DependencyRow, SnapshotPolicy, ToolchainKind, ToolchainSnapshot};
use probes::ProbeOutcome;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// Build the policy fingerprint for the current invocation.
pub fn policy_from(major: bool, firewall: bool) -> SnapshotPolicy {
    SnapshotPolicy {
        major,
        min_release_age_days: security::min_release_age_days(),
        uv_exclude_newer: Some(security::exclude_newer_date()),
        firewall,
    }
}

/// Toolchain groups eligible for probing in this repo, in fixed report order.
pub fn eligible_kinds(root: &Path) -> Vec<ToolchainKind> {
    let mut kinds = vec![ToolchainKind::Proto];
    if root.join("Cargo.toml").is_file() && root.join("Cargo.lock").is_file() {
        kinds.push(ToolchainKind::Rust);
    }
    kinds.push(ToolchainKind::Bun);
    if !probes::uv::uv_projects(root).is_empty() {
        kinds.push(ToolchainKind::Uv);
    }
    if !workspace::project_roots(root, "go", "go.mod").is_empty() {
        kinds.push(ToolchainKind::Go);
    }
    // Preserve canonical order.
    ToolchainKind::ORDER
        .into_iter()
        .filter(|k| kinds.contains(k))
        .collect()
}

/// Run all eligible probes in parallel behind a live panel; return per-toolchain snapshots.
pub async fn plan(
    root: &Path,
    _policy: &SnapshotPolicy,
    quiet: bool,
    probe_title: &str,
    result_title: &str,
    console: &ui::LunaConsole,
) -> miette::Result<Vec<ToolchainSnapshot>> {
    let kinds = eligible_kinds(root);
    let projects = workspace::discover_projects(root);
    let root = Arc::new(root.to_path_buf());

    let panel = ui::StatusPanel::new(quiet);
    for kind in &kinds {
        panel.register(kind.label());
    }
    panel.set_work_total(kinds.len());

    let results: Arc<Mutex<Vec<ToolchainSnapshot>>> = Arc::new(Mutex::new(Vec::new()));

    let panel_live = panel.clone();
    let console_live = console.clone();
    let probe_title = probe_title.to_string();
    let root_workers = Arc::clone(&root);
    let projects_workers = projects.clone();
    let panel_workers = panel.clone();
    let results_workers = Arc::clone(&results);

    let live_future = panel_live.run_live(&console_live, &probe_title);
    let work_future = tokio::task::spawn_blocking(move || {
        let mut handles = Vec::new();
        for kind in kinds {
            let panel = panel_workers.clone();
            let results = Arc::clone(&results_workers);
            let root = Arc::clone(&root_workers);
            let projects = projects_workers.clone();

            handles.push(std::thread::spawn(move || {
                panel.start(kind.label());
                let started = Instant::now();
                let mut outcome = run_probe(kind, root.as_path());
                enrich_rows(kind, &mut outcome.rows, &projects);
                let elapsed_ms = started.elapsed().as_millis() as u64;
                panel.finish(kind.label(), outcome.state);
                let snapshot = ToolchainSnapshot {
                    kind,
                    label: kind.label().to_string(),
                    state: outcome.state,
                    elapsed_ms,
                    started_at: None,
                    finished_at: None,
                    rows: outcome.rows,
                    diagnostics: outcome.diagnostics,
                };
                if let Ok(mut r) = results.lock() {
                    r.push(snapshot);
                }
                panel.signal_done();
            }));
        }

        for handle in handles {
            let _ = handle.join();
        }
    });

    let (live_result, work_result) = tokio::join!(live_future, work_future);
    live_result?;
    work_result.map_err(|err| miette::miette!("outdated probe worker failed: {err}"))?;

    panel.render_frozen(console, result_title)?;

    let mut snapshots = Arc::try_unwrap(results)
        .ok()
        .and_then(|m| m.into_inner().ok())
        .unwrap_or_default();
    snapshots.sort_by_key(|s| {
        ToolchainKind::ORDER
            .iter()
            .position(|k| *k == s.kind)
            .unwrap_or(usize::MAX)
    });
    Ok(snapshots)
}

fn run_probe(kind: ToolchainKind, root: &Path) -> ProbeOutcome {
    match kind {
        ToolchainKind::Proto => probes::proto::probe(root),
        ToolchainKind::Rust => probes::cargo::probe(root),
        ToolchainKind::Bun => probes::bun::probe(root),
        ToolchainKind::Uv => probes::uv::probe(root),
        ToolchainKind::Go => probes::go::probe(root),
    }
}

/// Fill release-age (registry) and Workspace (Moon project) fields per row.
fn enrich_rows(kind: ToolchainKind, rows: &mut [DependencyRow], projects: &[Project]) {
    for row in rows.iter_mut() {
        if let Some(newest) = row.newest.clone() {
            row.newest_release_age_days =
                registry::release_age_days(kind, &row.dependency, &newest);
        }
        if let Some(latest) = row.latest.clone() {
            row.latest_release_age_days =
                registry::release_age_days(kind, &row.dependency, &latest);
        }
        if row.workspaces.is_empty() {
            if let Some(path) = row.source_path.as_ref() {
                let names = workspace::project_names_for_path(projects, Path::new(path));
                if !names.is_empty() {
                    row.workspaces = names;
                }
            }
        }
    }
}

/// Toolchain kinds reported as outdated (used by `luna update` to select work).
pub fn outdated_kinds(snapshots: &[ToolchainSnapshot]) -> Vec<ToolchainKind> {
    snapshots
        .iter()
        .filter(|s| s.has_updates())
        .map(|s| s.kind)
        .collect()
}
