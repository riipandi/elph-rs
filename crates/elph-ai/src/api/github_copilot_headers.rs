use std::collections::HashMap;

use crate::types::{ContentBlock, Message, UserContent};

pub fn infer_copilot_initiator(messages: &[Message]) -> &'static str {
    match messages.last() {
        Some(Message::User { .. }) | None => "user",
        _ => "agent",
    }
}

pub fn has_copilot_vision_input(messages: &[Message]) -> bool {
    messages.iter().any(|msg| match msg {
        Message::User { content, .. } => match content {
            UserContent::Blocks(blocks) => blocks.iter().any(|b| matches!(b, ContentBlock::Image { .. })),
            UserContent::Text(_) => false,
        },
        Message::ToolResult { content, .. } => content.iter().any(|b| matches!(b, ContentBlock::Image { .. })),
        _ => false,
    })
}

pub fn build_copilot_dynamic_headers(messages: &[Message], has_images: bool) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("X-Initiator".to_string(), infer_copilot_initiator(messages).to_string());
    headers.insert("Openai-Intent".to_string(), "conversation-edits".to_string());
    if has_images {
        headers.insert("Copilot-Vision-Request".to_string(), "true".to_string());
    }
    headers
}
