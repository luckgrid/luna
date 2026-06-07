pub mod bun;
pub mod cargo;
pub mod go;
pub mod pixi;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::model::{DependencyRow, ToolchainState};

    #[test]
    fn probe_outcome_up_to_date() {
        let o = ProbeOutcome::up_to_date();
        assert_eq!(o.state, ToolchainState::UpToDate);
        assert!(o.rows.is_empty());
        assert!(o.diagnostics.is_empty());
    }

    #[test]
    fn probe_outcome_outdated_with_rows() {
        let row = DependencyRow::outdated(
            ToolchainKind::Bun,
            "vite",
            "7.3.4",
            Some("7.3.5".into()),
            None,
        );
        let o = ProbeOutcome::outdated(vec![row]);
        assert_eq!(o.state, ToolchainState::Outdated);
        assert_eq!(o.rows.len(), 1);
    }

    #[test]
    fn probe_outcome_outdated_empty_rows_becomes_up_to_date() {
        let o = ProbeOutcome::outdated(Vec::new());
        assert_eq!(o.state, ToolchainState::UpToDate);
        assert!(o.rows.is_empty());
    }

    #[test]
    fn probe_outcome_failed() {
        let o = ProbeOutcome::failed("something broke");
        assert_eq!(o.state, ToolchainState::Failed);
        assert_eq!(o.diagnostics, vec!["something broke"]);
    }

    #[test]
    fn probe_outcome_with_diagnostic_appends() {
        let o = ProbeOutcome::failed("first").with_diagnostic("second");
        assert_eq!(o.diagnostics.len(), 2);
        assert_eq!(o.diagnostics[0], "first");
        assert_eq!(o.diagnostics[1], "second");
    }

    #[test]
    fn update_outcome_state_mapping() {
        assert_eq!(UpdateOutcome::Done.state(), ToolchainState::UpToDate);
        assert_eq!(UpdateOutcome::Blocked.state(), ToolchainState::Blocked);
        assert_eq!(
            UpdateOutcome::Failed("err".into()).state(),
            ToolchainState::Failed
        );
    }

    #[test]
    fn adapter_for_returns_correct_kind() {
        for kind in ToolchainKind::ORDER {
            let adapter = adapter_for(kind);
            assert_eq!(
                adapter.kind(),
                kind,
                "adapter_for({:?}).kind() mismatch",
                kind
            );
        }
    }

    #[test]
    fn update_opts_fields() {
        let opts = UpdateOpts {
            major: true,
            firewall: true,
        };
        assert!(opts.major);
        assert!(opts.firewall);
    }
}
