use super::agent_mode::AgentMode;

/// Multiline prompt state shared by tuie shell hosts.
#[derive(Debug, Clone)]
pub struct PromptState {
    text: String,
    pub mode: AgentMode,
    pub model_name: String,
    pub show_help: bool,
    /// Allow Tab / Ctrl+Tab to cycle agent mode (disabled for simple shells).
    pub enable_mode_cycle: bool,
}

impl PromptState {
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            text: String::new(),
            mode: AgentMode::default(),
            model_name: model_name.into(),
            show_help: false,
            enable_mode_cycle: true,
        }
    }

    pub fn value(&self) -> String {
        self.text.clone()
    }

    pub fn set_value(&mut self, text: impl AsRef<str>) {
        self.text = text.as_ref().to_string();
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }

    pub fn cycle_mode(&mut self) {
        self.mode = self.mode.next();
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }
}
