//! Prompt submit and slash-command side effects.

use super::demos::{
    demo_answer_line, demo_meta_notice, demo_skill_prompt, demo_thinking_pair, demo_tool_failed, demo_tool_success,
};
use super::kinds::{DEMO_MULTI_OPTION_COUNT, OverlayKind};
use crate::common::transcript::{TranscriptMessage, TranscriptStyle};
use crate::seed::seed_transcript;
use iocraft::prelude::*;
use std::time::{Duration, Instant};

const HELP_TEXT: &str = "Dialogs: /demo-mode /demo-model /demo-multi /demo-input /demo-confirm \
/demo-confirm-buttons /demo-todo /demo-progress · Transcript: /demo-tool /demo-tool-fail \
/demo-thinking /demo-skill /demo-meta · /demo-busy simulates a turn · /clear /compact";

pub fn clear_draft(draft: &mut State<String>, live_draft: &mut Ref<String>, suppress_enter: &mut Ref<bool>) {
    draft.set(String::new());
    live_draft.set(String::new());
    suppress_enter.set(true);
}

fn push_messages(
    messages: &mut State<Vec<TranscriptMessage>>,
    messages_revision: &mut State<u64>,
    list: Vec<TranscriptMessage>,
) {
    messages.set(list);
    messages_revision.set(messages_revision.get().wrapping_add(1));
}

fn append_messages(
    messages: &mut State<Vec<TranscriptMessage>>,
    messages_revision: &mut State<u64>,
    extra: Vec<TranscriptMessage>,
) {
    let mut list = messages.read().clone();
    list.extend(extra);
    push_messages(messages, messages_revision, list);
}

fn slash_name(trimmed: &str) -> Option<&str> {
    trimmed
        .strip_prefix('/')
        .map(|body| body.split_whitespace().next().unwrap_or(""))
}

fn record_skill_and_open(
    messages: &mut State<Vec<TranscriptMessage>>,
    messages_revision: &mut State<u64>,
    overlay: &mut State<Option<OverlayKind>>,
    command: &str,
    kind: OverlayKind,
) {
    append_messages(
        messages,
        messages_revision,
        vec![TranscriptMessage::text(command, TranscriptStyle::SkillPrompt)],
    );
    overlay.set(Some(kind));
}

fn reset_multi_checked(multi_checked: &mut State<Vec<bool>>) {
    multi_checked.set(vec![false; DEMO_MULTI_OPTION_COUNT]);
}

#[allow(clippy::too_many_arguments)]
fn start_busy_turn(
    turn_count: &mut State<u32>,
    busy: &mut State<bool>,
    busy_started: &mut Ref<Option<Instant>>,
    elapsed: &mut State<f64>,
    activity: &mut State<String>,
    reply_text: &mut Ref<String>,
    reply_due: &mut Ref<Option<Instant>>,
    overlay: &mut State<Option<OverlayKind>>,
    prompt: &str,
) {
    turn_count.set(turn_count.get().saturating_add(1));
    busy.set(true);
    busy_started.set(Some(Instant::now()));
    elapsed.set(0.0);
    activity.set("Thinking".to_string());
    reply_text.set(prompt.to_string());
    reply_due.set(Some(Instant::now() + Duration::from_secs(2)));
    overlay.set(Some(OverlayKind::TodoProgress));
}

