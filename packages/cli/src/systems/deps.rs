use crate::systems::model::{
    DependencyRow, PackageUpdateResult, PackageUpdateStatus, SnapshotPolicy, ToolchainKind,
    ToolchainSnapshot,
};
use crate::systems::snapshot::{self, OutdatedSnapshot};
use crate::systems::workspace::{self, Project};
use crate::systems::{registry, security};
use crate::toolchains::{self, adapter_for, UpdateOpts, UpdateOutcome};
use crate::ui::Emitter;
use miette::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tokio::task::JoinSet;

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
    if !toolchains::uv::uv_projects(root).is_empty() {
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

/// Toolchain kinds reported as outdated (used by `luna update` to select work).
pub fn outdated_kinds(snapshots: &[ToolchainSnapshot]) -> Vec<ToolchainKind> {
    snapshots
        .iter()
        .filter(|s| s.has_updates())
        .map(|s| s.kind)
        .collect()
}

/// Read and validate a cached snapshot for reuse (`luna update` snapshot-first path).
pub fn load_snapshot(
    root: &Path,
    policy: &SnapshotPolicy,
    ttl_secs: u64,
) -> Option<OutdatedSnapshot> {
    snapshot::read_valid(root, policy, ttl_secs).ok()
}

/// Probe all eligible toolchains concurrently behind a live panel; return per-toolchain snapshots.
///
/// Concurrency is driven by [`tokio::task::JoinSet`]; each adapter wraps its
/// blocking package-manager calls in `spawn_blocking`, so probes run in parallel
/// without manual thread or mutex management.
pub async fn plan(
    root: &Path,
    policy: &SnapshotPolicy,
    emitter: &Emitter,
    probe_title: &str,
    result_title: &str,
) -> Result<Vec<ToolchainSnapshot>> {
    // The policy is recorded in the snapshot by the caller; probing itself is
    // policy-agnostic (release-age cutoffs are applied inside each adapter).
    let _ = policy;

    let kinds = eligible_kinds(root);
    let projects = workspace::discover_projects(root);
    emitter.register_work(&kinds);

    let root = root.to_path_buf();
    let work = {
        let emitter = emitter.clone();
        let kinds = kinds.clone();
        let projects = projects.clone();
        let root = root.clone();
        async move {
            let mut set: JoinSet<ToolchainSnapshot> = JoinSet::new();
            for kind in kinds {
                let emitter = emitter.clone();
                let projects = projects.clone();
                let root = root.clone();
                set.spawn(async move { probe_one(kind, root, projects, emitter).await });
            }
            let mut snaps = Vec::new();
            while let Some(joined) = set.join_next().await {
                snaps.push(joined.map_err(|err| miette::miette!("probe task failed: {err}"))?);
            }
            Ok::<Vec<ToolchainSnapshot>, miette::Report>(snaps)
        }
    };

    let live = emitter.run_live(probe_title);
    let (live_res, work_res) = tokio::join!(live, work);
    live_res?;
    let mut snapshots = work_res?;

    snapshots.sort_by_key(|s| {
        ToolchainKind::ORDER
            .iter()
            .position(|k| *k == s.kind)
            .unwrap_or(usize::MAX)
    });

    emitter.freeze(result_title)?;
    Ok(snapshots)
}

async fn probe_one(
    kind: ToolchainKind,
    root: PathBuf,
    projects: Vec<Project>,
    emitter: Emitter,
) -> ToolchainSnapshot {
    emitter.probe_started(kind);
    let started = Instant::now();
    let outcome = adapter_for(kind).probe(&root).await;
    let state = outcome.state;
    let diagnostics = outcome.diagnostics;
    let rows = enrich_blocking(kind, outcome.rows, projects).await;
    let elapsed_ms = started.elapsed().as_millis() as u64;
    emitter.probe_finished(kind, state);

    ToolchainSnapshot {
        kind,
        label: kind.label().to_string(),
        state,
        elapsed_ms,
        started_at: None,
        finished_at: None,
        rows,
        diagnostics,
    }
}

/// Run row enrichment (registry lookups + project mapping) on the blocking pool.
async fn enrich_blocking(
    kind: ToolchainKind,
    mut rows: Vec<DependencyRow>,
    projects: Vec<Project>,
) -> Vec<DependencyRow> {
    if rows.is_empty() {
        return rows;
    }
    tokio::task::spawn_blocking(move || {
        enrich_rows(kind, &mut rows, &projects);
        rows
    })
    .await
    .unwrap_or_default()
}

/// Fill release-age (registry) and Workspace (Moon project) fields per row.
fn enrich_rows(kind: ToolchainKind, rows: &mut [DependencyRow], projects: &[Project]) {
    for row in rows.iter_mut() {
        let lookup = row.registry_name.as_deref().unwrap_or(&row.dependency);
        if let Some(newest) = row.newest.clone() {
            row.newest_release_age_days = registry::release_age_days(kind, lookup, &newest);
        }
        if let Some(latest) = row.latest.clone() {
            row.latest_release_age_days = registry::release_age_days(kind, lookup, &latest);
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

/// Summary of an update run, keyed by toolchain with per-package results.
pub struct UpdateReport {
    pub outcomes: HashMap<ToolchainKind, UpdateOutcome>,
    pub results: Vec<PackageUpdateResult>,
}

impl UpdateReport {
    pub fn outcome(&self, kind: ToolchainKind) -> UpdateOutcome {
        self.outcomes
            .get(&kind)
            .cloned()
            .unwrap_or(UpdateOutcome::Done)
    }

    pub fn updated(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == PackageUpdateStatus::Updated)
            .count()
    }

    pub fn blocked(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == PackageUpdateStatus::Blocked)
            .count()
    }

    pub fn failed(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == PackageUpdateStatus::Failed)
            .count()
    }

    pub fn unchanged(&self) -> usize {
        self.results
            .iter()
            .filter(|r| r.status == PackageUpdateStatus::Unchanged)
            .count()
    }

    pub fn had_failures(&self) -> bool {
        self.failed() > 0
            || self
                .outcomes
                .values()
                .any(|o| matches!(o, UpdateOutcome::Failed(_)))
    }
}

/// Update the selected toolchains (Proto first, the rest concurrently).
pub async fn update(
    root: &Path,
    snapshots: &[ToolchainSnapshot],
    selected: &[ToolchainKind],
    opts: UpdateOpts,
    emitter: &Emitter,
) -> UpdateReport {
    for tc in snapshots {
        emitter.register(tc.kind);
    }
    for tc in snapshots {
        if !selected.contains(&tc.kind) {
            emitter.skipped(tc.kind);
        }
    }
    emitter.set_work_total(selected.len());

    let root = root.to_path_buf();
    let selected_vec = selected.to_vec();
    let work = {
        let emitter = emitter.clone();
        let root = root.clone();
        async move {
            let mut map: HashMap<ToolchainKind, UpdateOutcome> = HashMap::new();

            // Proto runs first (it can change the runtimes used by the rest).
            if selected_vec.contains(&ToolchainKind::Proto) {
                emitter.update_started(ToolchainKind::Proto);
                let outcome = adapter_for(ToolchainKind::Proto).update(&root, opts).await;
                emitter.update_finished(ToolchainKind::Proto, outcome.state());
                map.insert(ToolchainKind::Proto, outcome);
            }

            let mut set: JoinSet<(ToolchainKind, UpdateOutcome)> = JoinSet::new();
            for kind in selected_vec
                .iter()
                .copied()
                .filter(|k| *k != ToolchainKind::Proto)
            {
                let emitter = emitter.clone();
                let root = root.clone();
                set.spawn(async move {
                    emitter.update_started(kind);
                    let outcome = adapter_for(kind).update(&root, opts).await;
                    emitter.update_finished(kind, outcome.state());
                    (kind, outcome)
                });
            }
            while let Some(joined) = set.join_next().await {
                if let Ok((kind, outcome)) = joined {
                    map.insert(kind, outcome);
                }
            }
            map
        }
    };

    let live = emitter.run_live("Updating…");
    let (live_res, outcomes) = tokio::join!(live, work);
    let _ = live_res;
    let _ = emitter.freeze("Update results");

    let projects = workspace::discover_projects(&root);
    let results = build_update_results(&root, snapshots, selected, &outcomes, projects).await;

    UpdateReport { outcomes, results }
}

fn row_match_key(row: &DependencyRow) -> String {
    row.registry_name.clone().unwrap_or_else(|| {
        row.dependency
            .split_whitespace()
            .next()
            .unwrap_or(&row.dependency)
            .to_string()
    })
}

fn find_post_row<'a>(
    pre: &DependencyRow,
    post_rows: &'a [DependencyRow],
) -> Option<&'a DependencyRow> {
    let key = row_match_key(pre);
    post_rows.iter().find(|r| row_match_key(r) == key)
}

