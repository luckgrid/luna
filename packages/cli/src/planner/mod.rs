pub mod cache;
pub mod plan;
pub mod step;

pub use cache::PlanCacheKey;
pub use plan::Plan;
pub use step::Step;

use crate::adapters::{registry, AdapterKind, SyncOpts};
use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::execution::{self, ExecutionMode};
use crate::observability;
use crate::systems::state;
use miette::{miette, Result};
use std::path::Path;

/// Build a plan for the given Luna target verb (e.g. "build", "test", "lint").
pub fn build_plan(root: &Path, config: &LunaConfig, target: &str) -> Result<Plan> {
    let steps = match target {
        "sync" | "install" => plan_sync(root, config),
        "build" => plan_build(root, config),
        "dev" => plan_dev(root, config),
        "test" => plan_test(root, config),
        "lint" => plan_lint(root, config),
        "format" => plan_format(root, config),
        "typecheck" => plan_typecheck(root, config),
        "check" => plan_check(root, config),
        "fix" => plan_fix(root, config),
        "outdated" => plan_outdated(root, config),
        "update" => plan_update(root, config),
        _ => Err(miette!("unknown plan target: {target}")),
    }?;
    Ok(Plan {
        target: target.to_string(),
        workspace_root: root.display().to_string(),
        steps,
        fingerprint: None,
    })
}

/// Execute a plan's steps in dependency order via backend adapters.
pub fn execute(root: &Path, config: &LunaConfig, plan: &Plan, global: &GlobalArgs) -> Result<i32> {
    let mode = execution::ExecutionMode::from_config_and_flag(config, global);
    let locked = global.locked || global.frozen || config.policy.frozen_ci;
    let sync_opts = SyncOpts {
        locked,
        quiet: global.quiet,
    };

    observability::event(
        global,
        "plan.execute.start",
        serde_json::json!({ "target": plan.target, "steps": plan.steps.len(), "mode": format!("{:?}", mode) }),
    );

    let mut completed = std::collections::HashSet::new();
    let cache_root = state::cache_dir(root, config);

    for step in &plan.steps {
        for dep in &step.depends_on {
            if !completed.contains(dep) {
                return Err(miette!(
                    "plan step {} depends on {dep} which has not completed",
                    step.id
                ));
            }
        }

        if global.dry_run || matches!(mode, ExecutionMode::Inspect | ExecutionMode::Plan) {
            if !global.quiet && !global.json {
                eprintln!(
                    "  [dry-run] {} {} {}",
                    step.adapter,
                    step.program,
                    step.args.join(" ")
                );
            }
            completed.insert(step.id.clone());
            continue;
        }

        if !mode.allows_mutation() && step.mutability != "read-only" {
            return Err(miette!("execution mode blocks mutating step {}", step.id));
        }
        if !mode.allows_network(step) {
            return Err(miette!("offline mode blocks network step {}", step.id));
        }

        if step.cacheability == "local" && !global.no_cache {
            let key = cache_key_for_step(root, config, step);
            let hit = cache_root.join(key.fingerprint());
            if hit.is_file() {
                observability::event(
                    global,
                    "plan.step.cache_hit",
                    serde_json::json!({ "id": step.id }),
                );
                completed.insert(step.id.clone());
                continue;
            }
        }

        let kind = AdapterKind::from_label(&step.adapter);
        let adapter = registry::get(kind);
        observability::event(
            global,
            "plan.step.start",
            serde_json::json!({ "id": step.id, "adapter": step.adapter }),
        );

        let code = adapter.run_step(root, config, step, global, sync_opts)?;
        if code != 0 {
            observability::event(
                global,
                "plan.step.failed",
                serde_json::json!({ "id": step.id, "exit_code": code }),
            );
            return Ok(code);
        }

        if step.cacheability == "local" && !global.no_cache {
            let key = cache_key_for_step(root, config, step);
            let hit = cache_root.join(key.fingerprint());
            if let Some(parent) = hit.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            let _ = std::fs::write(&hit, b"ok");
        }

        completed.insert(step.id.clone());
        observability::event(
            global,
            "plan.step.done",
            serde_json::json!({ "id": step.id }),
        );
    }

    observability::event(global, "plan.execute.done", serde_json::json!({}));
    Ok(0)
}

