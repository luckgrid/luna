use crate::cli::Cli;
use starbase::AppSession;
use std::path::PathBuf;

/// Per-run session passed through Starbase's application lifecycle.
///
/// All command work happens in the main execution closure (see `main.rs`),
/// so the lifecycle phases keep their default no-op implementations.
#[derive(Debug, Clone)]
pub struct LunaSession {
    pub cli: Cli,
    pub root: PathBuf,
}

impl LunaSession {
    pub fn new(cli: Cli, root: PathBuf) -> Self {
        Self { cli, root }
    }
}

impl AppSession for LunaSession {}
