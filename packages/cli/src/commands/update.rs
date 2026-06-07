use crate::cli::{GlobalArgs, UpdateArgs};
use crate::output;
use crate::systems::deps::{self, UpdateReport};
use crate::systems::model::{DependencyRow, ToolchainKind, ToolchainSnapshot};
use crate::systems::snapshot::{self, OutdatedSnapshot};
use crate::systems::{security, tasks};
use crate::toolchains::{UpdateOpts, UpdateOutcome};
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
    let blocked_count = report.blocked();
    let failed_count = report.failed();
    let updated_count = report.updated();
    let skipped_count = snapshots.len().saturating_sub(selected.len());

    if !quiet {
        emitter.section_title("Re-syncing workspace (release-age enforced)")?;
    }
    let config = crate::config::load(root)?;
    let setup_code = tasks::sync_workspace_quiet(root, &config, global, emitter.console())
        .await
        .unwrap_or(1);

    if !quiet {
        render_update_table(emitter, &snapshots, &selected, &report)?;
    }

    if global.json {
        output::emit(&output::UpdateReportJson {
            schema_version: output::SCHEMA_VERSION.into(),
            workspace_root: root.display().to_string(),
            updated: updated_count,
            blocked: blocked_count,
            failed: failed_count,
            skipped: skipped_count,
            setup_ok: setup_code == 0,
        });
    }

    if !quiet {
        ui::render_update_summary(
            emitter.console(),
            &UpdateSummary {
                updated: updated_count,
                blocked: blocked_count,
                failed: failed_count,
                skipped: skipped_count,
                setup_ok: setup_code == 0,
                show_major_tip: !args.major,
            },
        )?;
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

fn render_update_table(
    emitter: &Emitter,
    snapshots: &[ToolchainSnapshot],
    selected: &[ToolchainKind],
    report: &UpdateReport,
) -> Result<()> {
    let mut groups: Vec<(ToolchainKind, Vec<DependencyRow>)> = Vec::new();
    for kind in ToolchainKind::ORDER {
        if !selected.contains(&kind) {
            continue;
        }
        let Some(tc) = snapshots.iter().find(|t| t.kind == kind) else {
            continue;
        };
        let outcome = report.outcome(kind);
        let rows: Vec<DependencyRow> = tc
            .rows
            .iter()
            .filter(|r| r.newest.is_some())
            .map(|r| update_row_from(r, &outcome))
            .collect();
        if !rows.is_empty() {
            groups.push((kind, rows));
        }
    }

    if groups.is_empty() {
        return Ok(());
    }

    let console = emitter.console();
    ui::render_update_table(console, &groups)?;
    ui::render_release_age_section(console)
}

fn update_row_from(row: &DependencyRow, outcome: &UpdateOutcome) -> DependencyRow {
    let mut out = row.clone();
    out.previous = Some(row.current.clone());
    match outcome {
        UpdateOutcome::Done => {
            out.new_version = row.newest.clone();
        }
        UpdateOutcome::Blocked => {
            out.new_version = None;
            if out.blocked_reason.is_none() {
                out.blocked_reason = Some("minimum-release-age".to_string());
            }
        }
        UpdateOutcome::Failed(_) => {
            out.new_version = None;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::model::ToolchainState;

    fn sample_row() -> DependencyRow {
        let mut row = DependencyRow::outdated(
            ToolchainKind::Bun,
            "vite",
            "7.3.4",
            Some("7.3.5".into()),
            Some("8.0.14".into()),
        );
        row.newest_release_age_days = Some(21);
        row.latest_release_age_days = Some(8);
        row
    }

    #[test]
    fn update_row_done_sets_new_version() {
        let row = update_row_from(&sample_row(), &UpdateOutcome::Done);
        assert_eq!(row.previous.as_deref(), Some("7.3.4"));
        assert_eq!(row.new_version.as_deref(), Some("7.3.5"));
    }

    #[test]
    fn update_row_blocked_has_no_new_version() {
        let row = update_row_from(&sample_row(), &UpdateOutcome::Blocked);
        assert!(row.new_version.is_none());
        assert_eq!(row.blocked_reason.as_deref(), Some("minimum-release-age"));
    }

    #[test]
    fn outcome_state_mapping() {
        assert_eq!(UpdateOutcome::Done.state(), ToolchainState::UpToDate);
        assert_eq!(UpdateOutcome::Blocked.state(), ToolchainState::Blocked);
        assert_eq!(
            UpdateOutcome::Failed("x".into()).state(),
            ToolchainState::Failed
        );
    }
}
