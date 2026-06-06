use crate::deps::model::{DependencyRow, ToolchainKind, ToolchainSnapshot, ToolchainState};
use crate::security;
use iocraft::prelude::*;
use miette::Result;
use starbase_console::ui::{
    ConsoleTheme, Container, List, ListItem, Progress, ProgressDisplay, ProgressReporter, Stack,
    Style, StyledText, Table, TableCol, TableHeader, TableRow, Text, Variant, View,
};
use starbase_console::{Console, ConsoleError, EmptyReporter};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Console instance used by Luna dependency commands.
pub type LunaConsole = Console<EmptyReporter>;

fn map_console(err: ConsoleError) -> miette::Report {
    miette::miette!("{err}")
}

/// Max visible width of the Dependency column before trimming with `…`.
const DEP_BUDGET: usize = 24;

const SPINNER: [char; 8] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];

// --------------------------------------------------------------------------
// Live status panel (shared by outdated probes and update)
// --------------------------------------------------------------------------

#[derive(Clone)]
struct PanelEntry {
    label: String,
    state: ToolchainState,
    started: Option<Instant>,
    elapsed_ms: u64,
}

/// Shared handle for a multi-row toolchain status panel.
#[derive(Clone)]
pub struct StatusPanel {
    entries: Arc<Mutex<Vec<PanelEntry>>>,
    done: Arc<AtomicUsize>,
    work_total: Arc<AtomicUsize>,
    quiet: bool,
}

impl StatusPanel {
    pub fn new(quiet: bool) -> Self {
        Self {
            entries: Arc::new(Mutex::new(Vec::new())),
            done: Arc::new(AtomicUsize::new(0)),
            work_total: Arc::new(AtomicUsize::new(0)),
            quiet,
        }
    }

    pub fn register(&self, label: impl Into<String>) {
        if let Ok(mut entries) = self.entries.lock() {
            entries.push(PanelEntry {
                label: label.into(),
                state: ToolchainState::Queued,
                started: None,
                elapsed_ms: 0,
            });
        }
    }

    pub fn start(&self, label: &str) {
        if let Ok(mut entries) = self.entries.lock() {
            if let Some(entry) = entries.iter_mut().find(|e| e.label == label) {
                entry.state = ToolchainState::Running;
                entry.started = Some(Instant::now());
            }
        }
    }

    pub fn finish(&self, label: &str, state: ToolchainState) {
        if let Ok(mut entries) = self.entries.lock() {
            if let Some(entry) = entries.iter_mut().find(|e| e.label == label) {
                entry.elapsed_ms = entry
                    .started
                    .map(|t| t.elapsed().as_millis() as u64)
                    .unwrap_or(0);
                entry.state = state;
            }
        }
    }

    pub fn set_work_total(&self, total: usize) {
        self.work_total.store(total, Ordering::Release);
    }

    pub fn signal_done(&self) {
        self.done.fetch_add(1, Ordering::Relaxed);
    }

    fn snapshot_entries(&self) -> Vec<PanelEntry> {
        self.entries.lock().map(|e| e.clone()).unwrap_or_default()
    }

    /// Animated live panel; exits when `signal_done` reaches `work_total`.
    pub async fn run_live(&self, console: &LunaConsole, title: &str) -> Result<()> {
        if self.quiet {
            return Ok(());
        }

        let entries = Arc::clone(&self.entries);
        let done = Arc::clone(&self.done);
        let work_total = Arc::clone(&self.work_total);
        let title = title.to_string();

        console
            .render_loop_err(element! {
                StatusListLive(
                    title,
                    entries,
                    done,
                    work_total,
                )
            })
            .await
            .map_err(map_console)
    }

    /// Static frozen result list (✓ / ✗ + outcome text).
    pub fn render_frozen(&self, console: &LunaConsole, result_title: &str) -> Result<()> {
        if self.quiet {
            return Ok(());
        }

        let entries = self.snapshot_entries();
        console
            .render_err(element! {
                Stack {
                    Text(content: result_title.to_string(), weight: Weight::Bold)
                    #(entries.iter().map(|entry| status_row_element(entry, 0, true)))
                }
            })
            .map_err(map_console)
    }
}

