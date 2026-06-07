use crate::cli::{DoctorArgs, GlobalArgs};
use crate::config::validate;
use crate::config::LunaConfig;
use crate::output;
use crate::systems::diagnostics::{DoctorCheck, DoctorStatus};
use miette::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub use crate::systems::diagnostics::{
    DoctorCheck as DiagnosticCheck, DoctorStatus as DiagnosticStatus,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub schema_version: String,
    pub workspace_root: String,
    pub checks: Vec<DoctorCheck>,
    pub summary: DoctorSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorSummary {
    pub ok: usize,
    pub warn: usize,
    pub fail: usize,
}

/// Run all doctor checks against the workspace.
pub fn run_doctor(
    root: &Path,
    config: &LunaConfig,
    global: &GlobalArgs,
    args: &DoctorArgs,
) -> Result<i32> {
    let mut checks = Vec::new();

    checks.push(check_root_detection(root));
    checks.push(check_luna_toml(root));
    checks.extend(check_config_drift(root, config));
    checks.extend(check_policy_compliance(root, config));
    checks.push(check_prototools(root));

    for adapter in [
        crate::adapters::registry::get(crate::adapters::AdapterKind::Bun),
        crate::adapters::registry::get(crate::adapters::AdapterKind::Uv),
        crate::adapters::registry::get(crate::adapters::AdapterKind::Cargo),
        crate::adapters::registry::get(crate::adapters::AdapterKind::Go),
        crate::adapters::registry::get(crate::adapters::AdapterKind::Pixi),
        crate::adapters::registry::get(crate::adapters::AdapterKind::Moon),
    ] {
        checks.extend(adapter.doctor(root, config));
    }

    checks.push(check_tool_presence(root, config));

    let ok_count = checks
        .iter()
        .filter(|c| c.status == DoctorStatus::Ok)
        .count();
    let warn_count = checks
        .iter()
        .filter(|c| c.status == DoctorStatus::Warn)
        .count();
    let fail_count = checks
        .iter()
        .filter(|c| c.status == DoctorStatus::Fail)
        .count();

    let report = DoctorReport {
        schema_version: output::SCHEMA_VERSION.into(),
        workspace_root: root.display().to_string(),
        checks: checks.clone(),
        summary: DoctorSummary {
            ok: ok_count,
            warn: warn_count,
            fail: fail_count,
        },
    };

    if global.json {
        output::emit(&report);
    } else if !global.quiet {
        render_doctor_report(&report);
    }

    let exit = if fail_count > 0 || (args.ci && warn_count > 0) {
        1
    } else {
        0
    };
    Ok(exit)
}

fn check_root_detection(root: &Path) -> DoctorCheck {
    let has_luna_toml = root.join("luna.toml").is_file();
    let has_prototools = root.join(".prototools").is_file();
    let has_package_json = root.join("package.json").is_file();

    if has_luna_toml {
        ok("root-detection", "Workspace root (luna.toml)")
    } else if has_prototools && has_package_json {
        warn(
            "root-detection",
            "Workspace root (legacy: .prototools + package.json)",
            "Consider adding luna.toml for canonical detection",
        )
    } else {
        fail("root-detection", "No recognized root marker found")
    }
}

fn check_luna_toml(root: &Path) -> DoctorCheck {
    match crate::config::load(root) {
        Ok(_) => ok("luna-toml", "luna.toml valid"),
        Err(e) => {
            if root.join("luna.toml").is_file() {
                fail("luna-toml", &format!("luna.toml invalid: {e}"))
            } else {
                warn(
                    "luna-toml",
                    "luna.toml not found",
                    "Using defaults from legacy files",
                )
            }
        }
    }
}

fn check_config_drift(root: &Path, config: &LunaConfig) -> Vec<DoctorCheck> {
    validate::validate_against_repo(config, root)
        .into_iter()
        .map(|w| warn("config-drift", &w, "Manifest/lock drift detected"))
        .collect()
}

fn check_policy_compliance(root: &Path, config: &LunaConfig) -> Vec<DoctorCheck> {
    let mut checks = Vec::new();

    if config.policy.min_release_age_days >= 1 {
        checks.push(ok(
            "policy-release-age",
            &format!(
                "Minimum release age: {} days",
                config.policy.min_release_age_days
            ),
        ));
    } else {
        checks.push(fail("policy-release-age", "Invalid min_release_age_days"));
    }

    if config.policy.frozen_ci {
        checks.push(ok("policy-frozen-ci", "frozen_ci enabled in luna.toml"));
    }

    if root.join("pixi.toml").is_file() {
        let text = std::fs::read_to_string(root.join("pixi.toml")).unwrap_or_default();
        if text.contains("exclude-newer") {
            checks.push(ok(
                "policy-pixi-exclude-newer",
                "pixi.toml exclude-newer configured",
            ));
        } else {
            checks.push(warn(
                "policy-pixi-exclude-newer",
                "pixi.toml missing exclude-newer",
                "Align with policy.min_release_age_days",
            ));
        }
    }

    checks
}

fn check_prototools(root: &Path) -> DoctorCheck {
    if root.join(".prototools").is_file() {
        ok("prototools", ".prototools present")
    } else {
        warn(
            "prototools",
            ".prototools not found",
            "Toolchain pins may be missing",
        )
    }
}

fn check_tool_presence(root: &Path, config: &LunaConfig) -> DoctorCheck {
    let mut tools = vec!["proto", "moon", "bun", "cargo", "go", "uv"];
    if config.adapters.pixi.enabled {
        tools.push("pixi");
    }
    let mut missing: Vec<String> = Vec::new();
    let mut present: Vec<String> = Vec::new();

    for tool in &tools {
        if crate::systems::runner::ensure_installed(tool, root).is_ok() {
            present.push(tool.to_string());
        } else {
            missing.push(tool.to_string());
        }
    }

    if missing.is_empty() {
        ok(
            "tool-presence",
            &format!("All core tools available: {}", present.join(", ")),
        )
    } else {
        warn(
            "tool-presence",
            &format!("Missing tools: {}", missing.join(", ")),
            &format!("Available: {}", present.join(", ")),
        )
    }
}

fn ok(id: &str, label: &str) -> DoctorCheck {
    DoctorCheck {
        id: id.into(),
        label: label.into(),
        status: DoctorStatus::Ok,
        detail: None,
    }
}

fn warn(id: &str, label: &str, detail: &str) -> DoctorCheck {
    DoctorCheck {
        id: id.into(),
        label: label.into(),
        status: DoctorStatus::Warn,
        detail: Some(detail.into()),
    }
}

fn fail(id: &str, label: &str) -> DoctorCheck {
    DoctorCheck {
        id: id.into(),
        label: label.into(),
        status: DoctorStatus::Fail,
        detail: None,
    }
}

fn render_doctor_report(report: &DoctorReport) {
    for check in &report.checks {
        let icon = match check.status {
            DoctorStatus::Ok => "\x1b[32m✓\x1b[0m",
            DoctorStatus::Warn => "\x1b[33m⚠\x1b[0m",
            DoctorStatus::Fail => "\x1b[31m✗\x1b[0m",
        };
        if let Some(detail) = &check.detail {
            eprintln!(" {icon} {} — {}", check.label, detail);
        } else {
            eprintln!(" {icon} {}", check.label);
        }
    }
    eprintln!();
    eprintln!(
        " Summary: {} ok, {} warn, {} fail",
        report.summary.ok, report.summary.warn, report.summary.fail
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doctor_report_serialization() {
        let report = DoctorReport {
            schema_version: "v1".into(),
            workspace_root: "/tmp/repo".into(),
            checks: vec![DoctorCheck {
                id: "root-detection".into(),
                label: "Workspace root".into(),
                status: DoctorStatus::Ok,
                detail: None,
            }],
            summary: DoctorSummary {
                ok: 1,
                warn: 0,
                fail: 0,
            },
        };
        let json = serde_json::to_string_pretty(&report).unwrap();
        let back: DoctorReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.summary.ok, 1);
    }
}
