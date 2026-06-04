use crate::cli::GlobalArgs;
use crate::runner;
use crate::workspace;
use miette::Result;
use std::path::Path;

/// Report outdated proto pins, Bun workspaces, uv lockfiles, and Go modules.
/// Always exits 0 after printing the report (outdated findings are informational).
pub fn run(root: &Path, _global: &GlobalArgs) -> Result<i32> {
    runner::ensure_installed("proto", root)?;

    let mut outdated_tiers: Vec<&str> = Vec::new();

    // proto pins
    section("proto — .prototools pins");
    let proto = runner::capture(
        "proto",
        &["outdated".to_string(), "--json".to_string()],
        root,
    )?;
    if proto_pins_outdated(&proto.stdout) {
        print!("{}", run_text("proto", &["outdated"], root));
        outdated_tiers.push("proto pins");
    } else {
        ok("all pins up to date");
    }

    // Rust / Cargo workspace
    if root.join("Cargo.toml").is_file() && root.join("Cargo.lock").is_file() {
        section("Rust / Cargo — workspace");
        match rust_outdated(root)? {
            RustOutdated::UpToDate => ok("all workspace dependencies up to date"),
            RustOutdated::Outdated(output) => {
                print!("{output}");
                outdated_tiers.push("Rust / Cargo");
            }
            RustOutdated::Skipped(msg) => {
                println!("\x1b[33m⊘\x1b[0m {msg}");
            }
        }
    }

    runner::ensure_installed("bun", root)?;

    // Bun workspaces
    section("Bun — workspace dependencies");
    let bun = runner::capture(
        "bun",
        &["outdated".to_string(), "--recursive".to_string()],
        root,
    )?;
    if bun_outdated(&bun.stdout, &bun.stderr) {
        print!("{}{}", bun.stdout, bun.stderr);
        outdated_tiers.push("Bun workspaces");
    } else {
        ok("all workspace dependencies up to date");
    }

    // Python / uv (single root lockfile when workspace is configured)
    if let Some(uv_root) = workspace::uv_workspace_root(root) {
        runner::ensure_installed("uv", root)?;
        section("Python / uv — workspace");
        let out = runner::capture(
            "uv",
            &[
                "lock".to_string(),
                "--upgrade".to_string(),
                "--dry-run".to_string(),
            ],
            &uv_root,
        )?;
        let combined = format!("{}{}", out.stdout, out.stderr);
        if uv_outdated(&combined) {
            for line in combined
                .lines()
                .filter(|l| l.trim_start().starts_with("Update "))
            {
                println!("{line}");
            }
            outdated_tiers.push("Python / uv lockfile");
        } else {
            ok("lockfile up to date");
        }
    } else {
        let uv_roots = workspace::project_roots(root, "python", "pyproject.toml");
        if uv_roots.is_empty() {
            section("Python / uv");
            ok("no uv projects discovered");
        } else {
            runner::ensure_installed("uv", root)?;
            let mut any = false;
            for project in &uv_roots {
                let label = rel(root, project);
                section(&format!("Python / uv — {label}"));
                let out = runner::capture(
                    "uv",
                    &[
                        "lock".to_string(),
                        "--upgrade".to_string(),
                        "--dry-run".to_string(),
                    ],
                    project,
                )?;
                let combined = format!("{}{}", out.stdout, out.stderr);
                if uv_outdated(&combined) {
                    for line in combined
                        .lines()
                        .filter(|l| l.trim_start().starts_with("Update "))
                    {
                        println!("{line}");
                    }
                    any = true;
                } else {
                    ok("lockfile up to date");
                }
            }
            if any {
                outdated_tiers.push("Python / uv lockfile(s)");
            }
        }
    }

    // Go modules
    let go_roots = workspace::project_roots(root, "go", "go.mod");
    if go_roots.is_empty() {
        section("Go");
        ok("no Go modules discovered");
    } else {
        runner::ensure_installed("go", root)?;
        let mut any = false;
        for module in &go_roots {
            let label = rel(root, module);
            section(&format!("Go — {label}"));
            if go_module_outdated(module) {
                any = true;
            } else {
                ok("module up to date");
            }
        }
        if any {
            outdated_tiers.push("Go module(s)");
        }
    }

    println!();
    if outdated_tiers.is_empty() {
        println!("\x1b[32m✓ All checks passed (nothing reported as outdated).\x1b[0m");
    } else {
        println!("\x1b[1;33mOutdated dependencies reported in:\x1b[0m");
        for tier in &outdated_tiers {
            println!("  \x1b[33m•\x1b[0m {tier}");
        }
        println!("\x1b[2mTo refresh, run: luna update\x1b[0m");
    }
    Ok(0)
}

