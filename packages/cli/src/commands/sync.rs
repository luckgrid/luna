use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::planner;
use miette::Result;
use std::path::Path;

/// Full bootstrap via planner: proto + CLI + workspace sync.
pub fn run_full(root: &Path, config: &LunaConfig, global: &GlobalArgs) -> Result<i32> {
    let plan = planner::build_plan(root, config, "sync")?;
    if global.dry_run {
        return render_plan_dry_run(&plan, global);
    }
    planner::execute(root, config, &plan, global)
}

/// Workspace-only sync (skip CLI bootstrap moon steps when workspace flag set).
pub fn run_workspace(root: &Path, config: &LunaConfig, global: &GlobalArgs) -> Result<i32> {
    if global.dry_run {
        let plan = planner::build_plan(root, config, "sync")?;
        return render_plan_dry_run(&plan, global);
    }
    crate::systems::tasks::bootstrap_workspace(root, config, global)
}

fn render_plan_dry_run(plan: &planner::Plan, global: &GlobalArgs) -> Result<i32> {
    if global.json {
        crate::output::emit(&crate::output::PlanReport::new(plan.clone()));
    } else if !global.quiet {
        eprintln!("Dry-run plan for: {}", plan.target);
        for step in &plan.steps {
            eprintln!(
                "  {} {} {}",
                step.adapter,
                step.program,
                step.args.join(" ")
            );
        }
    }
    Ok(0)
}
