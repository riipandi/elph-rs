use crate::agent::render_transcript_view;
use crate::theme::Theme;
use crate::transcript::TranscriptEntry;
use slt::{Context, KeyCode, ScrollState};

/// Default lines scrolled per Up/Down key press.
pub const DEFAULT_LINE_SCROLL_STEP: u16 = 3;

/// Use viewport height for Page Up/Down when zero.
pub const PAGE_SCROLL_VIEWPORT: u16 = 0;

/// Scrollable chat transcript backed by SLT [`ScrollState`].
pub struct ChatStreamState {
    pub scroll: ScrollState,
    pub messages: Vec<String>,
    pub entries: Vec<TranscriptEntry>,
    pub scroll_enabled: bool,
    pub auto_scroll: bool,
    pub line_scroll_step: u16,
    pub page_scroll_step: u16,
    pub show_thinking: bool,
}

impl ChatStreamState {
    pub fn new() -> Self {
        Self {
            scroll: ScrollState::new(),
            messages: Vec::new(),
            entries: Vec::new(),
            scroll_enabled: true,
            auto_scroll: true,
            line_scroll_step: DEFAULT_LINE_SCROLL_STEP,
            page_scroll_step: PAGE_SCROLL_VIEWPORT,
            show_thinking: true,
        }
    }

    pub fn with_messages(messages: Vec<String>) -> Self {
        Self {
            messages,
            ..Self::new()
        }
    }
}

impl Default for ChatStreamState {
    fn default() -> Self {
        Self::new()
    }
}

/// Handle keyboard scrolling for the transcript area.
pub fn handle_chat_scroll(ui: &mut Context, state: &mut ChatStreamState) {
    if !state.scroll_enabled {
        return;
    }

    let step = state.line_scroll_step.max(1);
    if ui.key_code(KeyCode::Up) {
        state.scroll.scroll_up(step as usize);
    }
    if ui.key_code(KeyCode::Down) {
        state.scroll.scroll_down(step as usize);
    }
    if ui.key_code(KeyCode::PageUp) {
        state.scroll.scroll_up(page_scroll_amount(state));
    }
    if ui.key_code(KeyCode::PageDown) {
        state.scroll.scroll_down(page_scroll_amount(state));
    }
    if ui.key_code(KeyCode::Home) {
        state.scroll.offset = 0;
    }
    if ui.key_code(KeyCode::End) && state.auto_scroll {
        let max = state
            .scroll
            .content_height()
            .saturating_sub(state.scroll.viewport_height()) as usize;
        state.scroll.offset = max;
    }
}

fn page_scroll_amount(state: &ChatStreamState) -> usize {
    if state.page_scroll_step == PAGE_SCROLL_VIEWPORT {
        state.scroll.viewport_height().max(1) as usize
    } else {
        state.page_scroll_step as usize
    }
}

/// Render scrollable chat content (plain messages or rich transcript entries).
pub fn render_chat_stream(ui: &mut Context, state: &mut ChatStreamState, theme: Theme) {
    handle_chat_scroll(ui, state);

    let _ = ui.scroll_col(&mut state.scroll, |ui| {
        if !state.entries.is_empty() {
            render_transcript_view(ui, &state.entries, state.show_thinking, theme);
        } else {
            for message in &state.messages {
                let color = theme.text_color();
                if let Some(c) = color {
                    ui.text(message).fg(c);
                } else {
                    ui.text(message);
                }
            }
        }
    });
}
