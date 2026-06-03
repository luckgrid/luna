use crate::cli::{GlobalArgs, PassthroughArgs, ProjectArgs, RunArgs, TaskArgs};
use crate::runner;
use miette::Result;
use std::path::Path;

/// Application-layer query mirroring the repo's root scripts.
const APP_LAYER_QUERY: &str = "projectLayer=application";

/// Build: `moon run :build` or `moon run <project>:build`
pub fn run_build(root: &Path, args: &TaskArgs, global: &GlobalArgs) -> Result<i32> {
    run_task(root, "build", args, global)
}

/// Test: `moon run :test` or `moon run <project>:test`
pub fn run_test(root: &Path, args: &TaskArgs, global: &GlobalArgs) -> Result<i32> {
    run_task(root, "test", args, global)
}

/// Dev: `moon run :dev` or `moon run <project>:dev` (persistent task)
pub fn run_dev(root: &Path, args: &ProjectArgs, global: &GlobalArgs) -> Result<i32> {
    run_persistent(root, "dev", args, global)
}

/// Start: `moon run :start` or `moon run <project>:start` (persistent task)
pub fn run_start(root: &Path, args: &ProjectArgs, global: &GlobalArgs) -> Result<i32> {
    run_persistent(root, "start", args, global)
}

/// Run targets directly: `moon run <targets...>`
pub fn run_targets(root: &Path, args: &RunArgs, global: &GlobalArgs) -> Result<i32> {
    let mut argv = vec!["run"];
    argv.extend(args.targets.iter().map(String::as_str));
    run_moon(root, &argv, global)
}

/// Graph: `moon project-graph`
pub fn run_graph(root: &Path, global: &GlobalArgs) -> Result<i32> {
    run_moon(root, &["project-graph"], global)
}

/// Tasks: `moon tasks`
pub fn run_tasks(root: &Path, global: &GlobalArgs) -> Result<i32> {
    run_moon(root, &["tasks"], global)
}

/// Projects: `moon projects`
pub fn run_projects(root: &Path, global: &GlobalArgs) -> Result<i32> {
    run_moon(root, &["projects"], global)
}

/// CI: `moon ci` with optional passthrough args
pub fn run_ci(root: &Path, args: &PassthroughArgs, global: &GlobalArgs) -> Result<i32> {
    let mut argv = vec!["ci"];
    argv.extend(args.args.iter().map(String::as_str));
    run_moon(root, &argv, global)
}

/// Run a task: `moon run <target>` with optional affected filtering
fn run_task(root: &Path, task: &str, args: &TaskArgs, global: &GlobalArgs) -> Result<i32> {
    match &args.project {
        Some(project) => {
            let target = format!("{project}:{task}");
            let argv = with_affected(vec!["run", &target], args.affected);
            run_moon(root, &argv, global)
        }
        None => {
            let target = format!(":{task}");
            let mut argv = vec!["run", &target, "--query", APP_LAYER_QUERY];
            if args.affected {
                argv.push("--affected");
            }
            run_moon(root, &argv, global)
        }
    }
}

/// Run a persistent task (dev/start) - no affected filtering
fn run_persistent(root: &Path, task: &str, args: &ProjectArgs, global: &GlobalArgs) -> Result<i32> {
    match &args.project {
        Some(project) => {
            let target = format!("{project}:{task}");
            run_moon(root, &["run", &target], global)
        }
        None => {
            let target = format!(":{task}");
            run_moon(root, &["run", &target, "--query", APP_LAYER_QUERY], global)
        }
    }
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

    runner::run("moon", &full, root, global.quiet)
}

/// Append `--affected` to a target list when requested.
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
