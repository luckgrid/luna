use crate::cli::{Cli, Commands};
use crate::commands::update::UpdateFeedback;
use starbase::{AppResult, AppSession};
use std::path::PathBuf;

/// Per-run session passed through Starbase's application lifecycle.
#[derive(Debug, Clone)]
pub struct LunaSession {
    pub cli: Cli,
    pub root: PathBuf,
    pub update_feedback: UpdateFeedback,
}

impl LunaSession {
    pub fn new(cli: Cli, root: PathBuf) -> Self {
        Self {
            cli,
            root,
            update_feedback: UpdateFeedback::default(),
        }
    }
}

#[async_trait::async_trait]
impl AppSession for LunaSession {
    /// Runs in parallel with `commands::dispatch` during `luna update` to show live progress.
    async fn execute(&mut self) -> AppResult {
        if !matches!(self.cli.command, Commands::Update(_)) {
            return Ok(None);
        }
        let quiet = self.cli.global.quiet;
        self.update_feedback.run_progress_ticker(quiet).await;
        Ok(None)
    }
}
