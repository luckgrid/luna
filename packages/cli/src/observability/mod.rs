use crate::cli::GlobalArgs;
use std::sync::OnceLock;
use tracing::info;

static TRACE_INIT: OnceLock<()> = OnceLock::new();

/// Initialize JSONL tracing when `--trace` is enabled.
pub fn init_trace(global: &GlobalArgs) {
    if !global.trace {
        return;
    }
    TRACE_INIT.get_or_init(|| {
        let _ = tracing_subscriber::fmt()
            .json()
            .with_env_filter("luna=debug")
            .with_writer(std::io::stderr)
            .try_init();
    });
}

/// Emit a structured trace event when tracing is enabled.
pub fn event(global: &GlobalArgs, name: &str, fields: serde_json::Value) {
    if global.trace {
        info!(target: "luna", event = name, payload = %fields);
    }
}