#[derive(Props)]
struct StatusListLiveProps {
    title: String,
    entries: Arc<Mutex<Vec<PanelEntry>>>,
    done: Arc<AtomicUsize>,
    work_total: Arc<AtomicUsize>,
}

impl Default for StatusListLiveProps {
    fn default() -> Self {
        Self {
            title: String::new(),
            entries: Arc::new(Mutex::new(Vec::new())),
            done: Arc::new(AtomicUsize::new(0)),
            work_total: Arc::new(AtomicUsize::new(0)),
        }
    }
}

#[component]
fn StatusListLive(props: &StatusListLiveProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut tick = hooks.use_state(|| 0usize);
    let mut should_exit = hooks.use_state(|| false);
    let mut system = hooks.use_context_mut::<SystemContext>();

    let entries = Arc::clone(&props.entries);
    let done = Arc::clone(&props.done);
    let work_total = Arc::clone(&props.work_total);

    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(120)).await;
            tick.set(tick.get().wrapping_add(1));

            let finished = done.load(Ordering::Relaxed);
            let total = work_total.load(Ordering::Relaxed);
            if total > 0 && finished >= total {
                should_exit.set(true);
                break;
            }
        }
    });

    if should_exit.get() {
        system.exit();
        return element!(View).into_any();
    }

    let snapshot = entries.lock().map(|e| e.clone()).unwrap_or_default();
    let frame = tick.get();

    element! {
        Stack {
            Text(content: &props.title, weight: Weight::Bold)
            #(snapshot.iter().map(|entry| {
                status_row_element(entry, frame, false)
            }))
        }
    }
    .into_any()
}

fn status_row_element(entry: &PanelEntry, tick: usize, show_notes: bool) -> AnyElement<'static> {
    let secs = elapsed_secs(entry);
    let live = !show_notes;
    let (icon, style) = state_icon(entry.state, tick, live);
    let note = if show_notes {
        state_note(entry.state)
    } else {
        String::new()
    };
    let timing = format!("{secs:>5.1}s");
    let line = if note.is_empty() {
        format!("  {icon} {:<8} {timing}", entry.label)
    } else {
        format!("  {icon} {:<8} {timing}  {note}", entry.label)
    };

    element! {
        StyledText(content: line, style)
    }
    .into_any()
}

fn elapsed_secs(entry: &PanelEntry) -> f64 {
    if entry.state == ToolchainState::Running {
        entry
            .started
            .map(|t| t.elapsed().as_secs_f64())
            .unwrap_or(0.0)
    } else {
        entry.elapsed_ms as f64 / 1000.0
    }
}

fn state_icon(state: ToolchainState, tick: usize, live: bool) -> (String, Style) {
    match state {
        ToolchainState::Queued if live => (SPINNER[tick % SPINNER.len()].to_string(), Style::Muted),
        ToolchainState::Queued => ("○".into(), Style::Muted),
        ToolchainState::Running => (SPINNER[tick % SPINNER.len()].to_string(), Style::Muted),
        ToolchainState::UpToDate => ("✓".into(), Style::Success),
        ToolchainState::Outdated | ToolchainState::Failed => ("✗".into(), Style::Failure),
        ToolchainState::Blocked => ("⊘".into(), Style::Caution),
        ToolchainState::Skipped => ("—".into(), Style::Muted),
    }
}

fn state_note(state: ToolchainState) -> String {
    match state {
        ToolchainState::UpToDate => "up to date".into(),
        ToolchainState::Outdated => "updates found".into(),
        ToolchainState::Blocked => "blocked".into(),
        ToolchainState::Failed => "check failed".into(),
        ToolchainState::Skipped => "skipped".into(),
        _ => String::new(),
    }
}

