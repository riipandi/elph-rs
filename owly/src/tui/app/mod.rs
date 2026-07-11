//! Owly interactive shell application (tuie).

mod events;
mod input;
mod run;
mod setup;

use elph_tui::AgentMode;
use elph_tui::{
    ActivityState, BannerInfo, PromptQueue, PromptState, Theme, ToolExecutionState, owly_builtin_commands, pick_tip,
    simple_banner_lines,
};
use tokio::sync::mpsc;

use crate::tui::banner::directory_display;
use crate::tui::setup::SetupWizardState;

use super::ask::PendingAsk;
use super::chat_stream::OwlyChatState;
use super::context::AppContext;
use super::entries::OwlyEntry;
use super::launch::LaunchState;
use crate::ui_events::AgentUiEvent;

pub use events::AppMessage;
pub use run::run_shell;

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
    pub provider: String,
    pub model: String,
    pub show_thinking: bool,
    pub should_exit: bool,
    pub submit_tx: mpsc::UnboundedSender<String>,
    pub turn: u32,
    pub session_label: String,
    pub activity: ActivityState,
    pub prompt_queue: PromptQueue,
    pub slash_commands: Vec<elph_tui::SlashCommand>,
    pub(super) pending_ask: Option<PendingAsk>,
}

impl OwlyApp {
    pub(super) fn from_launch(launch: LaunchState) -> Self {
        let show_thinking = launch.app_context.verbose();
        let mut entries = super::transcript::lines_to_entries(&launch.startup_lines);
        let directory = directory_display(launch.app_context.cwd());
        let banner = BannerInfo {
            app_name: "Owly",
            version: env!("CARGO_PKG_VERSION"),
            update_available: false,
            directory: &directory,
            model: if launch.model.is_empty() {
                None
            } else {
                Some(launch.model.as_str())
            },
            provider: if launch.provider.is_empty() {
                None
            } else {
                Some(launch.provider.as_str())
            },
            extensions: 0,
            commands: 0,
            skills: 0,
            tools: 0,
            mcp_connected: 0,
            mcp_total: 0,
            mcp_tools: 0,
            tip: pick_tip(&launch.session_id),
        };
        for line in simple_banner_lines(banner) {
            entries.insert(0, OwlyEntry::hint(line));
        }
        let setup = SetupWizardState::new(&launch.provider, &launch.model);

        let session_label = launch.session_label.clone();
        Self {
            context: launch.app_context,
            entries,
            live_tools: Vec::new(),
            prompt: {
                let mut prompt = PromptState::new(launch.model.clone());
                prompt.mode = AgentMode::Ask;
                prompt.enable_mode_cycle = false;
                prompt
            },
            chat: OwlyChatState,
            theme: Theme::detect(),
            running: false,
            setup_complete: !launch.pending_setup,
            setup,
            provider: launch.provider,
            model: launch.model,
            show_thinking,
            should_exit: false,
            submit_tx: launch.submit_tx,
            turn: 0,
            session_label,
            activity: ActivityState::default(),
            prompt_queue: PromptQueue::default(),
            slash_commands: owly_builtin_commands(),
            pending_ask: None,
        }
    }

    pub(super) fn open_ask_prompt(
        &mut self,
        tool_call_id: String,
        tool_name: String,
        question: String,
        kind: crate::ui_events::AskUserKind,
        response_tx: tokio::sync::oneshot::Sender<crate::ui_events::AskUserResponse>,
    ) {
        let default_index = match &kind {
            crate::ui_events::AskUserKind::Select { default_index, .. } => *default_index,
            crate::ui_events::AskUserKind::Confirm { default } => {
                if *default {
                    0
                } else {
                    1
                }
            }
            crate::ui_events::AskUserKind::Text { .. } => 0,
        };
        if let crate::ui_events::AskUserKind::Select { options, .. } = &kind
            && options.is_empty()
        {
            let _ = response_tx.send(crate::ui_events::AskUserResponse::Cancelled);
            return;
        }

        let pending = PendingAsk {
            _tool_call_id: tool_call_id,
            tool_name,
            question,
            kind,
            response_tx,
            _selected: default_index,
        };
        pending.push_transcript_notice(&mut self.entries);
        if let crate::ui_events::AskUserKind::Text { default: Some(default) } = &pending.kind
            && !default.is_empty()
        {
            self.prompt.set_value(default);
        }
        self.activity = ActivityState::awaiting_input();
        self.pending_ask = Some(pending);
    }

    pub(super) fn cancel_pending_ask(&mut self) {
        if let Some(pending) = self.pending_ask.take() {
            pending.finish_cancelled();
        }
    }

    fn apply_ui_event(&mut self, event: AgentUiEvent) {
        if let AgentUiEvent::SessionTitleUpdated { title } = &event {
            self.session_label = title.clone();
            return;
        }
        if let AgentUiEvent::AskUserRequired {
            tool_call_id,
            tool_name,
            question,
            kind,
            response_tx,
        } = event
        {
            self.open_ask_prompt(tool_call_id, tool_name, question, kind, response_tx);
            return;
        }

        if let AgentUiEvent::ToolStart { name, args_summary, .. } = &event {
            self.activity = ActivityState::running_tool(name, args_summary);
        }
        if let AgentUiEvent::TextDelta(_) = &event
            && self.running
            && self.live_tools.is_empty()
        {
            self.activity = ActivityState::responding();
        }

        if matches!(event, AgentUiEvent::RunCompleted { .. }) && self.running {
            self.activity.clear();
        }

        let tool_finished = matches!(event, AgentUiEvent::ToolEnd { .. });

        let mut applier =
            super::transcript::TranscriptApplier::new(&mut self.entries, &mut self.live_tools, self.show_thinking);
        applier.apply(event);

        if tool_finished && self.running && self.live_tools.is_empty() {
            self.activity = ActivityState::working();
        }
    }

    pub(super) fn handle_message(&mut self, message: events::AppMessage) {
        match message {
            events::AppMessage::UiEvent(event) => self.apply_ui_event(event),
            events::AppMessage::DispatchDone { lines, should_exit } => {
                self.running = false;
                self.activity.clear();
                self.live_tools.clear();
                self.cancel_pending_ask();
                super::transcript::append_shell_lines(&mut self.entries, &lines);
                if should_exit {
                    self.should_exit = true;
                } else {
                    self.drain_prompt_queue();
                }
            }
            events::AppMessage::DispatchError(err) => {
                self.running = false;
                self.activity.clear();
                self.live_tools.clear();
                self.cancel_pending_ask();
                elph_tui::push_capped(
                    &mut self.entries,
                    OwlyEntry::assistant(format!("Error: {err}")),
                    elph_tui::DEFAULT_TRANSCRIPT_CAP,
                );
                self.drain_prompt_queue();
            }
        }
    }
}
