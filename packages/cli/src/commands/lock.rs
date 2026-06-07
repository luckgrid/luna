use crate::cli::GlobalArgs;
use crate::config::LunaConfig;
use crate::output;
use crate::systems::ledger;
use miette::Result;
use std::path::Path;

pub fn run_lock(root: &Path, config: &LunaConfig, global: &GlobalArgs) -> Result<i32> {
    let locked = global.locked || global.frozen || config.policy.frozen_ci;
    let ledger = ledger::reconcile(root, config, locked, global.quiet)?;
    let rel = format!("{}/lock-ledger.json", config.state.dir);
    if global.json {
        output::emit(&output::LockLedgerReport::from_ledger(root, &ledger, &rel));
    } else if !global.quiet {
        eprintln!("\x1b[32m✓\x1b[0m Lock ledger written to {rel}");
        for entry in &ledger.adapters {
            let status = if entry.lock_ok { "ok" } else { "fail" };
            eprintln!("  {} [{status}] {} items", entry.adapter, entry.items.len());
        }
    }
    let failed = ledger.adapters.iter().any(|a| !a.lock_ok);
    Ok(if failed { 1 } else { 0 })
}