fn package_status(
    pre: &DependencyRow,
    post: Option<&DependencyRow>,
    outcome: &UpdateOutcome,
) -> (PackageUpdateStatus, Option<String>) {
    match outcome {
        UpdateOutcome::Failed(_) => (PackageUpdateStatus::Failed, None),
        UpdateOutcome::Blocked => (PackageUpdateStatus::Blocked, None),
        UpdateOutcome::Done => {
            if let Some(post_row) = post {
                if post_row.current != pre.current {
                    return (PackageUpdateStatus::Updated, Some(post_row.current.clone()));
                }
                if pre.blocked_reason.is_some() || post_row.blocked_reason.is_some() {
                    return (PackageUpdateStatus::Blocked, None);
                }
            }
            if pre.blocked_reason.is_some() {
                (PackageUpdateStatus::Blocked, None)
            } else {
                (PackageUpdateStatus::Unchanged, None)
            }
        }
    }
}

async fn build_update_results(
    root: &Path,
    snapshots: &[ToolchainSnapshot],
    selected: &[ToolchainKind],
    outcomes: &HashMap<ToolchainKind, UpdateOutcome>,
    projects: Vec<Project>,
) -> Vec<PackageUpdateResult> {
    let mut results = Vec::new();

    for &kind in selected {
        let Some(snapshot) = snapshots.iter().find(|s| s.kind == kind) else {
            continue;
        };
        let outcome = outcomes.get(&kind).cloned().unwrap_or(UpdateOutcome::Done);
        let pre_rows: Vec<&DependencyRow> = snapshot
            .rows
            .iter()
            .filter(|r| r.newest.is_some() || r.blocked_reason.is_some())
            .collect();

        if pre_rows.is_empty() {
            continue;
        }

        let post_outcome = adapter_for(kind).probe(root).await;
        let mut post_rows = post_outcome.rows;
        if !post_rows.is_empty() {
            post_rows = enrich_blocking(kind, post_rows, projects.clone()).await;
        }

        for pre in pre_rows {
            let post = find_post_row(pre, &post_rows);
            let (status, new_version) = package_status(pre, post, &outcome);
            results.push(PackageUpdateResult {
                toolchain: kind,
                workspaces: pre.workspaces.clone(),
                dependency: pre.dependency.clone(),
                registry_name: pre.registry_name.clone(),
                previous: pre.current.clone(),
                new_version,
                status,
            });
        }
    }

    results.sort_by(|a, b| {
        ToolchainKind::ORDER
            .iter()
            .position(|k| *k == a.toolchain)
            .unwrap_or(usize::MAX)
            .cmp(
                &ToolchainKind::ORDER
                    .iter()
                    .position(|k| *k == b.toolchain)
                    .unwrap_or(usize::MAX),
            )
            .then_with(|| a.workspaces.join(",").cmp(&b.workspaces.join(",")))
            .then_with(|| a.dependency.cmp(&b.dependency))
    });

    results
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::model::ToolchainState;
    use std::collections::HashMap;

    #[test]
    fn policy_from_defaults() {
        std::env::remove_var("LUNA_MIN_RELEASE_AGE");
        let p = policy_from(false, false);
        assert!(!p.major);
        assert_eq!(p.min_release_age_days, 14);
        assert!(p.uv_exclude_newer.is_some());
        assert!(!p.firewall);
    }

    #[test]
    fn policy_from_major_flag() {
        let p = policy_from(true, true);
        assert!(p.major);
        assert!(p.firewall);
    }

    #[test]
    fn update_report_counting_from_results() {
        let mut outcomes = HashMap::new();
        outcomes.insert(ToolchainKind::Bun, UpdateOutcome::Blocked);

        let report = UpdateReport {
            outcomes,
            results: vec![
                PackageUpdateResult {
                    toolchain: ToolchainKind::Bun,
                    workspaces: vec!["app".into()],
                    dependency: "vite".into(),
                    registry_name: None,
                    previous: "7.3.4".into(),
                    new_version: None,
                    status: PackageUpdateStatus::Blocked,
                },
                PackageUpdateResult {
                    toolchain: ToolchainKind::Go,
                    workspaces: vec!["web".into()],
                    dependency: "gohugoio/hugo".into(),
                    registry_name: Some("github.com/gohugoio/hugo".into()),
                    previous: "v0.145.0".into(),
                    new_version: Some("v0.146.0".into()),
                    status: PackageUpdateStatus::Updated,
                },
            ],
        };
        assert_eq!(report.updated(), 1);
        assert_eq!(report.blocked(), 1);
        assert_eq!(report.failed(), 0);
    }

    #[test]
    fn update_report_counting() {
        let mut outcomes = HashMap::new();
        outcomes.insert(ToolchainKind::Proto, UpdateOutcome::Done);
        outcomes.insert(ToolchainKind::Rust, UpdateOutcome::Done);
        outcomes.insert(ToolchainKind::Bun, UpdateOutcome::Blocked);
        outcomes.insert(ToolchainKind::Uv, UpdateOutcome::Failed("err".into()));

        let report = UpdateReport {
            outcomes,
            results: Vec::new(),
        };
        assert_eq!(report.updated(), 0);
        assert_eq!(report.blocked(), 0);
        assert_eq!(report.failed(), 0);
        assert!(report.had_failures());
    }

    #[test]
    fn update_report_no_failures() {
        let mut outcomes = HashMap::new();
        outcomes.insert(ToolchainKind::Proto, UpdateOutcome::Done);
        let report = UpdateReport {
            outcomes,
            results: Vec::new(),
        };
        assert!(!report.had_failures());
        assert_eq!(report.updated(), 0);
        assert_eq!(report.blocked(), 0);
        assert_eq!(report.failed(), 0);
    }

    #[test]
    fn update_report_outcome_default_is_done() {
        let report = UpdateReport {
            outcomes: HashMap::new(),
            results: Vec::new(),
        };
        assert!(matches!(
            report.outcome(ToolchainKind::Go),
            UpdateOutcome::Done
        ));
    }

    #[test]
    fn outdated_kinds_filters() {
        let snapshots = vec![
            ToolchainSnapshot {
                kind: ToolchainKind::Proto,
                label: "proto".into(),
                state: ToolchainState::UpToDate,
                elapsed_ms: 0,
                started_at: None,
                finished_at: None,
                rows: Vec::new(),
                diagnostics: Vec::new(),
            },
            ToolchainSnapshot {
                kind: ToolchainKind::Bun,
                label: "bun".into(),
                state: ToolchainState::Outdated,
                elapsed_ms: 0,
                started_at: None,
                finished_at: None,
                rows: vec![DependencyRow::outdated(
                    ToolchainKind::Bun,
                    "vite",
                    "7.3.4",
                    Some("7.3.5".into()),
                    None,
                )],
                diagnostics: Vec::new(),
            },
            ToolchainSnapshot {
                kind: ToolchainKind::Go,
                label: "go".into(),
                state: ToolchainState::Outdated,
                elapsed_ms: 0,
                started_at: None,
                finished_at: None,
                rows: Vec::new(),
                diagnostics: Vec::new(),
            },
        ];

        let kinds = outdated_kinds(&snapshots);
        assert_eq!(kinds, vec![ToolchainKind::Bun]);
    }
}
