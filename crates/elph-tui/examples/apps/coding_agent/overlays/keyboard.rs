//! Overlay and global keyboard routing.

use super::kinds::OverlayKind;
use super::submit::record_demo_answer;
use crate::common::lipsum_mock::mock_select_options;

use crate::common::transcript::{ToolCardDetail, TranscriptMessage, TranscriptStyle};
use crate::overlays::kinds::DEMO_MULTI_OPTION_COUNT;
use crate::shell::ThinkingLevel;
use elph_tui::prelude::*;
use elph_tui::text_editing::ShellFocus;

#[allow(clippy::too_many_arguments)]
pub fn handle_overlay_key(
    kind: OverlayKind,
    code: KeyCode,
    modifiers: KeyModifiers,
    overlay: &mut State<Option<OverlayKind>>,
    shell_focus: &mut State<ShellFocus>,
    messages: &mut State<Vec<TranscriptMessage>>,
    messages_revision: &mut State<u64>,
    dialog_selected: usize,
    user_answer: &State<String>,
) -> bool {
    match (modifiers, code) {
        (_, KeyCode::Esc) if modifiers.is_empty() => {
            overlay.set(None);
            shell_focus.set(ShellFocus::Prompt);
            true
        }
        (_, KeyCode::Enter) if modifiers.is_empty() && kind == OverlayKind::Question => {
            let options = mock_select_options(DEMO_MULTI_OPTION_COUNT);
            let detail = options
                .get(dialog_selected)
                .map(|o| o.name.clone())
                .unwrap_or_else(|| format!("option {}", dialog_selected.saturating_add(1)));
            record_demo_answer(messages, messages_revision, "Single choice", &detail);
            overlay.set(None);
            shell_focus.set(ShellFocus::Prompt);
            true
        }
        (_, KeyCode::Enter) if modifiers.is_empty() && kind == OverlayKind::UserInput => {
            let answer = user_answer.read().clone();
            if answer.trim().is_empty() {
                return false;
            }
            record_demo_answer(messages, messages_revision, "User input", answer.trim());
            overlay.set(None);
            shell_focus.set(ShellFocus::Prompt);
            true
        }
        (_, KeyCode::Char('y')) if kind == OverlayKind::Confirm => {
            messages.set({
                let mut list = messages.read().clone();
                list.push(TranscriptMessage {
                    content: String::new(),
                    style: TranscriptStyle::ToolSuccess,
                    tool: Some(ToolCardDetail {
                        name: "shell_exec".to_string(),
                        args: "cargo test -p elph-tui".to_string(),
                        output: "345 passed".to_string(),
                    }),
                });
                list
            });
            messages_revision.set(messages_revision.get().wrapping_add(1));
            overlay.set(None);
            shell_focus.set(ShellFocus::Prompt);
            true
        }
        (_, KeyCode::Char('n')) if kind == OverlayKind::Confirm => {
            messages.set({
                let mut list = messages.read().clone();
                list.push(TranscriptMessage {
                    content: String::new(),
                    style: TranscriptStyle::ToolFailed,
                    tool: Some(ToolCardDetail {
                        name: "shell_exec".to_string(),
                        args: "cargo test -p elph-tui".to_string(),
                        output: "Denied by user".to_string(),
                    }),
                });
                list
            });
            messages_revision.set(messages_revision.get().wrapping_add(1));
            overlay.set(None);
            shell_focus.set(ShellFocus::Prompt);
            true
        }
        _ => false,
    }
}

fn next_agent_mode(mode: DialogAgentMode) -> DialogAgentMode {
    let modes = DialogAgentMode::all();
    let idx = modes.iter().position(|m| *m == mode).unwrap_or(0);
    modes[(idx + 1) % modes.len()]
}

pub fn handle_global_shortcut(
    code: KeyCode,
    modifiers: KeyModifiers,
    should_exit: &mut State<bool>,
    overlay: &mut State<Option<OverlayKind>>,
    agent_mode: &mut State<DialogAgentMode>,
    thinking: &mut State<ThinkingLevel>,
) {
    match (modifiers, code) {
        (m, KeyCode::Char('d')) if m.contains(KeyModifiers::CONTROL) => should_exit.set(true),
        (m, KeyCode::Tab) if !m.contains(KeyModifiers::SHIFT) => {
            agent_mode.set(next_agent_mode(agent_mode.get()));
        }
        (m, KeyCode::BackTab) if m.contains(KeyModifiers::SHIFT) => {
            thinking.set(thinking.get().next());
        }
        (m, KeyCode::Char('m')) if m.contains(KeyModifiers::CONTROL) => {
            overlay.set(Some(OverlayKind::Mode));
        }
        (m, KeyCode::Char('l')) if m.contains(KeyModifiers::CONTROL) => {
            overlay.set(Some(OverlayKind::Question));
        }
        (m, KeyCode::Char('g')) if m.contains(KeyModifiers::CONTROL) => {
            overlay.set(Some(OverlayKind::TodoList));
        }
        (m, KeyCode::Char('p')) if m.contains(KeyModifiers::CONTROL) => {
            overlay.set(Some(OverlayKind::TodoProgress));
        }
        _ => {}
    }
}
