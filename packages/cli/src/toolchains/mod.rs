pub mod bun;
pub mod cargo;
pub mod go;
pub mod proto;
pub mod uv;

use crate::systems::model::{DependencyRow, ToolchainKind, ToolchainState};
use async_trait::async_trait;
use std::path::Path;

/// Outcome of a single ecosystem probe (timing is added by the planner).
pub struct ProbeOutcome {
    pub state: ToolchainState,
    pub rows: Vec<DependencyRow>,
    pub diagnostics: Vec<String>,
}

impl ProbeOutcome {
    pub fn up_to_date() -> Self {
        Self {
            state: ToolchainState::UpToDate,
            rows: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    pub fn outdated(rows: Vec<DependencyRow>) -> Self {
        if rows.is_empty() {
            Self::up_to_date()
        } else {
            Self {
                state: ToolchainState::Outdated,
                rows,
                diagnostics: Vec::new(),
            }
        }
    }

    pub fn failed(diagnostic: impl Into<String>) -> Self {
        Self {
            state: ToolchainState::Failed,
            rows: Vec::new(),
            diagnostics: vec![diagnostic.into()],
        }
    }

    pub fn with_diagnostic(mut self, diagnostic: impl Into<String>) -> Self {
        self.diagnostics.push(diagnostic.into());
        self
    }
}

/// Per-toolchain update result captured by the update report and live panel.
#[derive(Debug, Clone)]
pub enum UpdateOutcome {
    Done,
    Blocked,
    Failed(String),
}

impl UpdateOutcome {
    /// Map the update outcome to a panel/snapshot state.
    pub fn state(&self) -> ToolchainState {
        match self {
            UpdateOutcome::Done => ToolchainState::UpToDate,
            UpdateOutcome::Blocked => ToolchainState::Blocked,
            UpdateOutcome::Failed(_) => ToolchainState::Failed,
        }
    }
}

/// Options forwarded to a toolchain updater.
#[derive(Debug, Clone, Copy)]
pub struct UpdateOpts {
    pub major: bool,
    pub firewall: bool,
}

/// A per-ecosystem adapter: probes for outdated deps and applies updates.
///
/// Implementations wrap the underlying (blocking) package-manager calls in
/// `tokio::task::spawn_blocking`, so the [`DependencyService`](crate::systems::deps)
/// can schedule them concurrently on the async runtime.
#[async_trait]
pub trait ToolchainAdapter: Send + Sync {
    fn kind(&self) -> ToolchainKind;
    async fn probe(&self, root: &Path) -> ProbeOutcome;
    async fn update(&self, root: &Path, opts: UpdateOpts) -> UpdateOutcome;
}

/// Run a blocking closure on the runtime's blocking pool.
///
/// Adapters use this to wrap synchronous package-manager calls so the planner
/// can drive them concurrently without manual thread management.
pub(crate) async fn run_blocking<T, F>(f: F) -> T
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    tokio::task::spawn_blocking(f)
        .await
        .expect("toolchain task panicked")
}

/// Resolve the adapter for a toolchain kind.
pub fn adapter_for(kind: ToolchainKind) -> Box<dyn ToolchainAdapter> {
    match kind {
        ToolchainKind::Proto => Box::new(proto::ProtoAdapter),
        ToolchainKind::Rust => Box::new(cargo::CargoAdapter),
        ToolchainKind::Bun => Box::new(bun::BunAdapter),
        ToolchainKind::Uv => Box::new(uv::UvAdapter),
        ToolchainKind::Go => Box::new(go::GoAdapter),
    }
}
