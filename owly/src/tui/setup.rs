//! Setup wizard error state retained between tuie setup passes.

pub struct SetupWizardState {
    error: Option<String>,
}

impl SetupWizardState {
    pub fn new(_default_provider: &str, _default_model: &str) -> Self {
        Self { error: None }
    }

    pub fn set_error(&mut self, message: impl Into<String>) {
        self.error = Some(message.into());
    }

    pub fn clear_error(&mut self) {
        self.error = None;
    }
}
