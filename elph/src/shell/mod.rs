//! Interactive TUI application shell.
//!
//! ans: TUI reset — all complex shell infrastructure moved out.
//! The shell module now only provides launch options. The actual TUI
//! rendering lives in `crate::tui`.

/// Launch options for the interactive TUI.
#[derive(Debug, Clone, Default)]
pub struct TuiOptions {
    pub resume_id: Option<String>,
}
