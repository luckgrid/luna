use crate::systems::model::{DependencyRow, SnapshotPolicy, ToolchainKind, ToolchainSnapshot};
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

/// Summary of an update run, keyed by toolchain.
pub struct UpdateReport {
    pub outcomes: HashMap<ToolchainKind, UpdateOutcome>,
}

impl UpdateReport {
    pub fn outcome(&self, kind: ToolchainKind) -> UpdateOutcome {
        self.outcomes
            .get(&kind)
            .cloned()
            .unwrap_or(UpdateOutcome::Done)
    }

    pub fn updated(&self) -> usize {
        self.count(|o| matches!(o, UpdateOutcome::Done))
    }

    pub fn blocked(&self) -> usize {
        self.count(|o| matches!(o, UpdateOutcome::Blocked))
    }

    pub fn failed(&self) -> usize {
        self.count(|o| matches!(o, UpdateOutcome::Failed(_)))
    }

    pub fn had_failures(&self) -> bool {
        self.failed() > 0
    }

    fn count(&self, pred: impl Fn(&UpdateOutcome) -> bool) -> usize {
        self.outcomes.values().filter(|o| pred(o)).count()
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

    for (kind, outcome) in &outcomes {
        if let UpdateOutcome::Failed(detail) = outcome {
            let _ = emitter.failure_notice(kind.label(), detail);
        }
    }

    UpdateReport { outcomes }
}
