use super::{AdapterKind, BackendAdapter, LockOpts, LockOutcome, SyncOpts, SyncOutcome};
use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::planner::Step;
use crate::systems::diagnostics::{DoctorCheck, DoctorStatus};
use crate::systems::inventory::{self, InventoryItem};
use miette::Result;
use std::path::Path;

pub struct CargoBackend;

impl BackendAdapter for CargoBackend {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Cargo
    }

    fn detect(&self, root: &Path, config: &LunaConfig) -> bool {
        root.join(&config.adapters.cargo.manifest).is_file()
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
        let code = crate::systems::runner::run("cargo", &["build".to_string()], root, opts.quiet)?;
        Ok(SyncOutcome { exit_code: code })
    }

    fn run_step(
        &self,
        root: &Path,
        _config: &LunaConfig,
        step: &Step,
        global: &GlobalArgs,
        opts: SyncOpts,
    ) -> Result<i32> {
        crate::systems::runner::run(&step.program, &step.args, root, opts.quiet || global.quiet)
    }

    fn doctor(&self, root: &Path, config: &LunaConfig) -> Vec<DoctorCheck> {
        let mut checks = Vec::new();
        if root.join(&config.adapters.cargo.manifest).is_file() {
            checks.push(DoctorCheck {
                id: "cargo-manifest".into(),
                label: "Cargo.toml present".into(),
                status: DoctorStatus::Ok,
                detail: None,
            });
        }
        if root.join(&config.adapters.cargo.lockfile).is_file() {
            checks.push(DoctorCheck {
                id: "cargo-lockfile".into(),
                label: "Cargo.lock present".into(),
                status: DoctorStatus::Ok,
                detail: None,
            });
        }
        checks
    }

    fn export_inventory(&self, root: &Path, config: &LunaConfig) -> Result<Vec<InventoryItem>> {
        if !self.detect(root, config) {
            return Ok(Vec::new());
        }
        let lock_path = root.join(&config.adapters.cargo.lockfile);
        if !lock_path.is_file() {
            let manifest = root.join(&config.adapters.cargo.manifest);
            let text = std::fs::read_to_string(&manifest).unwrap_or_default();
            return Ok(inventory::parse_toml_dep_table(&text, "[dependencies]")
                .into_iter()
                .map(|(name, version)| {
                    InventoryItem::new("cargo", name, version)
                        .with_source(&config.adapters.cargo.manifest)
                        .with_ecosystem("cargo")
                })
                .collect());
        }
        let text = std::fs::read_to_string(&lock_path)
            .map_err(|e| miette::miette!("read {}: {e}", lock_path.display()))?;
        let mut items = Vec::new();
        let mut current_name = None;
        let mut current_version = None;
        for line in text.lines() {
            let trimmed = line.trim();
            if trimmed == "[[package]]" {
                if let (Some(name), Some(version)) = (current_name.take(), current_version.take()) {
                    items.push(
                        InventoryItem::new("cargo", name, version)
                            .with_source(&config.adapters.cargo.lockfile)
                            .with_ecosystem("cargo"),
                    );
                }
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("name = ") {
                current_name = Some(rest.trim_matches('"').to_string());
            }
            if let Some(rest) = trimmed.strip_prefix("version = ") {
                current_version = Some(rest.trim_matches('"').to_string());
            }
        }
        if let (Some(name), Some(version)) = (current_name, current_version) {
            items.push(
                InventoryItem::new("cargo", name, version)
                    .with_source(&config.adapters.cargo.lockfile)
                    .with_ecosystem("cargo"),
            );
        }
        Ok(items)
    }
}
