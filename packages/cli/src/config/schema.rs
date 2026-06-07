use serde::{Deserialize, Serialize};

/// Schema version constant — bump when making breaking config changes.
pub const SCHEMA_VERSION: u32 = 1;

/// Root `luna.toml` configuration model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LunaConfig {
    /// Schema version for forward compatibility.
    pub schema: u32,

    #[serde(default)]
    pub workspace: WorkspaceConfig,

    #[serde(default)]
    pub state: StateConfig,

    #[serde(default)]
    pub bootstrap: BootstrapConfig,

    #[serde(default)]
    pub policy: PolicyConfig,

    #[serde(default)]
    pub adapters: AdaptersConfig,

    #[serde(default)]
    pub compat: CompatConfig,

    #[serde(default)]
    pub commands: CommandsConfig,

    #[serde(default)]
    pub agent: AgentConfig,
}

impl Default for LunaConfig {
    fn default() -> Self {
        Self {
            schema: SCHEMA_VERSION,
            workspace: WorkspaceConfig::default(),
            state: StateConfig::default(),
            bootstrap: BootstrapConfig::default(),
            policy: PolicyConfig::default(),
            adapters: AdaptersConfig::default(),
            compat: CompatConfig::default(),
            commands: CommandsConfig::default(),
            agent: AgentConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    #[serde(default = "default_name")]
    pub name: String,
    #[serde(default = "default_root")]
    pub root: String,
    #[serde(default = "default_profile")]
    pub default_profile: String,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            name: default_name(),
            root: default_root(),
            default_profile: default_profile(),
        }
    }
}

fn default_name() -> String {
    "luna".into()
}
fn default_root() -> String {
    ".".into()
}
fn default_profile() -> String {
    "dev".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateConfig {
    #[serde(default = "default_state_dir")]
    pub dir: String,
    #[serde(default = "default_snapshot_ttl")]
    pub snapshot_ttl_hours: u64,
}

impl Default for StateConfig {
    fn default() -> Self {
        Self {
            dir: default_state_dir(),
            snapshot_ttl_hours: default_snapshot_ttl(),
        }
    }
}

fn default_state_dir() -> String {
    ".luna".into()
}
fn default_snapshot_ttl() -> u64 {
    8
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    #[serde(default = "default_bootstrap_provider")]
    pub provider: String,
    #[serde(default = "default_compat_providers")]
    pub compat_providers: Vec<String>,
    #[serde(default = "default_true_val")]
    pub auto_install_pixi: bool,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            provider: default_bootstrap_provider(),
            compat_providers: default_compat_providers(),
            auto_install_pixi: default_true_val(),
        }
    }
}

