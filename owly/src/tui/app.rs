//! Owly interactive shell application (SuperLightTUI).

use std::sync::{Arc, Mutex};

use elph_agent::try_block_on;
use elph_tui::{
    ActivityState, BannerInfo, DEFAULT_TRANSCRIPT_CAP, FooterInfo, PromptAction, PromptOpts, PromptQueue, PromptState,
    ShellChrome, ShellRegion, Theme, ToolExecutionState, consume_ctrl_char, default_activity_spinner,
    default_run_config, disable_keyboard_enhancement, enable_keyboard_enhancement, handle_prompt_input,
    is_quit_command, pick_tip, push_capped, render_agent_shell, render_prompt,
};
use slt::{Context, widgets::SpinnerState};
use tokio::sync::mpsc;

use crate::env;
use crate::onboarding::{self, SetupCredentials};
use crate::ui_events::AgentUiEvent;

use super::banner::directory_display;
use super::chat_stream::{OwlyChatState, render_owly_chat_stream};
use super::context::AppContext;
use super::entries::OwlyEntry;
use super::launch::LaunchState;
use super::setup::{SetupWizardState, render_setup_wizard};
use super::transcript::{TranscriptApplier, append_shell_lines, lines_to_entries};

#[derive(Debug)]
enum AppMessage {
    UiEvent(AgentUiEvent),
    DispatchDone { lines: Vec<String>, should_exit: bool },
    DispatchError(String),
}

pub struct OwlyApp {
    pub context: AppContext,
    pub entries: Vec<OwlyEntry>,
    pub live_tools: Vec<ToolExecutionState>,
    pub prompt: PromptState,
    pub chat: OwlyChatState,
    pub theme: Theme,
    pub running: bool,
    pub setup_complete: bool,
    pub setup: SetupWizardState,
    pub setup_error: Option<String>,
    pub provider: String,
    pub model: String,
    pub show_thinking: bool,
    pub should_exit: bool,
    pub submit_tx: mpsc::UnboundedSender<String>,
    pub tip: &'static str,
    pub turn: u32,
    pub session_id: String,
    pub activity: ActivityState,
    pub spinner: SpinnerState,
    pub prompt_queue: PromptQueue,
}

impl OwlyApp {
    fn from_launch(launch: LaunchState) -> Self {
        let show_thinking = launch.app_context.verbose();
        let startup_entries = lines_to_entries(&launch.startup_lines);
        let setup = SetupWizardState::new(&launch.provider, &launch.model);

        let session_id = launch.session_id.clone();
        Self {
            context: launch.app_context,
            entries: startup_entries,
            live_tools: Vec::new(),
            prompt: PromptState::new(launch.model.clone()),
            chat: OwlyChatState::default(),
            theme: Theme::detect(),
            running: false,
            setup_complete: !launch.pending_setup,
            setup,
            setup_error: None,
            provider: launch.provider,
            model: launch.model,
            show_thinking,
            should_exit: false,
            submit_tx: launch.submit_tx,
            tip: pick_tip(&session_id),
            turn: 0,
            session_id,
            activity: ActivityState::default(),
            spinner: default_activity_spinner(),
            prompt_queue: PromptQueue::default(),
        }
    }

    fn dispatch_prompt(&mut self, text: String) {
        let _ = self.submit_tx.send(text);
    }

    fn drain_prompt_queue(&mut self) {
        if self.running {
            return;
        }
        if let Some(next) = self.prompt_queue.pop_front() {
            self.dispatch_prompt(next);
        }
    }

    fn handle_message(&mut self, message: AppMessage) {
        match message {
            AppMessage::UiEvent(event) => {
                let mut applier = TranscriptApplier::new(&mut self.entries, &mut self.live_tools, self.show_thinking);
                applier.apply(event);
            }
            AppMessage::DispatchDone { lines, should_exit } => {
                self.running = false;
                self.activity.clear();
                self.live_tools.clear();
                append_shell_lines(&mut self.entries, &lines);
                if should_exit {
                    self.should_exit = true;
                } else {
                    self.drain_prompt_queue();
                }
            }
            AppMessage::DispatchError(err) => {
                self.running = false;
                self.activity.clear();
                self.live_tools.clear();
                push_capped(
                    &mut self.entries,
                    OwlyEntry::assistant(format!("Error: {err}")),
                    DEFAULT_TRANSCRIPT_CAP,
                );
                self.drain_prompt_queue();
            }
        }
    }

