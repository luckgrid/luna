use crate::adapters::{registry, INVENTORY_KINDS};
use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::output::{self, Component, SbomReport};
use crate::systems::inventory::InventoryItem;
use miette::Result;
use std::path::Path;

pub fn collect_inventory(root: &Path, config: &LunaConfig) -> Result<Vec<InventoryItem>> {
    let mut all = Vec::new();
    for kind in INVENTORY_KINDS {
        let adapter = registry::get(kind);
        if !adapter.detect(root, config) {
            continue;
        }
        all.extend(adapter.export_inventory(root, config)?);
    }
    Ok(all)
}

pub fn run_sbom(
    root: &Path,
    config: &LunaConfig,
    global: &GlobalArgs,
    format: SbomFormat,
) -> Result<i32> {
    let items = collect_inventory(root, config)?;
    let components: Vec<Component> = items.into_iter().map(Component::from).collect();

    if global.json || matches!(format, SbomFormat::Json) {
        let report = SbomReport {
            schema_version: output::SCHEMA_VERSION.into(),
            workspace_root: root.display().to_string(),
            format: format.label().into(),
            components: components.clone(),
        };
        if matches!(format, SbomFormat::CycloneDx) {
            output::emit(&cyclonedx_from_components(&components));
        } else {
            output::emit(&report);
        }
    } else if !global.quiet {
        eprintln!("SBOM ({} components):", components.len());
        for c in &components {
            eprintln!("  {} {}@{} ({})", c.adapter, c.name, c.version, c.ecosystem);
        }
    }
    Ok(0)
}

#[derive(Debug, Clone, Copy)]
pub enum SbomFormat {
    Json,
    CycloneDx,
}

impl SbomFormat {
    pub fn parse_format(s: &str) -> Self {
        match s {
            "cyclonedx" => Self::CycloneDx,
            _ => Self::Json,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Json => "luna",
            Self::CycloneDx => "cyclonedx",
        }
    }
}

fn cyclonedx_from_components(components: &[Component]) -> serde_json::Value {
    serde_json::json!({
        "bomFormat": "CycloneDX",
        "specVersion": "1.5",
        "version": 1,
        "components": components.iter().map(|c| {
            serde_json::json!({
                "type": "library",
                "name": c.name,
                "version": c.version,
                "purl": format!("pkg:{}@{}", c.ecosystem, c.name),
            })
        }).collect::<Vec<_>>(),
    })
}

pub fn run_inventory(root: &Path, config: &LunaConfig, global: &GlobalArgs) -> Result<i32> {
    run_sbom(root, config, global, SbomFormat::Json)
}
