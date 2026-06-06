use crate::systems::model::{DependencyRow, ToolchainKind, ToolchainSnapshot};
use crate::systems::security;
use crate::ui::{map_console, LunaConsole};
use iocraft::prelude::*;
use miette::Result;
use starbase_console::ui::{
    Container, List, ListItem, Stack, Style, StyledText, Table, TableCol, TableHeader, TableRow,
    Text, View,
};

/// Max visible width of the Dependency column before trimming with `…`.
const DEP_BUDGET: usize = 24;

fn section_header_line(title: &str) -> String {
    format!("--- {} ---", title.to_uppercase())
}

fn section_header_element(title: &str) -> AnyElement<'static> {
    element! {
        Text(content: section_header_line(title), weight: Weight::Bold)
    }
    .into_any()
}

fn summary_count_style(count: usize, highlight: Style) -> Style {
    if count > 0 {
        highlight
    } else {
        Style::Muted
    }
}

fn summary_count_element(label: &str, count: usize, style: Style) -> AnyElement<'static> {
    element! {
        StyledText(content: format!("{label} {count}"), style)
    }
    .into_any()
}

/// Trim a dependency name to the column budget with a trailing ellipsis.
pub fn trim_dependency(name: &str) -> String {
    if name.chars().count() <= DEP_BUDGET {
        return name.to_string();
    }
    let keep: String = name.chars().take(DEP_BUDGET.saturating_sub(1)).collect();
    format!("{keep}…")
}

pub fn workspaces_text(row: &DependencyRow) -> String {
    if row.workspaces.is_empty() {
        "—".to_string()
    } else {
        row.workspaces.join(", ")
    }
}

fn newest_style(age_days: Option<u32>) -> Style {
    match age_days {
        Some(age) if (age as u64) >= security::min_release_age_days() => Style::Success,
        Some(_) => Style::Failure,
        None => Style::Muted,
    }
}

fn latest_style(one_major_ahead: bool) -> Style {
    if one_major_ahead {
        Style::Caution
    } else {
        Style::Muted
    }
}

pub fn release_age_cell(row: &DependencyRow, newest_label: &str) -> String {
    let mut parts = Vec::new();
    if let Some(age) = row.newest_release_age_days {
        parts.push(format!("{newest_label} {age}d"));
    }
    if let Some(age) = row.latest_release_age_days {
        parts.push(format!("latest {age}d"));
    }
    if parts.is_empty() {
        "—".to_string()
    } else {
        parts.join(" · ")
    }
}

fn table_headers() -> Vec<TableHeader> {
    vec![
        TableHeader::new("Toolchain", Size::Length(10)),
        TableHeader::new("Workspace", Size::Length(12)),
        TableHeader::new("Dependency", Size::Length(14)),
        TableHeader::new("Current", Size::Length(10)),
        TableHeader::new("Newest", Size::Length(10)),
        TableHeader::new("Latest", Size::Length(10)),
        TableHeader::new("Release Age", Size::Auto),
    ]
}

fn update_table_headers() -> Vec<TableHeader> {
    vec![
        TableHeader::new("Workspace", Size::Length(12)),
        TableHeader::new("Dependency", Size::Length(14)),
        TableHeader::new("Previous", Size::Length(10)),
        TableHeader::new("New", Size::Length(10)),
        TableHeader::new("Latest", Size::Length(10)),
        TableHeader::new("Release Age", Size::Auto),
    ]
}

pub fn render_outdated_table(console: &LunaConsole, snapshots: &[ToolchainSnapshot]) -> Result<()> {
    let rows = outdated_rows(snapshots);
    if rows.is_empty() {
        return Ok(());
    }

    let headers = table_headers();
    console
        .render(element! {
            View(margin_top: 1) {
                Container {
                    Table(headers: headers) {
                        #(rows.iter().enumerate().map(|(row_idx, (toolchain, row))| {
                            outdated_row_element(row_idx, toolchain, row)
                        }))
                    }
                }
            }
        })
        .map_err(map_console)
}

fn outdated_rows(snapshots: &[ToolchainSnapshot]) -> Vec<(&'static str, &DependencyRow)> {
    let mut rows = Vec::new();
    for kind in ToolchainKind::ORDER {
        let Some(tc) = snapshots.iter().find(|t| t.kind == kind) else {
            continue;
        };
        if !tc.has_updates() {
            continue;
        }
        let mut kind_rows: Vec<&DependencyRow> = tc.rows.iter().collect();
        kind_rows.sort_by(|a, b| {
            (a.workspaces.join(","), &a.dependency).cmp(&(b.workspaces.join(","), &b.dependency))
        });
        for row in kind_rows {
            rows.push((kind.label(), row));
        }
    }
    rows
}

