//! Transcript snippets for `/demo-*` slash commands.

use crate::common::lipsum_mock::{mock_paragraph, mock_sentence, mock_title};
use crate::common::transcript::{ToolCardDetail, TranscriptMessage, TranscriptStyle};

pub fn demo_tool_success() -> Vec<TranscriptMessage> {
    vec![TranscriptMessage {
        content: String::new(),
        style: TranscriptStyle::ToolSuccess,
        tool: Some(ToolCardDetail {
            name: "read_file".to_string(),
            args: "crates/elph-tui/src/components/dialog_shell/frame.rs".to_string(),
            output: mock_sentence(),
        }),
    }]
}

pub fn demo_tool_failed() -> Vec<TranscriptMessage> {
    vec![TranscriptMessage {
        content: String::new(),
        style: TranscriptStyle::ToolFailed,
        tool: Some(ToolCardDetail {
            name: "bash".to_string(),
            args: "cargo test -p elph-tui".to_string(),
            output: "error: test failed (demo placeholder)".to_string(),
        }),
    }]
}

pub fn demo_thinking_pair() -> Vec<TranscriptMessage> {
    vec![
        TranscriptMessage::text(mock_sentence(), TranscriptStyle::Thinking),
        TranscriptMessage::text(mock_paragraph(), TranscriptStyle::Assistant),
    ]
}

pub fn demo_skill_prompt() -> Vec<TranscriptMessage> {
    vec![TranscriptMessage::text(
        format!("/demo-skill {}", mock_title()),
        TranscriptStyle::SkillPrompt,
    )]
}

pub fn demo_meta_notice() -> Vec<TranscriptMessage> {
    vec![TranscriptMessage::text(
        "Steering queued — will run after the current turn (demo)",
        TranscriptStyle::Meta,
    )]
}

pub fn demo_answer_line(kind: &str, detail: &str) -> Vec<TranscriptMessage> {
    vec![TranscriptMessage::text(
        format!("{kind}: {detail}"),
        TranscriptStyle::Meta,
    )]
}
