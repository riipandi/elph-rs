//! Initial transcript data for the coding-agent simulator.

use crate::common::lipsum_mock::{mock_paragraph, mock_sentence};
use crate::common::transcript::{ToolCardDetail, TranscriptMessage, TranscriptStyle};

pub fn seed_transcript() -> Vec<TranscriptMessage> {
    vec![
        TranscriptMessage::text("Show me the TUI component demos.", TranscriptStyle::User),
        TranscriptMessage::text("/help", TranscriptStyle::SkillPrompt),
        TranscriptMessage::text(
            "Type / to open the slash palette — try /demo-mode, /demo-multi, /demo-todo, /demo-tool, and /demo-busy.",
            TranscriptStyle::Assistant,
        ),
        TranscriptMessage {
            content: String::new(),
            style: TranscriptStyle::ToolSuccess,
            tool: Some(ToolCardDetail {
                name: "read_file".to_string(),
                args: "examples/apps/coding_agent/main.rs".to_string(),
                output: "//! Coding agent — full AI chat TUI simulator.".to_string(),
            }),
        },
        TranscriptMessage::text(mock_sentence(), TranscriptStyle::Thinking),
        TranscriptMessage::text(mock_paragraph(), TranscriptStyle::Assistant),
        TranscriptMessage::text("Steering queued — will run after current turn", TranscriptStyle::Meta),
    ]
}
