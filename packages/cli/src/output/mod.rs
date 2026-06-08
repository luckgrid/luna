use crate::planner::Plan;
use crate::systems::inventory::InventoryItem;
use crate::systems::ledger::LockLedger;
use crate::systems::model::ToolchainSnapshot;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub const SCHEMA_VERSION: &str = "v1";

/// Print schema-versioned JSON to stdout (ANSI-free).
pub fn emit<T: Serialize>(value: &T) {
    let json = serde_json::to_string_pretty(value).unwrap_or_else(|_| "{}".into());
    println!("{json}");
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedReport {
    pub schema_version: String,
    pub workspace_root: String,
    pub snapshot_path: String,
    pub toolchains: Vec<ToolchainSnapshot>,
    pub has_outdated: bool,
}

impl OutdatedReport {
    pub fn from_snapshots(
        root: &Path,
        snapshots: &[ToolchainSnapshot],
        snapshot_rel: &str,
    ) -> Self {
        Self {
            schema_version: SCHEMA_VERSION.into(),
            workspace_root: root.display().to_string(),
            snapshot_path: snapshot_rel.into(),
            has_outdated: snapshots.iter().any(|s| s.has_updates()),
            toolchains: snapshots.to_vec(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateReportJson {
    pub schema_version: String,
    pub workspace_root: String,
    pub updated: usize,
    pub blocked: usize,
    pub failed: usize,
    pub unchanged: usize,
    pub skipped: usize,
    pub setup_ok: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub packages: Vec<crate::systems::model::PackageUpdateResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectGraph {
    pub schema_version: String,
    pub workspace_root: String,
    pub projects: Vec<ProjectNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectNode {
    pub id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGraph {
    pub schema_version: String,
    pub workspace_root: String,
    pub target: Option<String>,
    pub raw: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanReport {
    pub schema_version: String,
    #[serde(flatten)]
    pub plan: Plan,
}

impl PlanReport {
    pub fn new(plan: Plan) -> Self {
        Self {
            schema_version: SCHEMA_VERSION.into(),
            plan,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigReport {
    pub schema_version: String,
    pub valid: bool,
    pub warnings: Vec<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub version: String,
    pub adapter: String,
    pub ecosystem: String,
    pub source_path: Option<String>,
}

impl From<InventoryItem> for Component {
    fn from(item: InventoryItem) -> Self {
        Self {
            name: item.name,
            version: item.version,
            adapter: item.adapter,
            ecosystem: item.ecosystem,
            source_path: item.source_path,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomReport {
    pub schema_version: String,
    pub workspace_root: String,
    pub format: String,
    pub components: Vec<Component>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockLedgerReport {
    pub schema_version: String,
    pub workspace_root: String,
    pub ledger_path: String,
    pub fingerprint: String,
    pub adapters: Vec<crate::systems::ledger::AdapterLedgerEntry>,
}

impl LockLedgerReport {
    pub fn from_ledger(root: &Path, ledger: &LockLedger, ledger_rel: &str) -> Self {
        Self {
            schema_version: SCHEMA_VERSION.into(),
            workspace_root: root.display().to_string(),
            ledger_path: ledger_rel.into(),
            fingerprint: crate::systems::ledger::ledger_fingerprint(ledger),
            adapters: ledger.adapters.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiStageReport {
    pub name: String,
    pub exit_code: i32,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiReport {
    pub schema_version: String,
    pub workspace_root: String,
    pub passed: bool,
    pub stages: Vec<CiStageReport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplyReport {
    pub schema_version: String,
    pub target: String,
    pub fingerprint: String,
    pub applied: bool,
    pub exit_code: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_constant() {
        assert_eq!(SCHEMA_VERSION, "v1");
    }
}