#[allow(clippy::too_many_arguments)]
pub fn handle_submit(
    text: String,
    messages: &mut State<Vec<TranscriptMessage>>,
    messages_revision: &mut State<u64>,
    overlay: &mut State<Option<OverlayKind>>,
    multi_checked: &mut State<Vec<bool>>,
    draft: &mut State<String>,
    live_draft: &mut Ref<String>,
    suppress_enter: &mut Ref<bool>,
    turn_count: &mut State<u32>,
    busy: &mut State<bool>,
    busy_started: &mut Ref<Option<Instant>>,
    elapsed: &mut State<f64>,
    activity: &mut State<String>,
    reply_text: &mut Ref<String>,
    reply_due: &mut Ref<Option<Instant>>,
) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return;
    }

    let Some(name) = slash_name(trimmed) else {
        append_messages(
            messages,
            messages_revision,
            vec![TranscriptMessage::text(trimmed, TranscriptStyle::User)],
        );
        start_busy_turn(
            turn_count,
            busy,
            busy_started,
            elapsed,
            activity,
            reply_text,
            reply_due,
            overlay,
            trimmed,
        );
        clear_draft(draft, live_draft, suppress_enter);
        return;
    };

    match name {
        "clear" => {
            push_messages(messages, messages_revision, seed_transcript());
            clear_draft(draft, live_draft, suppress_enter);
        }
        "help" => {
            append_messages(messages, messages_revision, {
                let mut list = messages.read().clone();
                list.push(TranscriptMessage::text(trimmed, TranscriptStyle::SkillPrompt));
                list.push(TranscriptMessage::text(HELP_TEXT, TranscriptStyle::Meta));
                list
            });
            clear_draft(draft, live_draft, suppress_enter);
        }
        "compact" => {
            append_messages(
                messages,
                messages_revision,
                vec![TranscriptMessage::text(
                    "History compacted (demo)",
                    TranscriptStyle::Meta,
                )],
            );
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-mode" | "mode" => {
            record_skill_and_open(messages, messages_revision, overlay, trimmed, OverlayKind::Mode);
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-model" | "model" => {
            record_skill_and_open(messages, messages_revision, overlay, trimmed, OverlayKind::Question);
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-multi" => {
            reset_multi_checked(multi_checked);
            record_skill_and_open(messages, messages_revision, overlay, trimmed, OverlayKind::MultiChoice);
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-input" => {
            record_skill_and_open(messages, messages_revision, overlay, trimmed, OverlayKind::UserInput);
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-confirm" | "bash" => {
            record_skill_and_open(messages, messages_revision, overlay, trimmed, OverlayKind::Confirm);
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-confirm-buttons" => {
            record_skill_and_open(messages, messages_revision, overlay, trimmed, OverlayKind::ConfirmButtons);
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-todo" | "goal" => {
            record_skill_and_open(messages, messages_revision, overlay, trimmed, OverlayKind::TodoList);
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-progress" | "progress" => {
            record_skill_and_open(messages, messages_revision, overlay, trimmed, OverlayKind::TodoProgress);
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-busy" => {
            append_messages(
                messages,
                messages_revision,
                vec![TranscriptMessage::text(trimmed, TranscriptStyle::SkillPrompt)],
            );
            start_busy_turn(
                turn_count,
                busy,
                busy_started,
                elapsed,
                activity,
                reply_text,
                reply_due,
                overlay,
                "demo busy turn",
            );
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-tool" => {
            append_messages(messages, messages_revision, {
                let mut list = messages.read().clone();
                list.push(TranscriptMessage::text(trimmed, TranscriptStyle::SkillPrompt));
                list.extend(demo_tool_success());
                list
            });
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-tool-fail" => {
            append_messages(messages, messages_revision, {
                let mut list = messages.read().clone();
                list.push(TranscriptMessage::text(trimmed, TranscriptStyle::SkillPrompt));
                list.extend(demo_tool_failed());
                list
            });
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-thinking" => {
            append_messages(messages, messages_revision, {
                let mut list = messages.read().clone();
                list.push(TranscriptMessage::text(trimmed, TranscriptStyle::SkillPrompt));
                list.extend(demo_thinking_pair());
                list
            });
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-skill" => {
            append_messages(messages, messages_revision, {
                let mut list = messages.read().clone();
                list.extend(demo_skill_prompt());
                list
            });
            clear_draft(draft, live_draft, suppress_enter);
        }
        "demo-meta" => {
            append_messages(messages, messages_revision, {
                let mut list = messages.read().clone();
                list.push(TranscriptMessage::text(trimmed, TranscriptStyle::SkillPrompt));
                list.extend(demo_meta_notice());
                list
            });
            clear_draft(draft, live_draft, suppress_enter);
        }
        _ => {
            append_messages(
                messages,
                messages_revision,
                vec![TranscriptMessage::text(
                    format!("Unknown slash command: /{name} — type /help"),
                    TranscriptStyle::Meta,
                )],
            );
            clear_draft(draft, live_draft, suppress_enter);
        }
    }
}

pub fn record_demo_answer(
    messages: &mut State<Vec<TranscriptMessage>>,
    messages_revision: &mut State<u64>,
    label: &str,
    detail: &str,
) {
    append_messages(messages, messages_revision, demo_answer_line(label, detail));
}
