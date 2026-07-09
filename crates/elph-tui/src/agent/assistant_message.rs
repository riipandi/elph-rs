use crate::diff::{MarkdownTheme, render_streaming_markdown_lines};
use crate::theme::{Theme, ThemeMode};
use crate::utils::strip_ansi;
use slt::Context;

fn streaming_cursor_visible() -> bool {
    let ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);
    (ms / 400) % 2 == 0
}

fn markdown_theme_for(theme: Theme) -> MarkdownTheme {
    match theme.mode {
        ThemeMode::Light => MarkdownTheme::light(),
        ThemeMode::Dark => MarkdownTheme::dark(),
    }
}

fn render_markdown_lines_ui(ui: &mut Context, lines: &[String]) {
    for line in lines {
        if line.is_empty() {
            let _ = ui.text(" ");
        } else {
            let _ = ui.text(line);
        }
    }
}

/// Renders an assistant message with markdown formatting and optional streaming cursor.
pub fn render_assistant_message(ui: &mut Context, content: &str, is_streaming: bool, theme: Theme) {
    let content = strip_ansi(content);
    if content.trim().is_empty() && !is_streaming {
        return;
    }

    if is_streaming {
        let width = ui.width().max(20) as u16;
        let md_theme = markdown_theme_for(theme);
        let show_cursor = streaming_cursor_visible();
        let lines = render_streaming_markdown_lines(&content, width, md_theme, true, show_cursor);
        render_markdown_lines_ui(ui, &lines);
        return;
    }

    if !content.trim().is_empty() {
        let _ = ui.markdown(&content);
    }
}
