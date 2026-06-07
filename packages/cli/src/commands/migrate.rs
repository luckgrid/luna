use crate::cli::GlobalArgs;
use crate::config::{self, compat, LunaConfig};
use crate::output;
use miette::{miette, Result};
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub enum MigrateMode {
    Migrate,
    Init,
}

pub fn run_migrate(
    root: &Path,
    global: &GlobalArgs,
    force: bool,
    mode: MigrateMode,
) -> Result<i32> {
    let path = root.join("luna.toml");
    if path.is_file() && !force {
        return Err(miette!(
            "luna.toml already exists at {} — pass --force to overwrite",
            path.display()
        ));
    }

    let luna_config = compat::from_legacy(root)?;
    if matches!(mode, MigrateMode::Init) && !root.join("pixi.toml").is_file() {
        write_pixi_scaffold(root)?;
    }

    let doc =
        toml::to_string_pretty(&luna_config).map_err(|e| miette!("serialize luna.toml: {e}"))?;
    std::fs::write(&path, doc).map_err(|e| miette!("write {}: {e}", path.display()))?;

    if global.json {
        output::emit(&output::ConfigReport {
            schema_version: output::SCHEMA_VERSION.into(),
            valid: true,
            warnings: Vec::new(),
            config: serde_json::to_value(&luna_config).ok(),
        });
    } else if !global.quiet {
        eprintln!("\x1b[32m✓\x1b[0m Wrote {}", path.display());
    }
    Ok(0)
}

fn write_pixi_scaffold(root: &Path) -> Result<()> {
    let pixi = root.join("pixi.toml");
    if pixi.is_file() {
        return Ok(());
    }
    let content = r#"[workspace]
name = "luna"
channels = ["conda-forge"]
platforms = ["osx-arm64", "osx-64", "linux-64", "win-64"]

[dependencies]
just = "*"
watchexec = "*"
"#;
    std::fs::write(&pixi, content).map_err(|e| miette!("write pixi.toml: {e}"))?;
    Ok(())
}

/// Load config for commands that may run before luna.toml exists.
pub fn load_for_bootstrap(root: &Path) -> Result<LunaConfig> {
    if root.join("luna.toml").is_file() {
        config::load(root)
    } else {
        compat::from_legacy(root)
    }
}
