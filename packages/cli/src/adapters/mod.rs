pub mod bun;
pub mod cargo;
pub mod go;
pub mod moon;
pub mod pixi;
pub mod proto;
pub mod registry;
pub mod uv;

use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::planner::Step;
use crate::systems::diagnostics::DoctorCheck;
use crate::systems::inventory::InventoryItem;
use miette::Result;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AdapterKind {
    Pixi,
    Moon,
    Proto,
    Bun,
    Uv,
    Cargo,
    Go,
    Native,
}

impl AdapterKind {
    pub fn from_label(label: &str) -> Self {
        match label {
            "pixi" => Self::Pixi,
            "moon" => Self::Moon,
            "proto" => Self::Proto,
            "bun" => Self::Bun,
            "uv" => Self::Uv,
            "cargo" | "rust" => Self::Cargo,
            "go" => Self::Go,
            _ => Self::Native,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Pixi => "pixi",
            Self::Moon => "moon",
            Self::Proto => "proto",
            Self::Bun => "bun",
            Self::Uv => "uv",
            Self::Cargo => "cargo",
            Self::Go => "go",
            Self::Native => "native",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SyncOpts {
    pub locked: bool,
    pub quiet: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct LockOpts {
    pub locked: bool,
    pub quiet: bool,
}

pub struct LockOutcome {
    pub ok: bool,
    pub message: Option<String>,
}

pub struct SyncOutcome {
    pub exit_code: i32,
}

pub use registry::INVENTORY_KINDS;

/// Backend adapter contract for sync, lock, run_task, and doctor operations.
pub trait BackendAdapter: Send + Sync {
    fn kind(&self) -> AdapterKind;
    fn detect(&self, root: &Path, config: &LunaConfig) -> bool;
    fn lock(&self, root: &Path, config: &LunaConfig, opts: LockOpts) -> Result<LockOutcome>;
    fn sync(&self, root: &Path, config: &LunaConfig, opts: SyncOpts) -> Result<SyncOutcome>;
    fn run_step(
        &self,
        root: &Path,
        config: &LunaConfig,
        step: &Step,
        global: &GlobalArgs,
        opts: SyncOpts,
    ) -> Result<i32>;
    fn doctor(&self, root: &Path, config: &LunaConfig) -> Vec<DoctorCheck>;
    fn export_inventory(&self, root: &Path, config: &LunaConfig) -> Result<Vec<InventoryItem>>;
}
