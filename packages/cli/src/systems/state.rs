use crate::config::LunaConfig;
use std::path::{Path, PathBuf};

/// Resolve Luna-owned state directory from config (default `.luna`).
pub fn state_dir(root: &Path, config: &LunaConfig) -> PathBuf {
    root.join(&config.state.dir)
}

pub fn snapshots_dir(root: &Path, config: &LunaConfig) -> PathBuf {
    state_dir(root, config).join("snapshots")
}

pub fn cache_dir(root: &Path, config: &LunaConfig) -> PathBuf {
    state_dir(root, config).join("cache")
}

pub fn runs_dir(root: &Path, config: &LunaConfig) -> PathBuf {
    state_dir(root, config).join("runs")
}

pub fn telemetry_dir(root: &Path, config: &LunaConfig) -> PathBuf {
    state_dir(root, config).join("telemetry")
}

pub fn lock_ledger_path(root: &Path, config: &LunaConfig) -> PathBuf {
    state_dir(root, config).join("lock-ledger.json")
}

pub fn outdated_snapshot_path(root: &Path, config: &LunaConfig) -> PathBuf {
    snapshots_dir(root, config).join("outdated.snapshot.json")
}

/// Legacy relative path for backward compat in JSON reports.
pub fn outdated_snapshot_rel(config: &LunaConfig) -> String {
    format!("{}/snapshots/outdated.snapshot.json", config.state.dir)
}
