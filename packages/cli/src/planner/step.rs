use serde::{Deserialize, Serialize};

/// A single step in an execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Stable identifier for this step (e.g. "pixi:sync", "moon:build").
    pub id: String,
    /// Which adapter/backend owns this step (e.g. "pixi", "moon", "bun", "uv", "cargo", "go", "native").
    pub adapter: String,
    /// Program to execute (e.g. "pixi", "moon", "bun", "cargo").
    pub program: String,
    /// Arguments to the program.
    pub args: Vec<String>,
    /// Working directory override (None = workspace root).
    pub cwd: Option<String>,
    /// Environment selector (e.g. "default", "ci", "docs").
    pub env_selector: Option<String>,
    /// IDs of steps this step depends on.
    pub depends_on: Vec<String>,
    /// Cache mode: "none", "local", "remote", "local-then-remote".
    pub cacheability: String,
    /// Mutability: "read-only", "lock-only", "install", "generate".
    pub mutability: String,
    /// Safety: "offline-safe", "network-required", "privileged".
    pub safety: String,
}
