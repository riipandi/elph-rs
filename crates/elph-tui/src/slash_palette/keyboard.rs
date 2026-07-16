//! Keyboard resolution for the slash command palette.

use iocraft::prelude::{KeyCode, KeyModifiers};

use super::SlashCommand;
use super::model::{PaletteSnapshot, complete_command, palette_visible, selected_command_name};

/// Palette-specific key outcome for the shell to apply.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SlashPaletteKeyAction {
    MoveSelection(usize),
    CompleteDraft { text: String, suppress_enter_newline: bool },
    Dismiss,
}

pub fn resolve_key_action(
    draft: &str,
    filtered_commands: &[SlashCommand],
    selected_index: usize,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> Option<SlashPaletteKeyAction> {
    if !palette_visible(draft) {
        return None;
    }

    match (modifiers, code) {
        (_, KeyCode::Esc) if modifiers.is_empty() => Some(SlashPaletteKeyAction::Dismiss),
        (_, KeyCode::Tab) | (_, KeyCode::Right) => {
            selected_command_name(filtered_commands, selected_index).map(|name| SlashPaletteKeyAction::CompleteDraft {
                text: complete_command(draft, name),
                suppress_enter_newline: false,
            })
        }
        (_, KeyCode::Enter) if modifiers.is_empty() => {
            selected_command_name(filtered_commands, selected_index).map(|name| SlashPaletteKeyAction::CompleteDraft {
                text: complete_command(draft, name),
                suppress_enter_newline: true,
            })
        }
        (_, KeyCode::Up) | (_, KeyCode::Down) if modifiers.is_empty() => {
            if filtered_commands.is_empty() {
                return None;
            }
            Some(SlashPaletteKeyAction::MoveSelection(index_after_key(
                selected_index,
                filtered_commands.len(),
                code,
                modifiers,
                super::model::FAST_SCROLL_STEP,
            )))
        }
        _ => None,
    }
}

pub fn resolve_snapshot_key_action(
    draft: &str,
    snapshot: &PaletteSnapshot,
    selected_index: usize,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> Option<SlashPaletteKeyAction> {
    if !snapshot.should_render() {
        return None;
    }
    resolve_key_action(draft, &snapshot.filtered, selected_index, code, modifiers)
}

fn index_after_key(current: usize, len: usize, code: KeyCode, modifiers: KeyModifiers, step: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let fast = modifiers.contains(KeyModifiers::SHIFT);
    let delta = if fast { step as isize } else { 1 };
    let change = match code {
        KeyCode::Up => Some(-delta),
        KeyCode::Down => Some(delta),
        _ => None,
    };
    let Some(change) = change else {
        return current;
    };
    let len = len as isize;
    let next = (current as isize + change).rem_euclid(len);
    next as usize
}
