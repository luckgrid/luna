use crate::systems::model::{SnapshotPolicy, ToolchainSnapshot};
use crate::systems::workspace;
use miette::{IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

/// Current on-disk snapshot schema. Bump on any incompatible field change.
pub const SCHEMA_VERSION: u32 = 1;

/// Snapshot lives at `<root>/.cache/outdated.snapshot.json` (single file, overwritten).
pub const SNAPSHOT_REL: &str = ".cache/outdated.snapshot.json";

/// Default reuse window enforced by `luna update` only (8 hours).
pub const DEFAULT_TTL_SECS: u64 = 8 * 60 * 60;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestFingerprint {
    pub path: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutdatedSnapshot {
    pub schema_version: u32,
    pub repo_root: String,
    pub created_at: String,
    pub created_at_unix: u64,
    pub policy: SnapshotPolicy,
    pub manifests: Vec<ManifestFingerprint>,
    pub toolchains: Vec<ToolchainSnapshot>,
}

impl OutdatedSnapshot {
    pub fn new(root: &Path, policy: SnapshotPolicy, toolchains: Vec<ToolchainSnapshot>) -> Self {
        let now = now_unix();
        Self {
            schema_version: SCHEMA_VERSION,
            repo_root: root.display().to_string(),
            created_at: crate::systems::security::format_ymd_from_unix_days(now / 86_400),
            created_at_unix: now,
            policy,
            manifests: fingerprint_manifests(root),
            toolchains,
        }
    }

    /// Age in seconds since the snapshot was written.
    pub fn age_secs(&self) -> u64 {
        now_unix().saturating_sub(self.created_at_unix)
    }
}

fn snapshot_path(root: &Path) -> PathBuf {
    root.join(SNAPSHOT_REL)
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Atomically write the snapshot (temp file + rename) so readers never see partial JSON.
pub fn write(root: &Path, snapshot: &OutdatedSnapshot) -> Result<()> {
    let path = snapshot_path(root);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).into_diagnostic()?;
    }
    let json = serde_json::to_vec_pretty(snapshot).into_diagnostic()?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, &json).into_diagnostic()?;
    std::fs::rename(&tmp, &path).into_diagnostic()?;
    Ok(())
}

/// Why a snapshot cannot be reused (for diagnostics/banners).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvalidReason {
    Missing,
    Unreadable,
    SchemaMismatch,
    RepoMismatch,
    Expired,
    ManifestChanged,
    PolicyChanged,
}

/// Load and validate the snapshot for reuse by `luna update`.
///
/// Returns the snapshot only when it exists, parses, matches schema/repo/policy,
/// is within `ttl_secs`, and all manifest fingerprints still match.
pub fn read_valid(
    root: &Path,
    policy: &SnapshotPolicy,
    ttl_secs: u64,
) -> std::result::Result<OutdatedSnapshot, InvalidReason> {
    let path = snapshot_path(root);
    let raw = std::fs::read_to_string(&path).map_err(|err| {
        if err.kind() == std::io::ErrorKind::NotFound {
            InvalidReason::Missing
        } else {
            InvalidReason::Unreadable
        }
    })?;
    let snapshot: OutdatedSnapshot =
        serde_json::from_str(&raw).map_err(|_| InvalidReason::SchemaMismatch)?;

    if snapshot.schema_version != SCHEMA_VERSION {
        return Err(InvalidReason::SchemaMismatch);
    }
    if snapshot.repo_root != root.display().to_string() {
        return Err(InvalidReason::RepoMismatch);
    }
    if snapshot.age_secs() > ttl_secs {
        return Err(InvalidReason::Expired);
    }
    if &snapshot.policy != policy {
        return Err(InvalidReason::PolicyChanged);
    }
    if !manifests_match(root, &snapshot.manifests) {
        return Err(InvalidReason::ManifestChanged);
    }
    Ok(snapshot)
}

/// Candidate manifest + lockfile paths whose edits should invalidate a snapshot.
fn manifest_paths(root: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = [
        ".prototools",
        "package.json",
        "pyproject.toml",
        "go.work",
        "Cargo.toml",
        "bun.lock",
        "uv.lock",
        "Cargo.lock",
    ]
    .iter()
    .map(|p| root.join(p))
    .collect();

    let mut go_dirs = workspace::go_work_use_paths(root);
    go_dirs.extend(workspace::project_roots(root, "go", "go.mod"));
    for dir in go_dirs {
        let go_mod = dir.join("go.mod");
        if !paths.contains(&go_mod) {
            paths.push(go_mod);
        }
    }
    paths
}

