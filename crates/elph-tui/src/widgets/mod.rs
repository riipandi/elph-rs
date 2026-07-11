//! tuie widget building blocks for the agent shell.

pub mod chrome_tuie;
pub mod command_palette;
pub mod prompt;
pub mod sidebar;
pub mod transcript;

pub use chrome_tuie::{build_activity_widget, build_footer_widget};
pub use command_palette::{
    CommandPaletteState, build_palette_widget, close_palette_popup, open_palette_popup, palette_visible,
};
pub use prompt::PromptPane;
pub use sidebar::SidebarPlaceholder;
pub use transcript::TranscriptPane;
