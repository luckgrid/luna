use crate::systems::model::ToolchainState;
use crate::ui::{map_console, LunaConsole};
use iocraft::prelude::*;
use miette::Result;
use starbase_console::ui::{Stack, Style, StyledText, Text, View};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

const SPINNER: [char; 8] = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧'];

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
