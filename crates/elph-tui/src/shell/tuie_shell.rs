//! tuie-based agent shell composition.

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use tuie::prelude::*;

use crate::keymap::{GlobalChordHandler, SIDEBAR_MIN_TOTAL_WIDTH, SIDEBAR_WIDTH, ShellAction, ShellActionSink};
use crate::shell::host::{ShellChromeData, ShellHost};
use crate::theme::apply_tuie_theme;
use crate::widgets::{
    CommandPaletteState, PromptPane, SidebarPlaceholder, TranscriptPane, build_activity_widget, build_footer_widget,
    build_palette_widget, close_palette_popup, open_palette_popup, palette_visible,
};

/// Main agent shell widget: transcript, chrome stack, and optional sidebar.
pub struct AgentShell {
    root: Box<dyn Widget>,
    host: Rc<RefCell<dyn ShellHost>>,
    action_sink: ShellActionSink,
    transcript_id: WidgetId<TranscriptPane>,
    prompt_id: WidgetId<PromptPane>,
    poll_task: TaskHandle,
    palette_state: CommandPaletteState,
    palette_popup_id: Option<WidgetId>,
    sidebar_visible: bool,
}

impl AgentShell {
    /// Builds the shell wrapped in [`GlobalChordHandler`].
    #[allow(clippy::new_ret_no_self)] // returns `Box<dyn Widget>` via chord wrapper
    pub fn new(host: Rc<RefCell<dyn ShellHost>>) -> Box<dyn Widget> {
        let action_sink = ShellActionSink::default();
        let shell = Self::build(host, action_sink.clone());
        GlobalChordHandler::new(shell, action_sink)
    }

    fn build(host: Rc<RefCell<dyn ShellHost>>, action_sink: ShellActionSink) -> Box<Self> {
        let (theme, chrome, lines, prompt, commands) = {
            let host_ref = host.borrow();
            (
                host_ref.theme(),
                host_ref.chrome(),
                host_ref.transcript_lines(),
                host_ref.prompt_text(),
                host_ref.commands(),
            )
        };
        let _ = apply_tuie_theme(theme);

        let mut transcript_id = WidgetId::EMPTY;
        let mut prompt_id = WidgetId::EMPTY;
        let width = tuie::get_runtime_info().size.x;
        let sidebar_visible = chrome.sidebar_open && width >= SIDEBAR_MIN_TOTAL_WIDTH;

        let root = Self::assemble_root(
            theme,
            &chrome,
            &lines,
            &prompt,
            sidebar_visible,
            &mut transcript_id,
            &mut prompt_id,
        );

        let mut shell = Box::new(Self {
            root,
            host,
            action_sink,
            transcript_id,
            prompt_id,
            poll_task: TaskHandle::EMPTY,
            palette_state: CommandPaletteState::default(),
            palette_popup_id: None,
            sidebar_visible,
        });

        let shell_id = shell.get_id();
        shell.poll_task = tuie::schedule(shell_id, Duration::from_millis(100), |shell| {
            shell.poll_host();
        });

        if chrome.palette_open && palette_visible(&prompt) {
            shell.open_palette(&commands);
        }

        shell
    }

    fn assemble_root(
        theme: crate::theme::Theme,
        chrome: &ShellChromeData,
        lines: &[String],
        prompt: &str,
        sidebar_visible: bool,
        transcript_id: &mut WidgetId<TranscriptPane>,
        prompt_id: &mut WidgetId<PromptPane>,
    ) -> Box<dyn Widget> {
        let mut transcript = TranscriptPane::new(theme);
        transcript.set_lines(lines.to_vec());
        let transcript = transcript.id(transcript_id);

        let mut prompt_widget = PromptPane::new(theme);
        prompt_widget.set_content(prompt);
        prompt_widget.set_running(chrome.running);
        let prompt_widget = prompt_widget.id(prompt_id);

        let activity = build_activity_widget(chrome, theme);
        let footer = build_footer_widget(chrome, theme);

        let bottom = Pane::new().vertical().gap(0).children([
            activity,
            prompt_widget as Box<dyn Widget>,
            footer as Box<dyn Widget>,
        ]);

        let main = Pane::new()
            .vertical()
            .children([transcript.flex(1) as Box<dyn Widget>, bottom]);

        Self::wrap_sidebar(main, theme, sidebar_visible)
    }

    fn wrap_sidebar(main: Box<Pane>, theme: crate::theme::Theme, visible: bool) -> Box<dyn Widget> {
        if visible {
            let sidebar = SidebarPlaceholder::new(theme)
                .min_width(SIDEBAR_WIDTH)
                .max_width(SIDEBAR_WIDTH);
            Split::new(
                SplitPane::new()
                    .horizontal()
                    .children([SplitPaneChild::from(main.flex(1)), SplitPaneChild::from(sidebar)]),
            )
        } else {
            main
        }
    }

