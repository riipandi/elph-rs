//! Root shell: layout zones, global keyboard handling, and session state.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use elph_tui::rgb;
use iocraft::prelude::*;
use tokio::sync::mpsc::UnboundedReceiver;

use crate::agent::{
    AgentUiEvent, CodingAgentSession, SlashDispatch, ToolApprovalChoice, dispatch_slash_command,
    slash_unimplemented_message,
};
use crate::platform::Paths;
use crate::types::{AgentMode, ThinkingLevel, is_quit_command};

use super::activity::activity_label_for_event;
use super::agent_bridge::{PromptQueue, SlashDispatcher, TranscriptEventApplier, TurnDispatcher};
use super::chrome::{ChromeStats, header_stats_from_chrome, read_git_branch, refresh_chrome_stats};
use super::footer::Footer;
use super::header::Header;
use super::labels::{project_footer_label, session_label};
use super::prompt_chrome::PromptChrome;
use super::session_prefs::persist_session_prefs;
use super::status_row::{StatusRow, format_elapsed_secs};
use super::tool_approval::{PendingToolApproval, ToolApprovalPrompt, choice_from_key};
use super::transcript::{TranscriptMessage, TranscriptPanel, TranscriptStyle};

const SHELL_TICK_MS: u64 = 50;
const CHROME_REFRESH_TICKS: u32 = 20;

#[derive(Props)]
pub struct MainShellProps {
    pub session_id: String,
    pub bootstrap_notice: Option<String>,
    pub initial_agent_mode: AgentMode,
    pub initial_thinking_level: ThinkingLevel,
    pub model_label: String,
    pub project_label: String,
    pub context_limit: u64,
    pub supports_images: bool,
    pub footer_token_display: String,
    pub sticky_scroll: bool,
    pub show_thinking: bool,
    pub agent_session: Option<Arc<CodingAgentSession>>,
    pub ui_events: Option<Arc<Mutex<UnboundedReceiver<AgentUiEvent>>>>,
}

impl Default for MainShellProps {
    fn default() -> Self {
        Self {
            session_id: "unavailable".to_string(),
            bootstrap_notice: None,
            initial_agent_mode: AgentMode::default(),
            initial_thinking_level: ThinkingLevel::default(),
            model_label: String::new(),
            project_label: String::new(),
            context_limit: 200_000,
            supports_images: false,
            footer_token_display: "both".to_string(),
            sticky_scroll: false,
            show_thinking: false,
            agent_session: None,
            ui_events: None,
        }
    }
}

struct BusyActivation<'a> {
    busy: &'a mut State<bool>,
    busy_started_at: &'a mut Ref<Option<Instant>>,
    elapsed_secs: &'a mut State<f64>,
    spinner_tick: &'a mut State<u32>,
    activity_label: &'a mut State<String>,
}

fn mark_busy(ctx: &mut BusyActivation<'_>, steer: bool) {
    ctx.busy.set(true);
    ctx.busy_started_at.set(Some(Instant::now()));
    ctx.elapsed_secs.set(0.0);
    ctx.spinner_tick.set(0);
    ctx.activity_label.set(if steer {
        "Steering".to_string()
    } else {
        "Thinking".to_string()
    });
}

