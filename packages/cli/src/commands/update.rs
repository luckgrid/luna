use crate::cli::{GlobalArgs, UpdateArgs};
use crate::workspace;
use crate::runner;
use miette::Result;
use std::path::Path;

/// Update toolchains and dependencies across every ecosystem, then re-run install.
///
/// Minimal delegating pipeline: proto pins, Bun workspaces, uv lockfiles, Go
/// modules. `go get -u` and `@latest` stay within a module's major path, so the
/// `--major` flag only widens the proto/Bun bumps.
pub fn run(root: &Path, args: &UpdateArgs, global: &GlobalArgs) -> Result<i32> {
    let major = args.major;
    let quiet = global.quiet;

    // proto pins
    runner::ensure_installed("proto", root)?;
    section("proto — update .prototools pins");
    let mut proto_args = vec!["outdated".to_string(), "--update".to_string()];
    if major {
        proto_args.push("--latest".to_string());
    }
    proto_args.push("-y".to_string());
    run_step("proto", &proto_args, root, quiet)?;
    section("proto — install pinned tools");
    run_step("proto", &["install".to_string()], root, quiet)?;

    // Bun workspaces
    runner::ensure_installed("bun", root)?;
    section("Bun — update workspace dependencies");
    let mut bun_args = vec!["update".to_string(), "--recursive".to_string()];
    if major {
        bun_args.push("--latest".to_string());
    }
    run_step("bun", &bun_args, root, quiet)?;

    // Python / uv
    let uv_roots = workspace::project_roots(root, "python", "pyproject.toml");
    if !uv_roots.is_empty() {
        runner::ensure_installed("uv", root)?;
        for project in &uv_roots {
            let label = rel(root, project);
            section(&format!("Python / uv — {label} (lock --upgrade + sync)"));
            run_step(
                "uv",
                &["lock".to_string(), "--upgrade".to_string()],
                project,
                quiet,
            )?;
            run_step("uv", &["sync".to_string()], project, quiet)?;
        }
    }

    // Go modules
    let go_roots = workspace::project_roots(root, "go", "go.mod");
    if !go_roots.is_empty() {
        runner::ensure_installed("go", root)?;
        for module in &go_roots {
            let label = rel(root, module);
            if workspace::is_go_tool_only(module) {
                section(&format!("Go — {label} (go get -tool @latest + tidy)"));
                for tool in workspace::go_tool_paths(module) {
                    run_step(
                        "go",
                        &[
                            "get".to_string(),
                            "-tool".to_string(),
                            format!("{tool}@latest"),
                        ],
                        module,
                        quiet,
                    )?;
                }
            } else {
                section(&format!("Go — {label} (go get -u all + tidy)"));
                run_step(
                    "go",
                    &["get".to_string(), "-u".to_string(), "all".to_string()],
                    module,
                    quiet,
                )?;
            }
            run_step(
                "go",
                &["mod".to_string(), "tidy".to_string()],
                module,
                quiet,
            )?;
        }
    }

    // Re-run workspace bootstrap.
    section("repo setup — proto install + bun install + moon builds");
    run_step("proto", &["install".to_string()], root, quiet)?;
    run_step(
        "bun",
        &["install".to_string(), "--ignore-scripts".to_string()],
        root,
        quiet,
    )?;
    run_step(
        "moon",
        &["run".to_string(), "cli:build".to_string()],
        root,
        quiet,
    )?;
    run_step(
        "moon",
        &["run".to_string(), "web:setup".to_string()],
        root,
        quiet,
    )?;
    run_step(
        "moon",
        &["run".to_string(), "api:build".to_string()],
        root,
        quiet,
    )?;

    println!("\nUpdate steps finished. Review changes before committing.");
    if !major {
        println!("Tip: re-run with `luna update --major` to also apply major-version bumps.");
    }
    Ok(0)
}

/// Run a step, aborting the pipeline if it fails.
fn run_step(program: &str, args: &[String], cwd: &Path, quiet: bool) -> Result<()> {
    let code = runner::run(program, args, cwd, quiet)?;
    if code != 0 {
        return Err(miette::miette!(
            "`{program} {}` failed with exit code {code}",
            args.join(" ")
        ));
    }
    Ok(())
}

fn section(title: &str) {
    println!("\n\x1b[1m== {title} ==\x1b[0m");
}

fn rel(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| path.display().to_string())
}