fn cache_key_for_step(root: &Path, _config: &LunaConfig, step: &Step) -> PlanCacheKey {
    let manifests = crate::systems::snapshot::fingerprint_manifests_public(root);
    PlanCacheKey {
        step_id: step.id.clone(),
        adapter: step.adapter.clone(),
        normalized_command: format!("{} {}", step.program, step.args.join(" ")),
        manifest_hashes: manifests.iter().map(|m| m.sha256.clone()).collect(),
        lockfile_hashes: Vec::new(),
        env_identity: step
            .env_selector
            .clone()
            .unwrap_or_else(|| "default".into()),
        platform_key: std::env::consts::ARCH.into(),
    }
}

fn plan_sync(root: &Path, config: &LunaConfig) -> Result<Vec<Step>> {
    let mut steps = Vec::new();

    let pixi_wanted =
        config.adapters.pixi.enabled && root.join(&config.adapters.pixi.manifest).is_file();
    let needs_proto =
        config.compat.moon.enabled || (pixi_wanted && config.bootstrap.auto_install_pixi);

    if needs_proto {
        steps.push(Step {
            id: "proto:install".into(),
            adapter: "proto".into(),
            program: "proto".into(),
            args: vec!["install".into()],
            cwd: None,
            env_selector: None,
            depends_on: Vec::new(),
            cacheability: "none".into(),
            mutability: "install".into(),
            safety: "network-required".into(),
        });
    }

    if config.compat.moon.enabled {
        steps.push(Step {
            id: "moon:build-cli".into(),
            adapter: "moon".into(),
            program: "moon".into(),
            args: vec!["run".into(), "cli:build".into()],
            cwd: None,
            env_selector: None,
            depends_on: if needs_proto {
                vec!["proto:install".into()]
            } else {
                Vec::new()
            },
            cacheability: "local".into(),
            mutability: "generate".into(),
            safety: "network-required".into(),
        });

        steps.push(Step {
            id: "moon:install-cli".into(),
            adapter: "moon".into(),
            program: "moon".into(),
            args: vec!["run".into(), "cli:install".into()],
            cwd: None,
            env_selector: None,
            depends_on: vec!["moon:build-cli".into()],
            cacheability: "none".into(),
            mutability: "install".into(),
            safety: "offline-safe".into(),
        });
    }

    if pixi_wanted {
        let pixi_depends = if config.compat.moon.enabled {
            vec!["moon:install-cli".into()]
        } else if needs_proto {
            vec!["proto:install".into()]
        } else {
            Vec::new()
        };
        steps.push(Step {
            id: "pixi:sync".into(),
            adapter: "pixi".into(),
            program: "pixi".into(),
            args: vec!["install".into()],
            cwd: None,
            env_selector: Some("default".into()),
            depends_on: pixi_depends,
            cacheability: "none".into(),
            mutability: "install".into(),
            safety: "network-required".into(),
        });
    }

    let pixi_dep = if pixi_wanted {
        vec!["pixi:sync".into()]
    } else {
        Vec::new()
    };

    if root.join(&config.adapters.bun.manifest).is_file() {
        steps.push(Step {
            id: "bun:install".into(),
            adapter: "bun".into(),
            program: "bun".into(),
            args: vec!["install".into()],
            cwd: None,
            env_selector: None,
            depends_on: pixi_dep.clone(),
            cacheability: "local".into(),
            mutability: "install".into(),
            safety: "network-required".into(),
        });
    }

    if !uv_project_dirs(root).is_empty() {
        steps.push(Step {
            id: "uv:sync".into(),
            adapter: "uv".into(),
            program: "uv".into(),
            args: vec!["sync".into()],
            cwd: None,
            env_selector: None,
            depends_on: pixi_dep.clone(),
            cacheability: "local".into(),
            mutability: "install".into(),
            safety: "network-required".into(),
        });
    }

    if root.join(&config.adapters.cargo.manifest).is_file() {
        steps.push(Step {
            id: "cargo:build".into(),
            adapter: "cargo".into(),
            program: "cargo".into(),
            args: vec!["build".into()],
            cwd: None,
            env_selector: None,
            depends_on: pixi_dep.clone(),
            cacheability: "local".into(),
            mutability: "generate".into(),
            safety: "network-required".into(),
        });
    }

    if root.join(&config.adapters.go.workspace).is_file() {
        steps.push(Step {
            id: "go:work-sync".into(),
            adapter: "go".into(),
            program: "go".into(),
            args: vec!["work".into(), "sync".into()],
            cwd: None,
            env_selector: None,
            depends_on: pixi_dep,
            cacheability: "none".into(),
            mutability: "lock-only".into(),
            safety: "network-required".into(),
        });

        steps.push(Step {
            id: "moon:web-setup".into(),
            adapter: "moon".into(),
            program: "moon".into(),
            args: vec!["run".into(), "web:setup".into()],
            cwd: None,
            env_selector: None,
            depends_on: vec!["go:work-sync".into()],
            cacheability: "local".into(),
            mutability: "install".into(),
            safety: "network-required".into(),
        });
    }

    Ok(steps)
}

