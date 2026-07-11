use crate::agent::CollapseState;
use crate::transcript::TranscriptEntry;

/// Transcript presentation style.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum TranscriptStyle {
    /// Pipe-column layout from `docs/tui.md`.
    #[default]
    Classic,
    /// Cursor Composer-style cards and blocks.
    Composer,
}

/// In-memory chat transcript state for tuie shell hosts.
#[derive(Debug, Default)]
pub struct ChatStreamState {
    pub messages: Vec<String>,
    pub entries: Vec<TranscriptEntry>,
    pub auto_scroll: bool,
    pub show_thinking: bool,
    pub collapse: CollapseState,
    pub style: TranscriptStyle,
}

impl ChatStreamState {
    pub fn new() -> Self {
        Self {
            auto_scroll: true,
            show_thinking: true,
            style: TranscriptStyle::default(),
            ..Self::default()
        }
    }

    pub fn with_messages(messages: Vec<String>) -> Self {
        Self {
            messages,
            ..Self::new()
        }
    }

    /// Re-pin the viewport to the tail (e.g. when the user submits a prompt).
    pub fn pin_to_tail(&mut self) {
        self.auto_scroll = true;
    }
}
