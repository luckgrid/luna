use serde::{Deserialize, Serialize};

/// Dependency toolchain groups handled by the planner, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolchainKind {
    Proto,
    Rust,
    Bun,
    Uv,
    Go,
}

impl ToolchainKind {
    /// Fixed report order: proto → rust → bun → uv → go.
    pub const ORDER: [ToolchainKind; 5] = [
        ToolchainKind::Proto,
        ToolchainKind::Rust,
        ToolchainKind::Bun,
        ToolchainKind::Uv,
        ToolchainKind::Go,
    ];

    /// User-facing label used in panels, divider rows, and banners.
    pub fn label(self) -> &'static str {
        match self {
            ToolchainKind::Proto => "proto",
            ToolchainKind::Rust => "rust",
            ToolchainKind::Bun => "bun",
            ToolchainKind::Uv => "uv",
            ToolchainKind::Go => "go",
        }
    }
}

/// Lifecycle state of a toolchain group across probe and update phases.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ToolchainState {
    Queued,
    Running,
    UpToDate,
    Outdated,
    Blocked,
    Failed,
    Skipped,
}

/// One normalized dependency row shared by outdated and update reports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyRow {
    pub toolchain: ToolchainKind,
    /// Moon project names only; joined with ", " for the Workspace column.
    pub workspaces: Vec<String>,
    pub dependency: String,
    pub current: String,
    pub newest: Option<String>,
    pub newest_release_age_days: Option<u32>,
    pub latest: Option<String>,
    pub latest_release_age_days: Option<u32>,
    /// True when `latest` is exactly one major version ahead of `current`.
    pub latest_one_major_ahead: bool,
    /// Version before update (update report only).
    pub previous: Option<String>,
    /// Version after update (update report only).
    pub new_version: Option<String>,
    pub result: Option<String>,
    pub blocked_reason: Option<String>,
    pub source_path: Option<String>,
}

impl DependencyRow {
    /// Build a bare outdated row; release-age fields are filled in later.
    pub fn outdated(
        toolchain: ToolchainKind,
        dependency: impl Into<String>,
        current: impl Into<String>,
        newest: Option<String>,
        latest: Option<String>,
    ) -> Self {
        let current = current.into();
        let latest_one_major_ahead = latest
            .as_deref()
            .map(|l| one_major_ahead(&current, l))
            .unwrap_or(false);
        Self {
            toolchain,
            workspaces: Vec::new(),
            dependency: dependency.into(),
            current,
            newest,
            newest_release_age_days: None,
            latest,
            latest_release_age_days: None,
            latest_one_major_ahead,
            previous: None,
            new_version: None,
            result: None,
            blocked_reason: None,
            source_path: None,
        }
    }
}

/// True when `latest`'s major version is exactly one greater than `current`'s.
pub fn one_major_ahead(current: &str, latest: &str) -> bool {
    match (major_of(current), major_of(latest)) {
        (Some(c), Some(l)) => l == c + 1,
        _ => false,
    }
}

fn major_of(version: &str) -> Option<u64> {
    let v = version.trim().trim_start_matches('v');
    let head = v.split(['.', '+', '-']).next()?;
    head.parse::<u64>().ok()
}

/// Per-toolchain probe result captured in the snapshot and live panel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolchainSnapshot {
    pub kind: ToolchainKind,
    pub label: String,
    pub state: ToolchainState,
    pub elapsed_ms: u64,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub rows: Vec<DependencyRow>,
    pub diagnostics: Vec<String>,
}

impl ToolchainSnapshot {
    pub fn has_updates(&self) -> bool {
        self.state == ToolchainState::Outdated && !self.rows.is_empty()
    }
}

/// Policy inputs that affect probe/update results; recorded for snapshot reuse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SnapshotPolicy {
    pub major: bool,
    pub min_release_age_days: u64,
    pub uv_exclude_newer: Option<String>,
    pub firewall: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_major_ahead_detects_single_major_bump() {
        assert!(one_major_ahead("7.3.4", "8.0.14"));
        assert!(one_major_ahead("v1.2.3", "2.0.0"));
    }

    #[test]
    fn one_major_ahead_rejects_same_or_multi_major() {
        assert!(!one_major_ahead("7.3.4", "7.4.0"));
        assert!(!one_major_ahead("7.3.4", "9.0.0"));
        assert!(!one_major_ahead("nightly", "8.0.0"));
    }
}
