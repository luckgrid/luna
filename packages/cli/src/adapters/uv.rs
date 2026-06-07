use super::{AdapterKind, BackendAdapter, LockOpts, LockOutcome, SyncOpts, SyncOutcome};
use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::planner::Step;
use crate::systems::diagnostics::{DoctorCheck, DoctorStatus};
use crate::systems::inventory::InventoryItem;
use crate::systems::{security, workspace};
use miette::Result;
use std::path::Path;

pub struct UvBackend;

impl BackendAdapter for UvBackend {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Uv
    }

    fn detect(&self, root: &Path, _config: &LunaConfig) -> bool {
        workspace::uv_workspace_root(root).is_some()
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
        crate::systems::runner::ensure_installed("uv", root)?;
        let mut args = vec!["sync".to_string()];
        if opts.locked {
            args.push("--locked".to_string());
        }
        let firewall = false;
        let code = crate::systems::runner::run_pm("uv", &args, root, opts.quiet, firewall)?;
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
        if step.id == "uv:sync" {
            return self.sync(root, config, opts).map(|o| o.exit_code);
        }
        let firewall = security::resolve_firewall(root, global, global.quiet);
        crate::systems::runner::run_pm(
            &step.program,
            &step.args,
            root,
            opts.quiet || global.quiet,
            firewall,
        )
    }

    fn doctor(&self, root: &Path, config: &LunaConfig) -> Vec<DoctorCheck> {
        let mut checks = Vec::new();
        if root.join(&config.adapters.uv.manifest).is_file() {
            checks.push(DoctorCheck {
                id: "uv-manifest".into(),
                label: "pyproject.toml present".into(),
                status: DoctorStatus::Ok,
                detail: None,
            });
        }
        if root.join(&config.adapters.uv.lockfile).is_file() {
            checks.push(DoctorCheck {
                id: "uv-lockfile".into(),
                label: format!("{} present", config.adapters.uv.lockfile),
                status: DoctorStatus::Ok,
                detail: None,
            });
        } else if self.detect(root, config) {
            checks.push(DoctorCheck {
                id: "uv-lockfile".into(),
                label: format!("{} missing", config.adapters.uv.lockfile),
                status: DoctorStatus::Warn,
                detail: Some("Run `uv sync`".into()),
            });
        }
        checks
    }

    fn export_inventory(&self, root: &Path, config: &LunaConfig) -> Result<Vec<InventoryItem>> {
        if !self.detect(root, config) {
            return Ok(Vec::new());
        }
        let manifest = root.join(&config.adapters.uv.manifest);
        let text = std::fs::read_to_string(&manifest).unwrap_or_default();
        let mut items = Vec::new();
        if let Ok(doc) = toml::from_str::<toml::Table>(&text) {
            if let Some(project) = doc.get("project").and_then(|p| p.as_table()) {
                if let Some(deps) = project.get("dependencies").and_then(|d| d.as_array()) {
                    for dep in deps {
                        if let Some(s) = dep.as_str() {
                            let (name, version) = s.split_once(">=").unwrap_or((s, "*"));
                            items.push(
                                InventoryItem::new("uv", name.trim(), version.trim())
                                    .with_source(&config.adapters.uv.manifest)
                                    .with_ecosystem("pypi"),
                            );
                        }
                    }
                }
            }
        }
        Ok(items)
    }
}
