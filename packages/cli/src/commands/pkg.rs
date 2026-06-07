use crate::cli::{GlobalArgs, PkgAddArgs, PkgRemoveArgs};
use crate::config::LunaConfig;
use crate::systems::runner;
use crate::systems::workspace;
use miette::{miette, Result};
use std::path::{Path, PathBuf};

pub fn add(
    root: &Path,
    config: &LunaConfig,
    args: &PkgAddArgs,
    global: &GlobalArgs,
) -> Result<i32> {
    let (program, argv) =
        resolve_pkg_cmd(root, config, args.project.as_deref(), &args.package, true)?;
    let code = runner::run(&program, &argv, root, global.quiet)?;
    if global.json {
        crate::output::emit(&serde_json::json!({
            "schemaVersion": crate::output::SCHEMA_VERSION,
            "action": "add",
            "program": program,
            "exit_code": code,
        }));
    }
    Ok(code)
}

pub fn remove(
    root: &Path,
    config: &LunaConfig,
    args: &PkgRemoveArgs,
    global: &GlobalArgs,
) -> Result<i32> {
    let (program, argv) =
        resolve_pkg_cmd(root, config, args.project.as_deref(), &args.package, false)?;
    let code = runner::run(&program, &argv, root, global.quiet)?;
    if global.json {
        crate::output::emit(&serde_json::json!({
            "schemaVersion": crate::output::SCHEMA_VERSION,
            "action": "remove",
            "program": program,
            "exit_code": code,
        }));
    }
    Ok(code)
}

fn resolve_pkg_cmd(
    root: &Path,
    config: &LunaConfig,
    project: Option<&str>,
    package: &str,
    add: bool,
) -> Result<(String, Vec<String>)> {
    let project_dir = resolve_project_dir(root, project)?;
    if project_dir.join(&config.adapters.bun.manifest).is_file() {
        let verb = if add { "add" } else { "remove" };
        return Ok(("bun".into(), vec![verb.into(), package.into()]));
    }
    if project_dir.join(&config.adapters.uv.manifest).is_file() {
        let verb = if add { "add" } else { "remove" };
        return Ok(("uv".into(), vec![verb.into(), package.into()]));
    }
    if project_dir.join(&config.adapters.cargo.manifest).is_file() {
        let verb = if add { "add" } else { "remove" };
        return Ok(("cargo".into(), vec![verb.into(), package.into()]));
    }
    if project_dir.join("go.mod").is_file() {
        let arg = if add {
            package.to_string()
        } else {
            format!("-{package}")
        };
        return Ok(("go".into(), vec!["get".into(), arg]));
    }
    Err(miette!("no supported package manifest found for project"))
}

fn resolve_project_dir(root: &Path, project: Option<&str>) -> Result<PathBuf> {
    if let Some(id) = project {
        let projects = workspace::discover_projects(root);
        if let Some(p) = projects.iter().find(|p| p.name == id) {
            return Ok(p.path.clone());
        }
        return Err(miette!("unknown project id: {id}"));
    }
    Ok(root.to_path_buf())
}
