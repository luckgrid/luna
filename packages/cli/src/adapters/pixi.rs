use super::{AdapterKind, BackendAdapter, LockOpts, LockOutcome, SyncOpts, SyncOutcome};
use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::planner::Step;
use crate::systems::diagnostics::{DoctorCheck, DoctorStatus};
use crate::systems::inventory::InventoryItem;
use crate::toolchains::pixi;
use miette::Result;
use std::path::Path;

pub struct PixiBackend;

impl BackendAdapter for PixiBackend {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Pixi
    }

    fn detect(&self, root: &Path, config: &LunaConfig) -> bool {
        config.adapters.pixi.enabled && root.join(&config.adapters.pixi.manifest).is_file()
    }

    fn lock(&self, root: &Path, config: &LunaConfig, opts: LockOpts) -> Result<LockOutcome> {
        if !self.detect(root, config) {
            return Ok(LockOutcome {
                ok: true,
                message: Some("pixi not configured".into()),
            });
        }
        pixi::ensure_pixi(root, config, false)?;
        let code = pixi::pixi_install(root, opts.locked)?;
        Ok(LockOutcome {
            ok: code == 0,
            message: if code == 0 {
                None
            } else {
                Some(format!("pixi install exited with {code}"))
            },
        })
    }

    fn sync(&self, root: &Path, config: &LunaConfig, opts: SyncOpts) -> Result<SyncOutcome> {
        if !self.detect(root, config) {
            return Ok(SyncOutcome { exit_code: 0 });
        }
        pixi::ensure_pixi(root, config, opts.quiet)?;
        let code = pixi::pixi_install(root, opts.locked)?;
        Ok(SyncOutcome { exit_code: code })
    }

    fn run_step(
        &self,
        root: &Path,
        config: &LunaConfig,
        step: &Step,
        _global: &GlobalArgs,
        opts: SyncOpts,
    ) -> Result<i32> {
        if step.program == "pixi" && step.args.first().map(String::as_str) == Some("install") {
            if !self.detect(root, config) {
                return Ok(0);
            }
            pixi::ensure_pixi(root, config, opts.quiet)?;
            return pixi::pixi_install(root, opts.locked);
        }
        let args: Vec<String> = step.args.clone();
        crate::systems::runner::run(&step.program, &args, root, opts.quiet)
    }

    fn doctor(&self, root: &Path, config: &LunaConfig) -> Vec<DoctorCheck> {
        if !config.adapters.pixi.enabled {
            return vec![DoctorCheck {
                id: "pixi-adapter".into(),
                label: "Pixi adapter disabled".into(),
                status: DoctorStatus::Ok,
                detail: None,
            }];
        }
        if root.join(&config.adapters.pixi.manifest).is_file() {
            vec![DoctorCheck {
                id: "pixi-adapter".into(),
                label: format!("{} present", config.adapters.pixi.manifest),
                status: DoctorStatus::Ok,
                detail: None,
            }]
        } else {
            vec![DoctorCheck {
                id: "pixi-adapter".into(),
                label: format!("{} missing", config.adapters.pixi.manifest),
                status: DoctorStatus::Warn,
                detail: Some("Run `pixi init` or enable sync".into()),
            }]
        }
    }

    fn export_inventory(&self, root: &Path, config: &LunaConfig) -> Result<Vec<InventoryItem>> {
        if !self.detect(root, config) {
            return Ok(Vec::new());
        }
        let path = root.join(&config.adapters.pixi.manifest);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| miette::miette!("read {}: {e}", path.display()))?;
        let mut items = Vec::new();
        let mut in_deps = false;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                in_deps = trimmed == "[dependencies]" || trimmed.starts_with("[dependencies.");
                continue;
            }
            if !in_deps || trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            if let Some((name, rest)) = trimmed.split_once('=') {
                let version = rest.trim().trim_matches('"').to_string();
                items.push(
                    InventoryItem::new("pixi", name.trim(), version)
                        .with_source(&config.adapters.pixi.manifest)
                        .with_ecosystem("conda"),
                );
            }
        }
        Ok(items)
    }
}
