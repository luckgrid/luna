use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::planner::Step;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Inspect,
    Plan,
    Apply,
    Offline,
    Networked,
}

impl ExecutionMode {
    pub fn from_config_and_flag(_config: &LunaConfig, global: &GlobalArgs) -> Self {
        if let Some(mode) = &global.mode {
            return Self::parse_mode(mode);
        }
        // CLI commands apply by default; `[agent].safe_mode_default` is for MCP only.
        Self::Apply
    }

    pub fn from_agent_config(config: &LunaConfig) -> Self {
        Self::parse_mode(&config.agent.safe_mode_default)
    }

    pub fn parse_mode(s: &str) -> Self {
        match s {
            "plan" => Self::Plan,
            "apply" => Self::Apply,
            "offline" => Self::Offline,
            "networked" => Self::Networked,
            _ => Self::Inspect,
        }
    }

    pub fn allows_mutation(self) -> bool {
        matches!(self, Self::Apply | Self::Networked)
    }

    pub fn allows_network(self, step: &Step) -> bool {
        if self == Self::Offline {
            return step.safety != "network-required";
        }
        true
    }
}

pub fn should_execute_step(mode: ExecutionMode, step: &Step, dry_run: bool) -> Result<(), String> {
    if dry_run || matches!(mode, ExecutionMode::Inspect | ExecutionMode::Plan) {
        return Err(format!("dry-run: skip step {}", step.id));
    }
    if !mode.allows_mutation() && step.mutability != "read-only" {
        return Err(format!("mode {:?} blocks mutating step {}", mode, step.id));
    }
    if !mode.allows_network(step) {
        return Err(format!("offline mode blocks network step {}", step.id));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offline_blocks_network() {
        let step = Step {
            id: "x".into(),
            adapter: "bun".into(),
            program: "bun".into(),
            args: vec![],
            cwd: None,
            env_selector: None,
            depends_on: vec![],
            cacheability: "none".into(),
            mutability: "install".into(),
            safety: "network-required".into(),
        };
        assert!(!ExecutionMode::Offline.allows_network(&step));
    }

    #[test]
    fn cli_defaults_to_apply_without_mode_flag() {
        use crate::cli::Cli;
        use crate::config::LunaConfig;
        use clap::Parser;

        let config = LunaConfig {
            agent: crate::config::schema::AgentConfig {
                safe_mode_default: "inspect".into(),
                ..Default::default()
            },
            ..Default::default()
        };
        let cli = Cli::try_parse_from(["luna", "doctor"]).unwrap();
        assert_eq!(
            ExecutionMode::from_config_and_flag(&config, &cli.global),
            ExecutionMode::Apply
        );
    }
}
