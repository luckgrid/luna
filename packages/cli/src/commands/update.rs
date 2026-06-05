use crate::cli::{GlobalArgs, UpdateArgs};
use crate::commands::outdated;
use crate::runner::{self, Output};
use crate::security;
use crate::workspace;
use miette::Result;
use starbase::style::{Style, Stylize};
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Update toolchains and dependencies across every ecosystem, then re-run install.
///
/// Supply-chain precautions: Bun cooldown via `bunfig.toml` (not the CLI flag on `update`,
/// which breaks re-resolving existing pins), uv `--exclude-newer`, optional Socket Firewall.
pub fn run(
    root: &Path,
    args: &UpdateArgs,
    global: &GlobalArgs,
    feedback: &UpdateFeedback,
) -> Result<i32> {
    let major = args.major;
    let quiet = global.quiet;
    let firewall = security::resolve_firewall(root, global, quiet);
    let mut had_failures = false;

    // proto pins (must run before parallel package-manager updates)
    runner::ensure_installed("proto", root)?;
    feedback.register_task("proto");
    section("proto — update .prototools pins");
    feedback.start_task("proto");
    let mut proto_args = vec!["outdated".to_string(), "--update".to_string()];
    if major {
        proto_args.push("--latest".to_string());
    }
    proto_args.push("-y".to_string());
    let mut proto_ok = true;
    if !run_step(
        feedback,
        "proto — update pins",
        "proto",
        &proto_args,
        root,
        quiet,
    )? {
        proto_ok = false;
        had_failures = true;
    }
    section("proto — install pinned tools");
    if !run_step(
        feedback,
        "proto — install pinned tools",
        "proto",
        &["install".to_string()],
        root,
        quiet,
    )? {
        proto_ok = false;
        had_failures = true;
    }
    feedback.finish_task(
        "proto",
        if proto_ok {
            TaskStatus::Done
        } else {
            TaskStatus::Failed
        },
    );

    let has_cargo = root.join("Cargo.toml").is_file() && root.join("Cargo.lock").is_file();
    if has_cargo {
        feedback.register_task("Cargo");
    }
    feedback.register_task("Bun");

    let uv_projects = uv_update_projects(root);
    for project in &uv_projects {
        feedback.register_task(format!("uv — {}", rel(root, project)));
    }

    let go_roots = workspace::project_roots(root, "go", "go.mod");
    for module in &go_roots {
        feedback.register_task(format!("Go — {}", rel(root, module)));
    }

    if has_cargo {
        runner::ensure_installed("cargo", root)?;
    }
    runner::ensure_installed("bun", root)?;
    if !uv_projects.is_empty() {
        runner::ensure_installed("uv", root)?;
    }
    if !go_roots.is_empty() {
        runner::ensure_installed("go", root)?;
    }

    if root.join("go.work").is_file() {
        workspace::sync_go_toolchain(root, quiet)?;
    }

    if !quiet {
        section("Package managers — updating in parallel");
    }
    feedback.begin_live_panel();

    std::thread::scope(|scope| {
        if has_cargo {
            let fb = feedback.clone();
            let root = root.to_path_buf();
            scope.spawn(move || run_cargo_task(&fb, &root, firewall));
        }

        {
            let fb = feedback.clone();
            let root = root.to_path_buf();
            scope.spawn(move || run_bun_task(&fb, &root, major));
        }

        for project in uv_projects {
            let fb = feedback.clone();
            let task_name = format!("uv — {}", rel(root, &project));
            scope.spawn(move || run_uv_task(&fb, &task_name, &project, firewall));
        }

        for module in go_roots {
            let fb = feedback.clone();
            let task_name = format!("Go — {}", rel(root, &module));
            scope.spawn(move || run_go_task(&fb, &task_name, &module));
        }
    });

    feedback.freeze_panel(quiet);

    for task in feedback.tasks_snapshot() {
        if task.status == TaskStatus::Failed {
            had_failures = true;
        }
    }

    // Re-run workspace bootstrap (continue on individual failures).
    section("repo setup — proto install + bun install + moon builds");
    if !run_step(
        feedback,
        "repo setup — proto install",
        "proto",
        &["install".to_string()],
        root,
        quiet,
    )? {
        had_failures = true;
    }
    if !run_step(
        feedback,
        "repo setup — bun install",
        "bun",
        &bun_install_args(),
        root,
        quiet,
    )? {
        had_failures = true;
    }
    if !run_step(
        feedback,
        "repo setup — moon run cli:build",
        "moon",
        &["run".to_string(), "cli:build".to_string()],
        root,
        quiet,
    )? {
        had_failures = true;
    }
    if !run_step(
        feedback,
        "repo setup — moon run web:setup",
        "moon",
        &["run".to_string(), "web:setup".to_string()],
        root,
        quiet,
    )? {
        had_failures = true;
    }
    if workspace::uv_workspace_root(root).is_some() {
        runner::ensure_installed("uv", root)?;
        if !run_pm_step(
            feedback,
            "repo setup — uv sync",
            "uv",
            &["sync".to_string()],
            root,
            quiet,
            firewall,
        )? {
            had_failures = true;
        }
    } else if !run_step(
        feedback,
        "repo setup — moon run api:build",
        "moon",
        &["run".to_string(), "api:build".to_string()],
        root,
        quiet,
    )? {
        had_failures = true;
    }
    if root.join("go.work").is_file() {
        runner::ensure_installed("go", root)?;
        if !run_step(
            feedback,
            "repo setup — go work sync",
            "go",
            &["work".to_string(), "sync".to_string()],
            root,
            quiet,
        )? {
            had_failures = true;
        }
    }

    feedback.finish();
    feedback.print_summary(quiet);

    println!("\nUpdate steps finished. Review changes before committing.");
    if !major {
        println!("Tip: re-run with `luna update --major` to also apply major-version bumps.");
    }

    Ok(if had_failures { 1 } else { 0 })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TaskOutcome {
    Ok,
    Blocked,
    Failed,
}

fn uv_update_projects(root: &Path) -> Vec<PathBuf> {
    if let Some(uv_root) = workspace::uv_workspace_root(root) {
        vec![uv_root]
    } else {
        workspace::project_roots(root, "python", "pyproject.toml")
    }
}

fn run_cargo_task(feedback: &UpdateFeedback, root: &Path, firewall: bool) {
    feedback.start_task("Cargo");
    let outcome = match task_cargo_update(feedback, root, firewall) {
        Ok(o) => o,
        Err(_) => TaskOutcome::Failed,
    };
    feedback.finish_task("Cargo", task_status_from_outcome(outcome));
}

fn run_bun_task(feedback: &UpdateFeedback, root: &Path, major: bool) {
    feedback.start_task("Bun");
    let outcome = match task_bun_update(feedback, root, major) {
        Ok(o) => o,
        Err(_) => TaskOutcome::Failed,
    };
    feedback.finish_task("Bun", task_status_from_outcome(outcome));
}

fn run_uv_task(feedback: &UpdateFeedback, task_name: &str, project: &Path, firewall: bool) {
    feedback.start_task(task_name);
    let outcome = match task_uv_update(feedback, task_name, project, firewall) {
        Ok(o) => o,
        Err(_) => TaskOutcome::Failed,
    };
    feedback.finish_task(task_name, task_status_from_outcome(outcome));
}

fn run_go_task(feedback: &UpdateFeedback, task_name: &str, module: &Path) {
    feedback.start_task(task_name);
    let outcome = match task_go_update(feedback, task_name, module) {
        Ok(o) => o,
        Err(_) => TaskOutcome::Failed,
    };
    feedback.finish_task(task_name, task_status_from_outcome(outcome));
}

fn task_status_from_outcome(outcome: TaskOutcome) -> TaskStatus {
    match outcome {
        TaskOutcome::Ok => TaskStatus::Done,
        TaskOutcome::Blocked => TaskStatus::Blocked,
        TaskOutcome::Failed => TaskStatus::Failed,
    }
}

fn task_cargo_update(
    feedback: &UpdateFeedback,
    root: &Path,
    firewall: bool,
) -> Result<TaskOutcome> {
    let args = vec!["update".to_string()];
    let out = capture_pm("cargo", &args, root, firewall)?;
    if out.code != 0 {
        feedback.record_task_output("Cargo", format!("{}{}", out.stdout, out.stderr));
        feedback.record_step_failure("Cargo — cargo update");
        return Ok(TaskOutcome::Failed);
    }
    for line in format!("{}{}", out.stdout, out.stderr).lines() {
        let Some(rest) = line.trim().strip_prefix("Updating ") else {
            continue;
        };
        let Some((name, versions)) = rest.split_once(' ') else {
            continue;
        };
        let Some((from, to)) = versions.split_once(" -> ") else {
            continue;
        };
        feedback.push_package(PackageRow {
            ecosystem: "Cargo",
            package: name.to_string(),
            workspace: None,
            from_version: from.trim_start_matches('v').to_string(),
            to_version: to.trim_start_matches('v').to_string(),
            status: PackageStatus::Updated,
            detail: None,
        });
    }
    Ok(TaskOutcome::Ok)
}

fn task_bun_update(feedback: &UpdateFeedback, root: &Path, major: bool) -> Result<TaskOutcome> {
    let bun_before = outdated::bun_outdated_rows(root)?;
    let mut bun_args = vec!["update".to_string(), "--recursive".to_string()];
    if major {
        bun_args.push("--latest".to_string());
    }
    let bun_out = runner::capture("bun", &bun_args, root)?;
    if bun_out.code != 0 {
        if outdated::bun_failure_is_age_only(&bun_out.stdout, &bun_out.stderr) {
            record_bun_blocked_packages(feedback, &bun_out);
            let bun_after = outdated::bun_outdated_rows(root)?;
            merge_bun_package_rows(feedback, &bun_before, &bun_after);
            return Ok(TaskOutcome::Blocked);
        }
        feedback.record_task_output("Bun", format!("{}{}", bun_out.stdout, bun_out.stderr));
        feedback.record_step_failure("Bun — update workspace dependencies");
        return Ok(TaskOutcome::Failed);
    }

    let bun_after = outdated::bun_outdated_rows(root)?;
    merge_bun_package_rows(feedback, &bun_before, &bun_after);

    if feedback
        .report()
        .packages
        .iter()
        .any(|p| p.ecosystem == "Bun" && p.status == PackageStatus::Blocked)
    {
        return Ok(TaskOutcome::Blocked);
    }
    Ok(TaskOutcome::Ok)
}

fn task_uv_update(
    feedback: &UpdateFeedback,
    task_name: &str,
    project: &Path,
    firewall: bool,
) -> Result<TaskOutcome> {
    let lock_label = format!("{task_name} lock --upgrade");
    let mut lock_args = vec!["lock".to_string(), "--upgrade".to_string()];
    lock_args.extend(security::uv_exclude_newer_args());
    let lock_out = capture_pm("uv", &lock_args, project, firewall)?;
    if lock_out.code != 0 {
        feedback.record_task_output(task_name, format!("{}{}", lock_out.stdout, lock_out.stderr));
        feedback.record_step_failure(lock_label);
        return Ok(TaskOutcome::Failed);
    }
    for (package, from_ver, to_ver) in
        outdated::parse_uv_dry_run_updates(&format!("{}{}", lock_out.stdout, lock_out.stderr))
    {
        feedback.push_package(PackageRow {
            ecosystem: "uv",
            package,
            workspace: None,
            from_version: from_ver,
            to_version: to_ver,
            status: PackageStatus::Updated,
            detail: None,
        });
    }

    let sync_out = capture_pm("uv", &["sync".to_string()], project, firewall)?;
    if sync_out.code != 0 {
        feedback.record_task_output(task_name, format!("{}{}", sync_out.stdout, sync_out.stderr));
        feedback.record_step_failure(format!("{task_name} sync"));
        return Ok(TaskOutcome::Failed);
    }

    record_uv_still_blocked(feedback, project, firewall);

    if feedback
        .report()
        .packages
        .iter()
        .any(|p| p.ecosystem == "uv" && p.status == PackageStatus::Blocked)
    {
        return Ok(TaskOutcome::Blocked);
    }
    Ok(TaskOutcome::Ok)
}

fn task_go_update(
    feedback: &UpdateFeedback,
    task_name: &str,
    module: &Path,
) -> Result<TaskOutcome> {
    let mut failed = false;
    let mut last_output = String::new();

    if workspace::go_uses_tool_fast_path(module) {
        for tool in workspace::go_tool_paths(module) {
            let args = vec![
                "get".to_string(),
                "-tool".to_string(),
                format!("{tool}@latest"),
            ];
            let out = runner::capture("go", &args, module)?;
            if out.code != 0 {
                failed = true;
                last_output = format!("{}{}", out.stdout, out.stderr);
            }
        }
    } else if workspace::full_graph_enabled() {
        let out = runner::capture(
            "go",
            &["get".to_string(), "-u".to_string(), "all".to_string()],
            module,
        )?;
        if out.code != 0 {
            failed = true;
            last_output = format!("{}{}", out.stdout, out.stderr);
        }
    } else {
        let out = runner::capture("go", &["get".to_string(), "-u".to_string()], module)?;
        if out.code != 0 {
            failed = true;
            last_output = format!("{}{}", out.stdout, out.stderr);
        }
    }

    let tidy_out = runner::capture("go", &["mod".to_string(), "tidy".to_string()], module)?;
    if tidy_out.code != 0 {
        failed = true;
        last_output = format!("{}{}", tidy_out.stdout, tidy_out.stderr);
    }

    if failed {
        feedback.record_task_output(task_name, last_output);
        feedback.record_step_failure(format!("{task_name} update"));
        return Ok(TaskOutcome::Failed);
    }
    Ok(TaskOutcome::Ok)
}

fn record_bun_blocked_packages(feedback: &UpdateFeedback, bun_out: &Output) {
    let blocked =
        outdated::parse_bun_blocked_errors(&format!("{}{}", bun_out.stdout, bun_out.stderr));
    for (pkg, spec) in blocked {
        feedback.push_package(PackageRow {
            ecosystem: "Bun",
            package: pkg,
            workspace: None,
            from_version: spec,
            to_version: "—".into(),
            status: PackageStatus::Blocked,
            detail: Some("minimum-release-age".into()),
        });
    }
}

fn capture_pm(program: &str, args: &[String], cwd: &Path, firewall: bool) -> Result<Output> {
    let (program, args) = security::wrap(program, args, firewall);
    runner::capture(&program, &args, cwd)
}

fn bun_install_args() -> Vec<String> {
    vec![
        "install".to_string(),
        "--ignore-scripts".to_string(),
        security::bun_min_release_age_arg(),
    ]
}

fn merge_bun_package_rows(
    feedback: &UpdateFeedback,
    before: &[outdated::BunOutdatedRow],
    after: &[outdated::BunOutdatedRow],
) {
    let mut seen = HashMap::new();
    for row in before {
        let key = (row.package.clone(), row.workspace.clone());
        seen.insert(key, row.clone());
    }

    for row in after {
        let key = (row.package.clone(), row.workspace.clone());
        let prev = seen.get(&key);
        let from = prev
            .map(|p| p.current.as_str())
            .unwrap_or(row.current.as_str());
        let target = if row.update != row.current {
            &row.update
        } else {
            &row.current
        };

        let status = if prev.is_some_and(|p| p.current != row.current) {
            PackageStatus::Updated
        } else if row.latest_blocked_by_age || row.latest != row.update {
            PackageStatus::Blocked
        } else if prev.is_some_and(|p| p.current != p.update) && row.current == row.update {
            PackageStatus::Updated
        } else {
            PackageStatus::Unchanged
        };

        let report = feedback.report();
        if report.packages.iter().any(|p| {
            p.ecosystem == "Bun"
                && p.package == row.package
                && p.status == PackageStatus::Blocked
                && p.detail.as_deref() == Some("minimum-release-age")
        }) {
            continue;
        }

        if status == PackageStatus::Unchanged && prev.is_none() {
            continue;
        }

        let detail = if status == PackageStatus::Blocked && row.latest_blocked_by_age {
            Some(format!("latest {}", row.latest))
        } else {
            None
        };

        feedback.push_package(PackageRow {
            ecosystem: "Bun",
            package: row.package.clone(),
            workspace: row.workspace.clone(),
            from_version: from.to_string(),
            to_version: target.to_string(),
            status,
            detail,
        });
    }
}

/// Packages still blocked by `--exclude-newer` after a successful lock upgrade.
fn record_uv_still_blocked(feedback: &UpdateFeedback, cwd: &Path, firewall: bool) {
    let mut args = vec![
        "lock".to_string(),
        "--upgrade".to_string(),
        "--dry-run".to_string(),
    ];
    args.extend(security::uv_exclude_newer_args());
    let Ok(out) = capture_pm("uv", &args, cwd, firewall) else {
        return;
    };
    if out.code != 0 {
        return;
    }
    for (package, from_ver, to_ver) in
        outdated::parse_uv_dry_run_updates(&format!("{}{}", out.stdout, out.stderr))
    {
        feedback.push_package(PackageRow {
            ecosystem: "uv",
            package,
            workspace: None,
            from_version: from_ver,
            to_version: to_ver,
            status: PackageStatus::Blocked,
            detail: Some("exclude-newer".into()),
        });
    }
}

/// Run a step; returns `Ok(false)` on non-zero exit without aborting the pipeline.
fn run_step(
    feedback: &UpdateFeedback,
    step_label: impl Into<String>,
    program: &str,
    args: &[String],
    cwd: &Path,
    quiet: bool,
) -> Result<bool> {
    let label = step_label.into();
    feedback.set_step(label.clone());
    let code = runner::run(program, args, cwd, quiet)?;
    feedback.clear_step();
    if code != 0 {
        feedback.record_step_failure(label);
        return Ok(false);
    }
    Ok(true)
}

fn run_pm_step(
    feedback: &UpdateFeedback,
    step_label: impl Into<String>,
    program: &str,
    args: &[String],
    cwd: &Path,
    quiet: bool,
    firewall: bool,
) -> Result<bool> {
    let label = step_label.into();
    feedback.set_step(label.clone());
    let code = runner::run_pm(program, args, cwd, quiet, firewall)?;
    feedback.clear_step();
    if code != 0 {
        feedback.record_step_failure(label);
        return Ok(false);
    }
    Ok(true)
}

fn section(title: &str) {
    println!("\n\x1b[1m== {title} ==\x1b[0m");
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

/// Shared progress + results for `luna update` (read by the Starbase `execute` phase).
#[derive(Clone, Debug, Default)]
pub struct UpdateFeedback {
    inner: Arc<Mutex<FeedbackState>>,
}

#[derive(Debug, Default)]
struct FeedbackState {
    tasks: Vec<ToolchainTask>,
    live: bool,
    panel_lines: usize,
    finished: bool,
    active_step: Option<String>,
    report: UpdateReport,
}

/// Per-toolchain task tracked in the live progress panel.
#[derive(Debug, Clone)]
pub struct ToolchainTask {
    pub name: String,
    pub status: TaskStatus,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Queued,
    Running,
    Done,
    Blocked,
    Failed,
}

/// One package row in the final summary.
#[derive(Debug, Clone)]
pub struct PackageRow {
    pub ecosystem: &'static str,
    pub package: String,
    pub workspace: Option<String>,
    pub from_version: String,
    pub to_version: String,
    pub status: PackageStatus,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageStatus {
    Updated,
    Blocked,
    Unchanged,
}

#[derive(Clone, Debug, Default)]
pub struct UpdateReport {
    pub packages: Vec<PackageRow>,
    pub failed_steps: Vec<String>,
}

impl UpdateFeedback {
    pub fn register_task(&self, name: impl Into<String>) {
        if let Ok(mut s) = self.inner.lock() {
            let name = name.into();
            if s.tasks.iter().any(|t| t.name == name) {
                return;
            }
            s.tasks.push(ToolchainTask {
                name,
                status: TaskStatus::Queued,
                output: None,
            });
        }
    }

    pub fn start_task(&self, name: &str) {
        if let Ok(mut s) = self.inner.lock() {
            if let Some(task) = s.tasks.iter_mut().find(|t| t.name == name) {
                task.status = TaskStatus::Running;
            }
        }
    }

    pub fn finish_task(&self, name: &str, status: TaskStatus) {
        if let Ok(mut s) = self.inner.lock() {
            if let Some(task) = s.tasks.iter_mut().find(|t| t.name == name) {
                task.status = status;
            }
        }
    }

    pub fn record_task_output(&self, name: &str, output: impl Into<String>) {
        if let Ok(mut s) = self.inner.lock() {
            if let Some(task) = s.tasks.iter_mut().find(|t| t.name == name) {
                task.output = Some(output.into());
            }
        }
    }

    pub fn begin_live_panel(&self) {
        if let Ok(mut s) = self.inner.lock() {
            s.live = true;
        }
    }

    pub fn freeze_panel(&self, quiet: bool) {
        if quiet {
            if let Ok(mut s) = self.inner.lock() {
                s.live = false;
            }
            return;
        }
        let lines = self.panel_line_count();
        if lines > 0 {
            eprint!("\x1b[{lines}A");
        }
        self.render_panel(false);
        eprintln!();
        if let Ok(mut s) = self.inner.lock() {
            s.live = false;
            s.panel_lines = 0;
        }
    }

    pub fn set_step(&self, label: impl Into<String>) {
        if let Ok(mut s) = self.inner.lock() {
            s.active_step = Some(label.into());
        }
    }

    pub fn clear_step(&self) {
        if let Ok(mut s) = self.inner.lock() {
            s.active_step = None;
        }
    }

    pub fn finish(&self) {
        if let Ok(mut s) = self.inner.lock() {
            s.finished = true;
            s.live = false;
            s.active_step = None;
        }
    }

    pub fn record_step_failure(&self, label: impl Into<String>) {
        if let Ok(mut s) = self.inner.lock() {
            s.report.failed_steps.push(label.into());
        }
    }

    pub fn push_package(&self, row: PackageRow) {
        if let Ok(mut s) = self.inner.lock() {
            s.report.packages.push(row);
        }
    }

    pub fn report(&self) -> UpdateReport {
        self.inner
            .lock()
            .map(|s| UpdateReport {
                packages: s.report.packages.clone(),
                failed_steps: s.report.failed_steps.clone(),
            })
            .unwrap_or_default()
    }

    pub fn tasks_snapshot(&self) -> Vec<ToolchainTask> {
        self.inner
            .lock()
            .map(|s| s.tasks.clone())
            .unwrap_or_default()
    }

    pub fn is_finished(&self) -> bool {
        self.inner.lock().map(|s| s.finished).unwrap_or(true)
    }

    pub fn is_live(&self) -> bool {
        self.inner.lock().map(|s| s.live).unwrap_or(false)
    }

    pub fn active_step_label(&self) -> Option<String> {
        self.inner.lock().ok().and_then(|s| s.active_step.clone())
    }

    fn panel_line_count(&self) -> usize {
        self.inner.lock().map(|s| s.panel_lines).unwrap_or(0)
    }

    fn render_panel(&self, live: bool) {
        let (tasks, step) = self
            .inner
            .lock()
            .map(|s| (s.tasks.clone(), s.active_step.clone()))
            .unwrap_or_default();

        if tasks.is_empty() && step.is_none() {
            return;
        }

        let mut lines: Vec<String> = Vec::new();
        if !tasks.is_empty() {
            lines.push("Toolchain updates:".style(Style::Shell));
            for task in &tasks {
                lines.push(format_task_line(task, live));
            }
        }
        if let Some(step) = step {
            lines.push(format!("  › {step}"));
        }

        for line in &lines {
            eprintln!("\x1b[2K{line}");
        }

        if let Ok(mut s) = self.inner.lock() {
            s.panel_lines = lines.len();
        }
    }

    /// Background ticker for Starbase `AppSession::execute` (runs parallel to `luna update`).
    pub async fn run_progress_ticker(&self, quiet: bool) {
        if quiet {
            return;
        }

        let mut tick: u8 = 0;
        loop {
            tokio::time::sleep(Duration::from_millis(450)).await;
            if self.is_finished() {
                break;
            }

            if self.is_live() {
                let lines = self.panel_line_count();
                if lines > 0 {
                    eprint!("\x1b[{lines}A");
                }
                self.render_panel(true);
                tick = tick.wrapping_add(1);
                continue;
            }

            let Some(step) = self.active_step_label() else {
                continue;
            };
            let spinner = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'][tick as usize % 8];
            tick = tick.wrapping_add(1);
            eprint!("\r{spinner} {}", step.style(Style::Shell));
        }
        eprint!("\r\x1b[2K");
    }

    pub fn print_summary(&self, quiet: bool) {
        let report = self.report();
        let tasks = self.tasks_snapshot();
        let grouped = group_packages_by_ecosystem(&report.packages);
        let no_updates = toolchains_with_no_updates(&tasks, &report.packages);

        if report.packages.is_empty() && report.failed_steps.is_empty() && no_updates.is_empty() {
            return;
        }

        if quiet {
            let updated = report
                .packages
                .iter()
                .filter(|r| r.status == PackageStatus::Updated)
                .count();
            let blocked = report
                .packages
                .iter()
                .filter(|r| r.status == PackageStatus::Blocked)
                .count();
            eprintln!(
                "\n\x1b[1m== Update summary ==\x1b[0m  {} updated, {} blocked, {} failed, {} unchanged toolchain(s)",
                updated,
                blocked,
                report.failed_steps.len(),
                no_updates.len()
            );
            eprintln!("\x1b[2m  Re-run without -q for the full table.\x1b[0m");
            return;
        }

        println!("\n\x1b[1m== Update summary ==\x1b[0m");

        let failed_tasks: Vec<_> = tasks
            .iter()
            .filter(|t| t.status == TaskStatus::Failed)
            .collect();
        if !report.failed_steps.is_empty() || !failed_tasks.is_empty() {
            let count = report.failed_steps.len() + failed_tasks.len();
            println!(
                "\x1b[33m✗\x1b[0m {count} step(s) reported errors (other ecosystems continued)"
            );
            for step in &report.failed_steps {
                println!("  \x1b[2m•\x1b[0m {}", step.style(Style::Failure));
            }
            for task in failed_tasks {
                println!("  \x1b[2m•\x1b[0m {}", task.name.style(Style::Failure));
                if let Some(output) = &task.output {
                    let trimmed = output.trim();
                    if !trimmed.is_empty() {
                        for line in trimmed.lines().take(12) {
                            println!("    \x1b[2m{line}\x1b[0m");
                        }
                        if trimmed.lines().count() > 12 {
                            println!("    \x1b[2m…\x1b[0m");
                        }
                    }
                }
            }
        }

        if !no_updates.is_empty() {
            println!(
                "\x1b[2mNo updates:\x1b[0m {}",
                no_updates.join(", ").style(Style::Muted)
            );
        }

        for (ecosystem, rows) in grouped {
            println!("\n\x1b[1m== {ecosystem} ==\x1b[0m");
            print_package_table(&rows);
        }
    }
}

fn format_task_line(task: &ToolchainTask, live: bool) -> String {
    let (icon, status_style) = match task.status {
        TaskStatus::Queued => ("○", Style::Muted),
        TaskStatus::Running => {
            if live {
                ("⠋", Style::Shell)
            } else {
                ("…", Style::Shell)
            }
        }
        TaskStatus::Done => ("✓", Style::Success),
        TaskStatus::Blocked => ("⊘", Style::Caution),
        TaskStatus::Failed => ("✗", Style::Failure),
    };
    format!("  {icon} {}", task.name).style(status_style)
}

/// Group package rows by ecosystem for sectioned summary output.
pub fn group_packages_by_ecosystem(packages: &[PackageRow]) -> Vec<(String, Vec<PackageRow>)> {
    let mut map: BTreeMap<&str, Vec<PackageRow>> = BTreeMap::new();
    for row in packages {
        map.entry(row.ecosystem).or_default().push(row.clone());
    }
    map.into_iter()
        .map(|(eco, mut rows)| {
            rows.sort_by(|a, b| (&a.package, &a.workspace).cmp(&(&b.package, &b.workspace)));
            (eco.to_string(), rows)
        })
        .collect()
}

/// Toolchains that finished without producing any package rows.
pub fn toolchains_with_no_updates(tasks: &[ToolchainTask], packages: &[PackageRow]) -> Vec<String> {
    let ecosystems_with_rows: BTreeSet<&str> = packages.iter().map(|p| p.ecosystem).collect();

    let mut out = Vec::new();
    for task in tasks {
        if !matches!(
            task.status,
            TaskStatus::Done | TaskStatus::Blocked | TaskStatus::Failed
        ) {
            continue;
        }
        let eco = task_ecosystem_key(&task.name);
        if ecosystems_with_rows.contains(eco.as_str()) {
            continue;
        }
        out.push(task.name.clone());
    }
    out
}

fn task_ecosystem_key(task_name: &str) -> String {
    if task_name.starts_with("Go —") {
        "Go".into()
    } else if task_name.starts_with("uv —") || task_name.starts_with("Python / uv") {
        "uv".into()
    } else if task_name.starts_with("Bun") {
        "Bun".into()
    } else if task_name.starts_with("Cargo") || task_name.contains("Cargo") {
        "Cargo".into()
    } else if task_name.starts_with("proto") {
        "proto".into()
    } else {
        task_name.to_string()
    }
}

fn print_package_table(rows: &[PackageRow]) {
    let pkg_w = rows
        .iter()
        .map(|r| r.package.len())
        .max()
        .unwrap_or(7)
        .max(7);
    let ws_w = rows
        .iter()
        .map(|r| r.workspace.as_ref().map(|w| w.len()).unwrap_or(1))
        .max()
        .unwrap_or(1)
        .max(9);

    let header = format!(
        " {:pkg_w$} │ {:ws_w$} │ {:^12} │ {:^12} │ {}",
        "Package",
        "Workspace",
        "From",
        "To",
        "Status",
        pkg_w = pkg_w,
        ws_w = ws_w,
    );
    let rule_len = header.len().max(64);
    println!("╭{}╮", "─".repeat(rule_len.saturating_sub(2)));
    println!("│{header}│");
    println!("│{}│", "─".repeat(rule_len.saturating_sub(2)));

    for row in rows {
        let ws = row.workspace.as_deref().unwrap_or("—");
        let from = fmt_version(row.status, &row.from_version);
        let to = fmt_version(row.status, &row.to_version);
        let status = status_label(row.status, row.detail.as_deref());
        println!(
            "│ {:pkg_w$} │ {:ws_w$} │ {:^12} │ {:^12} │ {}",
            row.package,
            ws,
            from,
            to,
            status,
            pkg_w = pkg_w,
            ws_w = ws_w,
        );
    }
    println!("╰{}╯", "─".repeat(rule_len.saturating_sub(2)));
}

fn fmt_version(status: PackageStatus, ver: &str) -> String {
    let ver = ver.to_string();
    match status {
        PackageStatus::Updated => ver.style(Style::Success),
        PackageStatus::Blocked => ver.style(Style::Failure),
        PackageStatus::Unchanged => ver.style(Style::Muted),
    }
}

fn status_label(status: PackageStatus, detail: Option<&str>) -> String {
    let base = match status {
        PackageStatus::Updated => "updated",
        PackageStatus::Blocked => "blocked",
        PackageStatus::Unchanged => "unchanged",
    };
    match detail {
        Some(d) => format!("{base} ({d})"),
        None => base.to_string(),
    }
    .style(match status {
        PackageStatus::Updated => Style::Success,
        PackageStatus::Blocked => Style::Failure,
        PackageStatus::Unchanged => Style::Muted,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn group_packages_by_ecosystem_sorts_within_sections() {
        let packages = vec![
            PackageRow {
                ecosystem: "Bun",
                package: "vite".into(),
                workspace: None,
                from_version: "1".into(),
                to_version: "2".into(),
                status: PackageStatus::Updated,
                detail: None,
            },
            PackageRow {
                ecosystem: "uv",
                package: "fastapi".into(),
                workspace: None,
                from_version: "1".into(),
                to_version: "2".into(),
                status: PackageStatus::Updated,
                detail: None,
            },
            PackageRow {
                ecosystem: "Bun",
                package: "react".into(),
                workspace: None,
                from_version: "1".into(),
                to_version: "2".into(),
                status: PackageStatus::Updated,
                detail: None,
            },
        ];
        let grouped = group_packages_by_ecosystem(&packages);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].0, "Bun");
        assert_eq!(grouped[0].1[0].package, "react");
        assert_eq!(grouped[1].0, "uv");
    }

    #[test]
    fn toolchains_with_no_updates_lists_finished_tasks_without_rows() {
        let tasks = vec![
            ToolchainTask {
                name: "Cargo".into(),
                status: TaskStatus::Done,
                output: None,
            },
            ToolchainTask {
                name: "Bun".into(),
                status: TaskStatus::Blocked,
                output: None,
            },
            ToolchainTask {
                name: "Go — apps/web".into(),
                status: TaskStatus::Done,
                output: None,
            },
        ];
        let packages = vec![PackageRow {
            ecosystem: "Bun",
            package: "vite".into(),
            workspace: None,
            from_version: "1".into(),
            to_version: "2".into(),
            status: PackageStatus::Blocked,
            detail: Some("minimum-release-age".into()),
        }];
        let no_updates = toolchains_with_no_updates(&tasks, &packages);
        assert_eq!(no_updates, vec!["Cargo", "Go — apps/web"]);
    }
}