fn outdated_row_element(
    row_idx: usize,
    toolchain: &str,
    row: &DependencyRow,
) -> AnyElement<'static> {
    let newest = row.newest.clone().unwrap_or_else(|| "—".to_string());
    let latest = row.latest.clone().unwrap_or_else(|| "—".to_string());
    element! {
        TableRow(row: row_idx as i32) {
            TableCol(col: 0) { Text(content: toolchain.to_string()) }
            TableCol(col: 1) { Text(content: workspaces_text(row)) }
            TableCol(col: 2) { Text(content: trim_dependency(&row.dependency)) }
            TableCol(col: 3) { Text(content: row.current.clone()) }
            TableCol(col: 4) {
                StyledText(content: newest, style: newest_style(row.newest_release_age_days))
            }
            TableCol(col: 5) {
                StyledText(content: latest, style: latest_style(row.latest_one_major_ahead))
            }
            TableCol(col: 6) {
                StyledText(
                    content: release_age_cell(row, "newest"),
                    style: Style::Muted,
                )
            }
        }
    }
    .into_any()
}

pub fn render_update_table(
    console: &LunaConsole,
    groups: &[(ToolchainKind, Vec<DependencyRow>)],
) -> Result<()> {
    if groups.is_empty() {
        return Ok(());
    }

    console
        .render(element! {
            View(margin_top: 1) {
                Container {
                    #(groups.iter().enumerate().map(|(idx, (kind, rows))| {
                        update_group_element(idx, kind.label(), rows)
                    }))
                }
            }
        })
        .map_err(map_console)
}

fn update_group_element(idx: usize, label: &str, rows: &[DependencyRow]) -> AnyElement<'static> {
    let headers = update_table_headers();
    let mut sorted: Vec<&DependencyRow> = rows.iter().collect();
    sorted.sort_by(|a, b| {
        (a.workspaces.join(","), &a.dependency).cmp(&(b.workspaces.join(","), &b.dependency))
    });

    element! {
        View(flex_direction: FlexDirection::Column) {
            #(section_header_element(label))
            Table(headers: headers) {
                #(sorted.iter().enumerate().map(|(row_idx, row)| {
                    update_row_element(idx * 100 + row_idx, row)
                }))
            }
        }
    }
    .into_any()
}

fn update_row_element(row_idx: usize, row: &DependencyRow) -> AnyElement<'static> {
    let blocked = row.blocked_reason.is_some();
    let new_text = row
        .new_version
        .clone()
        .or_else(|| row.newest.clone())
        .unwrap_or_else(|| "—".to_string());
    let new_style = if blocked {
        Style::Failure
    } else {
        newest_style(row.newest_release_age_days)
    };
    let mut age = release_age_cell(row, "new");
    if let Some(reason) = &row.blocked_reason {
        age = format!("{age} · blocked: {reason}");
    }
    let previous = row.previous.clone().unwrap_or_else(|| row.current.clone());
    let latest = row.latest.clone().unwrap_or_else(|| "—".to_string());

    element! {
        TableRow(row: row_idx as i32) {
            TableCol(col: 0) { Text(content: workspaces_text(row)) }
            TableCol(col: 1) { Text(content: trim_dependency(&row.dependency)) }
            TableCol(col: 2) { StyledText(content: previous, style: Style::Muted) }
            TableCol(col: 3) { StyledText(content: new_text, style: new_style) }
            TableCol(col: 4) {
                StyledText(content: latest, style: latest_style(row.latest_one_major_ahead))
            }
            TableCol(col: 5) { StyledText(content: age, style: Style::Muted) }
        }
    }
    .into_any()
}