// --------------------------------------------------------------------------
// Tables, notices, sections
// --------------------------------------------------------------------------

fn section_header_line(title: &str) -> String {
    format!("--- {} ---", title.to_uppercase())
}

fn section_header_element(title: &str) -> AnyElement<'static> {
    element! {
        Text(content: section_header_line(title), weight: Weight::Bold)
    }
    .into_any()
}

fn trim_failure_body(body: &str) -> String {
    body.trim()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(12)
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Default, Props)]
struct FailureNoticeProps {
    title: String,
    body: String,
}

#[component]
fn FailureNotice(props: &FailureNoticeProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<ConsoleTheme>();
    let color = theme.variant(Variant::Failure);
    let title_color = if theme.supports_color {
        Some(color)
    } else {
        None
    };

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_color: color,
            border_edges: Edges::Left,
            border_style: BorderStyle::Round,
            margin_top: 1,
            margin_bottom: 0,
            padding_left: 1,
            padding_bottom: 0,
        ) {
            Text(
                content: props.title.to_uppercase(),
                color: title_color,
                weight: Weight::Bold,
            )
            StyledText(content: props.body.clone(), style: Style::Shell)
        }
    }
}

fn summary_count_style(count: usize, highlight: Style) -> Style {
    if count > 0 {
        highlight
    } else {
        Style::Muted
    }
}

fn summary_count_element(label: &str, count: usize, style: Style) -> AnyElement<'static> {
    element! {
        StyledText(content: format!("{label} {count}"), style)
    }
    .into_any()
}

/// Trim a dependency name to the column budget with a trailing ellipsis.
pub fn trim_dependency(name: &str) -> String {
    if name.chars().count() <= DEP_BUDGET {
        return name.to_string();
    }
    let keep: String = name.chars().take(DEP_BUDGET.saturating_sub(1)).collect();
    format!("{keep}…")
}

pub fn workspaces_text(row: &DependencyRow) -> String {
    if row.workspaces.is_empty() {
        "—".to_string()
    } else {
        row.workspaces.join(", ")
    }
}

fn newest_style(age_days: Option<u32>) -> Style {
    match age_days {
        Some(age) if (age as u64) >= security::min_release_age_days() => Style::Success,
        Some(_) => Style::Failure,
        None => Style::Muted,
    }
}

fn latest_style(one_major_ahead: bool) -> Style {
    if one_major_ahead {
        Style::Caution
    } else {
        Style::Muted
    }
}

pub fn release_age_cell(row: &DependencyRow, newest_label: &str) -> String {
    let mut parts = Vec::new();
    if let Some(age) = row.newest_release_age_days {
        parts.push(format!("{newest_label} {age}d"));
    }
    if let Some(age) = row.latest_release_age_days {
        parts.push(format!("latest {age}d"));
    }
    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join(" · ")
    }
}

fn table_headers() -> Vec<TableHeader> {
    vec![
        TableHeader::new("Toolchain", Size::Length(10)),
        TableHeader::new("Workspace", Size::Length(12)),
        TableHeader::new("Dependency", Size::Length(14)),
        TableHeader::new("Current", Size::Length(10)),
        TableHeader::new("Newest", Size::Length(10)),
        TableHeader::new("Latest", Size::Length(10)),
        TableHeader::new("Release Age", Size::Auto),
    ]
}

fn update_table_headers() -> Vec<TableHeader> {
    vec![
        TableHeader::new("Workspace", Size::Length(12)),
        TableHeader::new("Dependency", Size::Length(14)),
        TableHeader::new("Previous", Size::Length(10)),
        TableHeader::new("New", Size::Length(10)),
        TableHeader::new("Latest", Size::Length(10)),
        TableHeader::new("Release Age", Size::Auto),
    ]
}

