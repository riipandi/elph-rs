use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use elph_tui::{AgentShell, configure_runtime, start_shell};

use crate::platform::exit_message;
use crate::shell::ElphApp;
use crate::shell::shell_host::ElphShellHost;
use crate::tui::TurnDispatcher;

pub async fn run_sigint_watcher(app: Arc<Mutex<ElphApp>>) {
    let mut sigint = elph_tui::sigint_channel();
    while sigint.recv().await {
        if let Ok(mut guard) = app.lock() {
            if guard.agent_running {
                guard.activity.request_cancel();
                TurnDispatcher::spawn_abort(Arc::clone(&guard.session));
            } else if crate::platform::handle_prompt_interrupt_prompt(&mut guard.prompt) {
                guard.should_exit = true;
            }
        }
    }
}

pub fn run_tui(resume_id: Option<String>) -> std::io::Result<()> {
    let settings = crate::platform::Paths::resolve()
        .and_then(|paths| crate::platform::Settings::load(&paths))
        .map_err(std::io::Error::other)?;

    let app =
        elph_agent::block_on(ElphApp::bootstrap(settings, resume_id.as_deref())).map_err(std::io::Error::other)?;
    let app = Arc::new(Mutex::new(app));
    let watcher_app = Arc::clone(&app);

    std::thread::spawn(move || {
        elph_agent::block_on(run_sigint_watcher(watcher_app));
    });

    configure_runtime();

    let host: Rc<RefCell<dyn elph_tui::ShellHost>> = Rc::new(RefCell::new(ElphShellHost::new(app.clone())));
    let root = AgentShell::new(host);
    let _exit_code = start_shell(root)?;

    if let Ok(guard) = app.lock() {
        let snapshot = ElphApp::exit_snapshot_from(
            &guard.session_id,
            guard.total_api_secs,
            guard.started_at,
            &guard.project_dir,
            &guard.session,
        );
        exit_message::record(snapshot);
    }

    Ok(())
}
