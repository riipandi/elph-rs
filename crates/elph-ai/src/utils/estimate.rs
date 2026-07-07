use crate::types::Context;

const CHARS_PER_TOKEN: f64 = 4.0;

pub struct ContextTokenEstimate {
    pub tokens: u32,
}

pub fn estimate_context_tokens(context: &Context) -> ContextTokenEstimate {
    let mut chars = 0usize;
    if let Some(sp) = &context.system_prompt {
        chars += sp.len();
    }
    for msg in &context.messages {
        match msg {
            crate::types::Message::User { content, .. } => match content {
                crate::types::UserContent::Text(t) => chars += t.len(),
                crate::types::UserContent::Blocks(blocks) => {
                    for b in blocks {
                        match b {
                            crate::types::ContentBlock::Text { text } => chars += text.len(),
                            crate::types::ContentBlock::Image { .. } => chars += 1000,
                        }
                    }
                }
            },
            crate::types::Message::Assistant(m) => {
                for block in &m.content {
                    match block {
                        crate::types::AssistantContentBlock::Text(t) => chars += t.text.len(),
                        crate::types::AssistantContentBlock::Thinking(t) => chars += t.thinking.len(),
                        crate::types::AssistantContentBlock::ToolCall(tc) => {
                            chars += tc.name.len() + tc.id.len();
                        }
                    }
                }
            }
            crate::types::Message::ToolResult { content, .. } => {
                for b in content {
                    match b {
                        crate::types::ContentBlock::Text { text } => chars += text.len(),
                        crate::types::ContentBlock::Image { .. } => chars += 1000,
                    }
                }
            }
        }
    }
    if let Some(tools) = &context.tools {
        for t in tools {
            chars += t.name.len() + t.description.len();
        }
    }
    ContextTokenEstimate {
        tokens: (chars as f64 / CHARS_PER_TOKEN).ceil() as u32,
    }
}
