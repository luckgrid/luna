mod cli;
mod commands;
mod runner;
mod security;
mod session;
mod workspace;

use clap::Parser;
use cli::Cli;
use session::LunaSession;
use starbase::{App, MainResult};
use std::process::ExitCode;

#[tokio::main]
async fn main() -> MainResult {
    let cli = Cli::parse();

    let app = App::default();
    app.setup_diagnostics();
    let _guard = app.setup_tracing_with_defaults()?;

    let root = workspace::find_root()?;
    let session = LunaSession::new(cli, root);

    let code = app
        .run(session, |session| async move {
            commands::dispatch(session).await
        })
        .await?;

    Ok(ExitCode::from(code))
}
