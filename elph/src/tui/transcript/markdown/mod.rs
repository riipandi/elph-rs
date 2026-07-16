//! Buffered, part-based markdown rendering for assistant stream responses.

mod buffer;
mod layout;
mod partition;
pub(crate) mod render;

pub use buffer::AssistantMarkdownBuffer;
pub use layout::assistant_row_count;

/// Refresh markdown caches for all assistant messages in `messages`.
///
/// Returns `true` when any buffer was updated.
pub fn refresh_assistant_markdown(messages: &mut [super::types::TranscriptMessage], screen_width: u16) -> bool {
    use super::types::TranscriptStyle;

    let mut changed = false;
    for message in messages.iter_mut() {
        if message.style != TranscriptStyle::Assistant {
            continue;
        }
        let wrap_width = elph_tui::transcript_bubble_inner_width(screen_width, message.style.horizontal_padding());
        if message.markdown.is_none() {
            message.markdown = Some(AssistantMarkdownBuffer::new());
        }
        if let Some(buffer) = message.markdown.as_mut()
            && buffer.refresh_stable(&message.content, wrap_width)
        {
            changed = true;
        }
    }
    changed
}
