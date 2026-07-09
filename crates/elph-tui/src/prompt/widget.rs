use super::agent_mode::AgentMode;
use super::editing::consume_prompt_textarea_keys;
use super::prompt_keys::{consume_mode_cycle_key, consume_prompt_clear, consume_submit_enter};
use crate::theme::Theme;
use slt::{Color, Context, KeyCode, TextareaState};

const PROMPT_VISIBLE_ROWS: u32 = 2;

/// Visual options for [`render_prompt`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PromptOpts {
    /// Agent turn in flight — show a compact working state instead of the editor.
    pub running: bool,
}

/// Prompt field state backed by SLT [`TextareaState`].
#[derive(Debug, Clone)]
pub struct PromptState {
    pub textarea: TextareaState,
    pub mode: AgentMode,
    pub model_name: String,
    pub show_help: bool,
}

impl PromptState {
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            textarea: TextareaState::new(),
            mode: AgentMode::default(),
            model_name: model_name.into(),
            show_help: false,
        }
    }

    pub fn value(&self) -> String {
        self.textarea.value()
    }

    pub fn clear(&mut self) {
        self.textarea.set_value("");
    }

    pub fn cycle_mode(&mut self) {
        self.mode = self.mode.next();
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}

/// Actions the prompt layer can signal to the parent app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptAction {
    None,
    Submit(String),
    Clear,
    CycleMode,
}

/// Handle global prompt shortcuts before rendering the textarea.
pub fn handle_prompt_input(ui: &mut Context, state: &mut PromptState) -> PromptAction {
    if ui.key_code(KeyCode::Char('?')) {
        state.toggle_help();
        return PromptAction::None;
    }

    let text = state.value();

    if consume_mode_cycle_key(ui, &text) {
        state.cycle_mode();
        return PromptAction::CycleMode;
    }

    if consume_prompt_clear(ui) && !text.is_empty() {
        state.clear();
        return PromptAction::Clear;
    }

    if consume_submit_enter(ui) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            let submitted = trimmed.to_string();
            state.clear();
            return PromptAction::Submit(submitted);
        }
    }

    PromptAction::None
}

/// Renders a minimal bottom prompt (Codex / Pi / Claude CLI style).
pub fn render_prompt(ui: &mut Context, state: &mut PromptState, theme: Theme, opts: PromptOpts) {
    if opts.running {
        let _ = ui.row(|ui| {
            let _ = ui.text("◌").fg(theme.muted);
            let _ = ui.text(" working…").fg(theme.muted);
            let _ = ui.spacer();
            let _ = ui.text(&state.model_name).fg(theme.muted).dim();
        });
    } else {
        let _ = ui.row(|ui| {
            let _ = ui.text("❯ ").fg(theme.prompt_prefix);
            let _ = ui.container().grow(1).col(|ui| {
                let _ = ui.register_focusable_named("prompt");
                consume_prompt_textarea_keys(ui, &mut state.textarea, true);
                let _ = ui.textarea(&mut state.textarea, PROMPT_VISIBLE_ROWS);
            });
        });
    }

    if state.show_help {
        let _ = ui.help(&[
            ("Enter", "send"),
            ("Shift+Enter", "newline"),
            ("Esc", "clear"),
            ("?", "toggle help"),
            ("Shift+↑/↓", "scroll"),
            ("Shift+End", "jump tail"),
            ("Ctrl+T", "theme"),
        ]);
    }
}

/// Apply optional foreground color to text (None inherits terminal default).
pub fn text_with_theme(ui: &mut Context, content: impl AsRef<str>, color: Option<Color>) {
    if let Some(c) = color {
        ui.text(content.as_ref()).fg(c);
    } else {
        ui.text(content.as_ref());
    }
}
