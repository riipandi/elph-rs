use std::sync::Arc;

use elph_tui::{ModelSelectorState, SessionSelectorState, TranscriptEntry, TreeNavigatorState, push_capped};

use crate::agent::{
    CreateSessionOptions, create_coding_session_with_events, list_model_select_items, list_session_select_items,
    list_tree_select_items,
};
use crate::shell::{ActiveOverlay, ElphApp};
use crate::tui::transcript_from_branch;

#[allow(dead_code)] // tuie overlay port pending
impl ElphApp {
    pub(super) fn close_overlay(&mut self) {
        self.active_overlay = ActiveOverlay::None;
        self.overlay_items.clear();
        self.model_selector = ModelSelectorState::default();
        self.session_selector = SessionSelectorState::default();
        self.tree_navigator = TreeNavigatorState::default();
    }

    pub(super) fn rebuild_transcript_from_session(&mut self) {
        let session = Arc::clone(&self.session);
        let show_thinking = self.show_thinking;
        match elph_agent::block_on(async move { session.branch_entries().await }) {
            Ok(entries) => {
                self.chat.entries = transcript_from_branch(&entries, show_thinking);
                self.live_tools.clear();
                self.chat.pin_to_tail();
            }
            Err(err) => {
                push_capped(
                    &mut self.chat.entries,
                    TranscriptEntry::system(format!("Failed to load transcript: {err}")),
                    elph_tui::DEFAULT_TRANSCRIPT_CAP,
                );
            }
        }
    }

    pub(super) fn swap_session(&mut self, resume_id: Option<&str>) {
        if self.agent_running {
            push_capped(
                &mut self.chat.entries,
                TranscriptEntry::system("Cannot switch session while agent is running"),
                elph_tui::DEFAULT_TRANSCRIPT_CAP,
            );
            return;
        }

        let paths = self.paths.clone();
        let settings = self.settings.clone();
        let cwd = self.cwd.clone();
        let resume_id_owned = resume_id.map(str::to_string);

        match elph_agent::block_on(async move {
            create_coding_session_with_events(CreateSessionOptions {
                paths: &paths,
                settings: &settings,
                cwd: &cwd,
                resume_id: resume_id_owned.as_deref(),
                provider_override: None,
                model_override: None,
            })
            .await
        }) {
            Ok((session, ui_rx)) => {
                self.session = Arc::new(session);
                self.ui_rx = ui_rx;
                self.session_id = self.session.session_id().to_string();
                self.prompt.model_name = self.session.model_display();
                self.turn = 0;
                self.prompt_queue.clear();
                self.rebuild_transcript_from_session();
                let label = if resume_id.is_some() {
                    "Resumed session"
                } else {
                    "Started new session"
                };
                push_capped(
                    &mut self.chat.entries,
                    TranscriptEntry::system(label),
                    elph_tui::DEFAULT_TRANSCRIPT_CAP,
                );
            }
            Err(err) => {
                push_capped(
                    &mut self.chat.entries,
                    TranscriptEntry::system(format!("Session switch failed: {err}")),
                    elph_tui::DEFAULT_TRANSCRIPT_CAP,
                );
            }
        }
    }

    #[expect(dead_code)] // Invoked from /model slash handler when implemented.
    pub(super) fn open_model_selector(&mut self) {
        self.overlay_items = list_model_select_items();
        if self.overlay_items.is_empty() {
            push_capped(
                &mut self.chat.entries,
                TranscriptEntry::system("No models available"),
                elph_tui::DEFAULT_TRANSCRIPT_CAP,
            );
            return;
        }
        self.model_selector = ModelSelectorState::default();
        self.active_overlay = ActiveOverlay::Model;
    }

    #[expect(dead_code)] // Invoked from /resume slash handler when implemented.
    pub(super) fn open_session_selector(&mut self) {
        let session = Arc::clone(&self.session);
        match elph_agent::block_on(async move { list_session_select_items(session.session_manager()).await }) {
            Ok(items) if items.is_empty() => {
                push_capped(
                    &mut self.chat.entries,
                    TranscriptEntry::system("No sessions to resume"),
                    elph_tui::DEFAULT_TRANSCRIPT_CAP,
                );
            }
            Ok(items) => {
                self.overlay_items = items;
                self.session_selector = SessionSelectorState::default();
                self.active_overlay = ActiveOverlay::Session;
            }
            Err(err) => {
                push_capped(
                    &mut self.chat.entries,
                    TranscriptEntry::system(format!("Failed to list sessions: {err}")),
                    elph_tui::DEFAULT_TRANSCRIPT_CAP,
                );
            }
        }
    }

    #[expect(dead_code)] // Invoked from /tree and /fork slash handlers when implemented.
    pub(super) fn open_tree_navigator(&mut self) {
        if self.agent_running {
            push_capped(
                &mut self.chat.entries,
                TranscriptEntry::system("Cannot navigate tree while agent is running"),
                elph_tui::DEFAULT_TRANSCRIPT_CAP,
            );
            return;
        }
        let session = Arc::clone(&self.session);
        let entries = elph_agent::block_on(async move { session.harness().session_entries().await });
        self.overlay_items = list_tree_select_items(&entries);
        if self.overlay_items.is_empty() {
            push_capped(
                &mut self.chat.entries,
                TranscriptEntry::system("No navigable entries in session tree"),
                elph_tui::DEFAULT_TRANSCRIPT_CAP,
            );
            return;
        }
        self.tree_navigator = TreeNavigatorState::default();
        self.active_overlay = ActiveOverlay::Tree;
    }

    /// Overlay keyboard handling is deferred until tuie popup selectors land.
    pub(super) fn handle_overlay_input(&mut self) -> bool {
        self.overlay_visible()
    }
}
