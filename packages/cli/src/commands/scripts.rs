use crate::cli::GlobalArgs;
use crate::commands::moon;
use crate::runner;
use miette::Result;
use std::path::Path;

/// Bootstrap: proto install, bun install, cli build, web setup, api build.
pub fn install(root: &Path, global: &GlobalArgs) -> Result<i32> {
    runner::ensure_installed("proto", root)?;
    runner::ensure_installed("bun", root)?;

    run_step("proto", &["install".to_string()], root, global)?;
    run_step(
        "bun",
        &["install".to_string(), "--ignore-scripts".to_string()],
        root,
        global,
    )?;
    run_step_moon(&["run", "cli:build"], root, global)?;
    run_step_moon(&["run", "web:setup"], root, global)?;
    run_step_moon(&["run", "api:build"], root, global)?;
    Ok(0)
}

/// Clean: moon per-project clean, moon cache clean, git clean.
/// Matches old root package.json script: `moon run :clean && moon clean --all && git clean -fdx -- .cache node_modules`
pub fn clean(root: &Path, global: &GlobalArgs) -> Result<i32> {
    run_step_moon(&["run", ":clean"], root, global)?;
    run_step_moon(&["clean", "--all"], root, global)?;
    runner::run(
        "git",
        &[
            "clean".to_string(),
            "-fdx".to_string(),
            "--".to_string(),
            ".cache".to_string(),
            ".moon/cache".to_string(),
            "node_modules".to_string(),
            "target".to_string(),
        ],
        root,
        global.quiet,
    )
}

/// Lint all stacks: TS (oxlint) + Python (moon api:lint) + Rust (cargo clippy).
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

    // Python (moon handles uv sync dep)
    let api_task = if fix { "api:lint-fix" } else { "api:lint" };
    let code = moon::run_moon(root, &["run", api_task], global)?;
    if code != 0 {
        return Ok(code);
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

/// Format all stacks: TS (oxfmt) + Python (moon api:format) + Rust (cargo fmt).
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

    // Python (moon handles uv sync dep)
    let api_task = if check {
        "api:format-check"
    } else {
        "api:format"
    };
    let code = moon::run_moon(root, &["run", api_task], global)?;
    if code != 0 {
        return Ok(code);
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
    moon::run_moon(root, &["run", "web:typecheck"], global)
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

fn run_step(program: &str, args: &[String], cwd: &Path, global: &GlobalArgs) -> Result<()> {
    let code = runner::run(program, args, cwd, global.quiet)?;
    if code != 0 {
        return Err(miette::miette!(
            "`{program} {}` failed with exit code {code}",
            args.join(" ")
        ));
    }
    Ok(())
}

fn run_step_moon(args: &[&str], cwd: &Path, global: &GlobalArgs) -> Result<()> {
    let code = moon::run_moon(cwd, args, global)?;
    if code != 0 {
        return Err(miette::miette!(
            "`moon {}` failed with exit code {code}",
            args.join(" ")
        ));
    }
    Ok(())
}
