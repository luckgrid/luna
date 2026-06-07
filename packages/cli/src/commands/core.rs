use crate::adapters::moon::MoonBackend;
use crate::cli::{GlobalArgs, ProjectArgs, RunArgs, TaskArgs};
use crate::config::LunaConfig;
use crate::output::{self, ProjectGraph, TaskGraph};
use crate::planner;
use crate::systems::workspace;
use miette::Result;
use std::path::Path;

fn scope_query(config: &LunaConfig, command: &str) -> String {
    let scope = match command {
        "dev" => &config.commands.dev.default_scope,
        _ => &config.commands.build.default_scope,
    };
    format!(
        "projectLayer={}",
        scope.replace("applications", "application")
    )
}

fn run_moon_via_adapter(root: &Path, args: &[&str], global: &GlobalArgs) -> Result<i32> {
    MoonBackend::run_moon_argv(root, args, global)
}

pub fn run_planned(
    root: &Path,
    config: &LunaConfig,
    target: &str,
    global: &GlobalArgs,
    project: Option<&str>,
) -> Result<i32> {
    if let Some(project) = project {
        return run_task_target(root, &format!("{project}:{target}"), global);
    }

    let plan = planner::build_plan(root, config, target)?;
    if global.dry_run {
        if global.json {
            output::emit(&output::PlanReport::new(plan));
        } else if !global.quiet {
            eprintln!("Dry-run plan for: {target}");
            for step in &plan.steps {
                eprintln!(
                    "  {} {} {}",
                    step.adapter,
                    step.program,
                    step.args.join(" ")
                );
            }
        }
        return Ok(0);
    }
    planner::execute(root, config, &plan, global)
}

/// Build: planner-backed or project-scoped moon target.
pub fn run_build(
    root: &Path,
    config: &LunaConfig,
    args: &TaskArgs,
    global: &GlobalArgs,
) -> Result<i32> {
    run_task(root, config, "build", args, global)
}

/// Test: planner-backed or project-scoped moon target.
pub fn run_test(
    root: &Path,
    config: &LunaConfig,
    args: &TaskArgs,
    global: &GlobalArgs,
) -> Result<i32> {
    run_task(root, config, "test", args, global)
}

/// Dev: `moon run :dev` or project dev; optional watchexec when configured.
pub fn run_dev(
    root: &Path,
    config: &LunaConfig,
    args: &ProjectArgs,
    global: &GlobalArgs,
) -> Result<i32> {
    if config.commands.dev.watcher.as_deref() == Some("watchexec")
        && args.project.is_none()
        && crate::systems::runner::ensure_installed("watchexec", root).is_ok()
    {
        let args = vec![
            "--watch".to_string(),
            ".".to_string(),
            "--".to_string(),
            "moon".to_string(),
            "run".to_string(),
            ":dev".to_string(),
            "--query".to_string(),
            scope_query(config, "dev"),
        ];
        return crate::systems::runner::run("watchexec", &args, root, global.quiet);
    }
    run_persistent(root, config, "dev", args, global)
}

/// Start: `moon run :start` or `moon run <project>:start`
pub fn run_start(
    root: &Path,
    config: &LunaConfig,
    args: &ProjectArgs,
    global: &GlobalArgs,
) -> Result<i32> {
    run_persistent(root, config, "start", args, global)
}

/// Run targets directly: `moon run <targets...>`
pub fn run_targets(root: &Path, args: &RunArgs, global: &GlobalArgs) -> Result<i32> {
    let mut argv = vec!["run"];
    argv.extend(args.targets.iter().map(String::as_str));
    run_moon_via_adapter(root, &argv, global)
}

pub fn run_task_target(root: &Path, target: &str, global: &GlobalArgs) -> Result<i32> {
    run_moon_via_adapter(root, &["run", target], global)
}

/// Graph: structured project graph or legacy moon project-graph.
pub fn run_graph_project(root: &Path, global: &GlobalArgs) -> Result<i32> {
    if global.json {
        let projects = workspace::discover_projects(root)
            .into_iter()
            .map(|p| output::ProjectNode {
                id: p.name,
                path: p.path.display().to_string(),
            })
            .collect();
        output::emit(&ProjectGraph {
            schema_version: output::SCHEMA_VERSION.into(),
            workspace_root: root.display().to_string(),
            projects,
        });
        return Ok(0);
    }
    run_moon_via_adapter(root, &["project-graph"], global)
}

pub fn run_graph_task(root: &Path, target: &str, global: &GlobalArgs) -> Result<i32> {
    if global.json {
        let out = crate::adapters::moon::MoonBackend::capture_json(
            root,
            &["query", "tasks", "--json"],
            global.quiet,
        )?;
        let raw: serde_json::Value =
            serde_json::from_str(out.stdout.trim()).unwrap_or(serde_json::json!({}));
        output::emit(&TaskGraph {
            schema_version: output::SCHEMA_VERSION.into(),
            workspace_root: root.display().to_string(),
            target: Some(target.into()),
            raw,
        });
        return Ok(out.code);
    }
    run_moon_via_adapter(root, &["query", "tasks"], global)
}

/// Tasks: `moon tasks` with optional JSON envelope.
pub fn run_tasks(root: &Path, global: &GlobalArgs) -> Result<i32> {
    if global.json {
        let out = crate::adapters::moon::MoonBackend::capture_json(root, &["tasks"], global.quiet)?;
        let raw: serde_json::Value = serde_json::from_str(out.stdout.trim())
            .unwrap_or(serde_json::json!({ "raw": out.stdout }));
        output::emit(&TaskGraph {
            schema_version: output::SCHEMA_VERSION.into(),
            workspace_root: root.display().to_string(),
            target: None,
            raw,
        });
        return Ok(out.code);
    }
    run_moon_via_adapter(root, &["tasks"], global)
}

/// Projects: alias for structured project list.
pub fn run_projects(root: &Path, global: &GlobalArgs) -> Result<i32> {
    run_graph_project(root, global)
}

fn run_task(
    root: &Path,
    config: &LunaConfig,
    task: &str,
    args: &TaskArgs,
    global: &GlobalArgs,
) -> Result<i32> {
    match &args.project {
        Some(project) => {
            let target = format!("{project}:{task}");
            let argv = with_affected(vec!["run", &target], global.affected);
            run_moon_via_adapter(root, &argv, global)
        }
        None => {
            let target = format!(":{task}");
            let query = scope_query(config, task);
            let mut argv = vec!["run", &target, "--query", &query];
            if global.affected {
                argv.push("--affected");
            }
            run_moon_via_adapter(root, &argv, global)
        }
    }
}

fn run_persistent(
    root: &Path,
    config: &LunaConfig,
    task: &str,
    args: &ProjectArgs,
    global: &GlobalArgs,
) -> Result<i32> {
    match &args.project {
        Some(project) => {
            let target = format!("{project}:{task}");
            run_moon_via_adapter(root, &["run", &target], global)
        }
        None => {
            let target = format!(":{task}");
            let query = scope_query(config, task);
            run_moon_via_adapter(root, &["run", &target, "--query", &query], global)
        }
    }
}

fn with_affected(mut args: Vec<&str>, affected: bool) -> Vec<&str> {
    if affected {
        args.push("--affected");
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn with_affected_false() {
        let args = vec!["run", ":build"];
        assert_eq!(with_affected(args, false), vec!["run", ":build"]);
    }

    #[test]
    fn with_affected_true() {
        let args = vec!["run", ":build"];
        assert_eq!(
            with_affected(args, true),
            vec!["run", ":build", "--affected"]
        );
    }
}
