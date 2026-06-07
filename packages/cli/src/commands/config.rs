use crate::cli::GlobalArgs;
use crate::config;
use crate::config::validate;
use crate::output::{self, ConfigReport};
use miette::Result;
use std::path::Path;

pub fn validate_cmd(root: &Path, global: &GlobalArgs) -> Result<i32> {
    let config = config::load(root)?;
    validate::validate(&config)?;
    let warnings = validate::validate_against_repo(&config, root);

    let report = ConfigReport {
        schema_version: output::SCHEMA_VERSION.into(),
        valid: warnings.is_empty(),
        warnings: warnings.clone(),
        config: None,
    };

    if global.json {
        output::emit(&report);
    } else if !global.quiet {
        if warnings.is_empty() {
            eprintln!("\x1b[32m✓\x1b[0m luna.toml is valid");
        } else {
            eprintln!("\x1b[33m⚠\x1b[0m luna.toml valid with warnings:");
            for w in &warnings {
                eprintln!("  - {w}");
            }
        }
    }

    Ok(if warnings.is_empty() { 0 } else { 1 })
}

pub fn print_cmd(root: &Path, global: &GlobalArgs) -> Result<i32> {
    let config = config::load(root)?;
    validate::validate(&config)?;
    let warnings = validate::validate_against_repo(&config, root);
    let config_json = serde_json::to_value(&config).ok();

    let report = ConfigReport {
        schema_version: output::SCHEMA_VERSION.into(),
        valid: warnings.is_empty(),
        warnings,
        config: config_json,
    };

    if global.json {
        output::emit(&report);
    } else {
        let text = toml::to_string_pretty(&config).unwrap_or_default();
        println!("{text}");
    }
    Ok(0)
}
