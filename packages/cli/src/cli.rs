use clap::{Args, Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(
    name = "luna",
    version,
    about = "Luna monorepo CLI — policy-rich control plane for polyglot workspaces.",
    propagate_version = true,
    disable_help_subcommand = false
)]
pub struct Cli {
    #[command(flatten)]
    pub global: GlobalArgs,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Copy, ValueEnum, Default, PartialEq, Eq)]
pub enum Backend {
    #[default]
    Auto,
    Pixi,
    Moon,
    Native,
}

#[derive(Debug, Clone, Args)]
pub struct GlobalArgs {
    /// Increase logging verbosity (maps to `moon --log debug`).
    #[arg(short = 'v', long, global = true, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Silence Luna and Moon output (maps to `moon -q`).
    #[arg(short = 'q', long, global = true)]
    pub quiet: bool,

    /// Wrap supported package-manager commands with Socket Firewall (`sfw`).
    #[arg(long, global = true, env = "LUNA_FIREWALL", help_heading = "Security")]
    pub firewall: bool,

    /// Emit machine-readable JSON output (schema-versioned, ANSI-free).
    #[arg(long, global = true, help_heading = "Output")]
    pub json: bool,

    /// Print the execution plan without applying changes.
    #[arg(long, global = true, help_heading = "Execution")]
    pub dry_run: bool,

    /// Enforce locked/frozen install semantics for underlying package managers.
    #[arg(long, global = true, help_heading = "Execution")]
    pub locked: bool,

    /// Alias for `--locked` with stricter CI semantics.
    #[arg(long, global = true, help_heading = "Execution")]
    pub frozen: bool,

    /// Working directory before workspace root detection.
    #[arg(long, global = true, value_name = "PATH", help_heading = "Execution")]
    pub cwd: Option<PathBuf>,

    /// Select execution backend (auto, pixi, moon, native).
    #[arg(long, global = true, value_enum, default_value_t = Backend::Auto, help_heading = "Execution")]
    pub backend: Backend,

    /// Only run tasks affected by changed files (`moon --affected`).
    #[arg(long, global = true, help_heading = "Execution")]
    pub affected: bool,

    /// Disable planner/task caching.
    #[arg(long, global = true, help_heading = "Execution")]
    pub no_cache: bool,

    /// Emit structured JSONL trace events to stderr.
    #[arg(long, global = true, help_heading = "Output")]
    pub trace: bool,

