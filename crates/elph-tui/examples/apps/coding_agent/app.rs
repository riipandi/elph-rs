//! Root coding-agent shell — state, tick loop, layout.

use crate::common::chat_shell::{Header, PromptChrome, StatusRow, TranscriptPanel};
use crate::common::lipsum_mock::mock_paragraph;
use crate::common::slash_palette::coding_agent_registry;
use crate::common::transcript::{TranscriptMessage, TranscriptStyle};
use crate::overlays::kinds::DEMO_MULTI_OPTION_COUNT;
use crate::overlays::{
    OverlayDemoBodyProps, OverlayKind, handle_global_shortcut, handle_overlay_key, handle_submit, overlay_chrome,
    overlay_demo_body, overlay_header, record_demo_answer,
};
use crate::seed::seed_transcript;
use crate::shell::ThinkingLevel;
use elph_tui::prelude::*;
use elph_tui::slash_palette::{
    SlashPaletteKeyAction, build_snapshot, open_palette_draft, resolve_snapshot_key_action, sync_selection,
};
use elph_tui::text_editing::{ShellFocus, prompt_focus_char};
use std::time::{Duration, Instant};

const SHELL_TICK_MS: u64 = 50;

#[component]
pub fn CodingAgent(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (screen_width, screen_height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();

    let mut should_exit = hooks.use_state(|| false);
    let mut overlay = hooks.use_state(|| None::<OverlayKind>);
    let mut messages = hooks.use_state(seed_transcript);
    let mut messages_revision = hooks.use_state(|| 0u64);
    let mut draft = hooks.use_state(String::new);
    let mut live_draft = hooks.use_ref(String::new);
    let mut suppress_enter = hooks.use_ref(|| false);

    let mut busy = hooks.use_state(|| false);
    let mut elapsed = hooks.use_state(|| 0.0f64);
    let mut spinner_tick = hooks.use_state(|| 0u32);
    let mut activity = hooks.use_state(|| "Thinking".to_string());
    let mut busy_started = hooks.use_ref(|| None::<Instant>);
    let mut turn_count = hooks.use_state(|| 0u32);

    let mut agent_mode = hooks.use_state(DialogAgentMode::default);
    let mut thinking = hooks.use_state(ThinkingLevel::default);
    let model_label = hooks.use_state(|| "anthropic/claude-sonnet".to_string());
    let dialog_selected = hooks.use_state(|| 0usize);
    let multi_cursor = hooks.use_state(|| 0usize);
    let mut multi_checked = hooks.use_state(|| vec![false; DEMO_MULTI_OPTION_COUNT]);
    let user_answer = hooks.use_state(String::new);
    let confirm_focus = hooks.use_state(ConfirmButtonFocus::default);
    let mut palette_selected = hooks.use_state(|| 0usize);
    let mut palette_query_track = hooks.use_ref(String::new);
    let mut slash_palette_active = hooks.use_ref(|| false);
    let mut shell_focus = hooks.use_state(ShellFocus::default);
    let mut force_palette_sync = hooks.use_ref(|| false);
    let mut reply_due = hooks.use_ref(|| None::<Instant>);
    let mut reply_text = hooks.use_ref(String::new);

    let commands = coding_agent_registry();

    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(SHELL_TICK_MS)).await;

            if busy.get() {
                spinner_tick.set(spinner_tick.get().wrapping_add(1));
                if let Some(started) = busy_started.read().as_ref() {
                    elapsed.set(started.elapsed().as_secs_f64());
                    if elapsed.get() >= 1.0 {
                        activity.set("Responding".to_string());
                    }
                }
            }

            let due_snapshot = *reply_due.read();
            if let Some(due) = due_snapshot
                && Instant::now() >= due
            {
                reply_due.set(None);
                let user_copy = reply_text.read().clone();
                messages.set({
                    let mut list = messages.read().clone();
                    list.push(TranscriptMessage::text("Considering your request…", TranscriptStyle::Thinking));
                    list.push(TranscriptMessage::text(
                        format!("Re: \"{}\"\n\n{}", user_copy.trim(), mock_paragraph()),
                        TranscriptStyle::Assistant,
                    ));
                    list
                });
                messages_revision.set(messages_revision.get().wrapping_add(1));
                busy.set(false);
                busy_started.set(None);
                activity.set("Thinking".to_string());
                if overlay.get() == Some(OverlayKind::TodoProgress) {
                    overlay.set(None);
                }
            }
        }
    });

    let overlay_open = overlay.get().is_some();

    let commands_for_keys = commands.clone();
    hooks.use_terminal_events(move |event| {
        let draft_text = live_draft.read().clone();
        let palette_snapshot = build_snapshot(&draft_text, &commands_for_keys, screen_height);
        let TerminalEvent::Key(KeyEvent {
            code, kind, modifiers, ..
        }) = event
        else {
            return;
        };
        if kind == KeyEventKind::Release {
            return;
        }

        let active_overlay = overlay.get();
        if let Some(kind) = active_overlay {
            if handle_overlay_key(
                kind,
                code,
                modifiers,
                &mut overlay,
                &mut shell_focus,
                &mut messages,
                &mut messages_revision,
                dialog_selected.get(),
                &user_answer,
            ) {
                return;
            }
            // Dialog bodies handle their own keys (↑↓, Space, etc.).
            return;
        }

        if let Some(action) =
            resolve_snapshot_key_action(&draft_text, &palette_snapshot, palette_selected.get(), code, modifiers)
        {
            match action {
                SlashPaletteKeyAction::CompleteDraft {
                    text,
                    suppress_enter_newline,
                } => {
                    shell_focus.set(ShellFocus::Prompt);
                    draft.set(text.clone());
                    live_draft.set(text.clone());
                    suppress_enter.set(suppress_enter_newline);
                    force_palette_sync.set(true);
                    palette_query_track.set(String::new());
                    palette_selected.set(0);
                    if suppress_enter_newline {
                        handle_submit(
                            text,
                            &mut messages,
                            &mut messages_revision,
                            &mut overlay,
                            &mut multi_checked,
                            &mut draft,
                            &mut live_draft,
                            &mut suppress_enter,
                            &mut turn_count,
                            &mut busy,
                            &mut busy_started,
                            &mut elapsed,
                            &mut activity,
                            &mut reply_text,
                            &mut reply_due,
                        );
                    }
                }
                SlashPaletteKeyAction::MoveSelection(index) => {
                    shell_focus.set(ShellFocus::Prompt);
                    palette_selected.set(index);
                }
                SlashPaletteKeyAction::Dismiss => {
                    shell_focus.set(ShellFocus::Prompt);
                    draft.set(String::new());
                    live_draft.set(String::new());
                    palette_selected.set(0);
                    suppress_enter.set(true);
                }
            }
            return;
        }

        if shell_focus.get() == ShellFocus::Transcript
            && let Some(ch) = prompt_focus_char(code, modifiers)
        {
            shell_focus.set(ShellFocus::Prompt);
            let mut text = live_draft.read().clone();
            text.push(ch);
            draft.set(text.clone());
            live_draft.set(text);
            force_palette_sync.set(true);
            suppress_enter.set(false);
            return;
        }

        if modifiers.is_empty()
            && code == KeyCode::Char('/')
            && let Some(seeded) = open_palette_draft(&draft_text)
        {
            shell_focus.set(ShellFocus::Prompt);
            draft.set(seeded.clone());
            live_draft.set(seeded);
            palette_selected.set(0);
            force_palette_sync.set(true);
            return;
        }

        if modifiers.is_empty() && code == KeyCode::Esc && shell_focus.get() == ShellFocus::Transcript {
            shell_focus.set(ShellFocus::Prompt);
        }

        handle_global_shortcut(code, modifiers, &mut should_exit, &mut overlay, &mut agent_mode, &mut thinking);
    });

    if should_exit.get() {
        system.exit();
    }

    let draft_for_palette = live_draft.read().clone();
    let slash_palette_snapshot = build_snapshot(&draft_for_palette, &commands, screen_height);
    slash_palette_active.set(slash_palette_snapshot.visible && !overlay_open);
    {
        let old_index = palette_selected.get();
        let mut query = palette_query_track.write();
        let mut index = old_index;
        sync_selection(&mut query, &mut index, &slash_palette_snapshot);
        if index != old_index {
            palette_selected.set(index);
        }
    }

    let (accent_r, accent_g, accent_b) = agent_mode.get().accent_rgb();
    let scanner_accent = rgb(accent_r, accent_g, accent_b);
    let session_label = format!("Session: demo-8f3a | turn: {}", turn_count.get());
    let stats_label = "$0.00 | 12k | 6.0% (200k)".to_string();
    let active_overlay = overlay.get();
    let prompt_focused = shell_focus.get() == ShellFocus::Prompt && !overlay_open;
    let transcript_focused = shell_focus.get() == ShellFocus::Transcript && !overlay_open;

    let overlay_element: Option<AnyElement<'static>> = active_overlay.map(|kind| {
        let (chrome, list_height) = overlay_chrome(screen_width, screen_height, kind);
        let body_w = chrome.inner_body_width();
        let header = overlay_header(kind);

        element! {
            DialogShellOverlay(
                screen_width: screen_width,
                screen_height: screen_height,
                chrome: chrome,
                header: header,
            ) {
                #(overlay_demo_body(&mut OverlayDemoBodyProps {
                    kind,
                    width: body_w,
                    list_height,
                    selected: dialog_selected,
                    multi_cursor,
                    multi_checked,
                    user_answer,
                    confirm_focus,
                    on_confirm_yes: HandlerMut::from(move |_| {
                        record_demo_answer(&mut messages, &mut messages_revision, "Confirm", "Yes");
                        overlay.set(None);
                        shell_focus.set(ShellFocus::Prompt);
                    }),
                    on_confirm_no: HandlerMut::from(move |_| {
                        record_demo_answer(&mut messages, &mut messages_revision, "Confirm", "No");
                        overlay.set(None);
                        shell_focus.set(ShellFocus::Prompt);
                    }),
                    on_multi_submit: HandlerMut::from(move |picked: Vec<usize>| {
                        let options = crate::common::lipsum_mock::mock_select_options(DEMO_MULTI_OPTION_COUNT);
                        let labels: Vec<_> = picked
                            .iter()
                            .filter_map(|i| options.get(*i).map(|o| o.name.clone()))
                            .collect();
                        let detail = if labels.is_empty() {
                            "(none selected)".to_string()
                        } else {
                            labels.join(", ")
                        };
                        record_demo_answer(&mut messages, &mut messages_revision, "Multiple choice", &detail);
                        overlay.set(None);
                        shell_focus.set(ShellFocus::Prompt);
                    }),
                }))
            }
        }
        .into()
    });

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
            position: Position::Relative,
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
                sticky_scroll: true,
                keyboard_scroll: transcript_focused,
                has_focus: transcript_focused,
            )
            StatusRow(
                screen_width: screen_width,
                busy: busy.get(),
                activity_label: activity.read().clone(),
                accent: scanner_accent,
                spinner_tick: spinner_tick.get(),
                elapsed_secs: elapsed.get(),
            )
            PromptChrome(
                screen_width: screen_width,
                screen_height: screen_height,
                agent_mode: agent_mode.get(),
                thinking_level: thinking.get(),
                has_focus: prompt_focused,
                project_label: "~ elph [refactor-tui]".to_string(),
                model_label: model_label.read().clone(),
                supports_images: false,
                draft: Some(draft),
                live_draft: Some(live_draft),
                suppress_enter_newline: Some(suppress_enter),
                slash_palette_active: Some(slash_palette_active),
                force_palette_sync: Some(force_palette_sync),
                slash_palette_snapshot: slash_palette_snapshot,
                slash_palette_selected: Some(palette_selected),
                on_escape: move |_| {
                    shell_focus.set(ShellFocus::Transcript);
                },
                on_submit: move |text: String| {
                    shell_focus.set(ShellFocus::Prompt);
                    handle_submit(
                        text,
                        &mut messages,
                        &mut messages_revision,
                        &mut overlay,
                        &mut multi_checked,
                        &mut draft,
                        &mut live_draft,
                        &mut suppress_enter,
                        &mut turn_count,
                        &mut busy,
                        &mut busy_started,
                        &mut elapsed,
                        &mut activity,
                        &mut reply_text,
                        &mut reply_due,
                    );
                },
            )
            #(overlay_element)
        }
    }
}
