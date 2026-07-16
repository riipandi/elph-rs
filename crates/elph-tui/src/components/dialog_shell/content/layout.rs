//! Shared vertical rhythm for dialog body presets.
//!
//! Body presets use a single flex `gap` between blocks. Do not add `padding_top` for
//! "action" spacing on top of [`dialog_body_section_gap`] — that doubles the rhythm.

use crate::components::theme::UiTheme;

/// Gap between prompt, list, buttons, and keyboard hints inside a dialog body.
pub fn dialog_body_section_gap(theme: UiTheme) -> u16 {
    theme.dialog_section_gap()
}

/// Gap between rows in list-style dialog bodies.
pub fn dialog_body_row_gap(theme: UiTheme) -> u16 {
    theme.dialog_row_gap()
}