fn default_bootstrap_provider() -> String {
    "pixi".into()
}
fn default_compat_providers() -> Vec<String> {
    vec!["proto".into()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyConfig {
    #[serde(default = "default_min_release_age")]
    pub min_release_age_days: u64,
    #[serde(default)]
    pub firewall: FirewallPolicy,
    #[serde(default = "default_network")]
    pub network_default: String,
    #[serde(default)]
    pub allow_git_dependencies: bool,
    #[serde(default = "default_true")]
    pub ignore_lifecycle_scripts: bool,
    #[serde(default)]
    pub frozen_ci: bool,
}

impl Default for PolicyConfig {
    fn default() -> Self {
        Self {
            min_release_age_days: default_min_release_age(),
            firewall: FirewallPolicy::default(),
            network_default: default_network(),
            allow_git_dependencies: false,
            ignore_lifecycle_scripts: default_true(),
            frozen_ci: false,
        }
    }
}

fn default_min_release_age() -> u64 {
    14
}
fn default_network() -> String {
    "restricted".into()
}
fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum FirewallPolicy {
    #[serde(rename = "off")]
    Off,
    #[serde(rename = "opt-in")]
    #[default]
    OptIn,
    #[serde(rename = "socket")]
    Socket,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdaptersConfig {
    #[serde(default)]
    pub bun: BunAdapterConfig,
    #[serde(default)]
    pub uv: UvAdapterConfig,
    #[serde(default)]
    pub cargo: CargoAdapterConfig,
    #[serde(default)]
    pub go: GoAdapterConfig,
    #[serde(default)]
    pub pixi: PixiAdapterConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BunAdapterConfig {
    #[serde(default = "default_bun_manifest")]
    pub manifest: String,
    #[serde(default = "default_bun_lockfile")]
    pub lockfile: String,
    #[serde(default = "default_workspace_globs")]
    pub workspace_globs: Vec<String>,
}

impl Default for BunAdapterConfig {
    fn default() -> Self {
        Self {
            manifest: default_bun_manifest(),
            lockfile: default_bun_lockfile(),
            workspace_globs: default_workspace_globs(),
        }
    }
}

fn default_bun_manifest() -> String {
    "package.json".into()
}
fn default_bun_lockfile() -> String {
    "bun.lock".into()
}
fn default_workspace_globs() -> Vec<String> {
    vec!["apps/*".into(), "packages/*".into()]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UvAdapterConfig {
    #[serde(default = "default_uv_manifest")]
    pub manifest: String,
    #[serde(default = "default_uv_lockfile")]
    pub lockfile: String,
    #[serde(default)]
    pub workspace_members: Vec<String>,
}

impl Default for UvAdapterConfig {
    fn default() -> Self {
        Self {
            manifest: default_uv_manifest(),
            lockfile: default_uv_lockfile(),
            workspace_members: Vec::new(),
        }
    }
}

fn default_uv_manifest() -> String {
    "pyproject.toml".into()
}
fn default_uv_lockfile() -> String {
    "uv.lock".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CargoAdapterConfig {
    #[serde(default = "default_cargo_manifest")]
    pub manifest: String,
    #[serde(default = "default_cargo_lockfile")]
    pub lockfile: String,
}

impl Default for CargoAdapterConfig {
    fn default() -> Self {
        Self {
            manifest: default_cargo_manifest(),
            lockfile: default_cargo_lockfile(),
        }
    }
}

fn default_cargo_manifest() -> String {
    "Cargo.toml".into()
}
fn default_cargo_lockfile() -> String {
    "Cargo.lock".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoAdapterConfig {
    #[serde(default = "default_go_workspace")]
    pub workspace: String,
}

impl Default for GoAdapterConfig {
    fn default() -> Self {
        Self {
            workspace: default_go_workspace(),
        }
    }
}

fn default_go_workspace() -> String {
    "go.work".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PixiAdapterConfig {
    #[serde(default = "default_pixi_manifest")]
    pub manifest: String,
    #[serde(default = "default_true_val")]
    pub enabled: bool,
}

impl Default for PixiAdapterConfig {
    fn default() -> Self {
        Self {
            manifest: default_pixi_manifest(),
            enabled: default_true_val(),
        }
    }
}

fn default_pixi_manifest() -> String {
    "pixi.toml".into()
}
fn default_true_val() -> bool {
    true
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompatConfig {
    #[serde(default)]
    pub moon: MoonCompatConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoonCompatConfig {
    #[serde(default = "default_true_val")]
    pub enabled: bool,
    #[serde(default = "default_moon_workspace")]
    pub workspace: String,
}

impl Default for MoonCompatConfig {
    fn default() -> Self {
        Self {
            enabled: default_true_val(),
            workspace: default_moon_workspace(),
        }
    }
}

fn default_moon_workspace() -> String {
    ".moon/workspace.yml".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommandsConfig {
    #[serde(default)]
    pub build: CommandDefaults,
    #[serde(default)]
    pub dev: CommandDefaults,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandDefaults {
    #[serde(default = "default_scope")]
    pub default_scope: String,
    #[serde(default)]
    pub watcher: Option<String>,
}

impl Default for CommandDefaults {
    fn default() -> Self {
        Self {
            default_scope: default_scope(),
            watcher: None,
        }
    }
}

fn default_scope() -> String {
    "applications".into()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    #[serde(default)]
    pub json_default: bool,
    #[serde(default = "default_safe_mode")]
    pub safe_mode_default: String,
    #[serde(default)]
    pub mcp: bool,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            json_default: false,
            safe_mode_default: default_safe_mode(),
            mcp: false,
        }
    }
}

fn default_safe_mode() -> String {
    "inspect".into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_config() {
        let text = "schema = 1";
        let config: LunaConfig = toml::from_str(text).unwrap();
        assert_eq!(config.schema, 1);
        assert_eq!(config.workspace.name, "luna");
        assert_eq!(config.state.dir, ".luna");
        assert_eq!(config.policy.min_release_age_days, 14);
        assert!(config.adapters.pixi.enabled);
        assert!(config.compat.moon.enabled);
    }

    #[test]
    fn parse_full_config() {
        let text = r#"
schema = 1

[workspace]
name = "my-project"
root = "."
default_profile = "ci"

[state]
dir = ".luna"
snapshot_ttl_hours = 4

[bootstrap]
provider = "pixi"
compat_providers = ["proto"]

[policy]
min_release_age_days = 7
firewall = "socket"
network_default = "restricted"
allow_git_dependencies = false
ignore_lifecycle_scripts = true
frozen_ci = true

[adapters.bun]
manifest = "package.json"
lockfile = "bun.lock"
workspace_globs = ["apps/*", "packages/*"]

[adapters.uv]
manifest = "pyproject.toml"
lockfile = "uv.lock"
workspace_members = ["apps/api", "packages/py-demo"]

[adapters.cargo]
manifest = "Cargo.toml"
lockfile = "Cargo.lock"

[adapters.go]
workspace = "go.work"

[adapters.pixi]
manifest = "pixi.toml"
enabled = true

[compat.moon]
enabled = true
workspace = ".moon/workspace.yml"

[commands.build]
default_scope = "applications"

[commands.dev]
default_scope = "applications"
watcher = "watchexec"

[agent]
json_default = true
safe_mode_default = "plan"
mcp = false
"#;
        let config: LunaConfig = toml::from_str(text).unwrap();
        assert_eq!(config.schema, 1);
        assert_eq!(config.workspace.name, "my-project");
        assert_eq!(config.workspace.default_profile, "ci");
        assert_eq!(config.state.snapshot_ttl_hours, 4);
        assert_eq!(config.policy.min_release_age_days, 7);
        assert!(config.policy.frozen_ci);
        assert_eq!(config.adapters.uv.workspace_members.len(), 2);
        assert_eq!(config.commands.dev.watcher.as_deref(), Some("watchexec"));
        assert!(config.agent.json_default);
    }

    #[test]
    fn roundtrip_config() {
        let config = LunaConfig {
            schema: 1,
            workspace: WorkspaceConfig::default(),
            state: StateConfig::default(),
            bootstrap: BootstrapConfig::default(),
            policy: PolicyConfig::default(),
            adapters: AdaptersConfig::default(),
            compat: CompatConfig::default(),
            commands: CommandsConfig::default(),
            agent: AgentConfig::default(),
        };
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: LunaConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(deserialized.schema, config.schema);
        assert_eq!(deserialized.workspace.name, config.workspace.name);
        assert_eq!(deserialized.state.dir, config.state.dir);
    }

    #[test]
    fn firewall_policy_values() {
        let off: FirewallPolicy = serde_json::from_str("\"off\"").unwrap();
        assert!(matches!(off, FirewallPolicy::Off));
        let socket: FirewallPolicy = serde_json::from_str("\"socket\"").unwrap();
        assert!(matches!(socket, FirewallPolicy::Socket));
    }
}