fn default_scope_query(config: &LunaConfig) -> String {
    format!(
        "projectLayer={}",
        config
            .commands
            .build
            .default_scope
            .replace("applications", "application")
    )
}

fn plan_build(_root: &Path, config: &LunaConfig) -> Result<Vec<Step>> {
    if config.compat.moon.enabled {
        Ok(vec![Step {
            id: "moon:build".into(),
            adapter: "moon".into(),
            program: "moon".into(),
            args: vec![
                "run".into(),
                ":build".into(),
                "--query".into(),
                default_scope_query(config),
            ],
            cwd: None,
            env_selector: None,
            depends_on: Vec::new(),
            cacheability: "local".into(),
            mutability: "generate".into(),
            safety: "offline-safe".into(),
        }])
    } else {
        Ok(Vec::new())
    }
}

fn plan_dev(_root: &Path, config: &LunaConfig) -> Result<Vec<Step>> {
    if config.compat.moon.enabled {
        Ok(vec![Step {
            id: "moon:dev".into(),
            adapter: "moon".into(),
            program: "moon".into(),
            args: vec![
                "run".into(),
                ":dev".into(),
                "--query".into(),
                default_scope_query(config),
            ],
            cwd: None,
            env_selector: None,
            depends_on: Vec::new(),
            cacheability: "none".into(),
            mutability: "read-only".into(),
            safety: "network-required".into(),
        }])
    } else {
        Ok(Vec::new())
    }
}

fn plan_test(_root: &Path, config: &LunaConfig) -> Result<Vec<Step>> {
    if config.compat.moon.enabled {
        Ok(vec![Step {
            id: "moon:test".into(),
            adapter: "moon".into(),
            program: "moon".into(),
            args: vec![
                "run".into(),
                ":test".into(),
                "--query".into(),
                default_scope_query(config),
            ],
            cwd: None,
            env_selector: None,
            depends_on: Vec::new(),
            cacheability: "local".into(),
            mutability: "read-only".into(),
            safety: "offline-safe".into(),
        }])
    } else {
        Ok(Vec::new())
    }
}

fn plan_lint(_root: &Path, _config: &LunaConfig) -> Result<Vec<Step>> {
    Ok(vec![Step {
        id: "lint:all".into(),
        adapter: "native".into(),
        program: "luna".into(),
        args: vec!["lint".into()],
        cwd: None,
        env_selector: None,
        depends_on: Vec::new(),
        cacheability: "none".into(),
        mutability: "read-only".into(),
        safety: "offline-safe".into(),
    }])
}

fn plan_format(_root: &Path, _config: &LunaConfig) -> Result<Vec<Step>> {
    Ok(vec![Step {
        id: "format:all".into(),
        adapter: "native".into(),
        program: "luna".into(),
        args: vec!["format".into()],
        cwd: None,
        env_selector: None,
        depends_on: Vec::new(),
        cacheability: "none".into(),
        mutability: "lock-only".into(),
        safety: "offline-safe".into(),
    }])
}

