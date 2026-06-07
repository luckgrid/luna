use super::{AdapterKind, BackendAdapter, LockOpts, LockOutcome, SyncOpts, SyncOutcome};
use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::planner::Step;
use crate::systems::diagnostics::{DoctorCheck, DoctorStatus};
use crate::systems::inventory::InventoryItem;
use miette::Result;
use std::path::Path;

pub struct ProtoBackend;

impl BackendAdapter for ProtoBackend {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Proto
    }

    fn detect(&self, root: &Path, _config: &LunaConfig) -> bool {
        root.join(".prototools").is_file()
    }

    fn lock(&self, _root: &Path, _config: &LunaConfig, _opts: LockOpts) -> Result<LockOutcome> {
        Ok(LockOutcome {
            ok: true,
            message: None,
        })
    }

    fn sync(&self, root: &Path, _config: &LunaConfig, opts: SyncOpts) -> Result<SyncOutcome> {
        if !self.detect(root, _config) {
            return Ok(SyncOutcome { exit_code: 0 });
        }
        crate::systems::runner::ensure_installed("proto", root)?;
        let code =
            crate::systems::runner::run("proto", &["install".to_string()], root, opts.quiet)?;
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
        crate::systems::runner::ensure_installed("proto", root)?;
        crate::systems::runner::run(&step.program, &step.args, root, opts.quiet || global.quiet)
    }

    fn doctor(&self, root: &Path, _config: &LunaConfig) -> Vec<DoctorCheck> {
        if root.join(".prototools").is_file() {
            vec![DoctorCheck {
                id: "proto-adapter".into(),
                label: ".prototools present".into(),
                status: DoctorStatus::Ok,
                detail: None,
            }]
        } else {
            vec![DoctorCheck {
                id: "proto-adapter".into(),
                label: ".prototools missing".into(),
                status: DoctorStatus::Warn,
                detail: Some("Toolchain pins may be missing".into()),
            }]
        }
    }

    fn export_inventory(&self, root: &Path, _config: &LunaConfig) -> Result<Vec<InventoryItem>> {
        if !self.detect(root, _config) {
            return Ok(Vec::new());
        }
        let path = root.join(".prototools");
        let text = std::fs::read_to_string(&path)
            .map_err(|e| miette::miette!("read {}: {e}", path.display()))?;
        let items = text
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    return None;
                }
                let (tool, version) = trimmed.split_once('=')?;
                Some(
                    InventoryItem::new("proto", tool.trim(), version.trim())
                        .with_source(".prototools")
                        .with_ecosystem("proto"),
                )
            })
            .collect();
        Ok(items)
    }
}
