use crate::systems::model::{PackageUpdateResult, ToolchainSnapshot};
use crate::ui::tables::{
    render_outdated_table, render_release_age_section, render_update_result_footer,
    render_update_result_table, UpdateSummary,
};
use crate::ui::LunaConsole;
use miette::Result;

/// Render the outdated check table and release-age guidance.
pub fn render_outdated_report(
    console: &LunaConsole,
    snapshots: &[ToolchainSnapshot],
) -> Result<()> {
    render_outdated_table(console, snapshots)?;
    render_release_age_section(console)
}

/// Render the unified update result table and compact summary footer.
pub fn render_update_report(
    console: &LunaConsole,
    results: &[PackageUpdateResult],
    summary: &UpdateSummary,
) -> Result<()> {
    render_update_result_table(console, results)?;
    render_update_result_footer(console, summary)
}
