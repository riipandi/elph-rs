//! Auto session title generation via LLM.

use elph_ai::{Context, Message, Models, SimpleStreamOptions, StopReason};

use super::builtin::session_name::{
    SESSION_NAME_SYSTEM_PROMPT, build_session_name_prompt, extract_conversation_for_naming, sanitize_session_name,
};
use crate::types::AgentMessage;

/// Generate a short session title from the conversation transcript.
///
/// Returns `None` when there is no naming-worthy content or the model call fails.
pub async fn generate_session_name(
    messages: &[AgentMessage],
    models: &Models,
    model: &elph_ai::Model,
) -> Option<String> {
    let conversation = extract_conversation_for_naming(messages);
    if conversation.trim().is_empty() {
        return None;
    }

    let prompt = build_session_name_prompt(&conversation);
    let response = models
        .complete_simple(
            model,
            &Context {
                system_prompt: Some(SESSION_NAME_SYSTEM_PROMPT.to_string()),
                messages: vec![Message::User {
                    content: elph_ai::UserContent::Text(prompt),
                    timestamp: now_millis(),
                }],
                tools: None,
            },
            Some({
                let mut options = SimpleStreamOptions::from_stream(elph_ai::StreamOptions::default());
                options.base.max_tokens = Some(64);
                options
            }),
        )
        .await;

    if !matches!(response.stop_reason, StopReason::Stop | StopReason::Length) {
        return None;
    }

    let text = response
        .content
        .iter()
        .filter_map(|block| match block {
            elph_ai::AssistantContentBlock::Text(text) => Some(text.text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");
    let title = sanitize_session_name(&text);
    if title.is_empty() { None } else { Some(title) }
}

fn now_millis() -> i64 {
    (time::OffsetDateTime::now_utc().unix_timestamp_nanos() / 1_000_000) as i64
}
