use crate::deps::model::{DependencyRow, ToolchainKind};
use crate::deps::probes::ProbeOutcome;
use crate::runner;
use std::path::Path;

/// Probe `.prototools` pins via `proto outdated --json`.
pub fn probe(root: &Path) -> ProbeOutcome {
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
