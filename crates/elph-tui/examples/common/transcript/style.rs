//! Transcript style tokens — mirrors production / `chat_layout` spacing and colors.

use elph_tui::prelude::*;

const COLORED_CARD_PAD: u16 = 1;
const COLORED_CARD_PAD_H: u16 = COLORED_CARD_PAD + 1;
const COLORED_CARD_GAP: u16 = 1;
const THINKING_RESPONSE_GAP: u16 = 1;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TranscriptStyle {
    User,
    SkillPrompt,
    Thinking,
    Assistant,
    Error,
    Meta,
    ToolRunning,
    ToolSuccess,
    ToolFailed,
}

impl TranscriptStyle {
    pub fn is_sticky_prompt(self) -> bool {
        matches!(self, Self::User)
    }

    pub fn has_tinted_background(self) -> bool {
        !matches!(self.background_color(), Color::Reset)
    }

    pub fn is_flush_text(self) -> bool {
        matches!(self, Self::Thinking | Self::Assistant)
    }

    pub fn entry_gap_after(self, next: Option<TranscriptStyle>) -> u16 {
        match (self, next) {
            (Self::Thinking, Some(Self::Assistant)) => THINKING_RESPONSE_GAP,
            (Self::Assistant, Some(Self::Thinking)) => 0,
            (prev, Some(next)) if prev.is_flush_text() && next.has_tinted_background() => COLORED_CARD_GAP,
            _ if self.has_tinted_background() => COLORED_CARD_GAP,
            _ => 0,
        }
    }

    pub fn forms_flush_pair_with(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Thinking, Self::Assistant) | (Self::Assistant, Self::Thinking)
        )
    }

    pub fn sticky_padding_top(self) -> u16 {
        self.padding()
    }

    pub fn sticky_padding_bottom(self) -> u16 {
        self.padding()
    }

    pub fn sticky_bubble_padding_rows(self) -> u16 {
        self.sticky_padding_top().saturating_add(self.sticky_padding_bottom())
    }

    pub fn horizontal_padding(self) -> u16 {
        if self.is_flush_text() || self.has_tinted_background() {
            COLORED_CARD_PAD_H
        } else {
            0
        }
    }

    pub fn text_color(self) -> Color {
        match self {
            Self::Thinking => Color::DarkGrey,
            Self::SkillPrompt => rgb(149, 117, 205),
            Self::Meta => rgb(240, 198, 116),
            Self::User | Self::Assistant => rgb(212, 212, 212),
            Self::Error => rgb(204, 102, 102),
            Self::ToolRunning => rgb(128, 128, 128),
            Self::ToolSuccess => rgb(181, 189, 104),
            Self::ToolFailed => rgb(204, 102, 102),
        }
    }

    pub fn background_color(self) -> Color {
        match self {
            Self::Assistant | Self::Thinking => Color::Reset,
            Self::User => rgb(52, 53, 65),
            Self::Error => rgb(60, 40, 40),
            Self::SkillPrompt => rgb(45, 40, 56),
            Self::Meta => rgb(60, 55, 40),
            Self::ToolRunning => rgb(40, 40, 50),
            Self::ToolSuccess => rgb(40, 50, 40),
            Self::ToolFailed => rgb(60, 40, 40),
        }
    }

    pub fn padding(self) -> u16 {
        if self.has_tinted_background() {
            COLORED_CARD_PAD
        } else {
            0
        }
    }
}

pub fn tool_marker(style: TranscriptStyle) -> &'static str {
    match style {
        TranscriptStyle::ToolRunning => "○",
        TranscriptStyle::ToolSuccess => "●",
        TranscriptStyle::ToolFailed => "✕",
        _ => "○",
    }
}

/// Map transcript tool card style to a process lifecycle state.
pub fn tool_process_status(style: TranscriptStyle) -> ProcessStatus {
    match style {
        TranscriptStyle::ToolRunning => ProcessStatus::Running,
        TranscriptStyle::ToolSuccess => ProcessStatus::Done,
        TranscriptStyle::ToolFailed => ProcessStatus::Failed,
        _ => ProcessStatus::Queued,
    }
}

pub const TRANSCRIPT_SCROLL_STEP: i32 = 3;
pub const STICKY_MIN_SCROLL_ROWS: u16 = 3;
