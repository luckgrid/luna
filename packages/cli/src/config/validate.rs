use crate::config::schema::LunaConfig;
use miette::{miette, Result};
use std::path::Path;

/// Validate a loaded `LunaConfig` for internal consistency.
pub fn validate(config: &LunaConfig) -> Result<()> {
    if config.schema == 0 {
        return Err(miette!("luna.toml: schema version must be >= 1"));
    }
    if config.workspace.name.is_empty() {
        return Err(miette!("luna.toml: workspace.name must not be empty"));
    }
    if config.state.dir.is_empty() {
        return Err(miette!("luna.toml: state.dir must not be empty"));
    }
    if config.policy.min_release_age_days == 0 {
        return Err(miette!(
            "luna.toml: policy.min_release_age_days must be >= 1"
        ));
    }
    Ok(())
}

/// Validate that a `LunaConfig` is consistent with the on-disk repository.
pub fn validate_against_repo(config: &LunaConfig, root: &Path) -> Vec<String> {
    let mut warnings = Vec::new();

    if !root.join(&config.adapters.bun.manifest).is_file() {
        warnings.push(format!(
            "adapters.bun.manifest not found: {}",
            config.adapters.bun.manifest
        ));
    }
    if !root.join(&config.adapters.cargo.manifest).is_file() {
        warnings.push(format!(
            "adapters.cargo.manifest not found: {}",
            config.adapters.cargo.manifest
        ));
    }
    if !root.join(&config.adapters.go.workspace).is_file() {
        warnings.push(format!(
            "adapters.go.workspace not found: {}",
            config.adapters.go.workspace
        ));
    }
    if config.adapters.pixi.enabled && !root.join(&config.adapters.pixi.manifest).is_file() {
        warnings.push(format!(
            "adapters.pixi.manifest not found (pixi enabled): {}",
            config.adapters.pixi.manifest
        ));
    }
    if config.compat.moon.enabled && !root.join(&config.compat.moon.workspace).is_file() {
        warnings.push(format!(
            "compat.moon.workspace not found: {}",
            config.compat.moon.workspace
        ));
    }

    warnings
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::schema::*;

    fn valid_config() -> LunaConfig {
        LunaConfig {
            schema: 1,
            ..Default::default()
        }
    }

    #[test]
    fn validate_accepts_valid_config() {
        assert!(validate(&valid_config()).is_ok());
    }

    #[test]
    fn validate_rejects_zero_schema() {
        let mut config = valid_config();
        config.schema = 0;
        assert!(validate(&config).is_err());
    }

    #[test]
    fn validate_rejects_empty_name() {
        let mut config = valid_config();
        config.workspace.name = String::new();
        assert!(validate(&config).is_err());
    }

    #[test]
    fn validate_rejects_zero_min_release_age() {
        let mut config = valid_config();
        config.policy.min_release_age_days = 0;
        assert!(validate(&config).is_err());
    }
}
