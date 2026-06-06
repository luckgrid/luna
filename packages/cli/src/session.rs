use crate::cli::Cli;
use starbase::AppSession;
use std::path::PathBuf;

/// Per-run session passed through Starbase's application lifecycle.
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

#[async_trait::async_trait]
impl AppSession for LunaSession {}
