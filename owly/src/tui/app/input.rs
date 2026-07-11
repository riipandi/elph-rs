use elph_tui::{DEFAULT_TRANSCRIPT_CAP, push_capped};

use super::OwlyApp;
use crate::tui::entries::OwlyEntry;
use crate::tui::slash::normalize_dispatch_text;

impl OwlyApp {
    pub(crate) fn dispatch_prompt(&mut self, text: String) {
        let normalized = normalize_dispatch_text(&text);
        if normalized.is_empty() {
            return;
        }
        let _ = self.submit_tx.send(normalized);
    }

    pub(super) fn drain_prompt_queue(&mut self) {
        if self.running {
            return;
        }
        if let Some(next) = self.prompt_queue.pop_front() {
            self.dispatch_prompt(next);
        }
    }

    pub(crate) fn record_ask_answer(&mut self, answer: &str) {
        push_capped(
            &mut self.entries,
            OwlyEntry::user(format!("→ {answer}")),
            DEFAULT_TRANSCRIPT_CAP,
        );
    }

    pub(crate) fn resume_activity_after_ask(&mut self) {
        if self.running {
            self.activity = elph_tui::ActivityState::working();
        }
    }
}
