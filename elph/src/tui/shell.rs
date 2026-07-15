//! Root shell: layout zones, global keyboard handling, and session state.

use elph_tui::rgb;
use iocraft::prelude::*;
use std::time::Duration;

use crate::platform::Paths;
use crate::types::{AgentMode, ThinkingLevel, is_quit_command};

use super::header::Header;
use super::labels::session_label;
use super::prompt_chrome::PromptChrome;
use super::session_prefs::persist_session_prefs;
use super::status_row::StatusRow;
use super::transcript::{TranscriptMessage, TranscriptPanel, TranscriptStyle, seed_transcript_messages};

#[derive(Default, Props)]
pub struct MainShellProps {
    pub resume_id: Option<String>,
    pub initial_agent_mode: AgentMode,
    pub initial_thinking_level: ThinkingLevel,
    pub model_label: String,
    pub project_label: String,
    pub sticky_scroll: bool,
}

#[component]
pub fn MainShell(props: &mut MainShellProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (screen_width, screen_height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut should_exit = hooks.use_state(|| false);
    let mut agent_mode = hooks.use_state(|| props.initial_agent_mode);
    let mut thinking_level = hooks.use_state(|| props.initial_thinking_level);
    let mut draft = hooks.use_state(String::new);
    let mut live_draft = hooks.use_ref(String::new);
    let mut messages = hooks.use_state(seed_transcript_messages);
    let mut messages_revision = hooks.use_state(|| 0u64);
    let mut suppress_enter_newline = hooks.use_ref(|| false);
    let mut busy = hooks.use_state(|| false);
    let mut busy_generation = hooks.use_state(|| 0u64);
    let mut activity_label = hooks.use_state(|| "Working".to_string());
    let session_label = session_label(props.resume_id.as_deref());
    let paths = hooks.use_state(|| Paths::resolve().expect("resolve elph paths"));

    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(50)).await;
            if !busy.get() {
                continue;
            }
            let generation = busy_generation.get();
            tokio::time::sleep(Duration::from_secs(3)).await;
            if busy.get() && busy_generation.get() == generation {
                busy.set(false);
            }
        }
    });

    hooks.use_terminal_events({
        let paths = paths.read().clone();
        move |event| {
            let TerminalEvent::Key(KeyEvent {
                code, kind, modifiers, ..
            }) = event
            else {
                return;
            };
            if kind == KeyEventKind::Release {
                return;
            }

            match (modifiers, code) {
                (m, KeyCode::Char('a')) if m.contains(KeyModifiers::CONTROL) => {
                    let next = agent_mode.get().next();
                    agent_mode.set(next);
                    persist_session_prefs(&paths, next, thinking_level.get());
                }
                (m, KeyCode::BackTab) if m.contains(KeyModifiers::SHIFT) => {
                    let next = thinking_level.get().next();
                    thinking_level.set(next);
                    persist_session_prefs(&paths, agent_mode.get(), next);
                }
                (m, KeyCode::Char('d')) if m.contains(KeyModifiers::CONTROL) => should_exit.set(true),
                _ => {}
            }
        }
    });

    if should_exit.get() {
        system.exit();
    }

    let (accent_r, accent_g, accent_b) = agent_mode.get().label_rgb();
    let scanner_accent = rgb(accent_r, accent_g, accent_b);

    element! {
        View(
            width: screen_width,
            height: screen_height,
            background_color: Color::Reset,
            border_style: BorderStyle::None,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            margin: 0,
            padding: 0,
        ) {
            Header(
                screen_width: screen_width,
                session_label: session_label,
            )
            TranscriptPanel(
                screen_width: screen_width,
                messages: Some(messages),
                messages_revision: messages_revision.get(),
                sticky_scroll: props.sticky_scroll,
            )
            StatusRow(
                screen_width: screen_width,
                busy: busy.get(),
                activity_label: activity_label.read().clone(),
                accent: scanner_accent,
            )
            PromptChrome(
                screen_width: screen_width,
                screen_height: screen_height,
                agent_mode: agent_mode.get(),
                thinking_level: thinking_level.get(),
                project_label: props.project_label.clone(),
                model_label: props.model_label.clone(),
                draft: Some(draft),
                live_draft: Some(live_draft),
                suppress_enter_newline: Some(suppress_enter_newline),
                on_submit: move |text: String| {
                    if text.trim().is_empty() {
                        return;
                    }
                    if is_quit_command(&text) {
                        should_exit.set(true);
                        draft.set(String::new());
                        live_draft.set(String::new());
                        suppress_enter_newline.set(true);
                        return;
                    }
                    messages.set({
                        let mut list = messages.read().clone();
                        list.push(TranscriptMessage {
                            content: text,
                            style: TranscriptStyle::User,
                        });
                        list
                    });
                    messages_revision.set(messages_revision.get().wrapping_add(1));
                    busy.set(true);
                    busy_generation.set(busy_generation.get().saturating_add(1));
                    activity_label.set("Working".to_string());
                    draft.set(String::new());
                    live_draft.set(String::new());
                    suppress_enter_newline.set(true);
                },
            )
        }
    }
}