fn fingerprint_manifests(root: &Path) -> Vec<ManifestFingerprint> {
    let mut out = Vec::new();
    for path in manifest_paths(root) {
        if let Some(sha) = sha256_file(&path) {
            out.push(ManifestFingerprint {
                path: rel(root, &path),
                sha256: sha,
            });
        }
    }
    out
}

fn manifests_match(root: &Path, recorded: &[ManifestFingerprint]) -> bool {
    let current = fingerprint_manifests(root);
    if current.len() != recorded.len() {
        return false;
    }
    for rec in recorded {
        match current.iter().find(|c| c.path == rec.path) {
            Some(c) if c.sha256 == rec.sha256 => {}
            _ => return false,
        }
    }
    true
}

fn sha256_file(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    Some(format!("{:x}", hasher.finalize()))
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::systems::model::ToolchainKind;

    fn policy() -> SnapshotPolicy {
        SnapshotPolicy {
            major: false,
            min_release_age_days: 14,
            uv_exclude_newer: Some("2026-05-01".into()),
            firewall: false,
        }
    }

    fn write_temp_repo() -> tempdir_like::TempRepo {
        tempdir_like::TempRepo::new()
    }

    #[test]
    fn write_then_read_valid_roundtrip() {
        let repo = write_temp_repo();
        let snap = OutdatedSnapshot::new(repo.root(), policy(), Vec::new());
        write(repo.root(), &snap).unwrap();
        let loaded = read_valid(repo.root(), &policy(), DEFAULT_TTL_SECS).unwrap();
        assert_eq!(loaded.schema_version, SCHEMA_VERSION);
        assert_eq!(loaded.repo_root, repo.root().display().to_string());
    }

    #[test]
    fn missing_snapshot_is_invalid() {
        let repo = write_temp_repo();
        let err = read_valid(repo.root(), &policy(), DEFAULT_TTL_SECS).unwrap_err();
        assert_eq!(err, InvalidReason::Missing);
    }

    #[test]
    fn expired_snapshot_is_invalid() {
        let repo = write_temp_repo();
        let mut snap = OutdatedSnapshot::new(repo.root(), policy(), Vec::new());
        snap.created_at_unix -= DEFAULT_TTL_SECS + 60;
        write(repo.root(), &snap).unwrap();
        let err = read_valid(repo.root(), &policy(), DEFAULT_TTL_SECS).unwrap_err();
        assert_eq!(err, InvalidReason::Expired);
    }

    #[test]
    fn policy_change_invalidates() {
        let repo = write_temp_repo();
        let snap = OutdatedSnapshot::new(repo.root(), policy(), Vec::new());
        write(repo.root(), &snap).unwrap();
        let mut other = policy();
        other.major = true;
        let err = read_valid(repo.root(), &other, DEFAULT_TTL_SECS).unwrap_err();
        assert_eq!(err, InvalidReason::PolicyChanged);
    }

    #[test]
    fn manifest_edit_invalidates() {
        let repo = write_temp_repo();
        let snap = OutdatedSnapshot::new(repo.root(), policy(), Vec::new());
        write(repo.root(), &snap).unwrap();
        std::fs::write(repo.root().join("package.json"), "{\"changed\":true}").unwrap();
        let err = read_valid(repo.root(), &policy(), DEFAULT_TTL_SECS).unwrap_err();
        assert_eq!(err, InvalidReason::ManifestChanged);
    }

    #[test]
    fn has_updates_requires_outdated_state_and_rows() {
        let snap = ToolchainSnapshot {
            kind: ToolchainKind::Bun,
            label: "bun".into(),
            state: crate::systems::model::ToolchainState::UpToDate,
            elapsed_ms: 0,
            started_at: None,
            finished_at: None,
            rows: Vec::new(),
            diagnostics: Vec::new(),
        };
        assert!(!snap.has_updates());
    }

    /// Minimal temp repo helper (avoids adding a dev-dependency).
    mod tempdir_like {
        use std::path::{Path, PathBuf};

        pub struct TempRepo {
            root: PathBuf,
        }

        impl TempRepo {
            pub fn new() -> Self {
                let mut root = std::env::temp_dir();
                let unique = format!(
                    "luna-snap-test-{}-{}",
                    std::process::id(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_nanos()
                );
                root.push(unique);
                std::fs::create_dir_all(&root).unwrap();
                std::fs::write(root.join("package.json"), "{}").unwrap();
                Self { root }
            }

            pub fn root(&self) -> &Path {
                &self.root
            }
        }

        impl Drop for TempRepo {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.root);
            }
        }
    }
}
