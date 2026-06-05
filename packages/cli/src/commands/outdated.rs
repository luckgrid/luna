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
    if workspace::full_graph_enabled() {
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

/// Whether `bun outdated --recursive` reported any dependency rows.
fn bun_outdated(stdout: &str, stderr: &str) -> bool {
    !parse_bun_outdated_table(&format!("{stdout}{stderr}")).is_empty()
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

// --- Bun / uv parsing (used by `luna update` summary) ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BunOutdatedRow {
    pub package: String,
    pub current: String,
    pub update: String,
    pub latest: String,
    pub latest_blocked_by_age: bool,
    pub workspace: Option<String>,
}

/// Snapshot of `bun outdated --recursive` for before/after update comparisons.
pub fn bun_outdated_rows(root: &Path) -> Result<Vec<BunOutdatedRow>> {
    let out = runner::capture(
        "bun",
        &["outdated".to_string(), "--recursive".to_string()],
        root,
    )?;
    if out.code != 0 {
        return Ok(Vec::new());
    }
    Ok(parse_bun_outdated_table(&format!(
        "{}{}",
        out.stdout, out.stderr
    )))
}

/// True when Bun failed only because of `minimum-release-age` blocks (no other errors).
pub fn bun_failure_is_age_only(stdout: &str, stderr: &str) -> bool {
    let combined = format!("{stdout}{stderr}");
    let blocked = parse_bun_blocked_errors(&combined);
    if blocked.is_empty() {
        return false;
    }
    !combined.lines().any(|line| {
        let lower = line.to_lowercase();
        lower.contains("error:")
            && !lower.contains("blocked by minimum-release-age")
            && !lower.contains("no version matching")
    })
}

/// Packages Bun could not resolve because of `minimum-release-age` (from update stderr).
pub fn parse_bun_blocked_errors(text: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        if !line.contains("No version matching") || !line.contains("blocked by minimum-release-age")
        {
            continue;
        }
        let Some(pkg) = extract_quoted_field(line, "No version matching \"") else {
            continue;
        };
        let Some(spec) = extract_quoted_field(line, "found for specifier \"") else {
            continue;
        };
        if !out.iter().any(|(p, _)| p == &pkg) {
            out.push((pkg, spec));
        }
    }
    out
}

fn extract_quoted_field(line: &str, prefix: &str) -> Option<String> {
    let rest = line.split_once(prefix)?.1;
    let end = rest.find('"')?;
    Some(rest[..end].to_string())
}

pub fn parse_bun_outdated_table(stdout: &str) -> Vec<BunOutdatedRow> {
    let mut rows = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') {
            continue;
        }
        if trimmed.contains("Package") || trimmed.contains("---") {
            continue;
        }
        let cells: Vec<&str> = trimmed
            .split('|')
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .collect();
        if cells.len() < 4 {
            continue;
        }
        let latest_raw = cells[3];
        let latest_blocked = latest_raw.contains('*');
        let latest = latest_raw
            .trim_end_matches(|c: char| c == '*' || c.is_whitespace())
            .to_string();
        rows.push(BunOutdatedRow {
            package: cells[0].to_string(),
            current: cells[1].to_string(),
            update: cells[2].to_string(),
            latest,
            latest_blocked_by_age: latest_blocked,
            workspace: cells.get(4).map(|s| s.to_string()),
        });
    }
    rows
}

/// `uv lock --upgrade --dry-run` lines (`Update pkg vFROM -> vTO`).
pub fn parse_uv_dry_run_updates(text: &str) -> Vec<(String, String, String)> {
    let mut out = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if !line.starts_with("Update ") {
            continue;
        }
        let rest = line.strip_prefix("Update ").unwrap_or(line);
        let Some((left, right)) = rest.split_once(" -> ") else {
            continue;
        };
        let from_ver = left
            .rsplit_once(' ')
            .map(|(_, v)| strip_v_prefix(v))
            .unwrap_or_else(|| strip_v_prefix(left));
        let to_ver = strip_v_prefix(right);
        let package = left
            .rsplit_once(' ')
            .map(|(p, _)| p.to_string())
            .unwrap_or_else(|| left.to_string());
        out.push((package, from_ver, to_ver));
    }
    out
}

fn strip_v_prefix(version: &str) -> String {
    version.trim().trim_start_matches('v').to_string()
}

#[cfg(test)]
mod release_age_tests {
    use super::*;

    #[test]
    fn parse_bun_outdated_table_rows() {
        let text = r#"
| Package    | Current | Update | Latest   | Workspace |
|------------|---------|--------|----------|-----------|
| vite       | 7.3.5   | 7.3.5  | 8.0.14 * | app       |
"#;
        let rows = parse_bun_outdated_table(text);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].latest, "8.0.14");
        assert!(rows[0].latest_blocked_by_age);
        assert_eq!(rows[0].workspace.as_deref(), Some("app"));
    }

    #[test]
    fn bun_blocked_when_latest_differs_from_update() {
        let rows = vec![BunOutdatedRow {
            package: "vite".into(),
            current: "7.3.5".into(),
            update: "7.3.5".into(),
            latest: "8.0.14".into(),
            latest_blocked_by_age: true,
            workspace: None,
        }];
        assert_eq!(rows.iter().filter(|r| r.latest != r.update).count(), 1);
    }

    #[test]
    fn parse_bun_blocked_error_line() {
        let text = r#"error: No version matching "vite" found for specifier "7.3.5" (blocked by minimum-release-age: 1209600 seconds)"#;
        let blocked = parse_bun_blocked_errors(text);
        assert_eq!(blocked.len(), 1);
        assert_eq!(blocked[0].0, "vite");
        assert_eq!(blocked[0].1, "7.3.5");
    }

    #[test]
    fn bun_failure_is_age_only_true_for_cooldown_errors() {
        let stderr = r#"error: No version matching "vite" found for specifier "7.3.5" (blocked by minimum-release-age: 1209600 seconds)"#;
        assert!(bun_failure_is_age_only("", stderr));
    }

    #[test]
    fn bun_failure_is_age_only_false_for_other_errors() {
        let stderr = r#"error: Connection refused while resolving vite"#;
        assert!(!bun_failure_is_age_only("", stderr));
    }

    #[test]
    fn bun_failure_is_age_only_false_when_mixed_errors() {
        let stderr = r#"error: No version matching "vite" found for specifier "7.3.5" (blocked by minimum-release-age: 1209600 seconds)
error: Connection refused while resolving react"#;
        assert!(!bun_failure_is_age_only("", stderr));
    }

    #[test]
    fn parse_uv_dry_run_update_line() {
        let ups = parse_uv_dry_run_updates("Update fastapi v0.136.1 -> v0.136.3\n");
        assert_eq!(ups.len(), 1);
        assert_eq!(ups[0].0, "fastapi");
        assert_eq!(ups[0].1, "0.136.1");
        assert_eq!(ups[0].2, "0.136.3");
    }
}
