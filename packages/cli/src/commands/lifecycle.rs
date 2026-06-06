use crate::cli::GlobalArgs;
use crate::systems::tasks;
use miette::Result;
use std::path::Path;

/// Full bootstrap: proto + CLI + workspace.
pub fn install(root: &Path, global: &GlobalArgs) -> Result<i32> {
    let code = tasks::bootstrap_cli(root, global)?;
    if code != 0 {
        return Ok(code);
    }
    tasks::bootstrap_workspace(root, global)
}

/// Install workspace deps only; skip CLI bootstrap (for CI).
pub fn install_workspace(root: &Path, global: &GlobalArgs) -> Result<i32> {
    tasks::bootstrap_workspace(root, global)
}

/// Full reset: apps/packages → moon clean → root gitignored outputs → `.moon/cache` last.
pub fn clean(root: &Path, global: &GlobalArgs) -> Result<i32> {
    // 1. Per-project artifacts (venv, dist, cargo target via cli:clean, etc.)
    tasks::run_step_moon(
        &["run", ":clean", "--query", "projectLayer!=configuration"],
        root,
        global,
    )?;

    // 2. Prune moon task cache entries (still uses `.moon/cache` on disk)
    tasks::run_step_moon(&["clean", "--all"], root, global)?;

    // 3. Root install artifacts and caches (not `.moon/cache` — that is moon-owned)
    tasks::run_step_moon(&["run", "luna:clean"], root, global)?;

    // 4. Drop `.moon/cache` last; moon recreates it on every `moon` invocation until this step
    tasks::remove_moon_cache(root)?;
    Ok(0)
}
