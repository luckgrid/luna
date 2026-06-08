use crate::cli::{GlobalArgs, UpdateArgs};
use crate::output;
use crate::systems::deps;
use crate::systems::snapshot::{self, OutdatedSnapshot};
use crate::systems::{security, tasks};
use crate::toolchains::UpdateOpts;
use crate::ui::{self, Emitter, UpdateSummary};
use miette::Result;
use std::path::Path;

/// Snapshot-first update: reuse a fresh snapshot or preflight, then update
/// only outdated toolchains in parallel and re-run workspace bootstrap.
pub async fn run(
    root: &Path,
    args: &UpdateArgs,
    global: &GlobalArgs,
    emitter: &Emitter,
) -> Result<i32> {
    let quiet = global.quiet;
    let firewall = security::resolve_firewall(root, global, quiet);
    let policy = deps::policy_from(args.major, firewall);

    let mut announced_targets = false;

    // Phase A/B — obtain a valid set of toolchain snapshots.
    let snapshots = match deps::load_snapshot(root, &policy, snapshot::DEFAULT_TTL_SECS) {
        Some(snap) => {
            announce_reuse(emitter, &snap, quiet)?;
            announced_targets = true;
            snap.toolchains
        }
        None => {
            if !quiet {
                emitter.message("No recent snapshot — checking for outdated versions…")?;
            }
            let snapshots = deps::plan(
                root,
                &policy,
                emitter,
                "Checking for outdated versions…",
                "Outdated check results",
            )
            .await?;
            let snap = OutdatedSnapshot::new(root, policy.clone(), snapshots.clone());
            let _ = snapshot::write(root, &snap);
            snapshots
        }
    };

    let selected = deps::outdated_kinds(&snapshots);
    if selected.is_empty() {
        if !quiet {
            emitter.message("\n✓ All toolchains are up to date — nothing to update.")?;
        }
        return Ok(0);
    }

    if !quiet && !announced_targets {
        let names: Vec<&str> = selected.iter().map(|k| k.label()).collect();
        emitter.message(&format!(
            "\nUpdating outdated toolchains: {}",
            names.join(", ")
        ))?;
    } else if !quiet {
        emitter.message("\n")?;
    }

    // Phase C — run updates for selected toolchains (proto first, rest parallel).
    let opts = UpdateOpts {
        major: args.major,
        firewall,
    };
    let report = deps::update(root, &snapshots, &selected, opts, emitter).await;

    let had_failures = report.had_failures();
    let updated_count = report.updated();
    let blocked_count = report.blocked();
    let failed_count = report.failed();
    let unchanged_count = report.unchanged();
    let skipped_count = snapshots.len().saturating_sub(selected.len());

    if !quiet {
        ui::render_update_report(
            emitter.console(),
            &report.results,
            &UpdateSummary {
                updated: updated_count,
                blocked: blocked_count,
                failed: failed_count,
                unchanged: unchanged_count,
                skipped: skipped_count,
                setup_ok: true,
                show_major_tip: !args.major,
            },
        )?;
        emitter.section_title("Re-syncing workspace (release-age enforced)")?;
    }

    let config = crate::config::load(root)?;
    let setup_code = tasks::sync_workspace_quiet(root, &config, global, emitter.console())
        .await
        .unwrap_or(1);

    if global.json {
        output::emit(&output::UpdateReportJson {
            schema_version: output::SCHEMA_VERSION.into(),
            workspace_root: root.display().to_string(),
            updated: updated_count,
            blocked: blocked_count,
            failed: failed_count,
            unchanged: unchanged_count,
            skipped: skipped_count,
            setup_ok: setup_code == 0,
            packages: report.results.clone(),
        });
    }

    Ok(if had_failures || setup_code != 0 {
        1
    } else {
        0
    })
}

fn announce_reuse(emitter: &Emitter, snap: &OutdatedSnapshot, quiet: bool) -> Result<()> {
    if quiet {
        return Ok(());
    }
    let mins = snap.age_secs() / 60;
    let outdated: Vec<&str> = snap
        .toolchains
        .iter()
        .filter(|t| t.has_updates())
        .map(|t| t.kind.label())
        .collect();
    let target = if outdated.is_empty() {
        "none".to_string()
    } else {
        outdated.join(", ")
    };
    emitter.message(&format!(
        "Recent snapshot found ({mins}m ago). Updating outdated toolchains: {target}\n"
    ))
}

#[cfg(test)]
mod tests {
    use crate::systems::model::{
        PackageUpdateResult, PackageUpdateStatus, ToolchainKind, ToolchainState,
    };
    use crate::toolchains::UpdateOutcome;

    #[test]
    fn update_outcome_state_mapping() {
        assert_eq!(UpdateOutcome::Done.state(), ToolchainState::UpToDate);
        assert_eq!(UpdateOutcome::Blocked.state(), ToolchainState::Blocked);
        assert_eq!(
            UpdateOutcome::Failed("x".into()).state(),
            ToolchainState::Failed
        );
    }

    #[test]
    fn package_update_result_serializes_status() {
        let result = PackageUpdateResult {
            toolchain: ToolchainKind::Bun,
            workspaces: vec!["app".into()],
            dependency: "vite".into(),
            registry_name: None,
            previous: "7.3.4".into(),
            new_version: Some("7.3.5".into()),
            status: PackageUpdateStatus::Updated,
        };
        assert_eq!(result.status, PackageUpdateStatus::Updated);
    }
}
