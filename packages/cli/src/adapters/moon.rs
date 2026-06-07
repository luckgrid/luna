use super::{AdapterKind, BackendAdapter, LockOpts, LockOutcome, SyncOpts, SyncOutcome};
use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::planner::Step;
use crate::systems::diagnostics::{DoctorCheck, DoctorStatus};
use crate::systems::inventory::InventoryItem;
use crate::systems::runner::{run_moon, Output};
use miette::Result;
use std::path::Path;

pub struct MoonBackend;

impl BackendAdapter for MoonBackend {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Moon
    }

    fn detect(&self, root: &Path, config: &LunaConfig) -> bool {
        config.compat.moon.enabled && root.join(".moon").is_dir()
    }

    fn lock(&self, _root: &Path, _config: &LunaConfig, _opts: LockOpts) -> Result<LockOutcome> {
        Ok(LockOutcome {
            ok: true,
            message: None,
        })
    }

    fn sync(&self, _root: &Path, _config: &LunaConfig, _opts: SyncOpts) -> Result<SyncOutcome> {
        Ok(SyncOutcome { exit_code: 0 })
    }

    fn run_step(
        &self,
        root: &Path,
        _config: &LunaConfig,
        step: &Step,
        global: &GlobalArgs,
        _opts: SyncOpts,
    ) -> Result<i32> {
        let argv: Vec<&str> = step.args.iter().map(String::as_str).collect();
        run_moon(root, &argv, global)
    }

    fn doctor(&self, root: &Path, config: &LunaConfig) -> Vec<DoctorCheck> {
        if !config.compat.moon.enabled {
            return vec![DoctorCheck {
                id: "moon-adapter".into(),
                label: "Moon compatibility disabled".into(),
                status: DoctorStatus::Ok,
                detail: None,
            }];
        }
        if root.join(".moon").is_dir() {
            vec![DoctorCheck {
                id: "moon-adapter".into(),
                label: ".moon/ present".into(),
                status: DoctorStatus::Ok,
                detail: None,
            }]
        } else {
            vec![DoctorCheck {
                id: "moon-adapter".into(),
                label: ".moon/ missing".into(),
                status: DoctorStatus::Warn,
                detail: Some("Moon task graph unavailable".into()),
            }]
        }
    }
    fn export_inventory(&self, _root: &Path, _config: &LunaConfig) -> Result<Vec<InventoryItem>> {
        Ok(Vec::new())
    }
}

impl MoonBackend {
    pub fn run_moon_argv(root: &Path, args: &[&str], global: &GlobalArgs) -> Result<i32> {
        run_moon(root, args, global)
    }

    pub fn capture_json(root: &Path, args: &[&str], quiet: bool) -> Result<Output> {
        let mut full: Vec<String> = Vec::new();
        if quiet {
            full.push("-q".into());
        }
        full.extend(args.iter().map(|a| (*a).into()));
        crate::systems::runner::capture("moon", &full, root)
    }
}
