//! Applies [`AgentUiEvent`] updates to rich [`TranscriptEntry`] lists.

use std::collections::HashMap;

use elph_tui::{
    DEFAULT_TRANSCRIPT_CAP, ToolExecutionState, ToolExecutionStatus, TranscriptEntry, TranscriptRole, push_capped,
};

use crate::ui_events::AgentUiEvent;

/// Mutable transcript state for one agent run.
pub struct TranscriptApplier<'a> {
    entries: &'a mut Vec<TranscriptEntry>,
    tool_indexes: HashMap<String, usize>,
    show_thinking: bool,
}

impl<'a> TranscriptApplier<'a> {
    pub fn new(entries: &'a mut Vec<TranscriptEntry>, show_thinking: bool) -> Self {
        Self {
            entries,
            tool_indexes: HashMap::new(),
            show_thinking,
        }
    }

    pub fn apply(&mut self, event: AgentUiEvent) {
        match event {
            AgentUiEvent::CommandStart {
                command,
                provider,
                model,
            } => self.push_command_start(&command, &provider, &model),
            AgentUiEvent::CommandComplete { message, success } => self.push_command_complete(&message, success),
            AgentUiEvent::Status(line) => self.push_status(&line),
            AgentUiEvent::TextDelta(delta) => self.append_assistant_text(&delta),
            AgentUiEvent::ThinkingDelta(delta) if self.show_thinking => self.append_thinking(&delta),
            AgentUiEvent::ToolStart { id, name, args_summary } => self.start_tool(id, name, args_summary),
            AgentUiEvent::ToolUpdate { id, output } => self.update_tool_output(&id, &output),
            AgentUiEvent::ToolEnd { id, is_error, output } => self.end_tool(&id, is_error, &output),
            AgentUiEvent::RunCompleted { elapsed_secs } => self.complete_run(elapsed_secs),
            AgentUiEvent::ThinkingDelta(_) => {}
        }
    }

    fn push_capped(&mut self, entry: TranscriptEntry) {
        push_capped(self.entries, entry, DEFAULT_TRANSCRIPT_CAP);
    }

    fn push_command_start(&mut self, command: &str, provider: &str, model: &str) {
        self.push_capped(TranscriptEntry::assistant(format!(
            ">_ Owly {command}\nprovider: {provider}\nmodel: {model}"
        )));
    }

    fn push_command_complete(&mut self, message: &str, success: bool) {
        if message.is_empty() {
            return;
        }
        let icon = if success { "✓" } else { "✗" };
        self.push_capped(TranscriptEntry::assistant(format!("{icon} {message}")));
    }

    fn push_status(&mut self, line: &str) {
        if line.is_empty() {
            self.push_capped(TranscriptEntry::assistant(String::new()));
            return;
        }
        self.push_capped(TranscriptEntry::assistant(format!("[status] {line}")));
    }

    fn append_assistant_text(&mut self, delta: &str) {
        if delta.is_empty() {
            return;
        }
        if let Some(last) = self.entries.last_mut()
            && last.role == TranscriptRole::Assistant
            && last.is_streaming
        {
            last.content.push_str(delta);
            return;
        }
        self.push_capped(TranscriptEntry::assistant_streaming(delta));
    }

    fn append_thinking(&mut self, delta: &str) {
        if delta.is_empty() {
            return;
        }
        if let Some(last) = self.entries.last_mut()
            && last.role == TranscriptRole::Thinking
        {
            last.content.push_str(delta);
            return;
        }
        self.push_capped(TranscriptEntry::thinking(delta, true));
    }

    fn start_tool(&mut self, id: String, name: String, args_summary: String) {
        let state = ToolExecutionState::new(id.clone(), name)
            .with_args(args_summary)
            .with_status(ToolExecutionStatus::Running);
        self.tool_indexes.insert(id, self.entries.len());
        self.push_capped(TranscriptEntry::tool(state));
    }

    fn update_tool_output(&mut self, id: &str, output: &str) {
        let Some(&idx) = self.tool_indexes.get(id) else {
            return;
        };
        let Some(entry) = self.entries.get_mut(idx) else {
            return;
        };
        let Some(tool) = entry.tool.as_mut() else {
            return;
        };
        tool.output = output.to_string();
    }

    fn end_tool(&mut self, id: &str, is_error: bool, output: &str) {
        let Some(&idx) = self.tool_indexes.get(id) else {
            return;
        };
        let Some(entry) = self.entries.get_mut(idx) else {
            return;
        };
        let Some(tool) = entry.tool.as_mut() else {
            return;
        };
        tool.status = if is_error {
            ToolExecutionStatus::Error
        } else {
            ToolExecutionStatus::Success
        };
        if !output.is_empty() {
            tool.output = output.to_string();
        }
    }

    fn complete_run(&mut self, elapsed_secs: f64) {
        for entry in self.entries.iter_mut() {
            if entry.role == TranscriptRole::Assistant {
                entry.is_streaming = false;
            }
        }
        self.push_status(&format!("Completed in {elapsed_secs:.1}s"));
    }
}

