//! Shell-wide and prompt submit actions for the tuie agent UI.

/// Global keyboard actions handled above individual widgets.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellAction {
    ToggleSidebar,
    OpenPalette,
    ToggleTheme,
    Quit,
    Cancel,
    TranscriptScrollUp,
    TranscriptScrollDown,
    TranscriptJumpTail,
}

/// Resolved Enter-key intent for the prompt input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromptSubmitMode {
    /// Plain Enter — submit when idle, queue when busy.
    Submit,
    /// Ctrl+Enter — steer / interrupt the in-flight response.
    Steer,
}
