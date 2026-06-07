pub mod compat;
pub mod schema;
pub mod validate;

use miette::{miette, Result};
use std::path::{Path, PathBuf};

pub use schema::LunaConfig;

/// Find `luna.toml` by walking up from `start`.
pub fn find_config(start: &Path) -> Result<PathBuf> {
    let mut dir: &Path = start;
    loop {
        let candidate = dir.join("luna.toml");
        if candidate.is_file() {
            return Ok(candidate);
        }
        match dir.parent() {
            Some(parent) => dir = parent,
            None => {
                return Err(miette!(
                    "no luna.toml found in any parent of {}",
                    start.display()
                ))
            }
        }
    }
}

/// Load `luna.toml` from disk. Use `migrate::load_for_bootstrap` before first migrate.
pub fn load(root: &Path) -> Result<LunaConfig> {
    let path = root.join("luna.toml");
    if !path.is_file() {
        return Err(miette!("luna.toml not found at {}", path.display()));
    }
    let text = std::fs::read_to_string(&path)
        .map_err(|e| miette!("failed to read {}: {e}", path.display()))?;
    let config: LunaConfig =
        toml::from_str(&text).map_err(|e| miette!("invalid luna.toml: {e}"))?;
    validate::validate(&config)?;
    Ok(config)
}

/// Require `luna.toml`; does not fall back to legacy import.
pub fn load_required(root: &Path) -> Result<LunaConfig> {
    if !root.join("luna.toml").is_file() {
        return Err(miette!(
            "luna.toml not found — run `luna migrate` to generate it from legacy repo files"
        ));
    }
    load(root)
}

/// Deprecated: use `load_required` or `migrate::load_for_bootstrap`.
pub fn load_or_compat(root: &Path) -> Result<LunaConfig> {
    load_required(root)
}
