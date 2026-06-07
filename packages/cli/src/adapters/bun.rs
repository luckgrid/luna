use super::{AdapterKind, BackendAdapter, LockOpts, LockOutcome, SyncOpts, SyncOutcome};
use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::planner::Step;
use crate::systems::diagnostics::{DoctorCheck, DoctorStatus};
use crate::systems::inventory::InventoryItem;
use crate::systems::security;
use miette::Result;
use std::path::Path;

pub struct BunBackend;

impl BackendAdapter for BunBackend {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Bun
    }

    fn detect(&self, root: &Path, config: &LunaConfig) -> bool {
        root.join(&config.adapters.bun.manifest).is_file()
    }

    fn lock(&self, _root: &Path, _config: &LunaConfig, _opts: LockOpts) -> Result<LockOutcome> {
        Ok(LockOutcome {
            ok: true,
            message: None,
        })
    }

    fn sync(&self, root: &Path, config: &LunaConfig, opts: SyncOpts) -> Result<SyncOutcome> {
        if !self.detect(root, config) {
            return Ok(SyncOutcome { exit_code: 0 });
        }
        crate::systems::runner::ensure_installed("bun", root)?;
        let args = vec![
            "install".to_string(),
            "--ignore-scripts".to_string(),
            security::bun_min_release_age_arg(),
        ];
        let code = crate::systems::runner::run("bun", &args, root, opts.quiet)?;
        Ok(SyncOutcome { exit_code: code })
    }

    fn run_step(
        &self,
        root: &Path,
        config: &LunaConfig,
        step: &Step,
        global: &GlobalArgs,
        opts: SyncOpts,
    ) -> Result<i32> {
        if step.id == "bun:install" {
            return self.sync(root, config, opts).map(|o| o.exit_code);
        }
        crate::systems::runner::run(&step.program, &step.args, root, opts.quiet || global.quiet)
    }

    fn doctor(&self, root: &Path, config: &LunaConfig) -> Vec<DoctorCheck> {
        let mut checks = Vec::new();
        if root.join(&config.adapters.bun.manifest).is_file() {
            checks.push(DoctorCheck {
                id: "bun-manifest".into(),
                label: format!("{} present", config.adapters.bun.manifest),
                status: DoctorStatus::Ok,
                detail: None,
            });
        }
        if root.join(&config.adapters.bun.lockfile).is_file() {
            checks.push(DoctorCheck {
                id: "bun-lockfile".into(),
                label: format!("{} present", config.adapters.bun.lockfile),
                status: DoctorStatus::Ok,
                detail: None,
            });
        } else if self.detect(root, config) {
            checks.push(DoctorCheck {
                id: "bun-lockfile".into(),
                label: format!("{} missing", config.adapters.bun.lockfile),
                status: DoctorStatus::Warn,
                detail: Some("Run `bun install`".into()),
            });
        }
        checks
    }

    fn export_inventory(&self, root: &Path, config: &LunaConfig) -> Result<Vec<InventoryItem>> {
        if !self.detect(root, config) {
            return Ok(Vec::new());
        }
        let path = root.join(&config.adapters.bun.manifest);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| miette::miette!("read {}: {e}", path.display()))?;
        let parsed: serde_json::Value = serde_json::from_str(&text)
            .map_err(|e| miette::miette!("parse {}: {e}", path.display()))?;
        let mut items = Vec::new();
        for key in ["dependencies", "devDependencies"] {
            if let Some(deps) = parsed.get(key).and_then(|v| v.as_object()) {
                for (name, version) in deps {
                    let ver = version
                        .as_str()
                        .map(String::from)
                        .unwrap_or_else(|| version.to_string());
                    items.push(
                        InventoryItem::new("bun", name.clone(), ver)
                            .with_source(&config.adapters.bun.manifest)
                            .with_ecosystem("npm"),
                    );
                }
            }
        }
        Ok(items)
    }
}
