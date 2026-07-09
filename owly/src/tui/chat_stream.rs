//! Owly-specific chat stream with structured transcript layout.

use elph_tui::{Theme, ToolExecutionStatus, render_assistant_message};
use slt::{Color, Context, KeyCode, ScrollState};

use super::entries::{OwlyEntry, OwlyEntryKind};
use super::tool_display::{tool_transcript_body, tool_transcript_header};

pub struct OwlyChatState {
    pub scroll: ScrollState,
    pub scroll_enabled: bool,
}

impl Default for OwlyChatState {
    fn default() -> Self {
        Self {
            scroll: ScrollState::new(),
            scroll_enabled: true,
        }
    }
}

pub fn handle_owly_scroll(ui: &mut Context, state: &mut OwlyChatState) {
    if !state.scroll_enabled {
        return;
    }
    if ui.key_code(KeyCode::Up) {
        state.scroll.scroll_up(3);
    }
    if ui.key_code(KeyCode::Down) {
        state.scroll.scroll_down(3);
    }
    if ui.key_code(KeyCode::Home) {
        state.scroll.offset = 0;
    }
    if ui.key_code(KeyCode::End) {
        let max = state
            .scroll
            .content_height()
            .saturating_sub(state.scroll.viewport_height()) as usize;
        state.scroll.offset = max;
    }
}

pub fn render_owly_chat_stream(
    ui: &mut Context,
    state: &mut OwlyChatState,
    entries: &[OwlyEntry],
    theme: Theme,
    show_thinking: bool,
) {
    handle_owly_scroll(ui, state);
    let _ = ui.scroll_col(&mut state.scroll, |ui| {
        let mut prev_kind = None;
        for entry in entries {
            render_entry(ui, entry, theme, show_thinking, &mut prev_kind);
        }
    });
}

fn render_entry(
    ui: &mut Context,
    entry: &OwlyEntry,
    theme: Theme,
    show_thinking: bool,
    prev_kind: &mut Option<OwlyEntryKind>,
) {
    let gap = section_gap(*prev_kind, entry.kind);
    if gap > 0 {
        for _ in 0..gap {
            let _ = ui.text("");
        }
    }
    *prev_kind = Some(entry.kind);

    match entry.kind {
        OwlyEntryKind::Hint => {
            let content = entry.inner.content.trim();
            if !content.is_empty() {
                let _ = ui.text(content).fg(theme.muted);
            }
        }
        OwlyEntryKind::User => {
            if let Some(c) = theme.text_color() {
                let _ = ui.text(format_user(&entry.inner.content)).fg(c);
            } else {
                let _ = ui.text(format_user(&entry.inner.content));
            }
        }
        OwlyEntryKind::Assistant => {
            render_assistant_message(ui, &entry.inner.content, entry.inner.is_streaming, theme);
        }
        OwlyEntryKind::Thinking if show_thinking => {
            let label = if entry.inner.thinking_expanded {
                format!("Thinking:\n{}", entry.inner.content)
            } else {
                "Thinking…".to_string()
            };
            let _ = ui.text(label).fg(theme.muted);
        }
        OwlyEntryKind::Thinking => {}
        OwlyEntryKind::Status => {
            let content = entry.inner.content.trim();
            if !content.is_empty() {
                let _ = ui.text(format!("· {content}")).fg(theme.muted);
            }
        }
        OwlyEntryKind::CommandHeader => {
            let _ = ui.text(format!("▸ {}", entry.inner.content)).fg(Color::Cyan);
        }
        OwlyEntryKind::CommandResult => {
            let content = &entry.inner.content;
            if let Some(c) = command_result_color(content) {
                let _ = ui.text(content.clone()).fg(c);
            } else {
                let _ = ui.text(content.clone());
            }
        }
        OwlyEntryKind::ToolSummary => {
            if let Some(tool) = &entry.inner.tool {
                let header = tool_transcript_header(tool);
                if let Some(c) = tool_summary_color(tool.status) {
                    let _ = ui.text(header).fg(c);
                } else {
                    let _ = ui.text(header);
                }
                if let Some(body) = tool_transcript_body(tool) {
                    let _ = ui.text(body).fg(theme.muted);
                }
            }
        }
    }
}

fn section_gap(prev: Option<OwlyEntryKind>, current: OwlyEntryKind) -> u32 {
    let Some(prev) = prev else {
        return 0;
    };
    if prev == current {
        return match current {
            OwlyEntryKind::Status | OwlyEntryKind::ToolSummary => 0,
            _ => 1,
        };
    }
    match (prev, current) {
        (OwlyEntryKind::Hint, OwlyEntryKind::Hint) => 0,
        (OwlyEntryKind::Status, OwlyEntryKind::Status) => 0,
        (OwlyEntryKind::ToolSummary, OwlyEntryKind::ToolSummary) => 0,
        (OwlyEntryKind::CommandHeader, OwlyEntryKind::Status) => 0,
        (OwlyEntryKind::Status, OwlyEntryKind::Assistant) => 1,
        (OwlyEntryKind::User, _) => 2,
        (OwlyEntryKind::CommandHeader, _) => 2,
        (OwlyEntryKind::CommandResult, _) => 2,
        (OwlyEntryKind::Assistant, OwlyEntryKind::User) => 2,
        (OwlyEntryKind::Hint, OwlyEntryKind::User) => 2,
        _ => 1,
    }
}

fn format_user(message: &str) -> String {
    let trimmed = message.trim_end();
    let mut lines = trimmed.lines();
    let Some(first) = lines.next() else {
        return String::new();
    };
    let mut out = format!("> {first}");
    for line in lines {
        out.push('\n');
        out.push_str("  ");
        out.push_str(line);
    }
    out
}

fn command_result_color(content: &str) -> Option<Color> {
    if content.starts_with('✓') {
        Some(Color::Green)
    } else if content.starts_with('✗') {
        Some(Color::Red)
    } else {
        None
    }
}

fn tool_summary_color(status: ToolExecutionStatus) -> Option<Color> {
    match status {
        ToolExecutionStatus::Success => Some(Color::Green),
        ToolExecutionStatus::Error => Some(Color::Red),
        ToolExecutionStatus::Running | ToolExecutionStatus::Pending => Some(Color::Cyan),
        ToolExecutionStatus::Cancelled => Some(Color::Yellow),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elph_tui::{ToolExecutionState, ToolExecutionStatus};

    #[test]
    fn tool_transcript_renders_full_args_in_body() {
        let tool = ToolExecutionState::new("1", "bash")
            .with_args("ls -la /very/long/path/that/would/previously/be/truncated")
            .with_status(ToolExecutionStatus::Success);
        let body = crate::tui::tool_display::tool_transcript_body(&tool).expect("body");
        assert!(body.contains("ls -la /very/long/path/that/would/previously/be/truncated"));
    }

    #[test]
    fn section_gap_adds_space_before_user_turn() {
        assert_eq!(section_gap(Some(OwlyEntryKind::Assistant), OwlyEntryKind::User), 2);
        assert_eq!(section_gap(Some(OwlyEntryKind::Status), OwlyEntryKind::Status), 0);
    }
}
