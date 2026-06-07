use serde::{Deserialize, Serialize};

/// Cache key for a plan step, computed from the step's identity, inputs, and
/// environment state. Used for content-addressed local caching.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanCacheKey {
    pub step_id: String,
    pub adapter: String,
    pub normalized_command: String,
    pub manifest_hashes: Vec<String>,
    pub lockfile_hashes: Vec<String>,
    pub env_identity: String,
    pub platform_key: String,
}

impl PlanCacheKey {
    /// Compute a stable string representation suitable for hashing.
    pub fn fingerprint(&self) -> String {
        let mut parts = vec![
            self.step_id.clone(),
            self.adapter.clone(),
            self.normalized_command.clone(),
            self.env_identity.clone(),
            self.platform_key.clone(),
        ];
        parts.extend(self.manifest_hashes.iter().cloned());
        parts.extend(self.lockfile_hashes.iter().cloned());
        parts.join("::")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_fingerprint_stable() {
        let key = PlanCacheKey {
            step_id: "moon:build".into(),
            adapter: "moon".into(),
            normalized_command: "moon run :build".into(),
            manifest_hashes: vec!["abc123".into()],
            lockfile_hashes: vec!["def456".into()],
            env_identity: "default".into(),
            platform_key: "osx-arm64".into(),
        };
        let fp1 = key.fingerprint();
        let fp2 = key.fingerprint();
        assert_eq!(fp1, fp2);
        assert!(fp1.contains("moon:build"));
    }
}