fn push_transcript_message(
    messages: &mut State<Vec<TranscriptMessage>>,
    messages_revision: &mut State<u64>,
    message: TranscriptMessage,
) {
    messages.set({
        let mut list = messages.read().clone();
        list.push(message);
        list
    });
    messages_revision.set(messages_revision.get().wrapping_add(1));
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
    let bootstrap_notice = props.bootstrap_notice.clone();
    let mut messages = hooks.use_state(move || {
        bootstrap_notice
            .map(|notice| vec![TranscriptMessage::text(notice, TranscriptStyle::Meta)])
            .unwrap_or_default()
    });
    let mut messages_revision = hooks.use_state(|| 0u64);
    let mut suppress_enter_newline = hooks.use_ref(|| false);
    let mut busy = hooks.use_state(|| false);
    let mut activity_label = hooks.use_state(|| "Thinking".to_string());
    let mut elapsed_secs = hooks.use_state(|| 0.0f64);
    let mut spinner_tick = hooks.use_state(|| 0u32);
    let show_thinking = props.show_thinking;
    let mut busy_started_at = hooks.use_ref(|| None::<Instant>);
    let mut prompt_queue = hooks.use_ref(PromptQueue::default);
    let mut event_applier = hooks.use_ref(|| TranscriptEventApplier::new(props.show_thinking));
    let mut pending_tool_approval = hooks.use_ref(|| None::<PendingToolApproval>);

    let agent_session = props.agent_session.clone();
    let agent_session_for_loop = agent_session.clone();
    let agent_session_for_chrome = agent_session.clone();
    let ui_events = props.ui_events.clone();
    let paths = hooks.use_state(|| Paths::resolve().expect("resolve elph paths"));
    let mut turn_count = hooks.use_state(|| 0u32);
    let mut chrome_stats = hooks.use_state(|| ChromeStats {
        context_limit: props.context_limit,
        model_label: props.model_label.clone(),
        supports_images: props.supports_images,
        ..ChromeStats::default()
    });
    let mut project_label = hooks.use_state(|| props.project_label.clone());
    let mut chrome_tick = hooks.use_ref(|| 0u32);
    let fallback_context_limit = props.context_limit;
    let fallback_model_label = props.model_label.clone();
    let fallback_model_label_for_chrome = fallback_model_label.clone();
    let fallback_supports_images = props.supports_images;
    let footer_token_display = props.footer_token_display.clone();
    let session_id = props.session_id.clone();

    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(SHELL_TICK_MS)).await;

            chrome_tick.set(chrome_tick.get().wrapping_add(1));
            if chrome_tick.get() % CHROME_REFRESH_TICKS == 0 {
                let paths = paths.read().clone();
                let branch = read_git_branch(paths.project_dir());
                project_label.set(project_footer_label(&paths, branch.as_deref()));

                if let Some(session) = agent_session_for_chrome.as_ref() {
                    let stats = refresh_chrome_stats(
                        Arc::clone(session),
                        fallback_context_limit,
                        &fallback_model_label_for_chrome,
                        fallback_supports_images,
                    )
                    .await;
                    chrome_stats.set(stats);
                }
            }

            if busy.get() {
                if let Some(started) = busy_started_at.read().as_ref() {
                    let next = format_elapsed_secs(*started);
                    if (elapsed_secs.get() - next).abs() > f64::EPSILON {
                        elapsed_secs.set(next);
                    }
                }
                spinner_tick.set(spinner_tick.get().wrapping_add(1));
            }

            let mut transcript_changed = false;
            let mut run_completed = false;

            if let Some(rx) = ui_events.as_ref() {
                if let Ok(mut guard) = rx.lock() {
                    while let Ok(event) = guard.try_recv() {
                        if let AgentUiEvent::RunCompleted { .. } = &event {
                            run_completed = true;
                        }

                        if let AgentUiEvent::ToolApprovalRequired(req) = event {
                            let tool_name = req.tool_name.clone();
                            activity_label.set(format!("Approve: {tool_name}"));
                            pending_tool_approval.set(Some(PendingToolApproval::from_request(req)));
                            {
                                let mut msgs = messages.write();
                                msgs.push(TranscriptMessage::text(
                                    format!("Tool approval required: {tool_name} (y/n/a)"),
                                    TranscriptStyle::Meta,
                                ));
                            }
                            transcript_changed = true;
                            continue;
                        }

                        if let Some(label) = activity_label_for_event(&event, show_thinking) {
                            activity_label.set(label);
                        }
                        {
                            let mut msgs = messages.write();
                            if event_applier.write().apply(&mut msgs, event) {
                                transcript_changed = true;
                            }
                        }
                    }
                }
            }

            if transcript_changed {
                messages_revision.set(messages_revision.get().wrapping_add(1));
            }

            if run_completed {
                busy.set(false);
                busy_started_at.set(None);
                elapsed_secs.set(0.0);
                activity_label.set("Thinking".to_string());

                if let Some(next) = prompt_queue.write().pop_front() {
                    mark_busy(
                        &mut BusyActivation {
                            busy: &mut busy,
                            busy_started_at: &mut busy_started_at,
                            elapsed_secs: &mut elapsed_secs,
                            spinner_tick: &mut spinner_tick,
                            activity_label: &mut activity_label,
                        },
                        false,
                    );
                    if let Some(session) = agent_session_for_loop.as_ref() {
                        TurnDispatcher::spawn_turn(Arc::clone(session), next, false);
                    }
                }
            }
        }
    });

    hooks.use_terminal_events({
        let paths = paths.read().clone();
        let agent_session = agent_session.clone();
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

            let mut pending_tool_approval = pending_tool_approval;
            if pending_tool_approval.read().is_some()
                && let Some(choice) = choice_from_key(modifiers, code)
            {
                if let Some(pending) = pending_tool_approval.write().take() {
                    pending.respond(choice);
                }
                activity_label.set(match choice {
                    ToolApprovalChoice::Approve => "Running approved tool…".to_string(),
                    ToolApprovalChoice::AllowSession => "Running tool (session allow)…".to_string(),
                    ToolApprovalChoice::Reject => "Tool denied".to_string(),
                });
                return;
            }

            match (modifiers, code) {
                (m, KeyCode::Char('a')) if m.contains(KeyModifiers::CONTROL) => {
                    let next = agent_mode.get().next();
                    agent_mode.set(next);
                    persist_session_prefs(&paths, next, thinking_level.get());
                    if let Some(session) = agent_session.as_ref() {
                        let session = Arc::clone(session);
                        let mode = next;
                        tokio::spawn(async move {
                            if let Err(err) = session.set_agent_mode(mode).await {
                                log::warn!("failed to set agent mode: {err}");
                            }
                        });
                    }
                }
                (m, KeyCode::BackTab) if m.contains(KeyModifiers::SHIFT) => {
                    let next = thinking_level.get().next();
                    thinking_level.set(next);
                    persist_session_prefs(&paths, agent_mode.get(), next);
                    if let Some(session) = agent_session.as_ref() {
                        let session = Arc::clone(session);
                        let level = next;
                        tokio::spawn(async move {
                            if let Err(err) = session.set_thinking_level(level).await {
                                log::warn!("failed to set thinking level: {err}");
                            }
                        });
                    }
                }
                (m, KeyCode::Char('d')) if m.contains(KeyModifiers::CONTROL) => should_exit.set(true),
                (m, KeyCode::Char('c')) if m.contains(KeyModifiers::CONTROL) && busy.get() => {
                    activity_label.set("Cancelling…".to_string());
                    if let Some(session) = agent_session.as_ref() {
                        TurnDispatcher::spawn_abort(Arc::clone(session));
                    } else {
                        busy.set(false);
                        busy_started_at.set(None);
                        elapsed_secs.set(0.0);
                    }
                }
                _ => {}
            }
        }
    });

    if should_exit.get() {
        system.exit();
    }

    let (accent_r, accent_g, accent_b) = agent_mode.get().label_rgb();
    let scanner_accent = rgb(accent_r, accent_g, accent_b);
    let chrome = chrome_stats.read();
    let session_label = session_label(&session_id, turn_count.get());
    let stats_label = header_stats_from_chrome(&chrome, &footer_token_display);
    let model_label = if chrome.model_label.is_empty() {
        fallback_model_label.clone()
    } else {
        chrome.model_label.clone()
    };
    let supports_images = chrome.supports_images;

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
                stats_label: stats_label,
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
                spinner_tick: spinner_tick.get(),
                elapsed_secs: elapsed_secs.get(),
            )
            #(if let Some(pending) = pending_tool_approval.read().as_ref() {
                element! {
                    View(
                        width: screen_width,
                        flex_shrink: 0f32,
                        flex_direction: FlexDirection::Column,
                    ) {
                        ToolApprovalPrompt(
                            screen_width: screen_width,
                            tool_name: pending.tool_name.clone(),
                            args_summary: pending.args_summary.clone(),
                        )
                        Footer(
                            screen_width: screen_width,
                            project_label: project_label.read().clone(),
                            model_label: model_label.clone(),
                            thinking_level: thinking_level.get(),
                            supports_images: supports_images,
                        )
                    }
                }
            } else {
                element! {
                    View(
                        width: screen_width,
                        flex_shrink: 0f32,
                        flex_direction: FlexDirection::Column,
                    ) {
                        PromptChrome(
                        screen_width: screen_width,
                        screen_height: screen_height,
                        agent_mode: agent_mode.get(),
                        thinking_level: thinking_level.get(),
                        project_label: project_label.read().clone(),
                        model_label: model_label.clone(),
                        supports_images: supports_images,
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

                    push_transcript_message(
                        &mut messages,
                        &mut messages_revision,
                        TranscriptMessage::text(text.clone(), TranscriptStyle::for_user_submit(&text)),
                    );

                    if let Some(dispatch) = dispatch_slash_command(&text, None) {
                        match dispatch {
                            SlashDispatch::Quit => should_exit.set(true),
                            SlashDispatch::Compact | SlashDispatch::Goal { .. } => {
                                if let Some(session) = agent_session.as_ref() {
                                    SlashDispatcher::spawn(Arc::clone(session), dispatch);
                                } else {
                                    push_transcript_message(
                                        &mut messages,
                                        &mut messages_revision,
                                        TranscriptMessage::text(
                                            "Agent session required for this command.",
                                            TranscriptStyle::Meta,
                                        ),
                                    );
                                }
                            }
                            SlashDispatch::Unimplemented(command) => {
                                push_transcript_message(
                                    &mut messages,
                                    &mut messages_revision,
                                    TranscriptMessage::text(
                                        slash_unimplemented_message(&command),
                                        TranscriptStyle::Meta,
                                    ),
                                );
                            }
                        }
                    } else if busy.get() {
                        prompt_queue.write().push(text);
                    } else if let Some(session) = agent_session.as_ref() {
                        turn_count.set(turn_count.get().saturating_add(1));
                        mark_busy(
                            &mut BusyActivation {
                                busy: &mut busy,
                                busy_started_at: &mut busy_started_at,
                                elapsed_secs: &mut elapsed_secs,
                                spinner_tick: &mut spinner_tick,
                                activity_label: &mut activity_label,
                            },
                            false,
                        );
                        TurnDispatcher::spawn_turn(Arc::clone(session), text, false);
                    } else {
                        push_transcript_message(
                            &mut messages,
                            &mut messages_revision,
                            TranscriptMessage::text(
                                "Agent session unavailable — check logs or run `elph doctor`.",
                                TranscriptStyle::Meta,
                            ),
                        );
                    }

                    draft.set(String::new());
                    live_draft.set(String::new());
                    suppress_enter_newline.set(true);
                },
                        )
                    }
                }
            })
        }
    }
}
