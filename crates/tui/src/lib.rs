//! Terminal UI components and helpers for Elph agent applications.

pub mod components;
pub mod prompt;
pub mod terminal;
pub mod theme;

pub use components::{Label, LabelProps, frame};
pub use prompt::{
    AgentMode, ChatStream, ChatStreamProps, DEFAULT_LINE_SCROLL_STEP, EditAction, MacEditAction, PAGE_SCROLL_VIEWPORT,
    PromptInput, PromptInputProps, PromptTranscript, PromptTranscriptProps, edit_action, is_force_quit_key,
    is_interrupt_key, is_mode_cycle_key, is_newline_key, is_prompt_newline_key, is_quit_command, is_submit_key,
    is_theme_toggle_key, mac_edit_action,
};
pub use terminal::{
    SigintReceiver, disable_keyboard_enhancement, enable_keyboard_enhancement, key_combination, sigint_channel,
};
pub use theme::{Theme, ThemeMode};
