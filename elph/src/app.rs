use std::sync::{Arc, Mutex};

use elph_tui::{
    ChatStreamState, PromptAction, PromptOpts, PromptState, Theme, disable_keyboard_enhancement,
    enable_keyboard_enhancement, handle_prompt_input, is_quit_command, push_capped, render_chat_stream, render_prompt,
    sigint_channel,
};
use slt::{Context, KeyModifiers, RunConfig};

use crate::runtime::exit_message::ExitSnapshot;
use crate::runtime::{WAS_INTERRUPTED, exit_message, handle_prompt_interrupt};

pub struct ElphApp {
    pub prompt: PromptState,
    pub chat: ChatStreamState,
    pub theme: Theme,
    pub should_exit: bool,
    pub session_id: String,
}

impl ElphApp {
    pub fn new() -> Self {
        Self {
            prompt: PromptState::new("Claude Fable 5"),
            chat: ChatStreamState::new(),
            theme: Theme::detect(),
            should_exit: false,
            session_id: exit_message::new_session_id(),
        }
    }

    pub fn handle_global_keys(&mut self, ui: &mut Context) {
        if ui.key_mod('c', KeyModifiers::CONTROL) && handle_prompt_interrupt(&mut self.prompt.textarea) {
            self.should_exit = true;
            return;
        }
        if ui.key_mod('q', KeyModifiers::CONTROL) {
            self.should_exit = true;
            use std::sync::atomic::Ordering;
            WAS_INTERRUPTED.store(true, Ordering::Relaxed);
            #[cfg(unix)]
            crate::runtime::SHOULD_KILL_PARENT.store(true, Ordering::Relaxed);
        }
        if ui.key_mod('t', KeyModifiers::CONTROL) {
            self.theme = self.theme.toggle();
        }
    }

    pub fn handle_prompt(&mut self, ui: &mut Context) {
        match handle_prompt_input(ui, &mut self.prompt) {
            PromptAction::Submit(text) => {
                if is_quit_command(&text) {
                    self.prompt.clear();
                    self.should_exit = true;
                    return;
                }
                push_capped(&mut self.chat.messages, text, elph_tui::DEFAULT_TRANSCRIPT_CAP);
            }
            PromptAction::Clear => self.prompt.clear(),
            PromptAction::CycleMode | PromptAction::None => {}
        }
    }
}

pub fn render_app(ui: &mut Context, app: &mut ElphApp) {
    app.handle_global_keys(ui);
    app.theme.apply_to(ui);
    let _ = ui.container().grow(1).col(|ui| {
        let _ = ui.container().grow(1).col(|ui| {
            render_chat_stream(ui, &mut app.chat, app.theme);
        });
        app.handle_prompt(ui);
        render_prompt(ui, &mut app.prompt, app.theme, PromptOpts::default());
    });
}

pub async fn run_sigint_watcher(app: Arc<Mutex<ElphApp>>) {
    let mut sigint = sigint_channel();
    while sigint.recv().await {
        if let Ok(mut guard) = app.lock()
            && handle_prompt_interrupt(&mut guard.prompt.textarea)
        {
            guard.should_exit = true;
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

    let config = RunConfig::default();
    slt::run_with(config, move |ui: &mut Context| {
        let mut guard = app.lock().expect("elph app lock");
        if guard.should_exit {
            exit_message::record(ExitSnapshot {
                session_id: guard.session_id.clone(),
                has_history: !guard.chat.messages.is_empty(),
            });
            ui.quit();
        }
        render_app(ui, &mut guard);
    })
}
