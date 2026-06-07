use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::systems::tasks;
use miette::Result;
use std::path::Path;

/// Full bootstrap: proto + CLI + workspace.
pub fn install(root: &Path, config: &LunaConfig, global: &GlobalArgs) -> Result<i32> {
    let code = tasks::bootstrap_cli(root, global)?;
    if code != 0 {
        return Ok(code);
    }
    tasks::bootstrap_workspace(root, config, global)
}

/// Install workspace deps only; skip CLI bootstrap (for CI).
pub fn install_workspace(root: &Path, config: &LunaConfig, global: &GlobalArgs) -> Result<i32> {
    tasks::bootstrap_workspace(root, config, global)
}

/// Full reset: apps/packages → moon clean → root gitignored outputs → `.moon/cache` last.
pub fn clean(root: &Path, global: &GlobalArgs) -> Result<i32> {
    tasks::run_step_moon(
        &["run", ":clean", "--query", "projectLayer!=configuration"],
        root,
        global,
    )?;

    tasks::run_step_moon(&["clean", "--all"], root, global)?;

    tasks::run_step_moon(&["run", "luna:clean"], root, global)?;

    tasks::remove_moon_cache(root)?;
    Ok(0)
}
