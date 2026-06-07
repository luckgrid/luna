use super::step::Step;
use serde::{Deserialize, Serialize};

/// A complete execution plan for a Luna target.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    /// The target verb this plan resolves (e.g. "build", "sync").
    pub target: String,
    /// Absolute path of the workspace root.
    pub workspace_root: String,
    /// Ordered steps to execute. Order matters; `depends_on` may introduce
    /// DAG relationships that a runtime executor should respect.
    pub steps: Vec<Step>,
    /// Content fingerprint for stale-plan rejection (set by `plan --out`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::planner::step::Step;

    #[test]
    fn plan_serialization_roundtrip() {
        let plan = Plan {
            target: "build".into(),
            workspace_root: "/tmp/repo".into(),
            steps: vec![Step {
                id: "moon:build".into(),
                adapter: "moon".into(),
                program: "moon".into(),
                args: vec!["run".into(), ":build".into()],
                cwd: None,
                env_selector: None,
                depends_on: Vec::new(),
                cacheability: "local".into(),
                mutability: "generate".into(),
                safety: "offline-safe".into(),
            }],
            fingerprint: None,
        };
        let json = serde_json::to_string_pretty(&plan).unwrap();
        let back: Plan = serde_json::from_str(&json).unwrap();
        assert_eq!(back.target, "build");
        assert_eq!(back.steps.len(), 1);
        assert_eq!(back.steps[0].id, "moon:build");
    }
}
