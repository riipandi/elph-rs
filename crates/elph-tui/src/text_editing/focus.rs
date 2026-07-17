//! Prompt vs transcript focus for chat shells.

use iocraft::prelude::*;

use super::is_transcript_scroll_key;

/// Which region owns plain-key typing in a two-pane shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ShellFocus {
    #[default]
    Prompt,
    Transcript,
}

/// Plain letter, space, or `/` — refocus the prompt and seed the first keystroke.
pub fn prompt_focus_char(code: KeyCode, modifiers: KeyModifiers) -> Option<char> {
    if !modifiers.is_empty() {
        return None;
    }
    match code {
        KeyCode::Char(' ') => Some(' '),
        KeyCode::Char('/') => Some('/'),
        KeyCode::Char(c) if c.is_ascii_alphabetic() => Some(c),
        _ => None,
    }
}

/// Keys that scroll the transcript when it has focus (plain arrows and Shift+arrows).
pub fn transcript_nav_key(code: KeyCode, kind: KeyEventKind, modifiers: KeyModifiers) -> bool {
    if kind == KeyEventKind::Release {
        return false;
    }
    if is_transcript_scroll_key(code, kind, modifiers) {
        return true;
    }
    !modifiers.intersects(KeyModifiers::CONTROL | KeyModifiers::ALT | KeyModifiers::META)
        && matches!(
            code,
            KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompt_focus_char_accepts_slash() {
        assert_eq!(prompt_focus_char(KeyCode::Char('/'), KeyModifiers::empty()), Some('/'));
        assert_eq!(prompt_focus_char(KeyCode::Char('1'), KeyModifiers::empty()), None);
    }

    #[test]
    fn transcript_nav_includes_shift_arrows() {
        assert!(transcript_nav_key(KeyCode::Up, KeyEventKind::Press, KeyModifiers::SHIFT));
        assert!(!transcript_nav_key(
            KeyCode::Char('/'),
            KeyEventKind::Press,
            KeyModifiers::empty()
        ));
    }
}
