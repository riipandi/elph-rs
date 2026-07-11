//! TUI prompts for ask_* agent tools.

use elph_tui::{DEFAULT_TRANSCRIPT_CAP, push_capped};

use crate::tui::entries::OwlyEntry;
use crate::ui_events::{AskUserKind, AskUserResponse};

#[derive(Debug)]
pub struct PendingAsk {
    pub _tool_call_id: String,
    pub tool_name: String,
    pub question: String,
    pub kind: AskUserKind,
    pub response_tx: tokio::sync::oneshot::Sender<AskUserResponse>,
    pub _selected: usize,
}

impl PendingAsk {
    pub fn is_text(&self) -> bool {
        matches!(self.kind, AskUserKind::Text { .. })
    }

    pub fn push_transcript_notice(&self, entries: &mut Vec<OwlyEntry>) {
        let line = format!("{} asks: {}", self.tool_name, self.question);
        push_capped(entries, OwlyEntry::status(&line), DEFAULT_TRANSCRIPT_CAP);
    }

    pub fn finish_with_answer(self, answer: String) {
        let _ = self.response_tx.send(AskUserResponse::Answered(answer));
    }

    pub fn finish_cancelled(self) {
        let _ = self.response_tx.send(AskUserResponse::Cancelled);
    }
}

pub fn resolve_text_answer(text: String, kind: &AskUserKind) -> String {
    match kind {
        AskUserKind::Text { default } => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                default.clone().unwrap_or_default()
            } else {
                trimmed.to_string()
            }
        }
        _ => text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui_events::AskUserKind;

    #[test]
    fn resolve_text_uses_default_when_empty() {
        let kind = AskUserKind::Text {
            default: Some("fallback".into()),
        };
        assert_eq!(resolve_text_answer("".into(), &kind), "fallback");
        assert_eq!(resolve_text_answer("  ".into(), &kind), "fallback");
    }

    #[test]
    fn resolve_text_trims_submitted_answer() {
        let kind = AskUserKind::Text { default: None };
        assert_eq!(resolve_text_answer("  yes  ".into(), &kind), "yes");
    }

    #[test]
    fn pending_ask_identifies_text_kind() {
        let (tx, _rx) = tokio::sync::oneshot::channel();
        let pending = PendingAsk {
            _tool_call_id: "1".into(),
            tool_name: "ask_text".into(),
            question: "Name?".into(),
            kind: AskUserKind::Text { default: None },
            response_tx: tx,
            _selected: 0,
        };
        assert!(pending.is_text());
    }
}
