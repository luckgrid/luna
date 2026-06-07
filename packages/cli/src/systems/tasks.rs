use crate::adapters::{registry, AdapterKind, SyncOpts};
use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::systems::runner::{self, run_moon};
use crate::systems::{security, workspace};
use crate::ui::{self, LunaConsole};
use miette::{IntoDiagnostic, Result};
use std::path::Path;

/// Install pinned toolchains and build/install the `luna` CLI (moon tasks).
pub fn bootstrap_cli(root: &Path, global: &GlobalArgs) -> Result<i32> {
    runner::ensure_installed("proto", root)?;
    run_step("proto", &["install".to_string()], root, global)?;
    run_step_moon(&["run", "cli:build"], root, global)?;
    run_step_moon(&["run", "cli:install"], root, global)?;
    Ok(0)
}

/// Install workspace deps (Pixi first, then bun, uv, go, web) after CLI is available.
pub fn bootstrap_workspace(root: &Path, config: &LunaConfig, global: &GlobalArgs) -> Result<i32> {
    pixi_sync(root, config, global)?;

    let bun = registry::get(AdapterKind::Bun);
    let bun_out = bun.sync(
        root,
        config,
        SyncOpts {
            locked: global.locked || global.frozen,
            quiet: global.quiet,
        },
    )?;
    if bun_out.exit_code != 0 {
        return Ok(bun_out.exit_code);
    }

    if workspace::uv_workspace_root(root).is_some() {
        let uv = registry::get(AdapterKind::Uv);
        let uv_out = uv.sync(
            root,
            config,
            SyncOpts {
                locked: global.locked || global.frozen,
                quiet: global.quiet,
            },
        )?;
        if uv_out.exit_code != 0 {
            return Ok(uv_out.exit_code);
        }
    } else {
        run_step_moon(&["run", "api:build"], root, global)?;
    }

    if root.join("go.work").is_file() {
        let go = registry::get(AdapterKind::Go);
        let go_out = go.sync(
            root,
            config,
            SyncOpts {
                locked: global.locked || global.frozen,
                quiet: global.quiet,
            },
        )?;
        if go_out.exit_code != 0 {
            return Ok(go_out.exit_code);
        }
    }

    run_step_moon(&["run", "web:setup"], root, global)?;
    Ok(0)
}

fn pixi_sync(root: &Path, config: &LunaConfig, global: &GlobalArgs) -> Result<()> {
    if !config.adapters.pixi.enabled || !root.join(&config.adapters.pixi.manifest).is_file() {
        return Ok(());
    }
    crate::toolchains::pixi::ensure_pixi(root, config, global.quiet)?;
    let pixi = registry::get(AdapterKind::Pixi);
    let locked = global.locked || global.frozen || config.policy.frozen_ci;
    let outcome = pixi.sync(
        root,
        config,
        SyncOpts {
            locked,
            quiet: global.quiet,
        },
    )?;
    if outcome.exit_code != 0 {
        return Err(miette::miette!(
            "pixi install failed with exit code {}",
            outcome.exit_code
        ));
    }
    Ok(())
}

/// Post-update workspace sync with captured output behind a progress loader.
pub async fn sync_workspace_quiet(
    root: &Path,
    config: &LunaConfig,
    global: &GlobalArgs,
    console: &LunaConsole,
) -> Result<i32> {
    if global.quiet {
        return run_quiet_sync_capture(root, config, global);
    }

    let root_buf = root.to_path_buf();
    let config = config.clone();
    let quiet = global.quiet;
    let locked = global.locked || global.frozen;
    let firewall = security::resolve_firewall(root, global, quiet);

    let result = ui::run_with_loader(console, "Syncing workspace", move |reporter| {
        run_quiet_sync_steps(&root_buf, &config, quiet, locked, firewall, &reporter)
    })
    .await;

    match result {
        Ok(()) => Ok(0),
        Err(msg) => {
            let _ = ui::render_failure_notice(console, "Workspace sync", &msg);
            Ok(1)
        }
    }
}

fn run_quiet_sync_capture(root: &Path, config: &LunaConfig, global: &GlobalArgs) -> Result<i32> {
    let firewall = security::resolve_firewall(root, global, global.quiet);
    let reporter = starbase_console::ui::ProgressReporter::default();
    run_quiet_sync_steps(
        root,
        config,
        global.quiet,
        global.locked || global.frozen,
        firewall,
        &reporter,
    )
    .map_err(|e| miette::miette!("{e}"))?;
    Ok(0)
}

