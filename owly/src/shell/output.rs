//! Shell output sink — stdout, buffered lines, or live TUI events.

use tokio::sync::mpsc;

use crate::ui_events::AgentUiEvent;

/// Writes shell status lines to stdout, a transcript buffer, or live TUI events.
pub struct ShellWriter<'a> {
    lines: Option<&'a mut Vec<String>>,
    ui_events: Option<mpsc::UnboundedSender<AgentUiEvent>>,
}

impl ShellWriter<'_> {
    pub fn stdout() -> ShellWriter<'static> {
        ShellWriter {
            lines: None,
            ui_events: None,
        }
    }

    pub fn transcript(lines: &mut Vec<String>) -> ShellWriter<'_> {
        ShellWriter {
            lines: Some(lines),
            ui_events: None,
        }
    }

    pub fn live_ui(lines: &mut Vec<String>, ui_events: mpsc::UnboundedSender<AgentUiEvent>) -> ShellWriter<'_> {
        ShellWriter {
            lines: Some(lines),
            ui_events: Some(ui_events),
        }
    }

    pub fn is_transcript(&self) -> bool {
        self.lines.is_some()
    }

    pub fn has_live_ui(&self) -> bool {
        self.ui_events.is_some()
    }

    pub fn ui_sender(&self) -> Option<mpsc::UnboundedSender<AgentUiEvent>> {
        self.ui_events.clone()
    }

    pub fn line(&mut self, text: impl AsRef<str>) {
        let text = text.as_ref();
        if let Some(tx) = &self.ui_events {
            let _ = tx.send(AgentUiEvent::Status(text.to_string()));
            return;
        }
        if let Some(lines) = self.lines.as_mut() {
            lines.push(text.to_string());
        } else {
            println!("{text}");
        }
    }

    pub fn command_start(&mut self, command: &str, provider: &str, model: &str) {
        if let Some(tx) = &self.ui_events {
            let _ = tx.send(AgentUiEvent::CommandStart {
                command: command.to_string(),
                provider: provider.to_string(),
                model: model.to_string(),
            });
            return;
        }
        self.blank();
        self.line(format!(">_ Owly {command}"));
        self.line(format!("provider: {provider}"));
        self.line(format!("model: {model}"));
        self.blank();
    }

    pub fn command_complete(&mut self, message: &str, success: bool) {
        if message.is_empty() {
            return;
        }
        if let Some(tx) = &self.ui_events {
            let _ = tx.send(AgentUiEvent::CommandComplete {
                message: message.to_string(),
                success,
            });
            return;
        }
        self.blank();
        let prefix = if success { "✓ " } else { "✗ " };
        self.line(format!("{prefix}{message}"));
        self.blank();
    }

    pub fn blank(&mut self) {
        self.line("");
    }
}
