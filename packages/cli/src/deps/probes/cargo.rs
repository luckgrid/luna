use crate::deps::model::{DependencyRow, ToolchainKind};
use crate::deps::probes::ProbeOutcome;
use crate::runner;
use std::path::Path;

/// Probe the Cargo workspace via `cargo outdated --format json`.
///
/// Per product decision (OQ-1), the `Latest` column is omitted for Cargo in v1;
/// only the in-range `Compat` target populates `Newest`.
pub fn probe(root: &Path) -> ProbeOutcome {
    if !root.join("Cargo.toml").is_file() || !root.join("Cargo.lock").is_file() {
        return ProbeOutcome::up_to_date();
    }
    if runner::ensure_installed("cargo", root).is_err() {
        return ProbeOutcome::failed("cargo is not installed");
    }
    if !ensure_cargo_outdated(root) {
        return ProbeOutcome::failed(
            "install `cargo-outdated` (`cargo install cargo-outdated`) to check Rust deps",
        );
    }

    let out = match runner::capture(
        "cargo",
        &[
            "outdated".to_string(),
            "--format".to_string(),
            "json".to_string(),
        ],
        root,
    ) {
        Ok(o) => o,
        Err(err) => return ProbeOutcome::failed(format!("cargo outdated failed: {err}")),
    };
    // `cargo outdated` exits 1 when upgrades exist; both 0 and 1 carry valid JSON.
    if out.code != 0 && out.code != 1 {
        return ProbeOutcome::failed(format!(
            "`cargo outdated` failed (exit {}): {}",
            out.code,
            out.stderr.trim()
        ));
    }
    match parse_cargo_outdated(&out.stdout) {
        Some(rows) => ProbeOutcome::outdated(rows)
            .with_diagnostic("Latest column omitted for Cargo in v1 (OQ-1)"),
        None => ProbeOutcome::failed("could not parse `cargo outdated --format json`"),
    }
}

fn ensure_cargo_outdated(root: &Path) -> bool {
    let has = runner::capture(
        "cargo",
        &["outdated".to_string(), "--version".to_string()],
        root,
    )
    .map(|o| o.code == 0)
    .unwrap_or(false);
    if has {
        return true;
    }
    runner::run(
        "cargo",
        &[
            "install".to_string(),
            "cargo-outdated".to_string(),
            "--locked".to_string(),
        ],
        root,
        true,
    )
    .map(|code| code == 0)
    .unwrap_or(false)
}

/// Parse `cargo outdated --format json` dependencies into outdated rows.
pub fn parse_cargo_outdated(json: &str) -> Option<Vec<DependencyRow>> {
    let value: serde_json::Value = serde_json::from_str(json.trim()).ok()?;
    let deps = value.get("dependencies")?.as_array()?;
    let mut rows = Vec::new();
    for dep in deps {
        let name = dep.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let current = clean(dep.get("project").and_then(|v| v.as_str()));
        let compat = clean(dep.get("compat").and_then(|v| v.as_str()));
        if name.is_empty() {
            continue;
        }
        let Some(current) = current else { continue };
        // Newest = latest semver-compatible version, when it differs from current.
        let newest = compat.filter(|c| c != &current);
        if newest.is_none() {
            continue;
        }
        rows.push(DependencyRow::outdated(
            ToolchainKind::Rust,
            name.to_string(),
            current,
            newest,
            None,
        ));
    }
    Some(rows)
}

/// Normalize cargo-outdated cell values (`---`, `Removed`, empty → None).
fn clean(value: Option<&str>) -> Option<String> {
    let v = value?.trim();
    if v.is_empty() || v == "---" || v == "Removed" {
        None
    } else {
        Some(v.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_compat_upgrade_as_newest() {
        let json = r#"{
            "crate_name": "luna",
            "dependencies": [
                {"name": "serde", "project": "1.0.1", "compat": "1.0.9", "latest": "1.0.9", "kind": "normal", "platform": null},
                {"name": "clap", "project": "4.5.0", "compat": "---", "latest": "5.0.0", "kind": "normal", "platform": null}
            ]
        }"#;
        let rows = parse_cargo_outdated(json).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].dependency, "serde");
        assert_eq!(rows[0].current, "1.0.1");
        assert_eq!(rows[0].newest.as_deref(), Some("1.0.9"));
        assert!(rows[0].latest.is_none());
    }
}
