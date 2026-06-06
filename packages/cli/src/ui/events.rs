use crate::systems::model::{ToolchainKind, ToolchainState};
use crate::ui::{self, LunaConsole, StatusPanel};
use miette::Result;

/// Event sink that decouples the dependency service from console rendering.
///
/// The `systems` layer publishes progress through these methods; the `ui` layer
/// owns the live status panel and all rendering. The emitter holds its own
/// (cloned) console handle, so it can be moved into concurrent probe/update tasks.
#[derive(Clone)]
pub struct Emitter {
    panel: StatusPanel,
    console: LunaConsole,
}

impl Emitter {
    pub fn new(console: LunaConsole, quiet: bool) -> Self {
        Self {
            panel: StatusPanel::new(quiet),
            console,
        }
    }

    /// Borrow the console for table/notice rendering by callers.
    pub fn console(&self) -> &LunaConsole {
        &self.console
    }

    // --- Panel lifecycle ---------------------------------------------------

    /// Register a row and count it toward the live panel's work total.
    pub fn register_work(&self, kinds: &[ToolchainKind]) {
        for kind in kinds {
            self.panel.register(kind.label());
        }
        self.panel.set_work_total(kinds.len());
    }

    /// Register a row without counting it (e.g. toolchains shown as skipped).
    pub fn register(&self, kind: ToolchainKind) {
        self.panel.register(kind.label());
    }

    /// Set how many `finished` signals complete the live panel.
    pub fn set_work_total(&self, total: usize) {
        self.panel.set_work_total(total);
    }

    /// Mark a toolchain as skipped (rendered, but not counted as work).
    pub fn skipped(&self, kind: ToolchainKind) {
        self.panel.finish(kind.label(), ToolchainState::Skipped);
    }

    // --- Discrete progress events ------------------------------------------

    pub fn probe_started(&self, kind: ToolchainKind) {
        self.panel.start(kind.label());
    }

    pub fn probe_finished(&self, kind: ToolchainKind, state: ToolchainState) {
        self.panel.finish(kind.label(), state);
        self.panel.signal_done();
    }

    pub fn update_started(&self, kind: ToolchainKind) {
        self.panel.start(kind.label());
    }

    pub fn update_finished(&self, kind: ToolchainKind, state: ToolchainState) {
        self.panel.finish(kind.label(), state);
        self.panel.signal_done();
    }

    /// Drive the animated live panel until all counted work is finished.
    pub async fn run_live(&self, title: &str) -> Result<()> {
        self.panel.run_live(&self.console, title).await
    }

    /// Render the frozen result list after the live panel exits.
    pub fn freeze(&self, result_title: &str) -> Result<()> {
        self.panel.render_frozen(&self.console, result_title)
    }

    // --- Notices -----------------------------------------------------------

    pub fn message(&self, content: &str) -> Result<()> {
        ui::render_message(&self.console, content)
    }

    pub fn section_title(&self, title: &str) -> Result<()> {
        ui::render_section_title(&self.console, title)
    }

    pub fn failure_notice(&self, label: &str, body: &str) -> Result<()> {
        ui::render_failure_notice(&self.console, label, body)
    }

    pub fn snapshot_written(&self, rel_path: &str) -> Result<()> {
        ui::render_message(&self.console, &format!("\nSnapshot saved to `{rel_path}`."))
    }
}
