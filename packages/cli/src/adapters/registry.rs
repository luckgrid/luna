use super::{AdapterKind, BackendAdapter, LockOpts, LockOutcome, SyncOpts, SyncOutcome};
use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::planner::Step;
use crate::systems::diagnostics::DoctorCheck;
use crate::systems::inventory::InventoryItem;
use miette::Result;
use std::path::Path;

pub fn get(kind: AdapterKind) -> Box<dyn BackendAdapter> {
    match kind {
        AdapterKind::Pixi => Box::new(super::pixi::PixiBackend),
        AdapterKind::Moon => Box::new(super::moon::MoonBackend),
        AdapterKind::Proto => Box::new(super::proto::ProtoBackend),
        AdapterKind::Bun => Box::new(super::bun::BunBackend),
        AdapterKind::Uv => Box::new(super::uv::UvBackend),
        AdapterKind::Cargo => Box::new(super::cargo::CargoBackend),
        AdapterKind::Go => Box::new(super::go::GoBackend),
        AdapterKind::Native => Box::new(NativeBackend),
    }
}

struct NativeBackend;

impl BackendAdapter for NativeBackend {
    fn kind(&self) -> AdapterKind {
        AdapterKind::Native
    }

    fn detect(&self, _root: &Path, _config: &LunaConfig) -> bool {
        true
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
        _root: &Path,
        _config: &LunaConfig,
        step: &Step,
        _global: &GlobalArgs,
        _opts: SyncOpts,
    ) -> Result<i32> {
        Err(miette::miette!(
            "native adapter cannot execute step {} — invoke luna subcommands directly",
            step.id
        ))
    }

    fn doctor(&self, _root: &Path, _config: &LunaConfig) -> Vec<DoctorCheck> {
        Vec::new()
    }

    fn export_inventory(&self, _root: &Path, _config: &LunaConfig) -> Result<Vec<InventoryItem>> {
        Ok(Vec::new())
    }
}

/// All adapter kinds that participate in lock/inventory operations.
pub const INVENTORY_KINDS: [AdapterKind; 7] = [
    AdapterKind::Pixi,
    AdapterKind::Proto,
    AdapterKind::Bun,
    AdapterKind::Uv,
    AdapterKind::Cargo,
    AdapterKind::Go,
    AdapterKind::Moon,
];