    /// Execution mode: inspect, plan, apply, offline, networked.
    #[arg(long, global = true, help_heading = "Execution")]
    pub mode: Option<String>,
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
    /// Project/task graph operations (legacy: bare `graph` shows project graph).
    Graph {
        #[command(subcommand)]
        command: Option<GraphCommands>,
    },
    /// List all tasks (legacy alias for `task list`).
    Tasks,
    /// List all projects (legacy alias for `graph project`).
    Projects,
    /// Run all affected tasks in a CI environment (`moon ci`).
    Ci(PassthroughArgs),
    /// Bootstrap the workspace (proto + CLI + bun + moon builds).
    Install(InstallArgs),
    /// Reconcile workspace state (Pixi + proto + native PMs).
    Sync(SyncArgs),
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
    /// Probe all toolchains in parallel and cache a snapshot.
    Outdated,
    /// Update only outdated toolchains (snapshot-first).
    Update(UpdateArgs),
    /// Validate workspace health: manifests, lockfiles, tools, config consistency.
    Doctor(DoctorArgs),
    /// Render a structured execution plan for a target.
    Plan(PlanArgs),
    /// Workspace configuration operations.
    Config {
        #[command(subcommand)]
        command: ConfigCommands,
    },
    /// Environment inspection and sync (Pixi-backed).
    Env {
        #[command(subcommand)]
        command: EnvCommands,
    },
    /// Task graph operations.
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },
    /// Delegate package add/remove to native PMs.
    Pkg {
        #[command(subcommand)]
        command: PkgCommands,
    },
    /// Reconcile lockfiles and write `.luna/lock-ledger.json`.
    Lock,
    /// Export dependency inventory / SBOM.
    Sbom(SbomArgs),
    /// Alias for `sbom`.
    Inventory(SbomArgs),
    /// Write canonical `luna.toml` from legacy repo files.
    Migrate(MigrateArgs),
    /// Initialize workspace config (`migrate` + optional `pixi.toml` scaffold).
    Init(MigrateArgs),
    /// Apply a saved plan file (rejects stale fingerprints).
    Apply(ApplyArgs),
    /// Generate shell completions.
    Completions(CompletionsArgs),
    /// Agent / MCP bridge.
    Agent {
        #[command(subcommand)]
        command: AgentCommands,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum ConfigCommands {
    /// Validate luna.toml against schema and on-disk repo state.
    Validate,
    /// Print normalized luna.toml (use `--json` for structured output).
    Print,
}

#[derive(Debug, Clone, Subcommand)]
pub enum EnvCommands {
    /// List available Pixi environments.
    List,
    /// Sync/install a Pixi environment.
    Sync(EnvSyncArgs),
    /// Run a command in a Pixi environment.
    Exec(EnvExecArgs),
}

#[derive(Debug, Clone, Subcommand)]
pub enum TaskCommands {
    /// List all Moon tasks.
    List,
    /// Run a Moon target.
    Run(TaskRunArgs),
}

#[derive(Debug, Clone, Subcommand)]
pub enum GraphCommands {
    /// Emit the project graph.
    Project,
    /// Emit the task graph for a target.
    Task(GraphTaskArgs),
}

#[derive(Debug, Clone, Subcommand)]
pub enum PkgCommands {
    /// Add a dependency via the native PM for a project.
    Add(PkgAddArgs),
    /// Remove a dependency via the native PM for a project.
    Remove(PkgRemoveArgs),
}

#[derive(Debug, Clone, Args)]
pub struct TaskArgs {
    /// Limit to a single project (e.g. `app`); omit for all application-layer projects.
    pub project: Option<String>,
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

#[derive(Debug, Clone, Args)]
pub struct InstallArgs {
    /// Install workspace deps only; skip CLI bootstrap (for CI).
    #[arg(long)]
    pub workspace: bool,
}

#[derive(Debug, Clone, Args)]
pub struct SyncArgs {
    /// Install workspace deps only; skip CLI bootstrap (for CI).
    #[arg(long)]
    pub workspace: bool,
}

#[derive(Debug, Clone, Args)]
pub struct PlanArgs {
    /// Target to plan (e.g. build, test, sync, outdated).
    pub target: String,
    /// Write plan JSON (with fingerprint) to this path.
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Debug, Clone, Args)]
pub struct SbomArgs {
    /// Output format: luna (default) or cyclonedx.
    #[arg(long, default_value = "luna")]
    pub format: String,
}

#[derive(Debug, Clone, Args)]
pub struct MigrateArgs {
    /// Overwrite existing luna.toml.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, Clone, Args)]
pub struct ApplyArgs {
    /// Path to plan JSON written by `luna plan --out`.
    pub plan_file: PathBuf,
}

#[derive(Debug, Clone, Args)]
pub struct CompletionsArgs {
    /// Shell: bash, zsh, fish, powershell, elvish.
    pub shell: String,
}

#[derive(Debug, Clone, Subcommand)]
pub enum AgentCommands {
    /// Start stdio MCP server (requires `[agent].mcp = true`).
    Mcp,
}

#[derive(Debug, Clone, Args)]
pub struct DoctorArgs {
    /// CI mode: fail on warnings when frozen policy is enabled.
    #[arg(long)]
    pub ci: bool,
}

#[derive(Debug, Clone, Args)]
pub struct EnvSyncArgs {
    /// Pixi environment name.
    #[arg(short = 'e', long)]
    pub environment: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct EnvExecArgs {
    /// Pixi environment name.
    #[arg(short = 'e', long)]
    pub environment: Option<String>,
    /// Command and arguments to run.
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub command: Vec<String>,
}

#[derive(Debug, Clone, Args)]
pub struct TaskRunArgs {
    /// Moon target to run.
    pub target: String,
}

#[derive(Debug, Clone, Args)]
pub struct GraphTaskArgs {
    /// Task target for the graph.
    pub target: String,
}

#[derive(Debug, Clone, Args)]
pub struct PkgAddArgs {
    /// Package name to add.
    pub package: String,
    /// Project id (defaults to cwd project).
    #[arg(long)]
    pub project: Option<String>,
}

#[derive(Debug, Clone, Args)]
pub struct PkgRemoveArgs {
    /// Package name to remove.
    pub package: String,
    /// Project id (defaults to cwd project).
    #[arg(long)]
    pub project: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parse_build_no_args() {
        let cli = Cli::try_parse_from(["luna", "build"]).unwrap();
        assert!(matches!(cli.command, Commands::Build(_)));
    }

    #[test]
    fn parse_sync() {
        let cli = Cli::try_parse_from(["luna", "sync"]).unwrap();
        assert!(matches!(cli.command, Commands::Sync(_)));
    }

    #[test]
    fn parse_config_validate() {
        let cli = Cli::try_parse_from(["luna", "config", "validate"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Config {
                command: ConfigCommands::Validate
            }
        ));
    }

    #[test]
    fn parse_env_list() {
        let cli = Cli::try_parse_from(["luna", "env", "list"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Env {
                command: EnvCommands::List
            }
        ));
    }

    #[test]
    fn parse_global_flags() {
        let cli = Cli::try_parse_from([
            "luna",
            "--json",
            "--dry-run",
            "--locked",
            "--trace",
            "--cwd",
            "/tmp",
            "plan",
            "sync",
        ])
        .unwrap();
        assert!(cli.global.json);
        assert!(cli.global.dry_run);
        assert!(cli.global.locked);
        assert!(cli.global.trace);
        assert_eq!(
            cli.global.cwd.as_deref(),
            Some(PathBuf::from("/tmp").as_path())
        );
    }

    #[test]
    fn parse_graph_project() {
        let cli = Cli::try_parse_from(["luna", "graph", "project"]).unwrap();
        assert!(matches!(
            cli.command,
            Commands::Graph {
                command: Some(GraphCommands::Project)
            }
        ));
    }

    #[test]
    fn parse_graph_legacy() {
        let cli = Cli::try_parse_from(["luna", "graph"]).unwrap();
        assert!(matches!(cli.command, Commands::Graph { command: None }));
    }

    #[test]
    fn parse_task_run() {
        let cli = Cli::try_parse_from(["luna", "task", "run", "app:build"]).unwrap();
        if let Commands::Task {
            command: TaskCommands::Run(args),
        } = cli.command
        {
            assert_eq!(args.target, "app:build");
        } else {
            panic!("expected task run");
        }
    }
}
