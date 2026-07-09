use std::sync::{Arc, Mutex};

use elph_tui::{
    ActivityState, FooterInfo, FooterTokenDisplay, PromptAction, PromptOpts, PromptQueue, PromptState, ShellChrome,
    ShellRegion, SlashPaletteState, StatusBarInfo, Theme, ThinkingLevel, ToolExecutionState, ToolExecutionStatus,
    TranscriptEntry, TranscriptStyle, composer_demo_entries, consume_ctrl_char, consume_key_code_mod,
    default_activity_spinner, default_run_config, disable_keyboard_enhancement, elph_builtin_commands,
    enable_keyboard_enhancement, handle_prompt_input, handle_slash_palette_keys, is_quit_command, push_capped,
    read_git_branch, render_agent_shell, render_chat_stream_with_agent, render_prompt, sigint_channel,
    slash_palette_visible,
};
use slt::{Context, KeyCode, KeyModifiers, widgets::SpinnerState};

use crate::runtime::exit_message::ExitSnapshot;
use crate::runtime::{WAS_INTERRUPTED, exit_message, handle_prompt_interrupt};

pub struct ElphApp {
    pub prompt: PromptState,
    pub chat: elph_tui::ChatStreamState,
    pub theme: Theme,
    pub should_exit: bool,
    pub session_id: String,
    pub turn: u32,
    pub project_dir: String,
    pub thinking: ThinkingLevel,
    pub agent_running: bool,
    pub activity: ActivityState,
    pub spinner: SpinnerState,
    pub slash_palette: SlashPaletteState,
    pub slash_commands: Vec<elph_tui::SlashCommand>,
    pub git_branch: Option<String>,
    pub collapse: elph_tui::CollapseState,
    pub prompt_queue: PromptQueue,
}

impl ElphApp {
    pub fn new() -> Self {
        let session_id = exit_message::new_session_id();
        let project_dir = std::env::current_dir()
            .ok()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| ".".to_string());
        let git_branch = std::env::current_dir().ok().and_then(|p| read_git_branch(&p));
        let mut chat = elph_tui::ChatStreamState::new();
        chat.style = TranscriptStyle::Composer;
        chat.entries = composer_demo_entries();
        chat.show_thinking = true;

        Self {
            prompt: PromptState::new("elph"),
            chat,
            theme: Theme::detect(),
            should_exit: false,
            session_id,
            turn: 0,
            project_dir,
            thinking: ThinkingLevel::High,
            agent_running: false,
            activity: ActivityState::default(),
            spinner: default_activity_spinner(),
            slash_palette: SlashPaletteState::default(),
            slash_commands: elph_builtin_commands(),
            git_branch,
            collapse: elph_tui::CollapseState::default(),
            prompt_queue: PromptQueue::default(),
        }
    }

    pub fn handle_global_keys(&mut self, ui: &mut Context) {
        if self.agent_running {
            if consume_ctrl_char(ui, 'c') {
                self.activity.request_cancel();
            }
        } else if consume_ctrl_char(ui, 'c') && handle_prompt_interrupt(&mut self.prompt.textarea) {
            self.should_exit = true;
            return;
        }

        if !self.agent_running {
            if consume_ctrl_char(ui, 'x') || consume_ctrl_char(ui, 'd') {
                self.should_exit = true;
                return;
            }
            if consume_ctrl_char(ui, 'q') {
                self.should_exit = true;
                use std::sync::atomic::Ordering;
                WAS_INTERRUPTED.store(true, Ordering::Relaxed);
                #[cfg(unix)]
                crate::runtime::SHOULD_KILL_PARENT.store(true, Ordering::Relaxed);
                return;
            }
        }

        if consume_ctrl_char(ui, 'a') && !self.agent_running {
            self.prompt.cycle_mode();
        }
        if consume_ctrl_char(ui, 't') {
            self.theme = self.theme.toggle();
        }
        if consume_ctrl_char(ui, 'o') {
            let len = self.chat.entries.len();
            self.collapse.toggle_newest(len);
            self.chat.collapse = self.collapse.clone();
        }
        if consume_key_code_mod(ui, KeyCode::Tab, KeyModifiers::SHIFT) {
            self.thinking = self.thinking.next();
        }
    }

    fn start_turn(&mut self, user_text: &str) {
        self.turn = self.turn.saturating_add(1);
        self.agent_running = true;
        self.activity = ActivityState::responding();
        self.push_composer_turn(user_text);
        self.chat.pin_to_tail();
    }

    fn finish_turn(&mut self) {
        self.agent_running = false;
        self.activity.clear();
        self.drain_prompt_queue();
    }

    fn drain_prompt_queue(&mut self) {
        if self.agent_running {
            return;
        }
        if let Some(next) = self.prompt_queue.pop_front() {
            self.start_turn(&next);
            self.finish_turn();
        }
    }

    fn push_composer_turn(&mut self, user_text: &str) {
        push_capped(
            &mut self.chat.entries,
            TranscriptEntry::user(user_text),
            elph_tui::DEFAULT_TRANSCRIPT_CAP,
        );
        push_capped(
            &mut self.chat.entries,
            TranscriptEntry::thinking("Memproses permintaan…", false),
            elph_tui::DEFAULT_TRANSCRIPT_CAP,
        );
        push_capped(
            &mut self.chat.entries,
            TranscriptEntry::tool(
                ToolExecutionState::new("run", "shell")
                    .with_args("cargo check -p elph-tui")
                    .with_status(ToolExecutionStatus::Success)
                    .with_output("    Finished dev [unoptimized + debuginfo]"),
            ),
            elph_tui::DEFAULT_TRANSCRIPT_CAP,
        );
        push_capped(
            &mut self.chat.entries,
            TranscriptEntry::assistant(format!(
                "Selesai memproses: **{user_text}**\n\n\
                 Balasan agent akan muncul di sini setelah dispatch di-wire."
            )),
            elph_tui::DEFAULT_TRANSCRIPT_CAP,
        );
    }

    pub fn handle_prompt(&mut self, ui: &mut Context) {
        let input = self.prompt.value();
        if slash_palette_visible(&input) {
            match handle_slash_palette_keys(ui, &mut self.slash_palette, &input, &self.slash_commands) {
                elph_tui::SlashPaletteAction::Complete(cmd) => {
                    self.prompt.textarea.set_value(&cmd);
                    return;
                }
                elph_tui::SlashPaletteAction::Run(cmd) => {
                    self.prompt.textarea.set_value(&cmd);
                }
                _ => {}
            }
        }

        match handle_prompt_input(ui, &mut self.prompt, self.agent_running) {
            PromptAction::Submit(text) => {
                if is_quit_command(&text) {
                    self.prompt.clear();
                    self.should_exit = true;
                    return;
                }
                self.start_turn(&text);
                self.finish_turn();
            }
            PromptAction::Queue(text) => {
                if is_quit_command(&text) {
                    self.prompt.clear();
                    self.should_exit = true;
                    return;
                }
                self.prompt_queue.push_back(text);
            }
            PromptAction::Steer(text) => {
                if is_quit_command(&text) {
                    self.prompt.clear();
                    self.should_exit = true;
                    return;
                }
                self.activity.request_cancel();
                self.prompt_queue.push_front(text);
                if self.agent_running {
                    self.agent_running = false;
                    self.activity.clear();
                    self.drain_prompt_queue();
                }
            }
            PromptAction::Clear => self.prompt.clear(),
            PromptAction::CycleMode | PromptAction::None => {}
        }
    }
}

