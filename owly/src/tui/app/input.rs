use elph_tui::{
    ActivityState, DEFAULT_TRANSCRIPT_CAP, PromptAction, PromptOpts, SlashPaletteAction, consume_ctrl_char,
    handle_prompt_input, handle_slash_palette_keys, is_quit_command, push_capped, render_prompt, slash_palette_visible,
};
use slt::Context;

use super::OwlyApp;
use crate::tui::ask::{AskModalAction, handle_ask_modal_keys, resolve_text_answer};
use crate::tui::entries::OwlyEntry;
use crate::tui::slash::normalize_dispatch_text;

/// When `true`, Enter queues input instead of dispatching immediately.
fn prompt_treats_agent_as_busy(running: bool, pending_text_ask: bool) -> bool {
    running && !pending_text_ask
}

impl OwlyApp {
    pub(super) fn dispatch_prompt(&mut self, text: String) {
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

    pub(super) fn handle_global_keys(&mut self, ui: &mut Context) {
        if let Some(pending) = self.pending_ask.as_mut() {
            if ui.raw_key_code(slt::KeyCode::Esc) {
                if let Some(pending) = self.pending_ask.take() {
                    pending.finish_cancelled();
                }
                self.prompt.clear();
                return;
            }
            if !pending.is_text() {
                match handle_ask_modal_keys(ui, pending) {
                    AskModalAction::Answered(answer) => {
                        if let Some(pending) = self.pending_ask.take() {
                            self.record_ask_answer(&answer);
                            pending.finish_with_answer(answer);
                            self.resume_activity_after_ask();
                        }
                        return;
                    }
                    AskModalAction::None => return,
                }
            }
        }

        if self.running {
            if consume_ctrl_char(ui, 'c') {
                self.activity.request_cancel();
            }
        } else if consume_ctrl_char(ui, 'c') {
            self.prompt.clear();
            return;
        }

        if !self.running && consume_ctrl_char(ui, 'q') {
            self.should_exit = true;
            return;
        }
        if consume_ctrl_char(ui, 't') {
            self.theme = self.theme.toggle();
        }
    }

    pub(super) fn handle_prompt(&mut self, ui: &mut Context) {
        let input = self.prompt.value();
        let ask_modal_blocks_palette = self.pending_ask.as_ref().is_some_and(|pending| !pending.is_text());
        if !ask_modal_blocks_palette && slash_palette_visible(&input) {
            match handle_slash_palette_keys(ui, &mut self.slash_palette, &input, &self.slash_commands) {
                SlashPaletteAction::Complete(cmd) => {
                    self.prompt.textarea.set_value(&cmd);
                    return;
                }
                SlashPaletteAction::Run(cmd) => {
                    self.prompt.clear();
                    self.dispatch_prompt(cmd);
                    return;
                }
                SlashPaletteAction::MoveUp | SlashPaletteAction::MoveDown => return,
                SlashPaletteAction::None => {}
            }
        }

        let awaiting_text_ask = self.pending_ask.as_ref().is_some_and(|pending| pending.is_text());
        let agent_busy = prompt_treats_agent_as_busy(self.running, awaiting_text_ask);

        match handle_prompt_input(ui, &mut self.prompt, agent_busy) {
            PromptAction::Submit(text) => {
                if let Some(pending) = self.pending_ask.take() {
                    if pending.is_text() {
                        let answer = resolve_text_answer(text, &pending.kind);
                        self.record_ask_answer(&answer);
                        pending.finish_with_answer(answer);
                        self.prompt.clear();
                        self.resume_activity_after_ask();
                        return;
                    }
                    // Non-text prompts use the modal; restore state if we got here unexpectedly.
                    self.pending_ask = Some(pending);
                }
                if is_quit_command(&text) {
                    self.should_exit = true;
                    return;
                }
                self.dispatch_prompt(text);
            }
            PromptAction::Queue(text) => {
                if is_quit_command(&text) {
                    self.should_exit = true;
                    return;
                }
                self.prompt_queue.push_back(text);
            }
            PromptAction::Steer(text) => {
                if is_quit_command(&text) {
                    self.should_exit = true;
                    return;
                }
                // Queue for the next turn; the in-flight dispatcher still owns the current run.
                self.prompt_queue.push_front(text);
            }
            PromptAction::Clear => self.prompt.clear(),
            PromptAction::CycleMode | PromptAction::None => {}
        }
    }

    pub(super) fn render_input(&mut self, ui: &mut Context) {
        if let Some(pending) = self.pending_ask.as_ref()
            && !pending.is_text()
        {
            crate::tui::ask::render_ask_modal(ui, pending, self.theme);
        }
        let awaiting_text_ask = self.pending_ask.as_ref().is_some_and(|pending| pending.is_text());
        let ask_modal_blocks_prompt = self.pending_ask.as_ref().is_some_and(|pending| !pending.is_text());
        if !ask_modal_blocks_prompt {
            self.handle_prompt(ui);
        }
        render_prompt(
            ui,
            &mut self.prompt,
            self.theme,
            PromptOpts {
                running: prompt_treats_agent_as_busy(self.running, awaiting_text_ask),
                queued_count: self.prompt_queue.len(),
                show_mode: false,
                input_enabled: !ask_modal_blocks_prompt,
                ..Default::default()
            },
        );
        if self.prompt.show_help {
            self.render_prompt_help(ui);
        }
    }

    fn render_prompt_help(&self, ui: &mut Context) {
        if let Some(pending) = &self.pending_ask {
            let ask_help: &[(&str, &str)] = if pending.is_text() {
                &[
                    ("Enter", "submit answer"),
                    ("Esc", "cancel question"),
                    ("?", "toggle this help"),
                ]
            } else {
                &[
                    ("↑/↓", "choose option"),
                    ("Enter", "confirm selection"),
                    ("Esc", "cancel question"),
                    ("?", "toggle this help"),
                ]
            };
            let _ = ui.help(ask_help);
            return;
        }

        let _ = ui.help(&[
            ("Enter", "send message or slash command"),
            ("Ctrl+Enter", "queue follow-up for next turn"),
            ("Shift+Enter", "newline"),
            ("Ctrl+J", "newline"),
            ("/", "open slash command palette"),
            ("Tab", "complete slash command"),
            ("↑/↓", "navigate slash palette"),
            ("Esc", "clear prompt"),
            ("Ctrl+C", "cancel agent / clear prompt"),
            ("Ctrl+Q", "quit"),
            ("Ctrl+T", "toggle theme"),
            ("?", "toggle this help"),
        ]);
    }

    fn record_ask_answer(&mut self, answer: &str) {
        push_capped(
            &mut self.entries,
            OwlyEntry::user(format!("→ {answer}")),
            DEFAULT_TRANSCRIPT_CAP,
        );
    }

    fn resume_activity_after_ask(&mut self) {
        if self.running {
            self.activity = ActivityState::working();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::prompt_treats_agent_as_busy;

    #[test]
    fn text_ask_keeps_submit_path_while_agent_runs() {
        assert!(!prompt_treats_agent_as_busy(true, true));
    }

    #[test]
    fn agent_busy_when_running_without_text_ask() {
        assert!(prompt_treats_agent_as_busy(true, false));
        assert!(!prompt_treats_agent_as_busy(false, false));
    }
}
