use crate::cli::{GlobalArgs, UpdateArgs};
use crate::commands::scripts;
use crate::deps::model::{DependencyRow, ToolchainKind, ToolchainSnapshot, ToolchainState};
use crate::deps::probes::{bun as bun_probe, uv as uv_probe};
use crate::deps::snapshot::{self, OutdatedSnapshot};
use crate::deps::{self, ui};
use crate::runner::{self, Output};
use crate::security;
use crate::workspace;
use miette::Result;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Per-toolchain update outcome.
#[derive(Debug, Clone)]
enum Outcome {
    Done,
    Blocked,
    Failed(String),
}

/// Snapshot-first update: reuse a fresh snapshot or preflight, then update
/// only outdated toolchains in parallel and re-run workspace bootstrap.
pub async fn run(
    root: &Path,
    args: &UpdateArgs,
    global: &GlobalArgs,
    console: &ui::LunaConsole,
) -> Result<i32> {
    let quiet = global.quiet;
    let firewall = security::resolve_firewall(root, global, quiet);
    let policy = deps::policy_from(args.major, firewall);

    let mut announced_targets = false;

    // Phase A/B — obtain a valid set of toolchain snapshots.
    let snapshots = match snapshot::read_valid(root, &policy, snapshot::DEFAULT_TTL_SECS) {
        Ok(snap) => {
            announce_reuse(console, &snap, quiet)?;
            announced_targets = true;
            snap.toolchains
        }
        Err(_) => {
            if !quiet {
                ui::render_message(
                    console,
                    "No recent snapshot — checking for outdated versions…",
                )?;
            }
            let snapshots = deps::plan(
                root,
                &policy,
                quiet,
                "Checking for outdated versions…",
                "Outdated check results",
                console,
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
            ui::render_message(
                console,
                "\n✓ All toolchains are up to date — nothing to update.",
            )?;
        }
        return Ok(0);
    }

    if !quiet && !announced_targets {
        let names: Vec<&str> = selected.iter().map(|k| k.label()).collect();
        ui::render_message(
            console,
            &format!("\nUpdating outdated toolchains: {}", names.join(", ")),
        )?;
    } else if !quiet {
        ui::render_message(console, "\n")?;
    }

    // Phase C — run updates for selected toolchains (proto first, rest parallel).
    let outcomes = run_updates(
        root, &snapshots, &selected, args.major, firewall, quiet, console,
    )
    .await;

    let had_failures = outcomes.values().any(|o| matches!(o, Outcome::Failed(_)));
    let blocked_count = outcomes
        .values()
        .filter(|o| matches!(o, Outcome::Blocked))
        .count();
    let failed_count = outcomes
        .values()
        .filter(|o| matches!(o, Outcome::Failed(_)))
        .count();
    let updated_count = outcomes
        .values()
        .filter(|o| matches!(o, Outcome::Done))
        .count();
    let skipped_count = snapshots.len().saturating_sub(selected.len());

    if !quiet {
        ui::render_section_title(console, "Re-syncing workspace (release-age enforced)")?;
    }
    let setup_code = scripts::sync_workspace_quiet(root, global, console)
        .await
        .unwrap_or(1);

    if !quiet {
        render_update_table(console, &snapshots, &selected, &outcomes)?;
    }

    if !quiet {
        ui::render_update_summary(
            console,
            &ui::UpdateSummary {
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

fn announce_reuse(console: &ui::LunaConsole, snap: &OutdatedSnapshot, quiet: bool) -> Result<()> {
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
    ui::render_message(
        console,
        &format!("Recent snapshot found ({mins}m ago). Updating outdated toolchains: {target}\n"),
    )
}

async fn run_updates(
    root: &Path,
    snapshots: &[ToolchainSnapshot],
    selected: &[ToolchainKind],
    major: bool,
    firewall: bool,
    quiet: bool,
    console: &ui::LunaConsole,
) -> HashMap<ToolchainKind, Outcome> {
    let panel = ui::StatusPanel::new(quiet);
    for tc in snapshots {
        panel.register(tc.kind.label());
    }
    for tc in snapshots {
        if !selected.contains(&tc.kind) {
            panel.finish(tc.kind.label(), ToolchainState::Skipped);
        }
    }

    let parallel: Vec<ToolchainKind> = selected
        .iter()
        .copied()
        .filter(|k| *k != ToolchainKind::Proto)
        .collect();

    panel.set_work_total(selected.len());

    let outcomes: Arc<Mutex<HashMap<ToolchainKind, Outcome>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let root = Arc::new(root.to_path_buf());

    let panel_live = panel.clone();
    let console_live = console.clone();
    let live = tokio::spawn(async move { panel_live.run_live(&console_live, "Updating…").await });

    if selected.contains(&ToolchainKind::Proto) {
        panel.start(ToolchainKind::Proto.label());
        let outcome = update_proto(root.as_path(), major, firewall);
        panel.finish(ToolchainKind::Proto.label(), state_of(&outcome));
        outcomes
            .lock()
            .unwrap()
            .insert(ToolchainKind::Proto, outcome);
        panel.signal_done();
    }

    let mut handles = Vec::new();
    for kind in parallel {
        let panel = panel.clone();
        let outcomes = Arc::clone(&outcomes);
        let root = Arc::clone(&root);
        handles.push(std::thread::spawn(move || {
            panel.start(kind.label());
            let outcome = match kind {
                ToolchainKind::Rust => update_cargo(root.as_path(), firewall),
                ToolchainKind::Bun => update_bun(root.as_path(), major),
                ToolchainKind::Uv => update_uv(root.as_path(), firewall),
                ToolchainKind::Go => update_go(root.as_path()),
                ToolchainKind::Proto => Outcome::Done,
            };
            panel.finish(kind.label(), state_of(&outcome));
            outcomes.lock().unwrap().insert(kind, outcome);
            panel.signal_done();
        }));
    }

    for handle in handles {
        let _ = handle.join();
    }

    let _ = live.await;
    let _ = panel.render_frozen(console, "Update results");

    let map = Arc::try_unwrap(outcomes)
        .ok()
        .and_then(|m| m.into_inner().ok())
        .unwrap_or_default();

    if !quiet {
        for (kind, outcome) in &map {
            if let Outcome::Failed(detail) = outcome {
                let _ = ui::render_failure_notice(console, kind.label(), detail);
            }
        }
    }

    map
}

fn state_of(outcome: &Outcome) -> ToolchainState {
    match outcome {
        Outcome::Done => ToolchainState::UpToDate,
        Outcome::Blocked => ToolchainState::Blocked,
        Outcome::Failed(_) => ToolchainState::Failed,
    }
}

// --- Per-toolchain updaters ------------------------------------------------

fn update_proto(root: &Path, major: bool, _firewall: bool) -> Outcome {
    if runner::ensure_installed("proto", root).is_err() {
        return Outcome::Failed("proto is not installed".into());
    }
    let mut args = vec!["outdated".to_string(), "--update".to_string()];
    if major {
        args.push("--latest".to_string());
    }
    args.push("-y".to_string());
    if let Ok(out) = runner::capture("proto", &args, root) {
        if out.code != 0 {
            return Outcome::Failed(format!("{}{}", out.stdout, out.stderr));
        }
    }
    match runner::capture("proto", &["install".to_string()], root) {
        Ok(out) if out.code == 0 => Outcome::Done,
        Ok(out) => Outcome::Failed(format!("{}{}", out.stdout, out.stderr)),
        Err(err) => Outcome::Failed(err.to_string()),
    }
}

fn update_cargo(root: &Path, firewall: bool) -> Outcome {
    match capture_pm("cargo", &["update".to_string()], root, firewall) {
        Ok(out) if out.code == 0 => Outcome::Done,
        Ok(out) => Outcome::Failed(format!("{}{}", out.stdout, out.stderr)),
        Err(err) => Outcome::Failed(err.to_string()),
    }
}

fn update_bun(root: &Path, major: bool) -> Outcome {
    let mut args = vec!["update".to_string(), "--recursive".to_string()];
    if major {
        args.push("--latest".to_string());
    }
    match runner::capture("bun", &args, root) {
        Ok(out) if out.code == 0 => Outcome::Done,
        Ok(out) => {
            if bun_probe::bun_failure_is_age_only(&out.stdout, &out.stderr) {
                Outcome::Blocked
            } else {
                Outcome::Failed(format!("{}{}", out.stdout, out.stderr))
            }
        }
        Err(err) => Outcome::Failed(err.to_string()),
    }
}

fn update_uv(root: &Path, firewall: bool) -> Outcome {
    let projects = uv_probe::uv_projects(root);
    for project in &projects {
        let mut lock_args = vec!["lock".to_string(), "--upgrade".to_string()];
        lock_args.extend(security::uv_exclude_newer_args());
        match capture_pm("uv", &lock_args, project, firewall) {
            Ok(out) if out.code == 0 => {}
            Ok(out) => return Outcome::Failed(format!("{}{}", out.stdout, out.stderr)),
            Err(err) => return Outcome::Failed(err.to_string()),
        }
        match capture_pm("uv", &["sync".to_string()], project, firewall) {
            Ok(out) if out.code == 0 => {}
            Ok(out) => return Outcome::Failed(format!("{}{}", out.stdout, out.stderr)),
            Err(err) => return Outcome::Failed(err.to_string()),
        }
    }
    Outcome::Done
}

fn update_go(root: &Path) -> Outcome {
    let modules = workspace::project_roots(root, "go", "go.mod");
    for module in &modules {
        if workspace::go_uses_tool_fast_path(module) {
            for tool in workspace::go_tool_paths(module) {
                let args = vec![
                    "get".to_string(),
                    "-tool".to_string(),
                    format!("{tool}@latest"),
                ];
                if let Ok(out) = runner::capture("go", &args, module) {
                    if out.code != 0 {
                        return Outcome::Failed(format!("{}{}", out.stdout, out.stderr));
                    }
                }
            }
        } else {
            let mut args = vec!["get".to_string(), "-u".to_string()];
            if workspace::full_graph_enabled() {
                args.push("all".to_string());
            }
            if let Ok(out) = runner::capture("go", &args, module) {
                if out.code != 0 {
                    return Outcome::Failed(format!("{}{}", out.stdout, out.stderr));
                }
            }
        }
        match runner::capture("go", &["mod".to_string(), "tidy".to_string()], module) {
            Ok(out) if out.code == 0 => {}
            Ok(out) => return Outcome::Failed(format!("{}{}", out.stdout, out.stderr)),
            Err(err) => return Outcome::Failed(err.to_string()),
        }
    }
    Outcome::Done
}

fn capture_pm(program: &str, args: &[String], cwd: &Path, firewall: bool) -> Result<Output> {
    let (program, args) = security::wrap(program, args, firewall);
    runner::capture(&program, &args, cwd)
}

// --- Results table ---------------------------------------------------------

fn render_update_table(
    console: &ui::LunaConsole,
    snapshots: &[ToolchainSnapshot],
    selected: &[ToolchainKind],
    outcomes: &HashMap<ToolchainKind, Outcome>,
) -> Result<()> {
    let mut groups: Vec<(ToolchainKind, Vec<DependencyRow>)> = Vec::new();
    for kind in ToolchainKind::ORDER {
        if !selected.contains(&kind) {
            continue;
        }
        let Some(tc) = snapshots.iter().find(|t| t.kind == kind) else {
            continue;
        };
        let outcome = outcomes.get(&kind).cloned().unwrap_or(Outcome::Done);
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

    ui::render_update_table(console, &groups)?;
    ui::render_release_age_section(console)
}

fn update_row_from(row: &DependencyRow, outcome: &Outcome) -> DependencyRow {
    let mut out = row.clone();
    out.previous = Some(row.current.clone());
    match outcome {
        Outcome::Done => {
            out.new_version = row.newest.clone();
        }
        Outcome::Blocked => {
            out.new_version = None;
            if out.blocked_reason.is_none() {
                out.blocked_reason = Some("minimum-release-age".to_string());
            }
        }
        Outcome::Failed(_) => {
            out.new_version = None;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let row = update_row_from(&sample_row(), &Outcome::Done);
        assert_eq!(row.previous.as_deref(), Some("7.3.4"));
        assert_eq!(row.new_version.as_deref(), Some("7.3.5"));
    }

    #[test]
    fn update_row_blocked_has_no_new_version() {
        let row = update_row_from(&sample_row(), &Outcome::Blocked);
        assert!(row.new_version.is_none());
        assert_eq!(row.blocked_reason.as_deref(), Some("minimum-release-age"));
    }

    #[test]
    fn state_of_maps_outcomes() {
        assert_eq!(state_of(&Outcome::Done), ToolchainState::UpToDate);
        assert_eq!(state_of(&Outcome::Blocked), ToolchainState::Blocked);
        assert_eq!(
            state_of(&Outcome::Failed("x".into())),
            ToolchainState::Failed
        );
    }
}
