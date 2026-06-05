pub mod moon;
pub mod outdated;
pub mod scripts;
pub mod update;

use crate::cli::Commands;
use crate::session::LunaSession;
use starbase::AppResult;

/// Main execution: dispatch the parsed command and surface its exit code.
pub async fn dispatch(session: LunaSession) -> AppResult {
    let global = &session.cli.global;
    let root = session.root.as_path();

    let code: i32 = match &session.cli.command {
        Commands::Build(args) => moon::run_build(root, args, global)?,
        Commands::Test(args) => moon::run_test(root, args, global)?,
        Commands::Dev(args) => moon::run_dev(root, args, global)?,
        Commands::Start(args) => moon::run_start(root, args, global)?,
        Commands::Run(args) => moon::run_targets(root, args, global)?,
        Commands::Graph => moon::run_graph(root, global)?,
        Commands::Tasks => moon::run_tasks(root, global)?,
        Commands::Projects => moon::run_projects(root, global)?,
        Commands::Ci(args) => moon::run_ci(root, args, global)?,
        Commands::Install => scripts::install(root, global)?,
        Commands::Clean => scripts::clean(root, global)?,
        Commands::Lint(args) => scripts::lint(root, args.fix, global)?,
        Commands::Format(args) => scripts::format(root, args.check, global)?,
        Commands::Typecheck => scripts::typecheck(root, global)?,
        Commands::Check => scripts::check(root, global)?,
        Commands::Fix => scripts::fix(root, global)?,
        Commands::Outdated => outdated::run(root, global)?,
        Commands::Update(args) => update::run(root, args, global, &session.update_feedback)?,
    };

    Ok(Some(code.clamp(0, 255) as u8))
}