pub fn render_release_age_section(console: &LunaConsole) -> Result<()> {
    let days = security::min_release_age_days();
    console
        .render(element! {
            View(margin_top: 1, margin_bottom: 1) {
                View(flex_direction: FlexDirection::Column) {
                    #(section_header_element("Release Age"))
                    View(padding_left: 2, padding_top: 1) {
                        Stack(gap: 1) {
                            Stack(gap: 0) {
                                StyledText(
                                    content: format!(
                                        "Luna waits {days} days after a package is published before installing it. `LUNA_MIN_RELEASE_AGE`, enforced via `bunfig.toml`, `.npmrc`, and `uv --exclude-newer`. This reduces risk from freshly published malicious releases."
                                    ),
                                    style: Style::Muted,
                                )
                                StyledText(
                                    content: "The Release Age column shows how many days ago each version was published.",
                                    style: Style::Muted,
                                )
                            }
                            StyledText(
                                content: "Run `luna update` to apply in-range Newest versions that pass the release-age policy.",
                                style: Style::Muted,
                            )
                            View(margin_top: 1) {
                                Stack(gap: 0) {
                                    StyledText(
                                        content: "To bypass for a one-off or emergency:",
                                        style: Style::Muted,
                                    )
                                    List {
                                ListItem {
                                    StyledText(
                                        content: "`luna update --major` — widen semver range (still age-gated unless overridden)",
                                        style: Style::Muted,
                                    )
                                }
                                ListItem {
                                    StyledText(
                                        content: "`LUNA_MIN_RELEASE_AGE=0 luna update` — disable cooldown for that invocation",
                                        style: Style::Muted,
                                    )
                                }
                                ListItem {
                                    StyledText(
                                        content: "`bun add <pkg>@<version>` — pin one package in a Bun workspace",
                                        style: Style::Muted,
                                    )
                                }
                                ListItem {
                                    StyledText(
                                        content: "`uv lock --upgrade-package <pkg>` — upgrade one Python dependency",
                                        style: Style::Muted,
                                    )
                                }
                            }
                                }
                            }
                        }
                    }
                }
            }
        })
        .map_err(map_console)
}

pub struct UpdateSummary {
    pub updated: usize,
    pub blocked: usize,
    pub failed: usize,
    pub skipped: usize,
    pub setup_ok: bool,
    pub show_major_tip: bool,
}

pub fn render_update_summary(console: &LunaConsole, summary: &UpdateSummary) -> Result<()> {
    console
        .render(element! {
            View(margin_top: 1) {
                View(flex_direction: FlexDirection::Column) {
                    #(section_header_element("Update summary"))
                    View(padding_left: 2, padding_top: 1) {
                        Stack(gap: 0) {
                            Stack(gap: 0) {
                                #(summary_count_element(
                                    "Updated",
                                    summary.updated,
                                    summary_count_style(summary.updated, Style::Success),
                                ))
                                #(summary_count_element(
                                    "Blocked",
                                    summary.blocked,
                                    summary_count_style(summary.blocked, Style::Caution),
                                ))
                                #(summary_count_element(
                                    "Failed",
                                    summary.failed,
                                    summary_count_style(summary.failed, Style::Failure),
                                ))
                                #(summary_count_element("Skipped", summary.skipped, Style::Muted))
                            }
                            View(margin_top: 1) {}
                            StyledText(
                                content: "Review changes before committing.",
                                style: Style::Muted,
                            )
                            #(if !summary.setup_ok {
                                Some(element! {
                                    StyledText(
                                        content: "Workspace re-sync encountered errors — see messages above.",
                                        style: Style::Failure,
                                    )
                                }.into_any())
                            } else {
                                None
                            })
                            #(if summary.show_major_tip {
                                Some(element! {
                                    StyledText(
                                        content: "Tip: re-run with `luna update --major` to also apply major-version bumps.",
                                        style: Style::Muted,
                                    )
                                }.into_any())
                            } else {
                                None
                            })
                            View(margin_bottom: 1) {}
                        }
                    }
                }
            }
        })
        .map_err(map_console)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn section_header_line_is_fixed_format() {
        assert_eq!(section_header_line("bun"), "--- BUN ---");
        assert_eq!(section_header_line("Release Age"), "--- RELEASE AGE ---");
    }

    #[test]
    fn trim_dependency_adds_ellipsis() {
        let trimmed = trim_dependency("@scope/a-very-long-package-name-here");
        assert!(trimmed.ends_with('…'));
        assert_eq!(trimmed.chars().count(), DEP_BUDGET);
    }

    #[test]
    fn trim_dependency_keeps_short_names() {
        assert_eq!(trim_dependency("vite"), "vite");
    }

    #[test]
    fn newest_style_respects_cutoff() {
        std::env::remove_var("LUNA_MIN_RELEASE_AGE");
        assert!(matches!(newest_style(Some(21)), Style::Success));
        assert!(matches!(newest_style(Some(3)), Style::Failure));
        assert!(matches!(newest_style(None), Style::Muted));
    }

    #[test]
    fn release_age_cell_formats_both() {
        let mut row = DependencyRow::outdated(
            ToolchainKind::Bun,
            "vite",
            "7.3.4",
            Some("7.3.5".into()),
            Some("8.0.14".into()),
        );
        row.newest_release_age_days = Some(21);
        row.latest_release_age_days = Some(8);
        assert_eq!(release_age_cell(&row, "newest"), "newest 21d · latest 8d");
    }
}
