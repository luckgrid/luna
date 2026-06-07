use super::{AdapterKind, BackendAdapter, LockOpts, LockOutcome, SyncOpts, SyncOutcome};
use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::planner::Step;
use crate::systems::diagnostics::{DoctorCheck, DoctorStatus};
use crate::systems::inventory::InventoryItem;
use crate::systems::workspace;
use miette::Result;
use std::path::Path;

pub struct GoBackend;

impl BackendAdapter for GoBackend {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Go
    }

    fn detect(&self, root: &Path, config: &LunaConfig) -> bool {
        root.join(&config.adapters.go.workspace).is_file()
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
        workspace::sync_go_toolchain(root, opts.quiet)?;
        crate::systems::runner::ensure_installed("go", root)?;
        let code = crate::systems::runner::run(
            "go",
            &["work".to_string(), "sync".to_string()],
            root,
            opts.quiet,
        )?;
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
        if step.id == "go:work-sync" {
            return self.sync(root, config, opts).map(|o| o.exit_code);
        }
        crate::systems::runner::run(&step.program, &step.args, root, opts.quiet || global.quiet)
    }

    fn doctor(&self, root: &Path, config: &LunaConfig) -> Vec<DoctorCheck> {
        if root.join(&config.adapters.go.workspace).is_file() {
            vec![DoctorCheck {
                id: "go-workspace".into(),
                label: format!("{} present", config.adapters.go.workspace),
                status: DoctorStatus::Ok,
                detail: None,
            }]
        } else {
            vec![DoctorCheck {
                id: "go-workspace".into(),
                label: "go.work missing".into(),
                status: DoctorStatus::Warn,
                detail: None,
            }]
        }
    }

    fn export_inventory(&self, root: &Path, config: &LunaConfig) -> Result<Vec<InventoryItem>> {
        if !self.detect(root, config) {
            return Ok(Vec::new());
        }
        let mut items = Vec::new();
        let dirs = workspace::project_roots(root, "go", "go.mod");
        for dir in dirs {
            let go_mod = dir.join("go.mod");
            if !go_mod.is_file() {
                continue;
            }
            let text = std::fs::read_to_string(&go_mod).unwrap_or_default();
            let mut in_require = false;
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("require (") {
                    in_require = true;
                    continue;
                }
                if in_require && trimmed == ")" {
                    in_require = false;
                    continue;
                }
                if in_require || trimmed.starts_with("require ") {
                    let parts: Vec<&str> = trimmed
                        .trim_start_matches("require ")
                        .split_whitespace()
                        .collect();
                    if parts.len() >= 2 {
                        let rel = go_mod
                            .strip_prefix(root)
                            .map(|p| p.display().to_string())
                            .unwrap_or_else(|_| go_mod.display().to_string());
                        items.push(
                            InventoryItem::new("go", parts[0], parts[1])
                                .with_source(rel)
                                .with_ecosystem("go"),
                        );
                    }
                }
            }
        }
        Ok(items)
    }
}
