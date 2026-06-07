use crate::cli::Cli;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

pub fn run_completions(shell: Shell) -> miette::Result<i32> {
    let mut cmd = Cli::command();
    generate(shell, &mut cmd, "luna", &mut io::stdout());
    Ok(0)
}

pub fn parse_shell(s: &str) -> Shell {
    match s {
        "bash" => Shell::Bash,
        "elvish" => Shell::Elvish,
        "fish" => Shell::Fish,
        "powershell" => Shell::PowerShell,
        "zsh" => Shell::Zsh,
        _ => Shell::Bash,
    }
}
