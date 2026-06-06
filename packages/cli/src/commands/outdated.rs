use crate::cli::GlobalArgs;
use crate::deps::{self, snapshot, ui};
use crate::security;
use miette::Result;
use std::path::Path;

/// Report outdated dependencies across all toolchains in parallel, render one
/// grouped table, and persist a fresh snapshot. Always exits 0 (informational).
pub async fn run(root: &Path, global: &GlobalArgs, console: &ui::LunaConsole) -> Result<i32> {
    let firewall = security::resolve_firewall(root, global, global.quiet);
    let policy = deps::policy_from(false, firewall);

    let snapshots = deps::plan(
        root,
        &policy,
        global.quiet,
        "Checking outdated versions…",
        "Outdated check results",
        console,
    )
    .await?;

    // `luna outdated` always overwrites the snapshot (FR-16a).
    let snap = snapshot::OutdatedSnapshot::new(root, policy, snapshots.clone());
    if let Err(err) = snapshot::write(root, &snap) {
        let _ = ui::render_failure_notice(console, "Snapshot", &format!("could not write: {err}"));
    }

    if global.quiet {
        return Ok(0);
    }

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

    report_failures(console, &snapshots);

    ui::render_message(
        console,
        &format!("\nSnapshot saved to `{}`.", snapshot::SNAPSHOT_REL),
    )?;

    Ok(0)
}

fn report_failures(console: &ui::LunaConsole, snapshots: &[crate::deps::model::ToolchainSnapshot]) {
    use crate::deps::model::ToolchainState;
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
        let _ = ui::render_failure_notice(console, &tc.label, detail);
    }
}