/// Tools to show in the live tool panel while a turn is running.
pub fn tool_panel_entries(entries: &[TranscriptEntry], max_visible: usize) -> Vec<ToolExecutionState> {
    let max_visible = max_visible.max(1);
    let mut tools: Vec<ToolExecutionState> = entries
        .iter()
        .filter_map(|entry| entry.tool.clone())
        .filter(|tool| matches!(tool.status, ToolExecutionStatus::Running | ToolExecutionStatus::Pending))
        .collect();

    if tools.is_empty() {
        tools = entries
            .iter()
            .filter_map(|entry| entry.tool.clone())
            .rev()
            .take(max_visible)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect();
    }

    tools.truncate(max_visible);
    tools
}

/// Convert plain startup hint lines into transcript entries.
pub fn lines_to_entries(lines: &[String]) -> Vec<TranscriptEntry> {
    lines
        .iter()
        .map(|line| {
            if line.is_empty() {
                TranscriptEntry::assistant(String::new())
            } else {
                TranscriptEntry::assistant(line.clone())
            }
        })
        .collect()
}

/// Append static shell output lines after a command finishes.
pub fn append_shell_lines(entries: &mut Vec<TranscriptEntry>, lines: &[String]) {
    let mut applier = TranscriptApplier::new(entries, false);
    for line in lines {
        applier.push_status(line);
    }
}

/// Infer the active Owly command from user input for the activity bar.
pub fn command_label_for_input(input: &str) -> Option<&'static str> {
    let lower = input.trim().to_ascii_lowercase();
    if lower == "/init" || lower.starts_with("/init ") {
        Some("init")
    } else if lower == "/update" || lower.starts_with("/update ") {
        Some("update")
    } else if matches!(lower.as_str(), "/exit" | "/quit" | "exit" | "quit" | ":q")
        || matches!(lower.as_str(), "/help" | "help" | "/clear" | "clear")
    {
        None
    } else {
        Some("chat")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_deltas_append_to_streaming_assistant() {
        let mut entries = Vec::new();
        let mut applier = TranscriptApplier::new(&mut entries, false);
        applier.apply(AgentUiEvent::TextDelta("Hel".into()));
        applier.apply(AgentUiEvent::TextDelta("lo".into()));
        assert_eq!(entries.len(), 1);
        assert!(entries[0].is_streaming);
        assert_eq!(entries[0].content, "Hello");
    }

    #[test]
    fn command_start_renders_header_block() {
        let mut entries = Vec::new();
        let mut applier = TranscriptApplier::new(&mut entries, false);
        applier.apply(AgentUiEvent::CommandStart {
            command: "Init".into(),
            provider: "opencode".into(),
            model: "big-pickle".into(),
        });
        assert!(entries[0].content.contains(">_ Owly Init"));
        assert!(entries[0].content.contains("provider: opencode"));
    }

    #[test]
    fn command_complete_renders_success_marker() {
        let mut entries = Vec::new();
        let mut applier = TranscriptApplier::new(&mut entries, false);
        applier.apply(AgentUiEvent::CommandComplete {
            message: "Documentation updated.".into(),
            success: true,
        });
        assert_eq!(entries[0].content, "✓ Documentation updated.");
    }

    #[test]
    fn tool_lifecycle_updates_card_status_and_output() {
        let mut entries = Vec::new();
        let mut applier = TranscriptApplier::new(&mut entries, false);
        applier.apply(AgentUiEvent::ToolStart {
            id: "t1".into(),
            name: "bash".into(),
            args_summary: "ls".into(),
        });
        applier.apply(AgentUiEvent::ToolUpdate {
            id: "t1".into(),
            output: "file.txt".into(),
        });
        applier.apply(AgentUiEvent::ToolEnd {
            id: "t1".into(),
            is_error: false,
            output: "file.txt\n".into(),
        });
        assert_eq!(entries.len(), 1);
        assert_eq!(
            entries[0].tool.as_ref().map(|t| t.status),
            Some(ToolExecutionStatus::Success)
        );
        assert_eq!(entries[0].tool.as_ref().map(|t| t.output.as_str()), Some("file.txt\n"));
    }

    #[test]
    fn tool_panel_prefers_running_tools() {
        let entries = vec![
            TranscriptEntry::tool(ToolExecutionState::new("a", "read").with_status(ToolExecutionStatus::Success)),
            TranscriptEntry::tool(ToolExecutionState::new("b", "bash").with_status(ToolExecutionStatus::Running)),
        ];
        let panel = tool_panel_entries(&entries, 4);
        assert_eq!(panel.len(), 1);
        assert_eq!(panel[0].name, "bash");
    }

    #[test]
    fn run_completed_clears_streaming_flag() {
        let mut entries = vec![TranscriptEntry::assistant_streaming("Hi")];
        let mut applier = TranscriptApplier::new(&mut entries, false);
        applier.apply(AgentUiEvent::RunCompleted { elapsed_secs: 1.2 });
        assert!(!entries[0].is_streaming);
        assert!(entries.last().unwrap().content.contains("Completed"));
    }

    #[test]
    fn command_label_for_slash_commands() {
        assert_eq!(command_label_for_input("/init"), Some("init"));
        assert_eq!(command_label_for_input("/update docs"), Some("update"));
        assert_eq!(command_label_for_input("hello"), Some("chat"));
        assert_eq!(command_label_for_input("/exit"), None);
    }
}