pub fn render_outdated_table(console: &LunaConsole, snapshots: &[ToolchainSnapshot]) -> Result<()> {
    let rows = outdated_rows(snapshots);
    if rows.is_empty() {
        return Ok(());
    }

    let headers = table_headers();
    console
        .render(element! {
            View(margin_top: 1) {
                Container {
                    Table(headers: headers) {
                        #(rows.iter().enumerate().map(|(row_idx, (toolchain, row))| {
                            outdated_row_element(row_idx, toolchain, row)
                        }))
                    }
                }
            }
        })
        .map_err(map_console)
}

fn outdated_rows(snapshots: &[ToolchainSnapshot]) -> Vec<(&'static str, &DependencyRow)> {
    let mut rows = Vec::new();
    for kind in ToolchainKind::ORDER {
        let Some(tc) = snapshots.iter().find(|t| t.kind == kind) else {
            continue;
        };
        if !tc.has_updates() {
            continue;
        }
        let mut kind_rows: Vec<&DependencyRow> = tc.rows.iter().collect();
        kind_rows.sort_by(|a, b| {
            (a.workspaces.join(","), &a.dependency).cmp(&(b.workspaces.join(","), &b.dependency))
        });
        for row in kind_rows {
            rows.push((kind.label(), row));
        }
    }
    rows
}

fn outdated_row_element(
    row_idx: usize,
    toolchain: &str,
    row: &DependencyRow,
) -> AnyElement<'static> {
    let newest = row.newest.clone().unwrap_or_else(|| "—".to_string());
    let latest = row.latest.clone().unwrap_or_else(|| "—".to_string());
    element! {
        TableRow(row: row_idx as i32) {
            TableCol(col: 0) { Text(content: toolchain.to_string()) }
            TableCol(col: 1) { Text(content: workspaces_text(row)) }
            TableCol(col: 2) { Text(content: trim_dependency(&row.dependency)) }
            TableCol(col: 3) { Text(content: row.current.clone()) }
            TableCol(col: 4) {
                StyledText(content: newest, style: newest_style(row.newest_release_age_days))
            }
            TableCol(col: 5) {
                StyledText(content: latest, style: latest_style(row.latest_one_major_ahead))
            }
            TableCol(col: 6) {
                StyledText(
                    content: release_age_cell(row, "newest"),
                    style: Style::Muted,
                )
            }
        }
    }
    .into_any()
}

pub fn render_update_table(
    console: &LunaConsole,
    groups: &[(ToolchainKind, Vec<DependencyRow>)],
) -> Result<()> {
    if groups.is_empty() {
        return Ok(());
    }

    console
        .render(element! {
            View(margin_top: 1) {
                Container {
                    #(groups.iter().enumerate().map(|(idx, (kind, rows))| {
                        update_group_element(idx, kind.label(), rows)
                    }))
                }
            }
        })
        .map_err(map_console)
}

fn update_group_element(idx: usize, label: &str, rows: &[DependencyRow]) -> AnyElement<'static> {
    let headers = update_table_headers();
    let mut sorted: Vec<&DependencyRow> = rows.iter().collect();
    sorted.sort_by(|a, b| {
        (a.workspaces.join(","), &a.dependency).cmp(&(b.workspaces.join(","), &b.dependency))
    });

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(section_header_element(label))
            Table(headers: headers) {
                #(sorted.iter().enumerate().map(|(row_idx, row)| {
                    update_row_element(idx * 100 + row_idx, row)
                }))
            }
        }
    }
    .into_any()
}

