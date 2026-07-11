//! Scrollable transcript list for the tuie shell.

use crate::keymap::ShellAction;
use crate::theme::Theme;
use tuie::prelude::*;

const LINE_SCROLL_STEP: i32 = 1;

struct TranscriptRenderCtx {
    lines: Vec<String>,
    theme: Theme,
}

/// Virtualized transcript backed by tuie [`List`].
pub struct TranscriptPane {
    list: Box<List>,
    auto_scroll: bool,
    lines: Vec<String>,
}

impl TranscriptPane {
    pub fn new(theme: Theme) -> Box<Self> {
        let mut list = List::new();
        list.set_renderer(
            TranscriptRenderCtx {
                lines: Vec::new(),
                theme,
            },
            |ctx: &mut TranscriptRenderCtx, idx: usize| -> Option<Box<dyn Widget>> {
                let line = ctx.lines.get(idx)?.clone();
                Some(
                    Text::new()
                        .content(line)
                        .style(Style::new().fg(ctx.theme.foreground))
                        .overflow(TextOverflow::WRAP) as Box<dyn Widget>,
                )
            },
        );
        list.set_item_count(0);

        let list = list.scroll(Scrollbar::AutoHide);

        Box::new(Self {
            list,
            auto_scroll: true,
            lines: Vec::new(),
        })
    }

    pub fn set_lines(&mut self, lines: Vec<String>) {
        if let Some(ctx) = self.list.get_context_mut::<TranscriptRenderCtx>() {
            ctx.lines = lines;
            self.lines.clone_from(&ctx.lines);
        } else {
            self.lines = lines;
        }
        self.list.set_item_count(self.lines.len());
        if self.auto_scroll {
            self.list.set_scroll_progress(Axis2D::Y, 1.0);
        }
        self.list.dirty_layout();
    }

    pub fn set_auto_scroll(&mut self, auto_scroll: bool) {
        self.auto_scroll = auto_scroll;
        if auto_scroll {
            self.list.set_scroll_progress(Axis2D::Y, 1.0);
        }
    }

    pub fn auto_scroll(&self) -> bool {
        self.auto_scroll
    }

    pub fn scroll_up(&mut self, step: usize) {
        self.auto_scroll = false;
        self.list.scroll_by(-(step as i32).max(LINE_SCROLL_STEP));
    }

    pub fn scroll_down(&mut self, step: usize) {
        self.list.scroll_by(step as i32);
        if self.list.get_scroll_progress(Axis2D::Y) >= 1.0 {
            self.auto_scroll = true;
        }
    }

    pub fn jump_tail(&mut self) {
        self.auto_scroll = true;
        self.list.set_scroll_progress(Axis2D::Y, 1.0);
    }

    pub fn handle_shell_action(&mut self, action: ShellAction) -> bool {
        match action {
            ShellAction::TranscriptScrollUp => {
                self.scroll_up(1);
                true
            }
            ShellAction::TranscriptScrollDown => {
                self.scroll_down(1);
                true
            }
            ShellAction::TranscriptJumpTail => {
                self.jump_tail();
                true
            }
            _ => false,
        }
    }
}

impl DelegateWidget for TranscriptPane {
    tuie::delegate_widget!(list);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_lines_updates_count_and_follows_tail() {
        let mut pane = TranscriptPane::new(Theme::dark());
        pane.set_lines(vec!["one".into(), "two".into()]);
        assert_eq!(pane.lines.len(), 2);
        assert!(pane.auto_scroll());
    }

    #[test]
    fn scroll_up_disables_auto_scroll() {
        let mut pane = TranscriptPane::new(Theme::dark());
        pane.set_lines((0..20).map(|i| format!("line {i}")).collect());
        pane.scroll_up(1);
        assert!(!pane.auto_scroll());
    }

    #[test]
    fn jump_tail_re_enables_auto_scroll() {
        let mut pane = TranscriptPane::new(Theme::dark());
        pane.set_lines(vec!["a".into()]);
        pane.scroll_up(1);
        assert!(!pane.auto_scroll());
        pane.jump_tail();
        assert!(pane.auto_scroll());
    }

    #[test]
    fn shell_actions_are_consumed() {
        let mut pane = TranscriptPane::new(Theme::dark());
        pane.set_lines(vec!["a".into(), "b".into()]);
        assert!(pane.handle_shell_action(ShellAction::TranscriptScrollUp));
        assert!(!pane.handle_shell_action(ShellAction::ToggleSidebar));
    }
}
