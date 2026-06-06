use crate::cli::GlobalArgs;
use miette::{miette, IntoDiagnostic, Result};
use std::collections::HashMap;
use std::env;
use std::io::ErrorKind;
use std::path::Path;
use std::process::{Command, Stdio};

/// Captured output from a finished process.
pub struct Output {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Build a PATH string with workspace bins, proto shims, and `~/.cargo/bin`
/// prepended so tools resolve to `.prototools` pins when possible.
fn enriched_path(root: &Path) -> String {
    let node_bin = root.join("node_modules").join(".bin");
    let proto_shims = home::home_dir()
        .map(|h| h.join(".proto").join("shims"))
        .filter(|p| p.is_dir());
    let cargo_bin = home::home_dir()
        .map(|h| h.join(".cargo").join("bin"))
        .filter(|p| p.is_dir());
    let existing = env::var("PATH").unwrap_or_default();

    let mut prefixes = Vec::new();
    if node_bin.is_dir() {
        prefixes.push(node_bin.display().to_string());
    }
    if let Some(shims) = proto_shims {
        prefixes.push(shims.display().to_string());
    }
    if let Some(cb) = cargo_bin {
        prefixes.push(cb.display().to_string());
    }
    if prefixes.is_empty() {
        existing
    } else {
        format!("{}:{}", prefixes.join(":"), existing)
    }
}

/// Extra env vars so child tools use proto-pinned runtimes (e.g. `UV_PYTHON`).
fn toolchain_env(root: &Path) -> HashMap<String, String> {
    let mut vars = HashMap::new();
    if let Some(python) = crate::systems::workspace::proto_tool_binary(root, "python") {
        vars.insert("UV_PYTHON".into(), python.display().to_string());
    }
    vars
}

fn apply_toolchain_env(cmd: &mut Command, root: &Path) {
    cmd.env("PATH", enriched_path(root));
    for (key, value) in toolchain_env(root) {
        cmd.env(key, value);
    }
}

/// Apply workspace PATH and toolchain env for availability checks (e.g. `sfw --help`).
pub fn apply_toolchain_env_for_check(cmd: &mut Command, root: &Path) {
    apply_toolchain_env(cmd, root);
}

/// Run a tool via `proto run <tool> -- …` so the `.prototools` pin is used.
pub fn run_proto(tool: &str, args: &[String], cwd: &Path, quiet: bool) -> Result<i32> {
    let mut argv = vec!["run".to_string(), tool.to_string(), "--".to_string()];
    argv.extend(args.iter().cloned());
    run("proto", &argv, cwd, quiet)
}

/// Run a command with inherited stdio at `cwd`, returning its exit code.
///
/// Prepends `<cwd>/node_modules/.bin` to PATH so workspace bin tools are
/// found without needing `bunx`. Echoes the command unless `quiet`.
pub fn run(program: &str, args: &[String], cwd: &Path, quiet: bool) -> Result<i32> {
    if !quiet {
        eprintln!("\x1b[2m› {} {}\x1b[0m", program, args.join(" "));
    }

    let mut cmd = Command::new(program);
    cmd.args(args).current_dir(cwd);
    apply_toolchain_env(&mut cmd, cwd);
    let status = cmd
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|err| map_spawn_error(program, err))?;

    Ok(status.code().unwrap_or(1))
}

/// Run `moon <args...>` from the workspace root, prefixing global flags
/// (`-q` / `--log <level>`) derived from Luna's verbosity options.
pub fn run_moon(root: &Path, args: &[&str], global: &GlobalArgs) -> Result<i32> {
    let mut full: Vec<String> = Vec::with_capacity(args.len() + 2);

    if global.quiet {
        full.push("-q".to_string());
    } else if let Some(level) = global.log_level() {
        full.push("--log".to_string());
        full.push(level.to_string());
    }

    full.extend(args.iter().map(|a| (*a).to_string()));

    run("moon", &full, root, global.quiet)
}

/// Run a package-manager command, optionally prefixed with Socket Firewall (`sfw`).
pub fn run_pm(
    program: &str,
    args: &[String],
    cwd: &Path,
    quiet: bool,
    firewall_active: bool,
) -> Result<i32> {
    let (program, args) = crate::systems::security::wrap(program, args, firewall_active);
    run(&program, &args, cwd, quiet)
}

/// Run a command and capture its stdout/stderr instead of inheriting them.
pub fn capture(program: &str, args: &[String], cwd: &Path) -> Result<Output> {
    let mut cmd = Command::new(program);
    cmd.args(args).current_dir(cwd);
    apply_toolchain_env(&mut cmd, cwd);
    let out = cmd
        .stdin(Stdio::null())
        .output()
        .map_err(|err| map_spawn_error(program, err))?;

    Ok(Output {
        code: out.status.code().unwrap_or(1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    })
}

/// Verify a tool is installed by spawning `<program> --version`.
///
/// Uses the workspace PATH (with `node_modules/.bin`) so dev-tool binaries
/// like `tsc` and `oxlint` are found after `bun install`.
pub fn ensure_installed(program: &str, root: &Path) -> Result<()> {
    let mut cmd = Command::new(program);
    cmd.arg("--version");
    apply_toolchain_env(&mut cmd, root);
    match cmd
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
    {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == ErrorKind::NotFound => Err(miette!(
            "missing required command: `{program}` (is it installed and on PATH?)"
        )),
        Err(err) => Err(err).into_diagnostic(),
    }
}

fn map_spawn_error(program: &str, err: std::io::Error) -> miette::Report {
    if err.kind() == ErrorKind::NotFound {
        miette!("missing required command: `{program}` (is it installed and on PATH?)")
    } else {
        miette!("failed to spawn `{program}`: {err}")
    }
}
