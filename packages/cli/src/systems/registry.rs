use crate::systems::model::ToolchainKind;
use std::collections::HashMap;
use std::io::Read;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;

/// HTTP timeout per registry call; failures degrade to `None` (no Release Age).
const HTTP_TIMEOUT: Duration = Duration::from_secs(20);

/// Upper bound on a registry response body. npm packuments can be tens of MB
/// (e.g. `vite`), well past ureq's default 10 MB `into_string` cap.
const MAX_BODY_BYTES: u64 = 96 * 1024 * 1024;

type VersionTimes = HashMap<String, String>;

/// Cache keyed by `"npm:<pkg>"` / `"pypi:<pkg>"`; `None` = lookup failed/unsupported.
fn cache() -> &'static Mutex<HashMap<String, Option<VersionTimes>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<VersionTimes>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Days since `version` of `dependency` was published, when a registry can answer.
///
/// Only Bun (npm) and uv (PyPI) have registry sources here; other toolchains
/// return `None`. Any network/parse failure also yields `None` (best-effort).
pub fn release_age_days(kind: ToolchainKind, dependency: &str, version: &str) -> Option<u32> {
    let version = version.trim().trim_start_matches('v');
    if version.is_empty() {
        return None;
    }
    // Bun's table appends suffixes like "vite (dev)"; the registry wants the bare name.
    let dependency = dependency.split_whitespace().next().unwrap_or(dependency);
    let times = match kind {
        ToolchainKind::Bun => package_times("npm", dependency, fetch_npm_times)?,
        ToolchainKind::Uv => package_times("pypi", dependency, fetch_pypi_times)?,
        _ => return None,
    };
    let iso = times.get(version)?;
    age_days_from_iso(iso)
}

fn package_times(
    scheme: &str,
    dependency: &str,
    fetch: fn(&str) -> Option<VersionTimes>,
) -> Option<VersionTimes> {
    let key = format!("{scheme}:{dependency}");
    if let Ok(map) = cache().lock() {
        if let Some(entry) = map.get(&key) {
            return entry.clone();
        }
    }
    let fetched = fetch(dependency);
    if let Ok(mut map) = cache().lock() {
        map.insert(key, fetched.clone());
    }
    fetched
}

fn fetch_npm_times(dependency: &str) -> Option<VersionTimes> {
    let url = format!("https://registry.npmjs.org/{}", encode_pkg(dependency));
    let body = http_get_json(&url)?;
    let time = body.get("time")?.as_object()?;
    let mut out = VersionTimes::new();
    for (version, value) in time {
        if version == "created" || version == "modified" {
            continue;
        }
        if let Some(iso) = value.as_str() {
            out.insert(version.clone(), iso.to_string());
        }
    }
    Some(out)
}

fn fetch_pypi_times(dependency: &str) -> Option<VersionTimes> {
    let url = format!("https://pypi.org/pypi/{}/json", encode_pkg(dependency));
    let body = http_get_json(&url)?;
    let releases = body.get("releases")?.as_object()?;
    let mut out = VersionTimes::new();
    for (version, files) in releases {
        let iso = files
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|f| f.get("upload_time_iso_8601"))
            .and_then(|v| v.as_str());
        if let Some(iso) = iso {
            out.insert(version.clone(), iso.to_string());
        }
    }
    Some(out)
}

fn http_get_json(url: &str) -> Option<serde_json::Value> {
    let resp = ureq::get(url).timeout(HTTP_TIMEOUT).call().ok()?;
    let mut body = String::new();
    resp.into_reader()
        .take(MAX_BODY_BYTES)
        .read_to_string(&mut body)
        .ok()?;
    serde_json::from_str(&body).ok()
}

/// Scoped npm names keep their slash url-encoded as `%2f` per the registry API.
fn encode_pkg(dependency: &str) -> String {
    dependency.replace('/', "%2f")
}

/// Whole days between an RFC3339 publish timestamp and now (UTC).
pub fn age_days_from_iso(iso: &str) -> Option<u32> {
    let published = OffsetDateTime::parse(iso, &Rfc3339).ok()?;
    let now = OffsetDateTime::now_utc();
    let days = (now - published).whole_days();
    if days < 0 {
        Some(0)
    } else {
        Some(days as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn age_days_from_future_iso_clamps_to_zero() {
        assert_eq!(age_days_from_iso("2999-01-01T00:00:00Z"), Some(0));
    }

    #[test]
    fn age_days_from_old_iso() {
        // 2000-01-01 is well over 9000 days ago.
        let age = age_days_from_iso("2000-01-01T00:00:00Z").unwrap();
        assert!(age > 9000, "expected large age, got {age}");
    }

    #[test]
    fn age_days_from_garbage_is_none() {
        assert!(age_days_from_iso("not-a-date").is_none());
    }

    #[test]
    fn encode_scoped_package() {
        assert_eq!(encode_pkg("@scope/pkg"), "@scope%2fpkg");
        assert_eq!(encode_pkg("vite"), "vite");
    }

    #[test]
    fn release_age_unsupported_toolchains_return_none() {
        assert!(release_age_days(ToolchainKind::Go, "x", "1.0.0").is_none());
        assert!(release_age_days(ToolchainKind::Rust, "x", "1.0.0").is_none());
        assert!(release_age_days(ToolchainKind::Proto, "node", "1.0.0").is_none());
    }
}