    fn handle_global_keys(&mut self, ui: &mut Context) {
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

    fn handle_prompt(&mut self, ui: &mut Context) {
        match handle_prompt_input(ui, &mut self.prompt, self.running) {
            PromptAction::Submit(text) => {
                if is_quit_command(&text) {
                    self.dispatch_prompt("/exit".to_string());
                } else {
                    self.dispatch_prompt(text);
                }
            }
            PromptAction::Queue(text) => {
                if is_quit_command(&text) {
                    self.dispatch_prompt("/exit".to_string());
                } else {
                    self.prompt_queue.push_back(text);
                }
            }
            PromptAction::Steer(text) => {
                if is_quit_command(&text) {
                    self.dispatch_prompt("/exit".to_string());
                    return;
                }
                self.activity.request_cancel();
                self.prompt_queue.push_front(text);
                if self.running {
                    self.running = false;
                    self.activity.clear();
                    self.live_tools.clear();
                    self.drain_prompt_queue();
                }
            }
            PromptAction::Clear => self.prompt.clear(),
            PromptAction::CycleMode | PromptAction::None => {}
        }
    }

    fn complete_setup(&mut self, credentials: SetupCredentials) {
        self.setup.clear_error();
        let apply_context = self.context.clone();
        let persist_context = self.context.clone();

        let config = match try_block_on(async move {
            let snapshot = apply_context.config_snapshot().await;
            onboarding::apply_setup(credentials, &snapshot)
        }) {
            Ok(Ok(config)) => config,
            Ok(Err(err)) => {
                self.setup.set_error(format!("{err:#}"));
                return;
            }
            Err(_) => {
                self.setup.set_error("Failed to apply setup.".to_string());
                return;
            }
        };

        if let Err(err) = env::setup_environment(&config) {
            self.setup.set_error(format!("{err:#}"));
            return;
        }

        if try_block_on(persist_context.replace_config(config.clone())).is_err() {
            self.setup
                .set_error("Failed to update session configuration.".to_string());
            return;
        }

        self.provider = config.provider.clone();
        self.model = config.model_id.clone();
        self.prompt.model_name = config.model_id.clone();
        self.setup_complete = true;
    }
}

pub fn render_owly_app(ui: &mut Context, app: &mut OwlyApp) {
    if !app.setup_complete {
        if let Some(credentials) = app.setup.handle_keys(ui) {
            app.complete_setup(credentials);
        }
        if let Some(err) = &app.setup_error {
            app.setup.set_error(err.clone());
        }
        render_setup_wizard(ui, &mut app.setup, app.theme);
        return;
    }

    app.handle_global_keys(ui);
    app.theme.apply_to(ui);

    let directory = directory_display(app.context.cwd());
    let version = env!("CARGO_PKG_VERSION");
    let model_name = app.model.clone();
    let provider_name = app.provider.clone();
    let session_id = app.session_id.clone();
    let tip = app.tip;
    let model = if model_name.is_empty() {
        None
    } else {
        Some(model_name.as_str())
    };
    let provider = if provider_name.is_empty() {
        None
    } else {
        Some(provider_name.as_str())
    };

    let banner = BannerInfo {
        app_name: "Owly",
        version,
        update_available: false,
        directory: &directory,
        model,
        provider,
        extensions: 0,
        commands: 0,
        skills: 0,
        tools: 0,
        mcp_connected: 0,
        mcp_total: 0,
        mcp_tools: 0,
        tip,
    };
    let footer = FooterInfo {
        model_name: model,
        provider,
        thinking_level: "high",
        supports_images: false,
        cost_usd: 0.0,
        tokens_used: 0,
        context_pct: 0.0,
        context_limit: 262_000,
        token_display: Default::default(),
        project_dir: &directory,
        session_id: &session_id,
        mode: app.prompt.mode,
        turn: app.turn,
        branch: None,
        git_additions: 0,
        git_deletions: 0,
    };

    if app.running && !app.activity.visible {
        app.activity = ActivityState::working();
    }

    let theme = app.theme;
    let running = app.running;
    let show_thinking = app.show_thinking;

    let chrome = ShellChrome::simple(
        banner,
        footer,
        running,
        if running && app.activity.visible {
            Some(app.activity.clone())
        } else {
            None
        },
        app.spinner.clone(),
    );

    render_agent_shell(ui, theme, chrome, |ui, region| match region {
        ShellRegion::Chat => {
            render_owly_chat_stream(
                ui,
                &mut app.chat,
                &app.entries,
                &app.live_tools,
                theme,
                show_thinking,
                running,
            );
        }
        ShellRegion::Input => {
            app.handle_prompt(ui);
            render_prompt(
                ui,
                &mut app.prompt,
                theme,
                PromptOpts {
                    running,
                    queued_count: app.prompt_queue.len(),
                    ..Default::default()
                },
            );
        }
    });
}

pub async fn run_shell(mut launch: LaunchState) -> anyhow::Result<()> {
    let _ = enable_keyboard_enhancement();
    struct KeyboardGuard;
    impl Drop for KeyboardGuard {
        fn drop(&mut self) {
            let _ = disable_keyboard_enhancement();
        }
    }
    let _guard = KeyboardGuard;

    let initial = launch.initial.take();
    let mut submit_rx = launch.submit_rx.take().expect("submit receiver");
    let app = Arc::new(Mutex::new(OwlyApp::from_launch(launch)));
    let (msg_tx, mut msg_rx) = tokio::sync::mpsc::unbounded_channel::<AppMessage>();

    let app_dispatch = Arc::clone(&app);
    let dispatcher = tokio::spawn(async move {
        let mut pending_initial = initial;

        loop {
            let input = if let Some(text) = pending_initial.take() {
                text
            } else {
                match submit_rx.recv().await {
                    Some(text) => text,
                    None => break,
                }
            };

            let trimmed = input.trim();
            if !trimmed.is_empty()
                && let Ok(mut guard) = app_dispatch.lock()
            {
                guard.turn = guard.turn.saturating_add(1);
                guard.activity = ActivityState::working();
                push_capped(&mut guard.entries, OwlyEntry::user(trimmed), DEFAULT_TRANSCRIPT_CAP);
                guard.chat.pin_to_tail();
                guard.live_tools.clear();
                guard.running = true;
            }

            let context = app_dispatch.lock().expect("owly app lock").context.clone();
            let (event_tx, mut event_rx) = mpsc::unbounded_channel();
            let mut dispatch = Box::pin(context.dispatch(input, Some(event_tx)));

            let turn_result = loop {
                tokio::select! {
                    event = event_rx.recv() => {
                        let Some(event) = event else { continue };
                        let _ = msg_tx.send(AppMessage::UiEvent(event));
                    }
                    result = &mut dispatch => break result,
                }
            };

            match turn_result {
                Ok(result) => {
                    let _ = msg_tx.send(AppMessage::DispatchDone {
                        lines: result.lines,
                        should_exit: result.should_exit,
                    });
                    if result.should_exit {
                        break;
                    }
                }
                Err(err) => {
                    let _ = msg_tx.send(AppMessage::DispatchError(format!("{err:#}")));
                }
            }
        }
    });

    let app_ui = Arc::clone(&app);
    tokio::task::spawn_blocking(move || {
        let config = default_run_config();
        slt::run_with(config, move |ui: &mut Context| {
            let mut guard = app_ui.lock().expect("owly app lock");
            while let Ok(message) = msg_rx.try_recv() {
                guard.handle_message(message);
            }
            if guard.should_exit {
                ui.quit();
            }
            render_owly_app(ui, &mut guard);
        })
    })
    .await??;

    // UI exit (e.g. Ctrl+Q) leaves the dispatcher blocked on `submit_rx`; abort so shutdown completes.
    dispatcher.abort();
    let _ = dispatcher.await;
    Ok(())
}
