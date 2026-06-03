use clap::{Args, Parser, Subcommand};

#[derive(Debug, Clone, Parser)]
#[command(
    name = "luna",
    version,
    about = "Luna monorepo CLI — a Rust orchestrator over Moon and Proto.",
    propagate_version = true,
    disable_help_subcommand = false
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Args)]
pub struct GlobalArgs {
    /// Increase logging verbosity (maps to `moon --log debug`).
    #[arg(short = 'v', long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Silence Luna and Moon output (maps to `moon -q`).
    #[arg(short = 'q', long, global = true)]
    pub quiet: bool,
}

impl GlobalArgs {
    /// Moon `--log` level implied by the verbosity count, if any.
    pub fn log_level(&self) -> Option<&'static str> {
        match self.verbose {
            0 => None,
            1 => Some("debug"),
            _ => Some("trace"),
        }
    }
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    /// Build projects (`moon run :build`).
    Build(TaskArgs),
    /// Start dev servers (`moon run :dev`).
    Dev(ProjectArgs),
    /// Start production servers (`moon run :start`).
    Start(ProjectArgs),
    /// Run tests (`moon run :test`).
    Test(TaskArgs),
    /// Run one or more Moon targets directly (`moon run <targets...>`).
    Run(RunArgs),
    /// Display the project graph (`moon project-graph`).
    Graph,
    /// List all tasks (`moon tasks`).
    Tasks,
    /// List all projects (`moon projects`).
    Projects,
    /// Run all affected tasks in a CI environment (`moon ci`).
    Ci(PassthroughArgs),
    /// Bootstrap the workspace (proto + CLI + bun + moon builds).
    Install,
    /// Full reset: apps/packages, Moon cache, then root gitignored outputs.
    Clean,
    /// Lint all stacks (TS: oxlint, Python: ruff, Rust: clippy).
    Lint(FixArgs),
    /// Format all stacks (TS: oxfmt, Python: ruff, Rust: cargo fmt).
    Format(CheckArgs),
    /// Typecheck all stacks (TS: tsc, Go: hugo config).
    Typecheck,
    /// Lint + format check + typecheck.
    Check,
    /// Lint fix + format.
    Fix,
    /// Report outdated toolchains and dependencies (exit 1 if any are outdated).
    Outdated,
    /// Update toolchains and dependencies, then re-run install.
    Update(UpdateArgs),
}

#[derive(Debug, Clone, Args)]
pub struct TaskArgs {
    /// Limit to a single project (e.g. `app`); omit for all application-layer projects.
    pub project: Option<String>,

    /// Only run tasks affected by changed files (`moon --affected`).
    #[arg(long)]
    pub affected: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ProjectArgs {
    /// Limit to a single project (e.g. `app`); omit for all application-layer projects.
    pub project: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct RunArgs {
    /// Moon targets to run, e.g. `app:build api:dev`.
    #[arg(required = true, trailing_var_arg = true, allow_hyphen_values = true)]
    pub targets: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct PassthroughArgs {
    /// Extra arguments forwarded verbatim to the underlying command.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct FixArgs {
    /// Apply fixes (`lint:fix`) instead of reporting only.
    #[arg(long)]
    pub fix: bool,
}

#[derive(Debug, Clone, Args)]
pub struct CheckArgs {
    /// Check formatting (`format:check`) instead of writing changes.
    #[arg(long)]
    pub check: bool,
}

#[derive(Debug, Clone, Args)]
pub struct UpdateArgs {
    /// Also apply major-version bumps where the ecosystem supports them.
    #[arg(long)]
    pub major: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_build_no_args() {
        let cli = Cli::try_parse_from(["luna", "build"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Build(TaskArgs {
                project: None,
                affected: false
            })
        ));
    }

    #[test]
    fn parse_build_with_project() {
        let cli = Cli::try_parse_from(["luna", "build", "app"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Build(TaskArgs {
                project: Some(_),
                ..
            })
        ));
    }

    #[test]
    fn parse_build_affected() {
        let cli = Cli::try_parse_from(["luna", "build", "--affected"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Build(TaskArgs { affected: true, .. })
        ));
    }

    #[test]
    fn parse_verbose_quiet() {
        let cli = Cli::try_parse_from(["luna", "-v", "-q", "tasks"]).unwrap();
        assert_eq!(cli.global.verbose, 1);
        assert!(cli.global.quiet);
    }

    #[test]
    fn parse_update_major() {
        let cli = Cli::try_parse_from(["luna", "update", "--major"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Update(UpdateArgs { major: true })
        ));
    }

    #[test]
    fn log_level_mapping() {
        let g0 = GlobalArgs {
            verbose: 0,
            quiet: false,
        };
        assert!(g0.log_level().is_none());

        let g1 = GlobalArgs {
            verbose: 1,
            quiet: false,
        };
        assert_eq!(g1.log_level(), Some("debug"));

        let g2 = GlobalArgs {
            verbose: 2,
            quiet: false,
        };
        assert_eq!(g2.log_level(), Some("trace"));
    }

    #[test]
    fn parse_run_targets() {
        let cli = Cli::try_parse_from(["luna", "run", "app:build", "api:test"]).unwrap();
        if let Commands::Run(args) = cli.command {
            assert_eq!(args.targets, vec!["app:build", "api:test"]);
        } else {
            panic!("expected Run command");
        }
    }

    #[test]
    fn parse_binary_name_alias() {
        let cli = Cli::try_parse_from(["l", "check"]).unwrap();
        assert!(matches!(cli.command, Commands::Check));

        let cli = Cli::try_parse_from(["ln", "build", "--affected"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Build(TaskArgs { affected: true, .. })
        ));
    }
}
