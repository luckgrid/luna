use miette::{miette, IntoDiagnostic, Result};
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

/// Build a PATH string with `<root>/node_modules/.bin` and `~/.cargo/bin`
/// prepended so workspace-dev tools (tsc, oxlint, oxfmt) and Cargo binaries
/// are resolvable without relying on global PATH.
fn enriched_path(root: &Path) -> String {
    let node_bin = root.join("node_modules").join(".bin");
    let cargo_bin = home::home_dir()
        .map(|h| h.join(".cargo").join("bin"))
        .filter(|p| p.is_dir());
    let existing = env::var("PATH").unwrap_or_default();

    let mut prefixes = Vec::new();
    if node_bin.is_dir() {
        prefixes.push(node_bin.display().to_string());
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

/// Run a command with inherited stdio at `cwd`, returning its exit code.
///
/// Prepends `<cwd>/node_modules/.bin` to PATH so workspace bin tools are
/// found without needing `bunx`. Echoes the command unless `quiet`.
pub fn run(program: &str, args: &[String], cwd: &Path, quiet: bool) -> Result<i32> {
    if !quiet {
        eprintln!("\x1b[2m› {} {}\x1b[0m", program, args.join(" "));
    }

    let status = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .env("PATH", enriched_path(cwd))
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|err| map_spawn_error(program, err))?;

    Ok(status.code().unwrap_or(1))
}

/// Run a command and capture its stdout/stderr instead of inheriting them.
pub fn capture(program: &str, args: &[String], cwd: &Path) -> Result<Output> {
    let out = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .env("PATH", enriched_path(cwd))
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
    match Command::new(program)
        .arg("--version")
        .env("PATH", enriched_path(root))
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