pub fn render_app(ui: &mut Context, app: &mut ElphApp) {
    app.handle_global_keys(ui);
    app.theme.apply_to(ui);
    let project_dir = app.project_dir.clone();
    let project_name = elph_tui::path_basename(&project_dir).to_string();
    let model_name = app.prompt.model_name.clone();
    let session_id = app.session_id.clone();
    let thinking = app.thinking.label();
    let branch = app.git_branch.clone();
    let branch_ref = branch.as_deref();

    let model_ref = if model_name.is_empty() {
        None
    } else {
        Some(model_name.as_str())
    };

    let footer = FooterInfo {
        model_name: model_ref,
        provider: None,
        thinking_level: thinking,
        supports_images: false,
        cost_usd: 0.0,
        tokens_used: 82_000,
        context_pct: 41.0,
        context_limit: 200_000,
        token_display: FooterTokenDisplay::Both,
        project_dir: &project_name,
        session_id: &session_id,
        mode: app.prompt.mode,
        turn: app.turn,
        branch: branch_ref,
        git_additions: 1,
        git_deletions: 0,
    };

    let status_bar = StatusBarInfo {
        branch: branch_ref,
        directory: &project_dir,
        tokens_used: footer.tokens_used,
        context_limit: footer.context_limit,
        git_additions: footer.git_additions,
        git_deletions: footer.git_deletions,
        turn: app.turn.max(1),
        turn_total: Some(4),
    };

    let input = app.prompt.value();
    app.chat.collapse = app.collapse.clone();

    if app.agent_running && !app.activity.visible {
        app.activity = ActivityState::responding();
    }

    let slash_commands = app.slash_commands.clone();
    let slash_palette = app.slash_palette.clone();
    let theme = app.theme;
    let agent_running = app.agent_running;

    let chrome = ShellChrome::composer(
        status_bar,
        footer,
        &input,
        &slash_commands,
        &slash_palette,
        agent_running,
        if agent_running && app.activity.visible {
            Some(app.activity.clone())
        } else {
            None
        },
        app.spinner.clone(),
    );

    render_agent_shell(ui, theme, chrome, |ui, region| match region {
        ShellRegion::Chat => {
            render_chat_stream_with_agent(ui, &mut app.chat, theme, agent_running);
        }
        ShellRegion::Input => {
            app.handle_prompt(ui);
            render_prompt(
                ui,
                &mut app.prompt,
                theme,
                PromptOpts {
                    running: agent_running,
                    composer: true,
                    queued_count: app.prompt_queue.len(),
                },
            );
        }
    });
}

pub async fn run_sigint_watcher(app: Arc<Mutex<ElphApp>>) {
    let mut sigint = sigint_channel();
    while sigint.recv().await {
        if let Ok(mut guard) = app.lock() {
            if guard.agent_running {
                guard.activity.request_cancel();
            } else if handle_prompt_interrupt(&mut guard.prompt.textarea) {
                guard.should_exit = true;
            }
        }
    }
}

pub fn run_tui() -> std::io::Result<()> {
    let _ = enable_keyboard_enhancement();
    struct KeyboardGuard;
    impl Drop for KeyboardGuard {
        fn drop(&mut self) {
            let _ = disable_keyboard_enhancement();
        }
    }
    let _guard = KeyboardGuard;

    let app = Arc::new(Mutex::new(ElphApp::new()));
    let watcher_app = Arc::clone(&app);

    std::thread::spawn(move || {
        elph_agent::block_on(run_sigint_watcher(watcher_app));
    });

    let config = default_run_config();
    slt::run_with(config, move |ui: &mut Context| {
        let mut guard = app.lock().expect("elph app lock");
        if guard.should_exit {
            exit_message::record(ExitSnapshot {
                session_id: guard.session_id.clone(),
                has_history: !guard.chat.entries.is_empty(),
            });
            ui.quit();
        }
        render_app(ui, &mut guard);
    })
}
