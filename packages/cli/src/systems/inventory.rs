use serde::{Deserialize, Serialize};

/// A resolved dependency/component from an adapter inventory export.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InventoryItem {
    pub adapter: String,
    pub name: String,
    pub version: String,
    pub source_path: Option<String>,
    pub ecosystem: String,
}

impl InventoryItem {
    pub fn new(adapter: &str, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            adapter: adapter.into(),
            name: name.into(),
            version: version.into(),
            source_path: None,
            ecosystem: adapter.into(),
        }
    }

    pub fn with_source(mut self, path: impl Into<String>) -> Self {
        self.source_path = Some(path.into());
        self
    }

    pub fn with_ecosystem(mut self, ecosystem: impl Into<String>) -> Self {
        self.ecosystem = ecosystem.into();
        self
    }
}

/// Parse simple `name = "version"` lines from TOML manifest dependency tables.
pub fn parse_toml_dep_table(text: &str, table_prefix: &str) -> Vec<(String, String)> {
    let mut in_table = false;
    let mut out = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_table = trimmed == table_prefix || trimmed.starts_with(&format!("{table_prefix}."));
            continue;
        }
        if !in_table || trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        if let Some((key, rest)) = trimmed.split_once('=') {
            let key = key.trim().trim_matches('"').to_string();
            let version = rest
                .trim()
                .trim_matches('"')
                .trim_start_matches('{')
                .split(',')
                .next()
                .unwrap_or("")
                .trim()
                .trim_matches('"')
                .to_string();
            if !key.is_empty() && !version.is_empty() {
                out.push((key, version));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_toml_deps() {
        let text = r#"
[dependencies]
serde = "1.0"
clap = { version = "4.5", features = ["derive"] }
"#;
        let deps = parse_toml_dep_table(text, "[dependencies]");
        assert!(deps.iter().any(|(k, _)| k == "serde"));
        assert!(deps.iter().any(|(k, _)| k == "clap"));
    }
}
