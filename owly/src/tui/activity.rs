//! Live activity bar shown while the agent is running.

use elph_tui::{Theme, ToolExecutionCard, ToolExecutionStatus, TranscriptEntry};
use iocraft::prelude::*;

use super::transcript::tool_panel_entries;

#[derive(Default, Props)]
pub struct ActivityBarProps {
    pub command: Option<String>,
    pub entries: Option<State<Vec<TranscriptEntry>>>,
    pub theme: Theme,
}

#[component]
pub fn ActivityBar(props: &ActivityBarProps) -> impl Into<AnyElement<'static>> {
    let Some(entries) = props.entries else {
        return element! { View(width: 100pct, height: 0) };
    };

    let palette = props.theme;
    let tools = tool_panel_entries(&entries.read(), 4);
    let running_count = tools
        .iter()
        .filter(|tool| tool.status == ToolExecutionStatus::Running)
        .count();
    let command = props
        .command
        .as_deref()
        .map(|name| format!("Owly {name}"))
        .unwrap_or_else(|| "Owly".to_string());
    let status = if running_count > 0 {
        format!("{command} · {running_count} tool(s) running")
    } else {
        format!("{command} · thinking")
    };

    let cards: Vec<AnyElement<'static>> = tools
        .iter()
        .map(|tool| {
            element!(ToolExecutionCard(
                tool: tool.clone(),
                theme: palette,
                compact: true,
                on_approve: HandlerMut::default(),
                on_deny: HandlerMut::default(),
            ))
            .into_any()
        })
        .collect();

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
            flex_direction: FlexDirection::Column,
            gap: Gap::Length(1),
        ) {
            Text(content: format!("⠋ {status}"), color: Color::Cyan)
            #(if cards.is_empty() {
                None
            } else {
                Some(element! {
                    View(flex_direction: FlexDirection::Column, width: 100pct) {
                        #(cards)
                    }
                }.into_any())
            })
        }
    }
}
