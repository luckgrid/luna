use crate::cli::GlobalArgs;
use crate::output;
use crate::systems::deps;
use crate::systems::model::{ToolchainSnapshot, ToolchainState};
use crate::systems::security;
use crate::systems::snapshot::{self, OutdatedSnapshot};
use crate::ui::{self, Emitter};
use miette::Result;
use std::path::Path;

/// Report outdated dependencies across all toolchains in parallel, render one
/// grouped table, and persist a fresh snapshot. Always exits 0 (informational).
pub async fn run(root: &Path, global: &GlobalArgs, emitter: &Emitter) -> Result<i32> {
    let firewall = security::resolve_firewall(root, global, global.quiet);
    let policy = deps::policy_from(false, firewall);

    let snapshots = deps::plan(
        root,
        &policy,
        emitter,
        "Checking outdated versions…",
        "Outdated check results",
    )
    .await?;

    // `luna outdated` always overwrites the snapshot (FR-16a).
    let snap = OutdatedSnapshot::new(root, policy, snapshots.clone());
    if let Err(err) = snapshot::write(root, &snap) {
        let _ = emitter.failure_notice("Snapshot", &format!("could not write: {err}"));
    }

    if global.json {
        let report =
            output::OutdatedReport::from_snapshots(root, &snapshots, snapshot::SNAPSHOT_REL);
        output::emit(&report);
        return Ok(0);
    }

    if global.quiet {
        return Ok(0);
    }

    let console = emitter.console();
    let has_outdated = snapshots.iter().any(|s| s.has_updates());
    if !has_outdated {
        ui::render_message(
            console,
            "\n✓ All checks passed (nothing reported as outdated).",
        )?;
    } else {
        ui::render_outdated_table(console, &snapshots)?;
        ui::render_release_age_section(console)?;
    }

    report_failures(emitter, &snapshots);

    emitter.snapshot_written(snapshot::SNAPSHOT_REL)?;

    Ok(0)
}

fn report_failures(emitter: &Emitter, snapshots: &[ToolchainSnapshot]) {
    let failed: Vec<_> = snapshots
        .iter()
        .filter(|s| s.state == ToolchainState::Failed)
        .collect();
    if failed.is_empty() {
        return;
    }
    for tc in failed {
        let detail = tc
            .diagnostics
            .first()
            .map(|d| d.as_str())
            .unwrap_or("check failed");
        let _ = emitter.failure_notice(&tc.label, detail);
    }
}
