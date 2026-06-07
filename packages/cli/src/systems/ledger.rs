use crate::adapters::{registry, AdapterKind, LockOpts};
use crate::config::LunaConfig;
use crate::systems::inventory::InventoryItem;
use crate::systems::snapshot::{self, ManifestFingerprint};
use crate::systems::state;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

pub const LEDGER_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterLedgerEntry {
    pub adapter: String,
    pub lock_ok: bool,
    pub message: Option<String>,
    pub items: Vec<InventoryItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockLedger {
    pub schema_version: u32,
    pub repo_root: String,
    pub created_at_unix: u64,
    pub manifests: Vec<ManifestFingerprint>,
    pub adapters: Vec<AdapterLedgerEntry>,
}

impl LockLedger {
    pub fn new(root: &Path, adapters: Vec<AdapterLedgerEntry>) -> Self {
        Self {
            schema_version: LEDGER_SCHEMA_VERSION,
            repo_root: root.display().to_string(),
            created_at_unix: now_unix(),
            manifests: snapshot::fingerprint_manifests_public(root),
            adapters,
        }
    }
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

pub fn write(root: &Path, config: &LunaConfig, ledger: &LockLedger) -> Result<()> {
    let path = state::lock_ledger_path(root, config);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).into_diagnostic()?;
    }
    let json = serde_json::to_vec_pretty(ledger).into_diagnostic()?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &json).into_diagnostic()?;
    std::fs::rename(&tmp, &path).into_diagnostic()?;
    Ok(())
}

pub fn read(root: &Path, config: &LunaConfig) -> Result<LockLedger> {
    let path = state::lock_ledger_path(root, config);
    let raw = std::fs::read_to_string(&path).into_diagnostic()?;
    serde_json::from_str(&raw).into_diagnostic()
}

/// Run adapter lock operations and build a fresh ledger.
pub fn reconcile(
    root: &Path,
    config: &LunaConfig,
    locked: bool,
    quiet: bool,
) -> Result<LockLedger> {
    let mut entries = Vec::new();
    for kind in [
        AdapterKind::Pixi,
        AdapterKind::Proto,
        AdapterKind::Bun,
        AdapterKind::Uv,
        AdapterKind::Cargo,
        AdapterKind::Go,
    ] {
        let adapter = registry::get(kind);
        if !adapter.detect(root, config) {
            continue;
        }
        let outcome = adapter.lock(root, config, LockOpts { locked, quiet })?;
        let items = adapter.export_inventory(root, config)?;
        entries.push(AdapterLedgerEntry {
            adapter: kind.label().into(),
            lock_ok: outcome.ok,
            message: outcome.message,
            items,
        });
    }
    let ledger = LockLedger::new(root, entries);
    write(root, config, &ledger)?;
    Ok(ledger)
}

pub fn ledger_fingerprint(ledger: &LockLedger) -> String {
    let mut hasher = Sha256::new();
    hasher.update(ledger.repo_root.as_bytes());
    for m in &ledger.manifests {
        hasher.update(m.path.as_bytes());
        hasher.update(m.sha256.as_bytes());
    }
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ledger_fingerprint_stable() {
        let ledger = LockLedger {
            schema_version: 1,
            repo_root: "/tmp/repo".into(),
            created_at_unix: 0,
            manifests: vec![ManifestFingerprint {
                path: "Cargo.toml".into(),
                sha256: "abc".into(),
            }],
            adapters: Vec::new(),
        };
        assert_eq!(ledger_fingerprint(&ledger), ledger_fingerprint(&ledger));
    }
}
