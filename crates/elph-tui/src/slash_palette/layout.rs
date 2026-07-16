//! Anchor the palette above the editor without covering it.

use crate::layout_textarea;

/// Gap between palette bottom edge and editor top edge (terminal rows).
pub const PALETTE_EDITOR_GAP: u16 = 1;

/// Editor max height — kept in sync with chat shell prompt.
pub fn editor_max_height(screen_height: u16) -> u16 {
    (screen_height / 4).clamp(4, 12)
}

/// Visible editor block height in rows (border + textarea viewport).
pub fn editor_chrome_height(draft: &str, screen_width: u16, screen_height: u16) -> u16 {
    let textarea_width = screen_width.saturating_sub(2);
    let max_height = Some(editor_max_height(screen_height));
    let cursor = draft.len();
    let layout = layout_textarea(draft, cursor, textarea_width, 1, max_height);
    layout.viewport_height.saturating_add(2)
}

/// `bottom` offset for an absolutely positioned palette sitting above the editor.
pub fn palette_anchor_bottom(draft: &str, screen_width: u16, screen_height: u16) -> u16 {
    editor_chrome_height(draft, screen_width, screen_height).saturating_add(PALETTE_EDITOR_GAP)
}
