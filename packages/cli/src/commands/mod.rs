pub mod core;
pub mod lifecycle;
pub mod outdated;
pub mod quality;
pub mod update;

use crate::cli::Commands;
use crate::session::LunaSession;
use crate::ui::{self, Emitter};
use miette::IntoDiagnostic;
use starbase::AppResult;

/// Main execution: dispatch the parsed command and surface its exit code.
pub async fn dispatch(session: LunaSession) -> AppResult {
    let global = &session.cli.global;
    let root = session.root.as_path();

    let code: i32 = match &session.cli.command {
        Commands::Build(args) => core::run_build(root, args, global)?,
        Commands::Test(args) => core::run_test(root, args, global)?,
        Commands::Dev(args) => core::run_dev(root, args, global)?,
        Commands::Start(args) => core::run_start(root, args, global)?,
        Commands::Run(args) => core::run_targets(root, args, global)?,
        Commands::Graph => core::run_graph(root, global)?,
        Commands::Tasks => core::run_tasks(root, global)?,
        Commands::Projects => core::run_projects(root, global)?,
        Commands::Ci(args) => core::run_ci(root, args, global)?,
        Commands::Install(args) => {
            if args.workspace {
                lifecycle::install_workspace(root, global)?
            } else {
                lifecycle::install(root, global)?
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
    };

    Ok(Some(code.clamp(0, 255) as u8))
}
