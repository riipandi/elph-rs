use crokey::KeyCombination;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Builds a normalized [`KeyCombination`] from iocraft/crossterm key parts.
pub fn key_combination(code: KeyCode, modifiers: KeyModifiers) -> KeyCombination {
    KeyCombination::from(KeyEvent::new(code, modifiers))
}