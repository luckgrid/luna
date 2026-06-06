use crate::systems::model::{DependencyRow, ToolchainKind};
use crate::systems::runner;
use crate::toolchains::{run_blocking, ProbeOutcome, ToolchainAdapter, UpdateOpts, UpdateOutcome};
use async_trait::async_trait;
use std::path::Path;

pub struct ProtoAdapter;

#[async_trait]
impl ToolchainAdapter for ProtoAdapter {
    fn kind(&self) -> ToolchainKind {
        ToolchainKind::Proto
    }

    async fn probe(&self, root: &Path) -> ProbeOutcome {
        let root = root.to_path_buf();
        run_blocking(move || probe(&root)).await
    }

    async fn update(&self, root: &Path, opts: UpdateOpts) -> UpdateOutcome {
        let root = root.to_path_buf();
        run_blocking(move || update(&root, opts)).await
    }
}

/// Probe `.prototools` pins via `proto outdated --json`.
fn probe(root: &Path) -> ProbeOutcome {
    if runner::ensure_installed("proto", root).is_err() {
        return ProbeOutcome::failed("proto is not installed");
    }
    let out = match runner::capture(
        "proto",
        &["outdated".to_string(), "--json".to_string()],
        root,
    ) {
        Ok(o) => o,
        Err(err) => return ProbeOutcome::failed(format!("proto outdated failed: {err}")),
    };
    match parse_proto_outdated(&out.stdout) {
        Some(rows) => ProbeOutcome::outdated(rows),
        None => ProbeOutcome::failed("could not parse `proto outdated --json`"),
    }
}

/// Update pinned tools via `proto outdated --update` then `proto install`.
fn update(root: &Path, opts: UpdateOpts) -> UpdateOutcome {
    if runner::ensure_installed("proto", root).is_err() {
        return UpdateOutcome::Failed("proto is not installed".into());
    }
    let mut args = vec!["outdated".to_string(), "--update".to_string()];
    if opts.major {
        args.push("--latest".to_string());
    }
    args.push("-y".to_string());
    if let Ok(out) = runner::capture("proto", &args, root) {
        if out.code != 0 {
            return UpdateOutcome::Failed(format!("{}{}", out.stdout, out.stderr));
        }
    }
    match runner::capture("proto", &["install".to_string()], root) {
        Ok(out) if out.code == 0 => UpdateOutcome::Done,
        Ok(out) => UpdateOutcome::Failed(format!("{}{}", out.stdout, out.stderr)),
        Err(err) => UpdateOutcome::Failed(err.to_string()),
    }
}

/// Parse the `proto outdated --json` object into outdated rows.
pub fn parse_proto_outdated(json: &str) -> Option<Vec<DependencyRow>> {
    let value: serde_json::Value = serde_json::from_str(json.trim()).ok()?;
    let map = value.as_object()?;
    let mut rows = Vec::new();
    for (tool, entry) in map {
        let is_outdated = entry
            .get("is_outdated")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if !is_outdated {
            continue;
        }
        let current = field(entry, "current").unwrap_or_else(|| "—".to_string());
        let newest = field(entry, "newest");
        let latest = field(entry, "latest");
        rows.push(DependencyRow::outdated(
            ToolchainKind::Proto,
            tool.clone(),
            current,
            newest,
            latest,
        ));
    }
    Some(rows)
}

fn field(entry: &serde_json::Value, key: &str) -> Option<String> {
    entry
        .get(key)
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_outdated_pins_only() {
        let json = r#"{
            "node": {"current": "20.0.0", "newest": "20.5.0", "latest": "21.1.0", "is_outdated": true},
            "bun": {"current": "1.1.0", "newest": "1.1.0", "latest": "1.1.0", "is_outdated": false}
        }"#;
        let rows = parse_proto_outdated(json).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].dependency, "node");
        assert_eq!(rows[0].current, "20.0.0");
        assert_eq!(rows[0].newest.as_deref(), Some("20.5.0"));
        assert_eq!(rows[0].latest.as_deref(), Some("21.1.0"));
        assert!(rows[0].latest_one_major_ahead);
    }

    #[test]
    fn empty_when_all_current() {
        let json = r#"{"node": {"current": "20.0.0", "newest": "20.0.0", "latest": "20.0.0", "is_outdated": false}}"#;
        assert!(parse_proto_outdated(json).unwrap().is_empty());
    }
}
