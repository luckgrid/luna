pub mod events;
pub mod report;
pub mod status;
pub mod tables;

use iocraft::prelude::*;
use miette::Result;
use starbase_console::ui::{
    ConsoleTheme, Progress, ProgressDisplay, ProgressReporter, Style, StyledText, Text, Variant,
    View,
};
use starbase_console::{Console, ConsoleError, EmptyReporter};

pub use events::Emitter;
pub use report::{render_outdated_report, render_update_report};
pub use status::StatusPanel;
pub use tables::{
    render_outdated_table, render_release_age_section, render_update_result_footer,
    render_update_result_table, UpdateSummary,
};

/// Console instance used by Luna dependency commands.
pub type LunaConsole = Console<EmptyReporter>;

/// Build a Luna console with the empty reporter wired up.
pub fn new_console(quiet: bool) -> LunaConsole {
    let mut console: LunaConsole = Console::new(quiet);
    console.set_reporter(EmptyReporter);
    console
}

pub(crate) fn map_console(err: ConsoleError) -> miette::Report {
    miette::miette!("{err}")
}

fn trim_failure_body(body: &str) -> String {
    body.trim()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take(12)
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Default, Props)]
struct FailureNoticeProps {
    title: String,
    body: String,
}

#[component]
fn FailureNotice(props: &FailureNoticeProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = hooks.use_context::<ConsoleTheme>();
    let color = theme.variant(Variant::Failure);
    let title_color = if theme.supports_color {
        Some(color)
    } else {
        None
    };

    element! {
        View(
            flex_direction: FlexDirection::Column,
            border_color: color,
            border_edges: Edges::Left,
            border_style: BorderStyle::Round,
            margin_top: 1,
            margin_bottom: 0,
            padding_left: 1,
            padding_bottom: 0,
        ) {
            Text(
                content: props.title.to_uppercase(),
                color: title_color,
                weight: Weight::Bold,
            )
            StyledText(content: props.body.clone(), style: Style::Shell)
        }
    }
}

pub fn render_failure_notice(console: &LunaConsole, label: &str, body: &str) -> Result<()> {
    let trimmed = trim_failure_body(body);
    if trimmed.is_empty() {
        return Ok(());
    }

    let title = format!("{label} failed");
    console
        .render_err(element! {
            FailureNotice(title: title, body: trimmed)
        })
        .map_err(map_console)
}

pub fn render_message(console: &LunaConsole, content: &str) -> Result<()> {
    console
        .render(element! {
            Text(content: content.to_string())
        })
        .map_err(map_console)
}

pub fn render_section_title(console: &LunaConsole, title: &str) -> Result<()> {
    console
        .render(element! {
            View(margin_top: 1) {
                StyledText(content: title.to_string(), style: Style::Muted)
            }
        })
        .map_err(map_console)
}

/// Run work behind a single progress loader; `work` receives a reporter to update messages.
pub async fn run_with_loader<F>(
    console: &LunaConsole,
    initial_message: &str,
    work: F,
) -> Result<(), String>
where
    F: FnOnce(ProgressReporter) -> Result<(), String> + Send + 'static,
{
    let reporter = ProgressReporter::default();
    reporter.set_message(initial_message);

    let reporter_worker = reporter.clone();
    let handle = tokio::task::spawn_blocking(move || {
        let result = work(reporter_worker.clone());
        reporter_worker.exit();
        result
    });

    console
        .render_loop(element! {
            Progress(
                display: ProgressDisplay::Loader,
                default_message: initial_message.to_string(),
                reporter: Some(reporter.clone().into()),
            )
        })
        .await
        .map_err(|e| e.to_string())?;

    handle.await.map_err(|e| e.to_string())?
}
