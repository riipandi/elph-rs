//! Compact live activity bar shown while the agent is running.

use elph_tui::{Theme, ToolExecutionState, ToolExecutionStatus};
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct ActivityBarProps {
    pub command: Option<String>,
    pub live_tools: Option<State<Vec<ToolExecutionState>>>,
    pub theme: Theme,
}

#[component]
pub fn ActivityBar(props: &ActivityBarProps) -> impl Into<AnyElement<'static>> {
    let palette = props.theme;
    let command = props
        .command
        .as_deref()
        .map(|name| format!("owly {name}"))
        .unwrap_or_else(|| "owly".to_string());

    let running_count = props
        .live_tools
        .as_ref()
        .map(|state| {
            state
                .read()
                .iter()
                .filter(|tool| matches!(tool.status, ToolExecutionStatus::Running | ToolExecutionStatus::Pending))
                .count()
        })
        .unwrap_or(0);

    let status = if running_count > 0 {
        format!("⠋ {command} · {running_count} tool(s)")
    } else {
        format!("⠋ {command} · working")
    };

    let chips: Vec<AnyElement<'static>> = props
        .live_tools
        .as_ref()
        .map(|state| {
            state
                .read()
                .iter()
                .take(6)
                .map(|tool| {
                    let label = tool_chip_label(tool);
                    let color = tool_chip_color(tool.status);
                    element! {
                        Text(color: Some(color), content: label)
                    }
                    .into_any()
                })
                .collect()
        })
        .unwrap_or_default();

    element! {
        View(
            flex_shrink: 0.0,
            width: 100pct,
            padding_left: 1,
            padding_right: 1,
            padding_top: 0,
            padding_bottom: 0,
            border_style: BorderStyle::Single,
            border_color: palette.frame_border,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            gap: Gap::Length(1),
        ) {
            Text(content: status, color: Color::Cyan)
            #(if chips.is_empty() {
                None
            } else {
                Some(element! {
                    View(
                        flex_direction: FlexDirection::Row,
                        flex_grow: 1.0,
                        gap: Gap::Length(2),
                        align_items: AlignItems::Center,
                    ) {
                        #(chips)
                    }
                }.into_any())
            })
        }
    }
}

fn tool_chip_label(tool: &ToolExecutionState) -> String {
    let args = tool.args_summary.trim();
    if args.is_empty() {
        tool.name.clone()
    } else {
        format!("{} {}", tool.name, truncate_args(args))
    }
}

fn truncate_args(args: &str) -> String {
    const MAX: usize = 24;
    if args.len() <= MAX {
        args.to_string()
    } else {
        format!("{}…", &args[..MAX.saturating_sub(1)])
    }
}

fn tool_chip_color(status: ToolExecutionStatus) -> Color {
    match status {
        ToolExecutionStatus::Running | ToolExecutionStatus::Pending => Color::Cyan,
        ToolExecutionStatus::Success => Color::Green,
        ToolExecutionStatus::Error => Color::Red,
        ToolExecutionStatus::Cancelled => Color::Yellow,
    }
}
