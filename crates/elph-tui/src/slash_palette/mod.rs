//! Slash command palette model (draft → filtered commands).

mod fuzzy;
mod keyboard;
mod layout;
mod model;
mod state;

pub use fuzzy::filter_commands;
pub use keyboard::{SlashPaletteKeyAction, resolve_key_action, resolve_snapshot_key_action};
pub use layout::{PALETTE_EDITOR_GAP, editor_chrome_height, editor_max_height, palette_anchor_bottom};
pub use model::{
    FAST_SCROLL_STEP, MAX_VISIBLE_ROWS, PaletteSnapshot, build_snapshot, clamp_index, commands_to_options,
    complete_command, list_height, list_viewport_cap, open_palette_draft, palette_query, palette_visible,
    palette_window_start, selected_command_name,
};
pub use state::sync_selection;

/// Minimal command descriptor for palette filtering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
}

impl SlashCommand {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

/// List body height capped by terminal size (alias for [`list_height`]).
pub fn palette_list_height(match_count: usize, screen_height: u16) -> u16 {
    list_height(match_count, screen_height)
}