fn update_row_element(row_idx: usize, row: &DependencyRow) -> AnyElement<'static> {
    let blocked = row.blocked_reason.is_some();
    let new_text = row
        .new_version
        .clone()
        .or_else(|| row.newest.clone())
        .unwrap_or_else(|| "—".to_string());
    let new_style = if blocked {
        Style::Failure
    } else {
        newest_style(row.newest_release_age_days)
    };
    let mut age = release_age_cell(row, "new");
    if let Some(reason) = &row.blocked_reason {
        age = format!("{age} · blocked: {reason}");
    }
    let previous = row.previous.clone().unwrap_or_else(|| row.current.clone());
    let latest = row.latest.clone().unwrap_or_else(|| "—".to_string());

    element! {
        TableRow(row: row_idx as i32) {
            TableCol(col: 0) { Text(content: workspaces_text(row)) }
            TableCol(col: 1) { Text(content: trim_dependency(&row.dependency)) }
            TableCol(col: 2) { StyledText(content: previous, style: Style::Muted) }
            TableCol(col: 3) { StyledText(content: new_text, style: new_style) }
            TableCol(col: 4) {
                StyledText(content: latest, style: latest_style(row.latest_one_major_ahead))
            }
            TableCol(col: 5) { StyledText(content: age, style: Style::Muted) }
        }
    }
    .into_any()
}

pub fn render_failure_notice(console: &LunaConsole, label: &str, body: &str) -> Result<()> {
    let trimmed = trim_failure_body(body);
    if trimmed.is_empty() {
        return Ok(());
    }

    let title = format!("{label} failed");
    console
        .render_err(element! {
            FailureNotice(title: title, body: trimmed)
        })
        .map_err(map_console)
}

pub fn render_release_age_section(console: &LunaConsole) -> Result<()> {
    let days = security::min_release_age_days();
    console
        .render(element! {
            View(margin_top: 1, margin_bottom: 1) {
                View(flex_direction: FlexDirection::Column) {
                    #(section_header_element("Release Age"))
                    View(padding_left: 2, padding_top: 1) {
                        Stack(gap: 1) {
                            Stack(gap: 0) {
                                StyledText(
                                    content: format!(
                                        "Luna waits {days} days after a package is published before installing it. `LUNA_MIN_RELEASE_AGE`, enforced via `bunfig.toml`, `.npmrc`, and `uv --exclude-newer`. This reduces risk from freshly published malicious releases."
                                    ),
                                    style: Style::Muted,
                                )
                                StyledText(
                                    content: "The Release Age column shows how many days ago each version was published.",
                                    style: Style::Muted,
                                )
                            }
                            StyledText(
                                content: "Run `luna update` to apply in-range Newest versions that pass the release-age policy.",
                                style: Style::Muted,
                            )
                            View(margin_top: 1) {
                                Stack(gap: 0) {
                                    StyledText(
                                        content: "To bypass for a one-off or emergency:",
                                        style: Style::Muted,
                                    )
                                    List {
                                ListItem {
                                    StyledText(
                                        content: "`luna update --major` — widen semver range (still age-gated unless overridden)",
                                        style: Style::Muted,
                                    )
                                }
                                ListItem {
                                    StyledText(
                                        content: "`LUNA_MIN_RELEASE_AGE=0 luna update` — disable cooldown for that invocation",
                                        style: Style::Muted,
                                    )
                                }
                                ListItem {
                                    StyledText(
                                        content: "`bun add <pkg>@<version>` — pin one package in a Bun workspace",
                                        style: Style::Muted,
                                    )
                                }
                                ListItem {
                                    StyledText(
                                        content: "`uv lock --upgrade-package <pkg>` — upgrade one Python dependency",
                                        style: Style::Muted,
                                    )
                                }
                            }
                                }
                            }
                        }
                    }
                }
            }
        })
        .map_err(map_console)
}

pub struct UpdateSummary {
    pub updated: usize,
    pub blocked: usize,
    pub failed: usize,
    pub skipped: usize,
    pub setup_ok: bool,
    pub show_major_tip: bool,
}

