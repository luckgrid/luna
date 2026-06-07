use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::output::{self, ApplyReport};
use crate::planner::{self, Plan};
use crate::systems::ledger;
use miette::{miette, Result};
use std::path::{Path, PathBuf};

pub fn run_apply(
    root: &Path,
    config: &LunaConfig,
    global: &GlobalArgs,
    plan_path: &Path,
) -> Result<i32> {
    let text = std::fs::read_to_string(plan_path)
        .map_err(|e| miette!("read plan {}: {e}", plan_path.display()))?;
    let plan: Plan = serde_json::from_str(&text).map_err(|e| miette!("invalid plan JSON: {e}"))?;

    let current_fp = plan_fingerprint(root, config, &plan)?;
    let stored_fp = plan
        .fingerprint
        .as_deref()
        .ok_or_else(|| miette!("plan file missing fingerprint — re-run `luna plan --out`"))?;

    if current_fp != stored_fp {
        return Err(miette!(
            "stale plan: workspace manifests changed since plan was written"
        ));
    }

    let code = planner::execute(root, config, &plan, global)?;
    if global.json {
        output::emit(&ApplyReport {
            schema_version: output::SCHEMA_VERSION.into(),
            target: plan.target.clone(),
            fingerprint: current_fp,
            applied: true,
            exit_code: code,
        });
    }
    Ok(code)
}

pub fn write_plan_out(
    root: &Path,
    config: &LunaConfig,
    target: &str,
    out: &Path,
    global: &GlobalArgs,
) -> Result<i32> {
    let mut plan = planner::build_plan(root, config, target)?;
    plan.fingerprint = Some(plan_fingerprint(root, config, &plan)?);
    let json = serde_json::to_string_pretty(&plan).map_err(|e| miette!("serialize plan: {e}"))?;
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    std::fs::write(out, json).map_err(|e| miette!("write {}: {e}", out.display()))?;
    if global.json {
        output::emit(&output::PlanReport::new(plan));
    } else if !global.quiet {
        eprintln!("\x1b[32m✓\x1b[0m Plan written to {}", out.display());
    }
    Ok(0)
}

fn plan_fingerprint(root: &Path, config: &LunaConfig, plan: &Plan) -> Result<String> {
    let ledger_fp = if let Ok(ledger) = ledger::read(root, config) {
        ledger::ledger_fingerprint(&ledger)
    } else {
        String::new()
    };
    Ok(format!(
        "{}::{}::{}",
        plan.target,
        plan.steps.len(),
        ledger_fp
    ))
}

pub fn default_plan_path(root: &Path, config: &LunaConfig, target: &str) -> PathBuf {
    root.join(&config.state.dir)
        .join("plans")
        .join(format!("{target}.plan.json"))
}
