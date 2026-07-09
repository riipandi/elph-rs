//! Owly interactive shell application (SuperLightTUI).

use std::sync::{Arc, Mutex};

use elph_agent::try_block_on;
use elph_tui::{
    DEFAULT_TRANSCRIPT_CAP, PromptAction, PromptState, Theme, ToolExecutionState, disable_keyboard_enhancement,
    enable_keyboard_enhancement, handle_prompt_input, is_quit_command, push_capped, render_prompt,
};
use slt::{Context, KeyModifiers, RunConfig};
use tokio::sync::mpsc;

use crate::env;
use crate::onboarding::{self, SetupCredentials};
use crate::ui_events::AgentUiEvent;

use super::activity::{ActivityBarState, render_activity_bar};
use super::banner::{directory_display, render_banner};
use super::chat_stream::{OwlyChatState, render_owly_chat_stream};
use super::context::AppContext;
use super::entries::OwlyEntry;
use super::launch::LaunchState;
use super::setup::{SetupWizardState, render_setup_wizard};
use super::transcript::{TranscriptApplier, append_shell_lines, command_label_for_input, lines_to_entries};

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
    pub activity: ActivityBarState,
    pub theme: Theme,
    pub running: bool,
    pub active_command: Option<String>,
    pub setup_complete: bool,
    pub setup: SetupWizardState,
    pub setup_error: Option<String>,
    pub provider: String,
    pub model: String,
    pub show_thinking: bool,
    pub should_exit: bool,
    pub submit_tx: mpsc::UnboundedSender<String>,
}

impl OwlyApp {
    fn from_launch(launch: LaunchState) -> Self {
        let show_thinking = launch.app_context.verbose();
        let startup_entries = lines_to_entries(&launch.startup_lines);
        let setup = SetupWizardState::new(&launch.provider, &launch.model);

        Self {
            context: launch.app_context,
            entries: startup_entries,
            live_tools: Vec::new(),
            prompt: PromptState::new(launch.model.clone()),
            chat: OwlyChatState::default(),
            activity: ActivityBarState::default(),
            theme: Theme::detect(),
            running: false,
            active_command: None,
            setup_complete: !launch.pending_setup,
            setup,
            setup_error: None,
            provider: launch.provider,
            model: launch.model,
            show_thinking,
            should_exit: false,
            submit_tx: launch.submit_tx,
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
                self.active_command = None;
                self.live_tools.clear();
                append_shell_lines(&mut self.entries, &lines);
                if should_exit {
                    self.should_exit = true;
                }
            }
            AppMessage::DispatchError(err) => {
                self.running = false;
                self.active_command = None;
                self.live_tools.clear();
                push_capped(
                    &mut self.entries,
                    OwlyEntry::assistant(format!("Error: {err}")),
                    DEFAULT_TRANSCRIPT_CAP,
                );
            }
        }
    }

    fn handle_global_keys(&mut self, ui: &mut Context) {
        if ui.key_mod('c', KeyModifiers::CONTROL) {
            self.prompt.clear();
            return;
        }
        if ui.key_mod('q', KeyModifiers::CONTROL) {
            self.should_exit = true;
            return;
        }
        if ui.key_mod('t', KeyModifiers::CONTROL) {
            self.theme = self.theme.toggle();
        }
    }

    fn handle_prompt(&mut self, ui: &mut Context) {
        if self.running {
            return;
        }
        match handle_prompt_input(ui, &mut self.prompt) {
            PromptAction::Submit(text) => {
                if is_quit_command(&text) {
                    let _ = self.submit_tx.send("/exit".to_string());
                } else {
                    let _ = self.submit_tx.send(text);
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

    let _ = ui.col(|ui| {
        render_banner(ui, &app.provider, &app.model, &directory, version, app.theme);
        render_owly_chat_stream(ui, &mut app.chat, &app.entries, app.theme, app.show_thinking);
        if app.running {
            render_activity_bar(
                ui,
                &mut app.activity,
                app.active_command.as_deref(),
                &app.live_tools,
                app.theme,
            );
        }
        app.handle_prompt(ui);
        render_prompt(ui, &mut app.prompt, app.theme);
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
                push_capped(&mut guard.entries, OwlyEntry::user(trimmed), DEFAULT_TRANSCRIPT_CAP);
                guard.active_command = command_label_for_input(trimmed).map(str::to_string);
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
        let config = RunConfig::default();
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
