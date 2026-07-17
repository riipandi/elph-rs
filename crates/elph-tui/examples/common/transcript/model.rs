//! Transcript message types for demo chat shells.

use super::style::{TranscriptStyle, tool_marker};

#[derive(Clone)]
pub struct ToolCardDetail {
    pub name: String,
    pub args: String,
    pub output: String,
}

#[derive(Clone)]
pub struct TranscriptMessage {
    pub content: String,
    pub style: TranscriptStyle,
    pub tool: Option<ToolCardDetail>,
}

impl TranscriptMessage {
    pub fn text(content: impl Into<String>, style: TranscriptStyle) -> Self {
        Self {
            content: content.into(),
            style,
            tool: None,
        }
    }

    pub fn layout_text(&self) -> String {
        if let Some(tool) = &self.tool {
            let mut lines = vec![format!("{} {}", tool_marker(self.style), tool.name)];
            if !tool.args.is_empty() {
                lines.push(tool.args.clone());
            }
            if !tool.output.is_empty() {
                lines.push(String::new());
                lines.extend(tool.output.lines().map(str::to_string));
            }
            lines.join("\n")
        } else {
            self.content.clone()
        }
    }
}
