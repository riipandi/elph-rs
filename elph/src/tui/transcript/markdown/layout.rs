//! Scroll row measurement for markdown assistant cards.

use elph_tui::wrapped_transcript_row_count;

use super::buffer::AssistantMarkdownBuffer;

pub fn markdown_part_row_count(source: &str, wrap_width: u16) -> u16 {
    wrapped_transcript_row_count(source, wrap_width)
}

pub fn assistant_row_count(content: &str, markdown: Option<&AssistantMarkdownBuffer>, wrap_width: u16) -> u16 {
    let Some(md) = markdown else {
        return wrapped_transcript_row_count(content, wrap_width);
    };
    let stable_rows: u16 = md.parts.iter().map(|part| part.row_count).sum();
    let tail = md.tail(content);
    stable_rows.saturating_add(wrapped_transcript_row_count(tail, wrap_width))
}
