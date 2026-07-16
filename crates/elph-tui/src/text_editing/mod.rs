//! GUI-style text editing helpers for terminal inputs.

mod actions;
mod focus;
mod input;
mod line;
mod submit;
mod wire;

pub use actions::TextEditAction;
pub use actions::{apply_action, delete_to_line_end, delete_to_line_start, delete_word_backward, delete_word_forward};
pub use actions::{insert_newline_at_cursor, wire_insert_newline};
pub use focus::{ShellFocus, prompt_focus_char, transcript_nav_key};
pub use input::wire_input_shortcuts;
pub use line::{is_word_char, line_end_offset, line_start_offset, next_word_offset, prev_word_offset};
pub use submit::{PASTE_BURST_WINDOW, PASTE_ECHO_GUARD_BASE, PASTE_ECHO_GUARD_PER_CHAR, PASTE_SUBMIT_GUARD_WINDOW};
pub use submit::{
    is_cursor_navigation_key, is_plain_submit_enter, is_slash_palette_capture_key, is_transcript_scroll_key,
};
pub use submit::{key_event_in_paste_burst, paste_echo_guard_duration, paste_submit_guarded, should_submit_on_enter};
pub use wire::WireEditResult;
pub use wire::{
    apply_wire_edit_key, match_key_to_action, wire_edit_apply_result, wire_edit_apply_to_cursor, wire_edit_handle_key,
};