pub fn render_update_summary(console: &LunaConsole, summary: &UpdateSummary) -> Result<()> {
    console
        .render(element! {
            View(margin_top: 1) {
                View(flex_direction: FlexDirection::Column) {
                    #(section_header_element("Update summary"))
                    View(padding_left: 2, padding_top: 1) {
                        Stack(gap: 0) {
                            Stack(gap: 0) {
                                #(summary_count_element(
                                    "Updated",
                                    summary.updated,
                                    summary_count_style(summary.updated, Style::Success),
                                ))
                                #(summary_count_element(
                                    "Blocked",
                                    summary.blocked,
                                    summary_count_style(summary.blocked, Style::Caution),
                                ))
                                #(summary_count_element(
                                    "Failed",
                                    summary.failed,
                                    summary_count_style(summary.failed, Style::Failure),
                                ))
                                #(summary_count_element("Skipped", summary.skipped, Style::Muted))
                            }
                            View(margin_top: 1) {}
                            StyledText(
                                content: "Review changes before committing.",
                                style: Style::Muted,
                            )
                            #(if !summary.setup_ok {
                                Some(element! {
                                    StyledText(
                                        content: "Workspace re-sync encountered errors — see messages above.",
                                        style: Style::Failure,
                                    )
                                }.into_any())
                            } else {
                                None
                            })
                            #(if summary.show_major_tip {
                                Some(element! {
                                    StyledText(
                                        content: "Tip: re-run with `luna update --major` to also apply major-version bumps.",
                                        style: Style::Muted,
                                    )
                                }.into_any())
                            } else {
                                None
                            })
                            View(margin_bottom: 1) {}
                        }
                    }
                }
            }
        })
        .map_err(map_console)
}

pub fn render_message(console: &LunaConsole, content: &str) -> Result<()> {
    console
        .render(element! {
            Text(content: content.to_string())
        })
        .map_err(map_console)
}

pub fn render_section_title(console: &LunaConsole, title: &str) -> Result<()> {
    console
        .render(element! {
            View(margin_top: 1) {
                StyledText(content: title.to_string(), style: Style::Muted)
            }
        })
        .map_err(map_console)
}

/// Run work behind a single progress loader; `work` receives a reporter to update messages.
pub async fn run_with_loader<F>(
    console: &LunaConsole,
    initial_message: &str,
    work: F,
) -> Result<(), String>
where
    F: FnOnce(ProgressReporter) -> Result<(), String> + Send + 'static,
{
    let reporter = ProgressReporter::default();
    reporter.set_message(initial_message);

    let reporter_worker = reporter.clone();
    let handle = tokio::task::spawn_blocking(move || {
        let result = work(reporter_worker.clone());
        reporter_worker.exit();
        result
    });

    console
        .render_loop(element! {
            Progress(
                display: ProgressDisplay::Loader,
                default_message: initial_message.to_string(),
                reporter: Some(reporter.clone().into()),
            )
        })
        .await
        .map_err(|e| e.to_string())?;

    handle.await.map_err(|e| e.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_header_line_is_fixed_format() {
        assert_eq!(section_header_line("bun"), "--- BUN ---");
        assert_eq!(section_header_line("Release Age"), "--- RELEASE AGE ---");
    }

    #[test]
    fn trim_dependency_adds_ellipsis() {
        let trimmed = trim_dependency("@scope/a-very-long-package-name-here");
        assert!(trimmed.ends_with('…'));
        assert_eq!(trimmed.chars().count(), DEP_BUDGET);
    }

    #[test]
    fn trim_dependency_keeps_short_names() {
        assert_eq!(trim_dependency("vite"), "vite");
    }

    #[test]
    fn newest_style_respects_cutoff() {
        std::env::remove_var("LUNA_MIN_RELEASE_AGE");
        assert!(matches!(newest_style(Some(21)), Style::Success));
        assert!(matches!(newest_style(Some(3)), Style::Failure));
        assert!(matches!(newest_style(None), Style::Muted));
    }

    #[test]
    fn release_age_cell_formats_both() {
        let mut row = DependencyRow::outdated(
            ToolchainKind::Bun,
            "vite",
            "7.3.4",
            Some("7.3.5".into()),
            Some("8.0.14".into()),
        );
        row.newest_release_age_days = Some(21);
        row.latest_release_age_days = Some(8);
        assert_eq!(release_age_cell(&row, "newest"), "newest 21d · latest 8d");
    }
}
