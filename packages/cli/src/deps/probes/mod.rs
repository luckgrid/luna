pub mod bun;
pub mod cargo;
pub mod go;
pub mod proto;
pub mod uv;

use crate::deps::model::{DependencyRow, ToolchainState};

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