fn run_quiet_sync_steps(
    root: &Path,
    config: &LunaConfig,
    quiet: bool,
    locked: bool,
    firewall: bool,
    reporter: &starbase_console::ui::ProgressReporter,
) -> Result<(), String> {
    reporter.set_message("Syncing Pixi environment");
    pixi_sync(
        root,
        config,
        &GlobalArgs {
            verbose: 0,
            quiet,
            firewall,
            json: false,
            dry_run: false,
            locked,
            frozen: locked,
            cwd: None,
            backend: crate::cli::Backend::Auto,
            affected: false,
            no_cache: false,
            trace: false,
            mode: None,
        },
    )
    .map_err(|e| e.to_string())?;

    reporter.set_message("Installing JS dependencies");
    runner::ensure_installed("bun", root).map_err(|e| e.to_string())?;
    let bun_args = vec![
        "install".to_string(),
        "--ignore-scripts".to_string(),
        security::bun_min_release_age_arg(),
    ];
    let out = runner::capture("bun", &bun_args, root).map_err(|e| e.to_string())?;
    if out.code != 0 {
        return Err(format!("{}{}", out.stdout, out.stderr));
    }

    if workspace::uv_workspace_root(root).is_some() {
        reporter.set_message("Syncing Python environment");
        runner::ensure_installed("uv", root).map_err(|e| e.to_string())?;
        let (program, args) = security::wrap("uv", &["sync".to_string()], firewall);
        let out = runner::capture(&program, &args, root).map_err(|e| e.to_string())?;
        if out.code != 0 {
            return Err(format!("{}{}", out.stdout, out.stderr));
        }
    } else {
        reporter.set_message("Building Python API");
        let out = capture_moon(root, &["run", "api:build"], quiet).map_err(|e| e.to_string())?;
        if out.code != 0 {
            return Err(format!("{}{}", out.stdout, out.stderr));
        }
    }

    if root.join("go.work").is_file() {
        reporter.set_message("Syncing Go workspace");
        sync_go_toolchain_capture(root).map_err(|e| e.to_string())?;
        runner::ensure_installed("go", root).map_err(|e| e.to_string())?;
        let out = runner::capture("go", &["work".to_string(), "sync".to_string()], root)
            .map_err(|e| e.to_string())?;
        if out.code != 0 {
            return Err(format!("{}{}", out.stdout, out.stderr));
        }
    }

    reporter.set_message("Running web setup");
    let out = capture_moon(root, &["run", "web:setup"], quiet).map_err(|e| e.to_string())?;
    if out.code != 0 {
        return Err(format!("{}{}", out.stdout, out.stderr));
    }

    Ok(())
}

fn capture_moon(root: &Path, args: &[&str], quiet: bool) -> Result<runner::Output> {
    let mut full: Vec<String> = Vec::with_capacity(args.len() + 2);
    if quiet {
        full.push("-q".to_string());
    }
    full.extend(args.iter().map(|a| (*a).to_string()));
    runner::capture("moon", &full, root)
}

fn sync_go_toolchain_capture(root: &Path) -> Result<()> {
    let Some(version) = workspace::prototools_pin(root, "go") else {
        return Ok(());
    };
    if !root.join("go.work").is_file() {
        return Ok(());
    }

    runner::ensure_installed("proto", root)?;

    let edit_go = format!("-go={version}");
    let work_args = vec![
        "run".to_string(),
        "go".to_string(),
        "--".to_string(),
        "work".to_string(),
        "edit".to_string(),
        edit_go.clone(),
    ];
    let out = runner::capture("proto", &work_args, root)?;
    if out.code != 0 {
        return Err(miette::miette!(
            "`proto run go -- work edit -go={version}` failed: {}{}",
            out.stdout,
            out.stderr
        ));
    }

    for module in workspace::go_work_use_paths(root) {
        if !module.join("go.mod").is_file() {
            continue;
        }
        let mod_args = vec![
            "run".to_string(),
            "go".to_string(),
            "--".to_string(),
            "mod".to_string(),
            "edit".to_string(),
            edit_go.clone(),
        ];
        let out = runner::capture("proto", &mod_args, &module)?;
        if out.code != 0 {
            return Err(miette::miette!(
                "`proto run go -- mod edit -go={version}` failed: {}{}",
                out.stdout,
                out.stderr
            ));
        }
    }

    Ok(())
}

pub(crate) fn remove_moon_cache(root: &Path) -> Result<()> {
    let moon_cache = root.join(".moon").join("cache");
    if moon_cache.is_dir() {
        std::fs::remove_dir_all(&moon_cache).into_diagnostic()?;
    }
    Ok(())
}

pub(crate) fn run_step(
    program: &str,
    args: &[String],
    cwd: &Path,
    global: &GlobalArgs,
) -> Result<()> {
    let code = runner::run(program, args, cwd, global.quiet)?;
    if code != 0 {
        return Err(miette::miette!(
            "`{program} {}` failed with exit code {code}",
            args.join(" ")
        ));
    }
    Ok(())
}

pub(crate) fn run_step_moon(args: &[&str], cwd: &Path, global: &GlobalArgs) -> Result<()> {
    let code = run_moon(cwd, args, global)?;
    if code != 0 {
        return Err(miette::miette!(
            "`moon {}` failed with exit code {code}",
            args.join(" ")
        ));
    }
    Ok(())
}
