//! Auto session naming prompts and text helpers.

/// Maximum characters for an auto-generated session title.
pub const SESSION_NAME_MAX_LEN: usize = 60;

/// System prompt for the naming LLM call.
pub const SESSION_NAME_SYSTEM_PROMPT: &str =
    "You produce short conversation titles. Output only the title text, nothing else.";

/// Build the user prompt for session title generation.
pub fn build_session_name_prompt(conversation: &str) -> String {
    format!(
        "You are naming a conversation session. Based on the conversation below, produce a single short title \
         (max {SESSION_NAME_MAX_LEN} characters, no quotes). Be specific — mention the main task, file, or topic. \
         Use sentence case.\n\n<conversation>\n{conversation}\n</conversation>"
    )
}

/// Normalize a raw model title into a display-safe session name.
pub fn sanitize_session_name(raw: &str) -> String {
    let stripped: String = raw
        .trim()
        .chars()
        .filter(|ch| !matches!(ch, '"' | '\'' | '“' | '”' | '‘' | '’'))
        .collect();
    let oneline = stripped.split_whitespace().collect::<Vec<_>>().join(" ");
    if oneline.chars().count() > SESSION_NAME_MAX_LEN {
        oneline.chars().take(SESSION_NAME_MAX_LEN).collect()
    } else {
        oneline
    }
}

/// Extract user messages from the transcript for naming (tool results omitted).
pub fn extract_conversation_for_naming(messages: &[crate::types::AgentMessage]) -> String {
    use elph_ai::{ContentBlock, Message, UserContent};

    let mut parts = Vec::new();
    for message in messages {
        let Some(Message::User { content, .. }) = message.as_llm() else {
            continue;
        };
        let text = match content {
            UserContent::Text(value) => value.clone(),
            UserContent::Blocks(blocks) => blocks
                .iter()
                .filter_map(|block| match block {
                    ContentBlock::Text { text } => Some(text.as_str()),
                    _ => None,
                })
                .collect::<Vec<_>>()
                .join("\n"),
        };
        let trimmed = text.trim();
        if !trimmed.is_empty() {
            parts.push(format!("User: {trimmed}"));
        }
    }
    parts.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::AgentMessage;
    use elph_ai::{Message, UserContent};

    #[test]
    fn sanitize_strips_quotes_and_truncates() {
        let long = "a".repeat(80);
        assert_eq!(sanitize_session_name(&format!("\"{long}\"")), "a".repeat(60));
        assert_eq!(sanitize_session_name("  Fix login bug  "), "Fix login bug");
    }

    #[test]
    fn extract_conversation_collects_user_messages() {
        let messages = vec![
            AgentMessage::Llm(Box::new(Message::User {
                content: UserContent::Text("Explain auth flow".into()),
                timestamp: 0,
            })),
            AgentMessage::Llm(Box::new(Message::User {
                content: UserContent::Text("What about OAuth?".into()),
                timestamp: 0,
            })),
        ];
        let conversation = extract_conversation_for_naming(&messages);
        assert!(conversation.contains("User: Explain auth flow"));
        assert!(conversation.contains("User: What about OAuth?"));
    }
}