    fn prompt_text(&self) -> String {
        self.root
            .get_widget(self.prompt_id)
            .map(PromptPane::content)
            .unwrap_or_default()
    }

    fn open_palette(&mut self, commands: &[crate::diff::SlashCommand]) {
        let input = self.prompt_text();
        if !self.palette_state.forced && !palette_visible(&input) {
            return;
        }
        let theme = self.host.borrow().theme();
        let widget = build_palette_widget(commands, &input, &self.palette_state, theme);
        self.palette_popup_id = Some(open_palette_popup(widget).untyped());
    }

    fn rebuild_root(&mut self) {
        let (theme, chrome, lines, prompt) = {
            let host = self.host.borrow();
            (host.theme(), host.chrome(), host.transcript_lines(), host.prompt_text())
        };
        let width = tuie::get_runtime_info().size.x;
        self.sidebar_visible = chrome.sidebar_open && width >= SIDEBAR_MIN_TOTAL_WIDTH;
        self.root = Self::assemble_root(
            theme,
            &chrome,
            &lines,
            &prompt,
            self.sidebar_visible,
            &mut self.transcript_id,
            &mut self.prompt_id,
        );
        self.root.dirty_layout();
    }

    fn sync_from_host(&mut self) {
        let (theme, chrome, lines, prompt, commands, palette_open) = {
            let host = self.host.borrow();
            (
                host.theme(),
                host.chrome(),
                host.transcript_lines(),
                host.prompt_text(),
                host.commands(),
                host.palette_open(),
            )
        };
        let _ = apply_tuie_theme(theme);

        let width = tuie::get_runtime_info().size.x;
        let want_sidebar = chrome.sidebar_open && width >= SIDEBAR_MIN_TOTAL_WIDTH;
        if want_sidebar != self.sidebar_visible {
            self.rebuild_root();
        }

        if let Some(transcript) = self.root.get_widget_mut(self.transcript_id) {
            let follow = chrome.running || transcript.auto_scroll();
            transcript.set_lines(lines);
            if follow {
                transcript.set_auto_scroll(true);
            }
        }

        if let Some(prompt_widget) = self.root.get_widget_mut(self.prompt_id) {
            if prompt_widget.content() != prompt {
                prompt_widget.set_content(&prompt);
            }
            prompt_widget.set_running(chrome.running);
        }

        let prompt = self.prompt_text();
        let show_palette = palette_open && (self.palette_state.forced || palette_visible(&prompt));
        if show_palette {
            self.palette_state.sync_filter(&prompt);
            self.open_palette(&commands);
        } else {
            self.palette_state.forced = false;
            if let Some(id) = self.palette_popup_id.take() {
                close_palette_popup(id);
            }
        }
    }

    fn dispatch_shell_actions(&mut self, actions: Vec<ShellAction>) {
        for action in actions {
            if let Some(transcript) = self.root.get_widget_mut(self.transcript_id)
                && transcript.handle_shell_action(action)
            {
                continue;
            }

            match action {
                ShellAction::ToggleSidebar => {
                    let next = {
                        let host = self.host.borrow();
                        !host.sidebar_open()
                    };
                    self.host.borrow_mut().set_sidebar_open(next);
                    self.rebuild_root();
                }
                ShellAction::OpenPalette => {
                    self.palette_state.forced = true;
                    self.host.borrow_mut().set_palette_open(true);
                    let commands = self.host.borrow().commands();
                    self.open_palette(&commands);
                }
                ShellAction::ToggleTheme => {
                    let mut host = self.host.borrow_mut();
                    let next = host.theme().toggle();
                    host.set_theme(next);
                    let _ = apply_tuie_theme(next);
                }
                ShellAction::Quit => tuie::quit(0),
                other => self.host.borrow_mut().on_shell_action(other),
            }
        }
    }

    fn dispatch_prompt_actions(&mut self) {
        let action = self
            .root
            .get_widget_mut(self.prompt_id)
            .and_then(PromptPane::take_action);
        if let Some(action) = action {
            self.host.borrow_mut().on_prompt_action(action);
        }
    }

    fn poll_host(&mut self) {
        {
            let mut host = self.host.borrow_mut();
            host.poll();
            if host.should_exit() {
                tuie::quit(0);
                return;
            }
        }
        self.sync_from_host();
        let actions = self.action_sink.take();
        if !actions.is_empty() {
            self.dispatch_shell_actions(actions);
            self.sync_from_host();
        }
        self.dispatch_prompt_actions();
    }
}

impl DelegateWidget for AgentShell {
    tuie::delegate_widget!(root);

    fn after_on_input(&mut self, _result: InputResult) {
        let actions = self.action_sink.take();
        if !actions.is_empty() {
            self.dispatch_shell_actions(actions);
        }
        self.dispatch_prompt_actions();
        self.sync_from_host();
    }

    fn after_before_layout(&mut self) {
        self.poll_host();
    }
}
