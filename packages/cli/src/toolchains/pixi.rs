use miette::Result;
use std::path::Path;

/// Run `pixi install` to sync the root environment.
pub fn pixi_install(root: &Path, locked: bool) -> Result<i32> {
    let mut args = vec!["install".to_string()];
    if locked {
        args.push("--locked".to_string());
    }
    crate::systems::runner::run("pixi", &args, root, false)
}

/// Run `pixi exec` for ephemeral tool execution.
pub fn pixi_exec(root: &Path, tool: &str, tool_args: &[String]) -> Result<i32> {
    let mut args = vec!["exec".to_string(), tool.to_string()];
    args.extend(tool_args.iter().cloned());
    crate::systems::runner::run("pixi", &args, root, false)
}

/// Check whether `pixi` resolves on the current PATH without Luna PATH enrichment.
///
/// Must not call `runner::ensure_installed` — that path enriches PATH via `pixi_available`.
fn pixi_on_path() -> bool {
    std::process::Command::new("pixi")
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Whether the workspace uses Pixi (`pixi.toml` present and `pixi` on PATH).
pub fn pixi_available(root: &Path) -> bool {
    root.join("pixi.toml").is_file() && pixi_on_path()
}

const PIXI_GIT: &str = "https://github.com/prefix-dev/pixi.git";

/// Install Pixi via proto-pinned cargo when missing and auto-install is enabled.
pub fn ensure_pixi(
    root: &Path,
    config: &crate::config::LunaConfig,
    quiet: bool,
) -> miette::Result<()> {
    if !config.adapters.pixi.enabled {
        return Ok(());
    }
    if !root.join(&config.adapters.pixi.manifest).is_file() {
        return Ok(());
    }
    if !config.bootstrap.auto_install_pixi {
        return Ok(());
    }
    if pixi_available(root) {
        return Ok(());
    }
    crate::systems::runner::ensure_installed("proto", root)?;
    crate::systems::runner::run("proto", &["install".to_string()], root, quiet)?;
    crate::systems::runner::ensure_installed("cargo", root)?;
    let args = vec![
        "install".to_string(),
        "--locked".to_string(),
        "--git".to_string(),
        PIXI_GIT.to_string(),
        "pixi".to_string(),
    ];
    let code = crate::systems::runner::run("cargo", &args, root, quiet)?;
    if code != 0 {
        return Err(miette::miette!(
            "failed to install pixi via cargo (exit {code}) — install manually: https://pixi.prefix.dev/latest/installation/"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn pixi_manifest_missing_in_temp() {
        let tmp = std::env::temp_dir().join(format!("luna-test-{}", std::process::id()));
        let _ = std::fs::create_dir_all(&tmp);
        assert!(!tmp.join("pixi.toml").is_file());
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
