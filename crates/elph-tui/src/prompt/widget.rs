use super::agent_mode::AgentMode;
use super::prompt_keys::should_cycle_agent_mode;
use crate::theme::Theme;
use slt::{Border, Color, Context, KeyCode, KeyModifiers, TextareaState};

const PROMPT_VISIBLE_ROWS: u32 = 4;

/// Prompt field state backed by SLT [`TextareaState`].
#[derive(Debug, Clone)]
pub struct PromptState {
    pub textarea: TextareaState,
    pub mode: AgentMode,
    pub model_name: String,
}

impl PromptState {
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            textarea: TextareaState::new(),
            mode: AgentMode::default(),
            model_name: model_name.into(),
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
    let text = state.value();

    if ui.raw_key_mod('\t', KeyModifiers::CONTROL)
        || (ui.raw_key_code(KeyCode::Tab) && should_cycle_agent_mode(&text, &KeyCode::Tab, KeyModifiers::NONE))
    {
        state.cycle_mode();
        return PromptAction::CycleMode;
    }

    if ui.raw_key_code(KeyCode::Esc) && !text.is_empty() {
        state.clear();
        return PromptAction::Clear;
    }

    if ui.raw_key_code(KeyCode::Enter) && !ui.key_mod('\n', KeyModifiers::SHIFT) {
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            let submitted = trimmed.to_string();
            state.clear();
            return PromptAction::Submit(submitted);
        }
    }

    PromptAction::None
}

/// Renders the agent prompt: bordered textarea plus mode/model footer.
pub fn render_prompt(ui: &mut Context, state: &mut PromptState, theme: Theme) {
    let mode_color = theme.mode_accent(state.mode);
    let footer = format!("{} • {}", state.model_name, state.mode.label());

    let _ = ui
        .bordered(Border::Rounded)
        .border_fg(theme.frame_border)
        .title("Prompt")
        .p(1)
        .gap(1)
        .col(|ui| {
            let _ = ui.textarea(&mut state.textarea, PROMPT_VISIBLE_ROWS);
            let _ = ui.row(|ui| {
                let _ = ui.text(footer).fg(mode_color);
                let _ = ui.spacer();
                let _ = ui.text("Enter submit · Tab mode · Esc clear").dim();
            });
        });
}

/// Apply optional foreground color to text (None inherits terminal default).
pub fn text_with_theme(ui: &mut Context, content: impl AsRef<str>, color: Option<Color>) {
    if let Some(c) = color {
        ui.text(content.as_ref()).fg(c);
    } else {
        ui.text(content.as_ref());
    }
}
