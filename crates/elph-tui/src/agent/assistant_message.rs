use crate::theme::Theme;
use crate::utils::strip_ansi;
use slt::Context;
use slt::widgets::StreamingMarkdownState;

/// Renders an assistant message with markdown formatting and optional streaming cursor.
pub fn render_assistant_message(ui: &mut Context, content: &str, is_streaming: bool, theme: Theme) {
    let _ = theme;
    let content = strip_ansi(content);

    let _ = ui.container().pl(1).grow(1).col(|ui| {
        if is_streaming {
            let mut state = StreamingMarkdownState::new();
            state.content = content;
            state.streaming = true;
            let _ = ui.streaming_markdown(&mut state);
        } else if !content.trim().is_empty() {
            let _ = ui.markdown(&content);
        }
    });
}
