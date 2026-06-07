use crate::cli::Cli;
use crate::config::LunaConfig;
use starbase::AppSession;
use std::path::PathBuf;

/// Per-run session passed through Starbase's application lifecycle.
#[derive(Debug, Clone)]
pub struct LunaSession {
    pub cli: Cli,
    pub root: PathBuf,
    pub config: LunaConfig,
}

impl LunaSession {
    pub fn new(cli: Cli, root: PathBuf, config: LunaConfig) -> Self {
        Self { cli, root, config }
    }
}

#[async_trait::async_trait]
impl AppSession for LunaSession {}
