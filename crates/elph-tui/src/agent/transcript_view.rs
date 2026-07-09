use super::assistant_message::render_assistant_message;
use super::tool_execution::render_tool_execution_card;
use crate::components::text_optional_color;
use crate::theme::Theme;
use crate::transcript::{TranscriptEntry, TranscriptRole};
use slt::Context;

/// Renders a transcript column from the given entries.
pub fn render_transcript_view(ui: &mut Context, entries: &[TranscriptEntry], show_thinking: bool, theme: Theme) {
    let _ = ui.container().grow(1).col(|ui| {
        for entry in entries {
            render_entry(ui, entry, theme, show_thinking);
        }
    });
}

fn render_entry(ui: &mut Context, entry: &TranscriptEntry, theme: Theme, show_thinking: bool) {
    match entry.role {
        TranscriptRole::User => {
            text_optional_color(ui, format_user(&entry.content), theme.text_color());
        }
        TranscriptRole::Assistant => {
            render_assistant_message(ui, &entry.content, entry.is_streaming, theme);
        }
        TranscriptRole::Thinking if show_thinking => {
            let label = if entry.thinking_expanded {
                format!("Thinking:\n{}", entry.content)
            } else {
                "Thinking…".to_string()
            };
            text_optional_color(ui, &label, Some(theme.muted));
        }
        TranscriptRole::Thinking => {}
        TranscriptRole::Tool => {
            if let Some(tool) = entry.tool.as_ref() {
                render_tool_execution_card(ui, tool, theme, true);
            }
        }
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
