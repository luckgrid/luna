pub mod apply;
pub mod ci;
pub mod completions;
pub mod config;
pub mod core;
pub mod doctor;
pub mod env;
pub mod lifecycle;
pub mod lock;
pub mod migrate;
pub mod outdated;
pub mod pkg;
pub mod quality;
pub mod sbom;
pub mod sync;
pub mod update;

use crate::cli::{
    AgentCommands, Commands, ConfigCommands, EnvCommands, GraphCommands, PkgCommands, TaskCommands,
};
use crate::output::{self, PlanReport};
use crate::planner;
use crate::session::LunaSession;
use crate::ui::{self, Emitter};
use miette::IntoDiagnostic;
use starbase::AppResult;

/// Main execution: dispatch the parsed command and surface its exit code.
pub async fn dispatch(session: LunaSession) -> AppResult {
    let global = &session.cli.global;
    let root = session.root.as_path();
    let config = &session.config;

    let code: i32 = match &session.cli.command {
        Commands::Build(args) => {
            core::run_planned(root, config, "build", global, args.project.as_deref())?
        }
        Commands::Test(args) => {
            core::run_planned(root, config, "test", global, args.project.as_deref())?
        }
        Commands::Dev(args) => core::run_dev(root, config, args, global)?,
        Commands::Start(args) => core::run_start(root, config, args, global)?,
        Commands::Run(args) => core::run_targets(root, args, global)?,
        Commands::Graph { command } => match command {
            None | Some(GraphCommands::Project) => core::run_graph_project(root, global)?,
            Some(GraphCommands::Task(args)) => core::run_graph_task(root, &args.target, global)?,
        },
        Commands::Tasks => core::run_tasks(root, global)?,
        Commands::Projects => core::run_graph_project(root, global)?,
        Commands::Ci(args) => ci::run_ci(root, config, args, global)?,
        Commands::Install(args) => {
            if args.workspace {
                sync::run_workspace(root, config, global)?
            } else {
                sync::run_full(root, config, global)?
            }
        }
        Commands::Sync(args) => {
            if args.workspace {
                sync::run_workspace(root, config, global)?
            } else {
                sync::run_full(root, config, global)?
            }
        }
        Commands::Clean => lifecycle::clean(root, global)?,
        Commands::Lint(args) => quality::lint(root, args.fix, global)?,
        Commands::Format(args) => quality::format(root, args.check, global)?,
        Commands::Typecheck => quality::typecheck(root, global)?,
        Commands::Check => quality::check(root, global)?,
        Commands::Fix => quality::fix(root, global)?,
        Commands::Outdated => {
            let mut console = ui::new_console(global.quiet);
            let emitter = Emitter::new(console.clone(), global.quiet);
            let result = outdated::run(root, global, &emitter).await?;
            console.close().into_diagnostic()?;
            result
        }
        Commands::Update(args) => {
            let mut console = ui::new_console(global.quiet);
            let emitter = Emitter::new(console.clone(), global.quiet);
            let result = update::run(root, args, global, &emitter).await?;
            console.close().into_diagnostic()?;
            result
        }
        Commands::Doctor(args) => doctor::run_doctor(root, config, global, args)?,
        Commands::Plan(args) => {
            if let Some(out) = &args.out {
                apply::write_plan_out(root, config, &args.target, out, global)?
            } else {
                render_plan(root, config, &args.target, global)?
            }
        }
        Commands::Config { command } => match command {
            ConfigCommands::Validate => config::validate_cmd(root, global)?,
            ConfigCommands::Print => config::print_cmd(root, global)?,
        },
        Commands::Env { command } => match command {
            EnvCommands::List => env::list(root, config, global)?,
            EnvCommands::Sync(args) => {
                env::sync(root, config, global, args.environment.as_deref())?
            }
            EnvCommands::Exec(args) => env::exec(
                root,
                config,
                global,
                args.environment.as_deref(),
                &args.command,
            )?,
        },
        Commands::Task { command } => match command {
            TaskCommands::List => core::run_tasks(root, global)?,
            TaskCommands::Run(args) => core::run_task_target(root, &args.target, global)?,
        },
        Commands::Pkg { command } => match command {
            PkgCommands::Add(args) => pkg::add(root, config, args, global)?,
            PkgCommands::Remove(args) => pkg::remove(root, config, args, global)?,
        },
        Commands::Lock => lock::run_lock(root, config, global)?,
        Commands::Sbom(args) => sbom::run_sbom(
            root,
            config,
            global,
            sbom::SbomFormat::parse_format(&args.format),
        )?,
        Commands::Inventory(args) => sbom::run_sbom(
            root,
            config,
            global,
            sbom::SbomFormat::parse_format(&args.format),
        )?,
        Commands::Migrate(args) => {
            migrate::run_migrate(root, global, args.force, migrate::MigrateMode::Migrate)?
        }
        Commands::Init(args) => {
            migrate::run_migrate(root, global, args.force, migrate::MigrateMode::Init)?
        }
        Commands::Apply(args) => apply::run_apply(root, config, global, &args.plan_file)?,
        Commands::Completions(args) => {
            completions::run_completions(completions::parse_shell(&args.shell))?
        }
        Commands::Agent { command } => match command {
            AgentCommands::Mcp => {
                if !config.agent.mcp {
                    return Err(miette::miette!(
                        "MCP disabled — set [agent].mcp = true in luna.toml"
                    ));
                }
                crate::agent::run_mcp_stdio(&session)?
            }
        },
    };

    Ok(Some(code.clamp(0, 255) as u8))
}

fn render_plan(
    root: &std::path::Path,
    config: &crate::config::LunaConfig,
    target: &str,
    global: &crate::cli::GlobalArgs,
) -> miette::Result<i32> {
    let plan = planner::build_plan(root, config, target)?;
    if global.json {
        output::emit(&PlanReport::new(plan));
    } else if !global.quiet {
        eprintln!("Plan for: {}", plan.target);
        eprintln!("Workspace: {}", plan.workspace_root);
        eprintln!();
        for step in &plan.steps {
            let dep_str = if step.depends_on.is_empty() {
                String::new()
            } else {
                format!(" (after {})", step.depends_on.join(", "))
            };
            eprintln!(
                "  {} {} {}{}",
                step.adapter,
                step.program,
                step.args.join(" "),
                dep_str,
            );
        }
    }
    Ok(0)
}
