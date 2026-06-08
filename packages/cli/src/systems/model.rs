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
    /// Full registry key (e.g. Go module path) when `dependency` is a short display name.
    pub registry_name: Option<String>,
}

/// Per-package outcome after an update run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageUpdateStatus {
    Updated,
    Blocked,
    Failed,
    Unchanged,
    Skipped,
}

/// One row in the unified update result table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageUpdateResult {
    pub toolchain: ToolchainKind,
    pub workspaces: Vec<String>,
    pub dependency: String,
    pub registry_name: Option<String>,
    pub previous: String,
    pub new_version: Option<String>,
    pub status: PackageUpdateStatus,
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
            registry_name: None,
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

    #[test]
    fn toolchain_kind_order_is_fixed() {
        assert_eq!(ToolchainKind::ORDER.len(), 5);
        assert_eq!(
            ToolchainKind::ORDER,
            [
                ToolchainKind::Proto,
                ToolchainKind::Rust,
                ToolchainKind::Bun,
                ToolchainKind::Uv,
                ToolchainKind::Go,
            ]
        );
    }

    #[test]
    fn toolchain_kind_labels() {
        assert_eq!(ToolchainKind::Proto.label(), "proto");
        assert_eq!(ToolchainKind::Rust.label(), "rust");
        assert_eq!(ToolchainKind::Bun.label(), "bun");
        assert_eq!(ToolchainKind::Uv.label(), "uv");
        assert_eq!(ToolchainKind::Go.label(), "go");
    }

    #[test]
    fn dependency_row_outdated_sets_latest_one_major_ahead() {
        let row = DependencyRow::outdated(
            ToolchainKind::Bun,
            "vite",
            "7.3.4",
            Some("7.3.5".into()),
            Some("8.0.14".into()),
        );
        assert!(row.latest_one_major_ahead);
        assert_eq!(row.current, "7.3.4");
        assert_eq!(row.newest.as_deref(), Some("7.3.5"));
        assert_eq!(row.latest.as_deref(), Some("8.0.14"));
        assert!(row.previous.is_none());
        assert!(row.new_version.is_none());
        assert!(row.blocked_reason.is_none());
    }

    #[test]
    fn dependency_row_outdated_no_latest_means_not_ahead() {
        let row = DependencyRow::outdated(
            ToolchainKind::Bun,
            "vite",
            "7.3.4",
            Some("7.3.5".into()),
            None,
        );
        assert!(!row.latest_one_major_ahead);
        assert!(row.latest.is_none());
    }

    #[test]
    fn dependency_row_outdated_same_major_latest_not_ahead() {
        let row = DependencyRow::outdated(
            ToolchainKind::Bun,
            "vite",
            "7.3.4",
            Some("7.3.5".into()),
            Some("7.4.0".into()),
        );
        assert!(!row.latest_one_major_ahead);
    }

    #[test]
    fn toolchain_snapshot_has_updates_requires_outdated_and_rows() {
        let snap = ToolchainSnapshot {
            kind: ToolchainKind::Bun,
            label: "bun".into(),
            state: ToolchainState::Outdated,
            elapsed_ms: 0,
            started_at: None,
            finished_at: None,
            rows: Vec::new(),
            diagnostics: Vec::new(),
        };
        assert!(
            !snap.has_updates(),
            "outdated with no rows should not have updates"
        );

        let snap_with_row = ToolchainSnapshot {
            kind: ToolchainKind::Bun,
            label: "bun".into(),
            state: ToolchainState::Outdated,
            elapsed_ms: 0,
            started_at: None,
            finished_at: None,
            rows: vec![DependencyRow::outdated(
                ToolchainKind::Bun,
                "vite",
                "7.3.4",
                Some("7.3.5".into()),
                None,
            )],
            diagnostics: Vec::new(),
        };
        assert!(snap_with_row.has_updates());
    }

    #[test]
    fn snapshot_policy_equality() {
        let a = SnapshotPolicy {
            major: false,
            min_release_age_days: 14,
            uv_exclude_newer: Some("2026-05-01".into()),
            firewall: false,
        };
        let b = SnapshotPolicy {
            major: false,
            min_release_age_days: 14,
            uv_exclude_newer: Some("2026-05-01".into()),
            firewall: false,
        };
        assert_eq!(a, b);

        let c = SnapshotPolicy {
            major: true,
            ..a.clone()
        };
        assert_ne!(a, c);
    }
}
