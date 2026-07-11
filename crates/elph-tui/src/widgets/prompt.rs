//! Multiline prompt input for the tuie shell.

use crate::keymap::PromptSubmitMode;
use crate::prompt::{PromptAction, is_quit_command, strip_submit_trigger};
use crate::theme::Theme;
use tuie::prelude::*;

const PROMPT_MIN_ROWS: u16 = 1;
const PROMPT_MAX_ROWS: u16 = 6;

fn visible_rows(text: &str) -> u16 {
    let mut lines = text.lines().count();
    if lines == 0 {
        lines = 1;
    }
    if lines == 1 && text.is_empty() {
        return PROMPT_MIN_ROWS;
    }
    (lines as u16).clamp(PROMPT_MIN_ROWS, PROMPT_MAX_ROWS)
}

fn enter_mode(chord: &Chord) -> Option<PromptSubmitMode> {
    if *chord == chord!(Enter) {
        return Some(PromptSubmitMode::Submit);
    }
    if *chord == chord!(Ctrl + Enter) {
        return Some(PromptSubmitMode::Steer);
    }
    None
}

/// Bordered multiline prompt backed by tuie [`Input`].
pub struct PromptPane {
    root: Box<Pane>,
    input_id: WidgetId<Input>,
    pending_action: Option<PromptAction>,
    running: bool,
}

impl PromptPane {
    pub fn new(theme: Theme) -> Box<Self> {
        let mut input_id = WidgetId::EMPTY;
        let input = Input::new()
            .multiline()
            .wrap()
            .min_height(visible_rows(""))
            .max_height(PROMPT_MAX_ROWS)
            .placeholder(
                Text::new()
                    .content("Message the agent…")
                    .style(Style::new().fg(theme.input_placeholder())),
            )
            .id(&mut input_id);

        let root = Pane::new()
            .border(Border::SINGLE)
            .border_style(Style::new().fg(theme.frame_border))
            .padding(Spacing::balanced(1))
            .child(input as Box<dyn Widget>);

        Box::new(Self {
            root,
            input_id,
            pending_action: None,
            running: false,
        })
    }

    pub fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    pub fn set_content(&mut self, text: &str) {
        if let Some(input) = self.root.get_widget_mut(self.input_id) {
            input.set_content(text.to_string());
            input.set_min_height(Some(visible_rows(text)));
        }
    }

    pub fn content(&self) -> String {
        self.root
            .get_widget(self.input_id)
            .map(Input::get_string)
            .unwrap_or_default()
    }

    pub fn take_action(&mut self) -> Option<PromptAction> {
        self.pending_action.take()
    }

    fn handle_chord(&mut self, chord: &Chord) -> bool {
        let text = self.content();

        if *chord == chord!(Esc) && !text.is_empty() && !self.running {
            self.set_content("");
            self.pending_action = Some(PromptAction::Clear);
            return true;
        }

        if should_cycle_mode(&text, chord) && !self.running {
            self.pending_action = Some(PromptAction::CycleMode);
            return true;
        }

        if let Some(mode) = enter_mode(chord) {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                return true;
            }
            let submitted = strip_submit_trigger(trimmed);
            if is_quit_command(trimmed) {
                tuie::quit(0);
                return true;
            }
            self.set_content("");
            self.pending_action = Some(match mode {
                PromptSubmitMode::Submit if self.running => PromptAction::Queue(submitted),
                PromptSubmitMode::Submit => PromptAction::Submit(submitted),
                PromptSubmitMode::Steer if self.running => PromptAction::Steer(submitted),
                PromptSubmitMode::Steer => PromptAction::Submit(submitted),
            });
            return true;
        }

        false
    }
}

fn should_cycle_mode(text: &str, chord: &Chord) -> bool {
    if *chord == chord!(Ctrl + Tab) {
        return true;
    }
    *chord == chord!(Tab) && text.is_empty()
}

impl DelegateWidget for PromptPane {
    tuie::delegate_widget!(root);

    fn override_on_input(&mut self, queue: &mut InputQueue) -> InputResult {
        if let Some(event) = queue.peek()
            && self.handle_chord(&event.chord)
        {
            queue.next();
            return InputResult::Handled;
        }

        let result = self.root.on_input(queue);
        if let Some(input) = self.root.get_widget_mut(self.input_id) {
            input.set_min_height(Some(visible_rows(&input.get_string())));
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tuie::emulator::Emulator;

    fn send_enter(pane: &mut Box<PromptPane>, steer: bool) {
        let chord = if steer { chord!(Ctrl + Enter) } else { chord!(Enter) };
        let mut term = Emulator::new(&mut **pane, Vec2::new(40, 6));
        term.update(&mut **pane, &[RuntimeEvent::from(chord)]);
    }

    #[test]
    fn submit_when_idle() {
        let mut pane = PromptPane::new(Theme::dark());
        pane.set_content("hello");
        send_enter(&mut pane, false);
        assert!(matches!(
            pane.take_action(),
            Some(PromptAction::Submit(s)) if s == "hello"
        ));
        assert!(pane.content().is_empty());
    }

    #[test]
    fn queue_when_running() {
        let mut pane = PromptPane::new(Theme::dark());
        pane.set_running(true);
        pane.set_content("follow up");
        send_enter(&mut pane, false);
        assert!(matches!(
            pane.take_action(),
            Some(PromptAction::Queue(s)) if s == "follow up"
        ));
    }

    #[test]
    fn steer_when_running() {
        let mut pane = PromptPane::new(Theme::dark());
        pane.set_running(true);
        pane.set_content("interrupt");
        send_enter(&mut pane, true);
        assert!(matches!(
            pane.take_action(),
            Some(PromptAction::Steer(s)) if s == "interrupt"
        ));
    }

    #[test]
    fn esc_clears_only_when_idle() {
        let mut pane = PromptPane::new(Theme::dark());
        pane.set_content("draft");
        let mut term = Emulator::new(&mut *pane, Vec2::new(40, 6));
        term.update(&mut *pane, &[RuntimeEvent::from(chord!(Esc))]);
        assert!(matches!(pane.take_action(), Some(PromptAction::Clear)));
        assert!(pane.content().is_empty());

        pane.set_running(true);
        pane.set_content("busy");
        term.update(&mut *pane, &[RuntimeEvent::from(chord!(Esc))]);
        assert!(pane.take_action().is_none());
        assert_eq!(pane.content(), "busy");
    }

    #[test]
    fn strips_slash_trigger_on_submit() {
        let mut pane = PromptPane::new(Theme::dark());
        pane.set_content("/help");
        send_enter(&mut pane, false);
        assert!(matches!(
            pane.take_action(),
            Some(PromptAction::Submit(s)) if s == "help"
        ));
    }
}
