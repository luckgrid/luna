use crate::adapters::{registry, AdapterKind, SyncOpts};
use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::output;
use miette::{miette, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvListReport {
    pub schema_version: String,
    pub environments: Vec<EnvEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvEntry {
    pub name: String,
    pub features: Vec<String>,
}

pub fn list(root: &Path, config: &LunaConfig, global: &GlobalArgs) -> Result<i32> {
    let environments = parse_pixi_environments(root, config);

    let report = EnvListReport {
        schema_version: output::SCHEMA_VERSION.into(),
        environments,
    };

    if global.json {
        output::emit(&report);
    } else if !global.quiet {
        for env in &report.environments {
            let feats = if env.features.is_empty() {
                "(default)".to_string()
            } else {
                env.features.join(", ")
            };
            eprintln!("  {} — {}", env.name, feats);
        }
    }
    Ok(0)
}

pub fn sync(
    root: &Path,
    config: &LunaConfig,
    global: &GlobalArgs,
    environment: Option<&str>,
) -> Result<i32> {
    if !config.adapters.pixi.enabled || !root.join(&config.adapters.pixi.manifest).is_file() {
        if global.json {
            output::emit(&serde_json::json!({
                "schemaVersion": output::SCHEMA_VERSION,
                "skipped": true,
                "reason": "pixi not configured"
            }));
        }
        return Ok(0);
    }

    let locked = global.locked || global.frozen || config.policy.frozen_ci;
    let adapter = registry::get(AdapterKind::Pixi);
    let opts = SyncOpts {
        locked,
        quiet: global.quiet,
    };

    if let Some(env) = environment {
        let mut args = vec!["install".to_string()];
        args.push("-e".into());
        args.push(env.into());
        if locked {
            args.push("--locked".into());
        }
        let code = crate::systems::runner::run("pixi", &args, root, global.quiet)?;
        return Ok(code);
    }

    let outcome = adapter.sync(root, config, opts)?;
    Ok(outcome.exit_code)
}

pub fn exec(
    root: &Path,
    config: &LunaConfig,
    global: &GlobalArgs,
    environment: Option<&str>,
    command: &[String],
) -> Result<i32> {
    if command.is_empty() {
        return Err(miette!("env exec requires a command after `--`"));
    }

    if config.adapters.pixi.enabled && root.join(&config.adapters.pixi.manifest).is_file() {
        let mut args = vec!["run".to_string()];
        if let Some(env) = environment {
            args.push("-e".into());
            args.push(env.into());
        }
        args.push("--".into());
        args.extend(command.iter().cloned());
        return crate::systems::runner::run("pixi", &args, root, global.quiet);
    }

    let program = &command[0];
    let rest = &command[1..];
    crate::systems::runner::run(program, rest, root, global.quiet)
}

fn parse_pixi_environments(root: &Path, config: &LunaConfig) -> Vec<EnvEntry> {
    let path = root.join(&config.adapters.pixi.manifest);
    let Ok(text) = std::fs::read_to_string(&path) else {
        return vec![EnvEntry {
            name: "default".into(),
            features: Vec::new(),
        }];
    };

    let Ok(doc) = toml::from_str::<toml::Value>(&text) else {
        return vec![EnvEntry {
            name: "default".into(),
            features: Vec::new(),
        }];
    };

    let Some(environments) = doc.get("environments").and_then(|e| e.as_table()) else {
        return vec![EnvEntry {
            name: "default".into(),
            features: Vec::new(),
        }];
    };

    environments
        .iter()
        .map(|(name, value)| {
            let features = match value {
                toml::Value::Array(arr) => arr
                    .iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect(),
                _ => Vec::new(),
            };
            EnvEntry {
                name: name.clone(),
                features,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_environments_from_pixi_toml() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = LunaConfig::default();
        let envs = parse_pixi_environments(&root, &config);
        assert!(envs.iter().any(|e| e.name == "default"));
        assert!(envs.iter().any(|e| e.name == "js"));
    }
}
