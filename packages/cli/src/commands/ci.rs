use crate::cli::{Backend, DoctorArgs, GlobalArgs, PassthroughArgs};
use crate::commands::{doctor, quality, sync};
use crate::config::LunaConfig;
use crate::output::{self, CiReport, CiStageReport};
use crate::planner;
use crate::systems::runner::run_moon;
use miette::Result;
use std::path::Path;
use std::time::Instant;

pub fn run_ci(
    root: &Path,
    config: &LunaConfig,
    args: &PassthroughArgs,
    global: &GlobalArgs,
) -> Result<i32> {
    if global.backend == Backend::Moon {
        let mut argv = vec!["ci"];
        argv.extend(args.args.iter().map(String::as_str));
        return run_moon(root, &argv, global);
    }

    type StageFn = fn(&Path, &LunaConfig, &GlobalArgs) -> Result<i32>;
    let stages: Vec<(&str, StageFn)> = vec![
        ("doctor", |r, c, g| {
            doctor::run_doctor(r, c, g, &DoctorArgs { ci: true })
        }),
        ("sync", |r, c, g| sync::run_workspace(r, c, g)),
        ("check", |r, _c, g| quality::check(r, g)),
        ("build", |r, c, g| {
            let plan = planner::build_plan(r, c, "build")?;
            if g.dry_run {
                return Ok(0);
            }
            planner::execute(r, c, &plan, g)
        }),
        ("test", |r, c, g| {
            let plan = planner::build_plan(r, c, "test")?;
            if g.dry_run {
                return Ok(0);
            }
            planner::execute(r, c, &plan, g)
        }),
    ];

    let mut reports = Vec::new();
    let mut overall = 0i32;

    for (name, stage) in stages {
        let start = Instant::now();
        let code = stage(root, config, global)?;
        let elapsed_ms = start.elapsed().as_millis() as u64;
        reports.push(CiStageReport {
            name: name.into(),
            exit_code: code,
            elapsed_ms,
        });
        if code != 0 {
            overall = code;
            break;
        }
    }

    let passed = overall == 0;
    if global.json {
        output::emit(&CiReport {
            schema_version: output::SCHEMA_VERSION.into(),
            workspace_root: root.display().to_string(),
            passed,
            stages: reports,
        });
    } else if !global.quiet {
        eprintln!("CI {}", if passed { "passed" } else { "failed" });
        for s in &reports {
            eprintln!("  {} exit={} ({}ms)", s.name, s.exit_code, s.elapsed_ms);
        }
    }
    Ok(overall)
}