fn go_module_outdated(module: &Path) -> bool {
    if workspace::go_uses_tool_fast_path(module) {
        go_tools_outdated(module)
    } else {
        go_list_modules_outdated(module)
    }
}

fn go_tools_outdated(module: &Path) -> bool {
    let mut args = vec!["list".to_string(), "-m".to_string(), "-u".to_string()];
    args.extend(workspace::go_tool_paths(module));
    let Ok(out) = runner::capture("go", &args, module) else {
        return false;
    };
    print_go_list_upgrades(&out.stdout)
}

fn go_list_modules_outdated(module: &Path) -> bool {
    let mut args = vec!["list".to_string(), "-m".to_string(), "-u".to_string()];
    if workspace::go_full_graph_enabled() {
        args.push("all".to_string());
    }
    let Ok(out) = runner::capture("go", &args, module) else {
        return false;
    };
    print_go_list_upgrades(&out.stdout)
}

fn print_go_list_upgrades(stdout: &str) -> bool {
    let lines: Vec<_> = stdout
        .lines()
        .filter(|l| go_list_line_has_upgrade(l.trim()))
        .collect();
    for line in &lines {
        println!("{line}");
    }
    !lines.is_empty()
}

enum RustOutdated {
    UpToDate,
    Outdated(String),
    Skipped(String),
}

fn rust_outdated(root: &Path) -> Result<RustOutdated> {
    runner::ensure_installed("cargo", root)?;
    let has_outdated_cmd = runner::capture(
        "cargo",
        &["outdated".to_string(), "--version".to_string()],
        root,
    )
    .map(|o| o.code == 0)
    .unwrap_or(false);

    if !has_outdated_cmd {
        let install_code = runner::run(
            "cargo",
            &[
                "install".to_string(),
                "cargo-outdated".to_string(),
                "--locked".to_string(),
            ],
            root,
            true,
        )?;
        if install_code != 0 {
            return Ok(RustOutdated::Skipped(
                "install `cargo-outdated` (`cargo install cargo-outdated`) to check Rust deps"
                    .to_string(),
            ));
        }
    }

    let out = match runner::capture("cargo", &["outdated".to_string()], root) {
        Ok(o) => o,
        Err(_) => {
            return Ok(RustOutdated::Skipped(
                "could not run `cargo outdated` (install with `cargo install cargo-outdated`)"
                    .to_string(),
            ));
        }
    };
    let combined = format!("{}{}", out.stdout, out.stderr);
    // `cargo outdated` exits 1 when upgrades are available.
    if out.code == 1 {
        Ok(RustOutdated::Outdated(combined))
    } else if out.code == 0 {
        Ok(RustOutdated::UpToDate)
    } else {
        Ok(RustOutdated::Skipped(format!(
            "`cargo outdated` failed (exit {}): {}",
            out.code,
            combined.trim()
        )))
    }
}

/// `proto outdated --json` marks at least one pin with `"is_outdated": true`.
fn proto_pins_outdated(json: &str) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json.trim()) else {
        return false;
    };
    let Some(map) = value.as_object() else {
        return false;
    };
    map.values().any(|row| {
        row.get("is_outdated")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    })
}

/// Heuristic for `bun outdated --recursive`: a table with at least one data row.
fn bun_outdated(stdout: &str, stderr: &str) -> bool {
    let combined = format!("{stdout}{stderr}");
    if !combined.contains("Package") {
        return false;
    }
    combined.lines().any(|line| {
        let trimmed = line.trim_start();
        (trimmed.starts_with('|') || trimmed.starts_with('│'))
            && !trimmed.contains("Package")
            && line.chars().any(|c| c.is_ascii_digit())
    })
}

fn uv_outdated(text: &str) -> bool {
    text.lines().any(|l| l.trim_start().starts_with("Update "))
}

fn go_list_line_has_upgrade(line: &str) -> bool {
    // `path current [newest]` — the bracket marks an available upgrade.
    let mut parts = line.split_whitespace();
    matches!((parts.next(), parts.next(), parts.next()), (Some(_), Some(_), Some(third)) if third.starts_with('['))
}

fn run_text(program: &str, args: &[&str], cwd: &Path) -> String {
    let argv: Vec<String> = args.iter().map(|s| s.to_string()).collect();
    match runner::capture(program, &argv, cwd) {
        Ok(out) => format!("{}{}", out.stdout, out.stderr),
        Err(_) => String::new(),
    }
}

fn section(title: &str) {
    println!("\n\x1b[1m== {title} ==\x1b[0m");
}

fn ok(msg: &str) {
    println!("\x1b[32m✓\x1b[0m {msg}");
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}
