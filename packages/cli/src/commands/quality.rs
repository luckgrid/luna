use crate::cli::GlobalArgs;
use crate::systems::runner::{self, run_moon};
use crate::systems::workspace;
use miette::Result;
use std::path::Path;

/// Lint all stacks: TS (oxlint) + Python (ruff) + Rust (cargo clippy).
pub fn lint(root: &Path, fix: bool, global: &GlobalArgs) -> Result<i32> {
    // TS root
    runner::ensure_installed("oxlint", root)?;
    let oxlint_args = if fix {
        vec!["--fix".to_string(), ".".to_string()]
    } else {
        vec![".".to_string()]
    };
    let code = runner::run("oxlint", &oxlint_args, root, global.quiet)?;
    if code != 0 {
        return Ok(code);
    }

    // Python (root ruff config; .ruff_cache handles incrementality)
    if workspace::uv_workspace_root(root).is_some() {
        runner::ensure_installed("uv", root)?;
        let ruff_args = if fix {
            vec![
                "run".to_string(),
                "ruff".to_string(),
                "check".to_string(),
                "--fix".to_string(),
                ".".to_string(),
            ]
        } else {
            vec![
                "run".to_string(),
                "ruff".to_string(),
                "check".to_string(),
                ".".to_string(),
            ]
        };
        let code = runner::run("uv", &ruff_args, root, global.quiet)?;
        if code != 0 {
            return Ok(code);
        }
    }

    // Rust
    let clippy_args = if fix {
        vec![
            "clippy".to_string(),
            "--fix".to_string(),
            "--allow-dirty".to_string(),
        ]
    } else {
        vec![
            "clippy".to_string(),
            "--".to_string(),
            "-D".to_string(),
            "warnings".to_string(),
        ]
    };
    runner::run("cargo", &clippy_args, root, global.quiet)
}

/// Format all stacks: TS (oxfmt) + Python (ruff) + Rust (cargo fmt).
pub fn format(root: &Path, check: bool, global: &GlobalArgs) -> Result<i32> {
    // TS root
    runner::ensure_installed("oxfmt", root)?;
    let oxfmt_args = if check {
        vec!["--list-different".to_string(), ".".to_string()]
    } else {
        vec![".".to_string()]
    };
    let code = runner::run("oxfmt", &oxfmt_args, root, global.quiet)?;
    if code != 0 {
        return Ok(code);
    }

    // Python (root ruff config)
    if workspace::uv_workspace_root(root).is_some() {
        runner::ensure_installed("uv", root)?;
        let ruff_args = if check {
            vec![
                "run".to_string(),
                "ruff".to_string(),
                "format".to_string(),
                "--check".to_string(),
                ".".to_string(),
            ]
        } else {
            vec![
                "run".to_string(),
                "ruff".to_string(),
                "format".to_string(),
                ".".to_string(),
            ]
        };
        let code = runner::run("uv", &ruff_args, root, global.quiet)?;
        if code != 0 {
            return Ok(code);
        }
    }

    // Rust
    let fmt_args = if check {
        vec!["fmt".to_string(), "--check".to_string()]
    } else {
        vec!["fmt".to_string()]
    };
    runner::run("cargo", &fmt_args, root, global.quiet)
}

/// Typecheck all stacks: TS (tsc) + Go/Hugo (moon web:typecheck).
pub fn typecheck(root: &Path, global: &GlobalArgs) -> Result<i32> {
    // TS (project references cover app, ds, ui)
    runner::ensure_installed("tsc", root)?;
    let code = runner::run(
        "tsc",
        &["--build".to_string(), "--verbose".to_string()],
        root,
        global.quiet,
    )?;
    if code != 0 {
        return Ok(code);
    }

    // Go/Hugo (moon handles PATH + deps)
    run_moon(root, &["run", "web:typecheck"], global)
}

/// Check: lint + format:check + typecheck across all stacks (stop on first failure).
pub fn check(root: &Path, global: &GlobalArgs) -> Result<i32> {
    let code = lint(root, false, global)?;
    if code != 0 {
        return Ok(code);
    }
    let code = format(root, true, global)?;
    if code != 0 {
        return Ok(code);
    }
    typecheck(root, global)
}

/// Fix: lint:fix + format across all stacks (stop on first failure).
pub fn fix(root: &Path, global: &GlobalArgs) -> Result<i32> {
    let code = lint(root, true, global)?;
    if code != 0 {
        return Ok(code);
    }
    format(root, false, global)
}
