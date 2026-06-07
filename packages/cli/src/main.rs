use clap::Parser;
use cli::config;
use cli::observability;
use cli::systems::workspace;
use cli::{commands, Cli, Commands, LunaSession};
use starbase::{App, MainResult};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> MainResult {
    let cli = Cli::parse();

    if let Some(cwd) = &cli.global.cwd {
        std::env::set_current_dir(cwd).map_err(|e| miette::miette!("--cwd invalid: {e}"))?;
    }

    observability::init_trace(&cli.global);

    let app = App::default();
    app.setup_diagnostics();
    let _guard = app.setup_tracing_with_defaults()?;

    let root = workspace::find_root()?;

    let luna_config = match &cli.command {
        Commands::Migrate(_) | Commands::Init(_) => commands::migrate::load_for_bootstrap(&root)?,
        _ => config::load_required(&root).map_err(|e| miette::miette!("{e}"))?,
    };

    let session = LunaSession::new(cli, root, luna_config);

    let code = app
        .run(session, |session| async move {
            commands::dispatch(session).await
        })
        .await?;

    Ok(ExitCode::from(code))
}
