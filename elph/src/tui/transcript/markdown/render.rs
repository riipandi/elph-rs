//! Themed markdown line rendering for assistant transcript cards.

use elph_tui::components::markdown::render_markdown_lines;
use iocraft::prelude::*;

use crate::tui::theme::{META_FG, TEXT_FG, TOOL_SUCCESS_FG};

/// Render one stable markdown slice into iocraft elements.
pub fn render_markdown_part(source: &str) -> Vec<AnyElement<'static>> {
    if source.is_empty() {
        return Vec::new();
    }
    // elph-tui renderer uses Grey/Cyan/Green; acceptable for phase 1.
    // Theme mapping deferred until inline styles land in shared renderer.
    let _ = (TEXT_FG, META_FG, TOOL_SUCCESS_FG);
    render_markdown_lines(source)
}