fn plan_typecheck(_root: &Path, _config: &LunaConfig) -> Result<Vec<Step>> {
    Ok(vec![Step {
        id: "typecheck:all".into(),
        adapter: "native".into(),
        program: "luna".into(),
        args: vec!["typecheck".into()],
        cwd: None,
        env_selector: None,
        depends_on: Vec::new(),
        cacheability: "none".into(),
        mutability: "read-only".into(),
        safety: "offline-safe".into(),
    }])
}

fn plan_check(root: &Path, config: &LunaConfig) -> Result<Vec<Step>> {
    let mut steps = Vec::new();
    steps.extend(plan_lint(root, config)?);
    steps.extend(plan_typecheck(root, config)?);
    Ok(steps)
}

fn plan_fix(_root: &Path, _config: &LunaConfig) -> Result<Vec<Step>> {
    Ok(vec![Step {
        id: "fix:all".into(),
        adapter: "native".into(),
        program: "luna".into(),
        args: vec!["fix".into()],
        cwd: None,
        env_selector: None,
        depends_on: Vec::new(),
        cacheability: "none".into(),
        mutability: "install".into(),
        safety: "offline-safe".into(),
    }])
}

fn plan_outdated(_root: &Path, _config: &LunaConfig) -> Result<Vec<Step>> {
    use crate::systems::model::ToolchainKind;
    Ok(ToolchainKind::ORDER
        .iter()
        .map(|k| Step {
            id: format!("outdated:{}", k.label()),
            adapter: k.label().into(),
            program: k.label().into(),
            args: vec!["outdated".into()],
            cwd: None,
            env_selector: None,
            depends_on: Vec::new(),
            cacheability: "none".into(),
            mutability: "read-only".into(),
            safety: "network-required".into(),
        })
        .collect())
}

fn plan_update(_root: &Path, _config: &LunaConfig) -> Result<Vec<Step>> {
    use crate::systems::model::ToolchainKind;
    let mut steps = Vec::new();
    steps.push(Step {
        id: "update:proto".into(),
        adapter: "proto".into(),
        program: "proto".into(),
        args: vec!["outdated".into(), "--update".into()],
        cwd: None,
        env_selector: None,
        depends_on: Vec::new(),
        cacheability: "none".into(),
        mutability: "install".into(),
        safety: "network-required".into(),
    });
    for kind in &ToolchainKind::ORDER[1..] {
        steps.push(Step {
            id: format!("update:{}", kind.label()),
            adapter: kind.label().into(),
            program: kind.label().into(),
            args: vec!["update".into()],
            cwd: None,
            env_selector: None,
            depends_on: vec!["update:proto".into()],
            cacheability: "none".into(),
            mutability: "install".into(),
            safety: "network-required".into(),
        });
    }
    Ok(steps)
}

fn uv_project_dirs(root: &Path) -> Vec<std::path::PathBuf> {
    crate::toolchains::uv::uv_projects(root)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::LunaConfig;

    #[test]
    fn build_plan_sync() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = LunaConfig::default();
        let plan = build_plan(&root, &config, "sync").unwrap();
        assert_eq!(plan.target, "sync");
        assert!(!plan.steps.is_empty());
    }

    #[test]
    fn sync_plan_runs_proto_before_pixi() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = LunaConfig::default();
        let plan = build_plan(&root, &config, "sync").unwrap();
        let ids: Vec<_> = plan.steps.iter().map(|s| s.id.as_str()).collect();
        let proto_idx = ids.iter().position(|id| *id == "proto:install");
        let pixi_idx = ids.iter().position(|id| *id == "pixi:sync");
        assert!(proto_idx.is_some() && pixi_idx.is_some());
        assert!(proto_idx.unwrap() < pixi_idx.unwrap());
        let pixi = plan.steps.iter().find(|s| s.id == "pixi:sync").unwrap();
        assert!(pixi.depends_on.iter().any(|d| d.starts_with("moon:")));
    }

    #[test]
    fn build_plan_unknown_target() {
        let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
        let config = LunaConfig::default();
        assert!(build_plan(&root, &config, "nonexistent").is_err());
    }
}
