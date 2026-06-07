use crate::config::schema::*;
use miette::Result;
use std::path::Path;

/// Construct a `LunaConfig` from the current repo's legacy files
/// (`.prototools`, `.moon/*`, native manifests) when no `luna.toml` exists.
pub fn from_legacy(root: &Path) -> Result<LunaConfig> {
    let name = root
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("luna")
        .to_string();

    let mut uv_members = Vec::new();
    if root.join("pyproject.toml").is_file() {
        if let Ok(text) = std::fs::read_to_string(root.join("pyproject.toml")) {
            if let Ok(doc) = toml::from_str::<toml::Table>(&text) {
                if let Some(members) = doc
                    .get("tool")
                    .and_then(|t| t.get("uv"))
                    .and_then(|u| u.get("workspace"))
                    .and_then(|w| w.get("members"))
                    .and_then(|m| m.as_array())
                {
                    for member in members {
                        if let Some(s) = member.as_str() {
                            uv_members.push(s.to_string());
                        }
                    }
                }
            }
        }
    }

    let pixi_enabled = root.join("pixi.toml").is_file();

    let min_release_age = std::env::var("LUNA_MIN_RELEASE_AGE")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&d: &u64| d > 0)
        .unwrap_or(14);

    let firewall = if std::env::var("LUNA_FIREWALL")
        .map(|v| !v.is_empty() && v != "0" && v != "false")
        .unwrap_or(false)
    {
        FirewallPolicy::Socket
    } else {
        FirewallPolicy::OptIn
    };

    let frozen_ci = root.join("pixi.toml").is_file();

    Ok(LunaConfig {
        schema: SCHEMA_VERSION,
        workspace: WorkspaceConfig {
            name,
            root: ".".into(),
            default_profile: "dev".into(),
        },
        state: StateConfig::default(),
        bootstrap: BootstrapConfig {
            provider: if pixi_enabled {
                "pixi".into()
            } else {
                "proto".into()
            },
            compat_providers: if pixi_enabled {
                vec!["proto".into()]
            } else {
                Vec::new()
            },
            auto_install_pixi: pixi_enabled,
        },
        policy: PolicyConfig {
            min_release_age_days: min_release_age,
            firewall,
            network_default: "restricted".into(),
            allow_git_dependencies: false,
            ignore_lifecycle_scripts: true,
            frozen_ci,
        },
        adapters: AdaptersConfig {
            bun: BunAdapterConfig::default(),
            uv: UvAdapterConfig {
                workspace_members: uv_members,
                ..UvAdapterConfig::default()
            },
            cargo: CargoAdapterConfig::default(),
            go: GoAdapterConfig::default(),
            pixi: PixiAdapterConfig {
                enabled: pixi_enabled,
                ..PixiAdapterConfig::default()
            },
        },
        compat: CompatConfig {
            moon: MoonCompatConfig {
                enabled: root.join(".moon").is_dir(),
                ..MoonCompatConfig::default()
            },
        },
        commands: CommandsConfig::default(),
        agent: AgentConfig::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn from_legacy_on_repo_root() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        std::env::remove_var("LUNA_MIN_RELEASE_AGE");
        std::env::remove_var("LUNA_FIREWALL");
        let config = from_legacy(&root).unwrap();
        assert_eq!(config.schema, SCHEMA_VERSION);
        assert_eq!(config.policy.min_release_age_days, 14);
        assert!(config.adapters.bun.manifest == "package.json");
        assert!(config.adapters.go.workspace == "go.work");
    }
}
