use crate::cli::GlobalArgs;
use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Default minimum package age before install/update (14 days).
pub const MIN_RELEASE_AGE_DAYS: u64 = 14;

const SECONDS_PER_DAY: u64 = 86_400;

/// Package managers Socket Firewall Free can wrap (see https://docs.socket.dev/docs/socket-firewall-free).
pub const SFW_SUPPORTED: &[&str] = &["npm", "yarn", "pnpm", "pip", "uv", "cargo"];

static SFW_MISSING_WARNED: AtomicBool = AtomicBool::new(false);

/// Minimum release age in days, overridable via `LUNA_MIN_RELEASE_AGE`.
pub fn min_release_age_days() -> u64 {
    std::env::var("LUNA_MIN_RELEASE_AGE")
        .ok()
        .and_then(|v| v.parse().ok())
        .filter(|&d| d > 0)
        .unwrap_or(MIN_RELEASE_AGE_DAYS)
}

/// Minimum release age in seconds (for Bun `--minimum-release-age`).
pub fn min_release_age_secs() -> u64 {
    min_release_age_days() * SECONDS_PER_DAY
}

/// Bun CLI flag for minimum release age.
pub fn bun_min_release_age_arg() -> String {
    format!("--minimum-release-age={}", min_release_age_secs())
}

/// `YYYY-MM-DD` cutoff for uv `--exclude-newer` (packages newer than this are skipped).
pub fn exclude_newer_date() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let target = now.saturating_sub(min_release_age_secs());
    format_ymd_from_unix_days(target / SECONDS_PER_DAY)
}

/// Extra args for `uv lock --upgrade` to enforce the release-age cooldown.
pub fn uv_exclude_newer_args() -> Vec<String> {
    vec!["--exclude-newer".to_string(), exclude_newer_date()]
}

/// Whether the user requested Socket Firewall via `--firewall` or `LUNA_FIREWALL`.
pub fn firewall_requested(global: &GlobalArgs) -> bool {
    global.firewall
}

/// Resolve firewall activation: opt-in only; warn once if `sfw` is missing.
pub fn resolve_firewall(root: &Path, global: &GlobalArgs, quiet: bool) -> bool {
    if !firewall_requested(global) {
        return false;
    }
    if sfw_available(root) {
        return true;
    }
    if !quiet && !SFW_MISSING_WARNED.swap(true, Ordering::Relaxed) {
        eprintln!(
            "\x1b[33m⚠ Socket Firewall requested but `sfw` not found; continuing without firewall.\x1b[0m"
        );
        eprintln!("\x1b[2m  Install: npm i -g sfw (or bun add -g sfw)\x1b[0m");
    }
    false
}

/// Check whether `sfw` is on PATH (uses workspace-enriched PATH).
pub fn sfw_available(root: &Path) -> bool {
    let mut cmd = Command::new("sfw");
    cmd.arg("--help");
    crate::runner::apply_toolchain_env_for_check(&mut cmd, root);
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// When active and supported, prefix with `sfw` so the command becomes `sfw <program> …`.
pub fn wrap(program: &str, args: &[String], active: bool) -> (String, Vec<String>) {
    if !active || !SFW_SUPPORTED.contains(&program) {
        return (program.to_string(), args.to_vec());
    }
    let mut wrapped = vec![program.to_string()];
    wrapped.extend(args.iter().cloned());
    ("sfw".to_string(), wrapped)
}

/// Convert days since Unix epoch (1970-01-01) to `YYYY-MM-DD`.
pub fn format_ymd_from_unix_days(days: u64) -> String {
    let (y, m, d) = civil_from_days(days as i64);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Howard Hinnant civil-date algorithm (days since 1970-01-01).
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u32;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i32 + era as i32 * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_min_release_age_secs_is_14_days() {
        std::env::remove_var("LUNA_MIN_RELEASE_AGE");
        assert_eq!(min_release_age_days(), 14);
        assert_eq!(min_release_age_secs(), 14 * SECONDS_PER_DAY);
        assert_eq!(min_release_age_secs(), 1_209_600);
    }

    #[test]
    fn min_release_age_days_env_override() {
        std::env::set_var("LUNA_MIN_RELEASE_AGE", "7");
        assert_eq!(min_release_age_days(), 7);
        assert_eq!(min_release_age_secs(), 7 * SECONDS_PER_DAY);
        std::env::remove_var("LUNA_MIN_RELEASE_AGE");
    }

    #[test]
    fn exclude_newer_date_is_iso_format() {
        let date = exclude_newer_date();
        assert_eq!(date.len(), 10);
        assert!(date.as_bytes()[4] == b'-');
        assert!(date.as_bytes()[7] == b'-');
        let parts: Vec<_> = date.split('-').collect();
        assert_eq!(parts.len(), 3);
        assert!(parts[0].parse::<u32>().is_ok());
        assert!(parts[1].parse::<u32>().is_ok());
        assert!(parts[2].parse::<u32>().is_ok());
    }

    #[test]
    fn wrap_adds_sfw_for_supported_tools() {
        let args = vec!["update".to_string()];
        let (prog, wrapped) = wrap("cargo", &args, true);
        assert_eq!(prog, "sfw");
        assert_eq!(wrapped, vec!["cargo", "update"]);

        let (prog, wrapped) = wrap("uv", &["sync".to_string()], true);
        assert_eq!(prog, "sfw");
        assert_eq!(wrapped[0], "uv");
    }

    #[test]
    fn wrap_skips_unsupported_tools() {
        let args = vec!["install".to_string()];
        let (prog, wrapped) = wrap("bun", &args, true);
        assert_eq!(prog, "bun");
        assert_eq!(wrapped, args);

        let (prog, _wrapped) = wrap("go", &["mod".to_string(), "tidy".to_string()], true);
        assert_eq!(prog, "go");
    }

    #[test]
    fn wrap_inactive_passthrough() {
        let args = vec!["update".to_string()];
        let (prog, wrapped) = wrap("cargo", &args, false);
        assert_eq!(prog, "cargo");
        assert_eq!(wrapped, args);
    }
}
