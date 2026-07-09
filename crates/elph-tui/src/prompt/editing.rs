//! Prompt textarea keybindings.
//!
//! All cursor navigation and delete-word bindings are handled here *before*
//! [`Context::textarea`] runs, using [`Context::key_presses_when`] so they work
//! reliably even when chat scroll or focus order would otherwise swallow arrow
//! keys. Plain arrows, Home/End, and Emacs/word-style shortcuts are included.

use slt::{Context, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, TextareaState};
use unicode_segmentation::UnicodeSegmentation;

/// Grapheme-cluster count (matches SLT cursor indexing).
fn grapheme_count(s: &str) -> usize {
    s.graphemes(true).count()
}

fn byte_index_for_grapheme(s: &str, cluster_index: usize) -> usize {
    if cluster_index == 0 {
        return 0;
    }
    s.grapheme_indices(true)
        .nth(cluster_index)
        .map_or(s.len(), |(idx, _)| idx)
}

fn cluster_is_alphanumeric(cluster: &str) -> bool {
    cluster.chars().next().is_some_and(|c| c.is_alphanumeric())
}

fn prev_word_col(line: &str, col: usize) -> usize {
    let clusters: Vec<&str> = line.graphemes(true).collect();
    let mut pos = col.min(clusters.len());
    while pos > 0 && !cluster_is_alphanumeric(clusters[pos - 1]) {
        pos -= 1;
    }
    while pos > 0 && cluster_is_alphanumeric(clusters[pos - 1]) {
        pos -= 1;
    }
    pos
}

fn next_word_col(line: &str, col: usize) -> usize {
    let clusters: Vec<&str> = line.graphemes(true).collect();
    let mut pos = col.min(clusters.len());
    while pos < clusters.len() && !cluster_is_alphanumeric(clusters[pos]) {
        pos += 1;
    }
    while pos < clusters.len() && cluster_is_alphanumeric(clusters[pos]) {
        pos += 1;
    }
    pos
}

fn current_line(state: &TextareaState) -> &str {
    &state.lines[state.cursor_row.min(state.lines.len().saturating_sub(1))]
}

fn normalize_cursor(state: &mut TextareaState) {
    if state.lines.is_empty() {
        state.lines.push(String::new());
    }
    state.cursor_row = state.cursor_row.min(state.lines.len().saturating_sub(1));
    state.cursor_col = state.cursor_col.min(grapheme_count(&state.lines[state.cursor_row]));
}

fn move_char_left(state: &mut TextareaState) {
    normalize_cursor(state);
    if state.cursor_col > 0 {
        state.cursor_col -= 1;
    } else if state.cursor_row > 0 {
        state.cursor_row -= 1;
        state.cursor_col = grapheme_count(current_line(state));
    }
}

fn move_char_right(state: &mut TextareaState) {
    normalize_cursor(state);
    let line_len = grapheme_count(current_line(state));
    if state.cursor_col < line_len {
        state.cursor_col += 1;
    } else if state.cursor_row + 1 < state.lines.len() {
        state.cursor_row += 1;
        state.cursor_col = 0;
    }
}

fn move_line_up(state: &mut TextareaState) {
    normalize_cursor(state);
    if state.cursor_row > 0 {
        state.cursor_row -= 1;
        state.cursor_col = state.cursor_col.min(grapheme_count(&state.lines[state.cursor_row]));
    }
}

fn move_line_down(state: &mut TextareaState) {
    normalize_cursor(state);
    if state.cursor_row + 1 < state.lines.len() {
        state.cursor_row += 1;
        state.cursor_col = state.cursor_col.min(grapheme_count(&state.lines[state.cursor_row]));
    }
}

fn move_word_left(state: &mut TextareaState) {
    normalize_cursor(state);
    if state.cursor_col > 0 {
        state.cursor_col = prev_word_col(current_line(state), state.cursor_col);
    } else if state.cursor_row > 0 {
        state.cursor_row -= 1;
        state.cursor_col = grapheme_count(current_line(state));
    }
}

fn move_word_right(state: &mut TextareaState) {
    normalize_cursor(state);
    let line_len = grapheme_count(current_line(state));
    if state.cursor_col < line_len {
        state.cursor_col = next_word_col(current_line(state), state.cursor_col);
    } else if state.cursor_row + 1 < state.lines.len() {
        state.cursor_row += 1;
        state.cursor_col = 0;
    }
}

fn move_line_start(state: &mut TextareaState) {
    normalize_cursor(state);
    state.cursor_col = 0;
}

fn move_line_end(state: &mut TextareaState) {
    normalize_cursor(state);
    state.cursor_col = grapheme_count(current_line(state));
}

fn delete_range(state: &mut TextareaState, start_col: usize, end_col: usize) {
    normalize_cursor(state);
    let start_col = start_col.min(end_col);
    let end_col = end_col.max(start_col);
    if start_col == end_col {
        return;
    }
    let line = &mut state.lines[state.cursor_row];
    let start = byte_index_for_grapheme(line, start_col);
    let end = byte_index_for_grapheme(line, end_col);
    line.replace_range(start..end, "");
    state.cursor_col = start_col;
}

fn delete_word_backward(state: &mut TextareaState) {
    normalize_cursor(state);
    if state.cursor_col > 0 {
        let target = prev_word_col(current_line(state), state.cursor_col);
        delete_range(state, target, state.cursor_col);
    } else if state.cursor_row > 0 {
        let current = state.lines.remove(state.cursor_row);
        state.cursor_row -= 1;
        state.cursor_col = grapheme_count(&state.lines[state.cursor_row]);
        state.lines[state.cursor_row].push_str(&current);
    }
}

fn delete_word_forward(state: &mut TextareaState) {
    normalize_cursor(state);
    let line_len = grapheme_count(current_line(state));
    if state.cursor_col < line_len {
        let target = next_word_col(current_line(state), state.cursor_col);
        delete_range(state, state.cursor_col, target);
    } else if state.cursor_row + 1 < state.lines.len() {
        let next = state.lines.remove(state.cursor_row + 1);
        state.lines[state.cursor_row].push_str(&next);
    }
}

fn delete_to_line_start(state: &mut TextareaState) {
    normalize_cursor(state);
    if state.cursor_col > 0 {
        delete_range(state, 0, state.cursor_col);
    }
}

fn insert_newline(state: &mut TextareaState) {
    normalize_cursor(state);
    let split_index = byte_index_for_grapheme(&state.lines[state.cursor_row], state.cursor_col);
    let remainder = state.lines[state.cursor_row].split_off(split_index);
    state.cursor_row += 1;
    state.lines.insert(state.cursor_row, remainder);
    state.cursor_col = 0;
}

fn is_newline_key(key: &KeyEvent) -> bool {
    if key.kind != KeyEventKind::Press {
        return false;
    }
    match key.code {
        KeyCode::Enter if key.modifiers.contains(KeyModifiers::SHIFT) => true,
        KeyCode::Char('\n') => true,
        KeyCode::Char(c) if key.modifiers.contains(KeyModifiers::CONTROL) && (c == 'j' || c == 'J') => true,
        _ => false,
    }
}

fn delete_to_line_end(state: &mut TextareaState) {
    normalize_cursor(state);
    let line_len = grapheme_count(current_line(state));
    if state.cursor_col < line_len {
        delete_range(state, state.cursor_col, line_len);
    }
}

fn has_word_modifier(modifiers: KeyModifiers) -> bool {
    modifiers.contains(KeyModifiers::CONTROL)
        || modifiers.contains(KeyModifiers::ALT)
        || modifiers.contains(KeyModifiers::SUPER)
        || modifiers.contains(KeyModifiers::META)
}

fn is_super_only(modifiers: KeyModifiers) -> bool {
    modifiers.contains(KeyModifiers::SUPER)
        && !modifiers.contains(KeyModifiers::ALT)
        && !modifiers.contains(KeyModifiers::CONTROL)
}

fn is_word_nav_modifier(modifiers: KeyModifiers) -> bool {
    if is_super_only(modifiers) {
        return false;
    }
    modifiers.contains(KeyModifiers::ALT)
        || modifiers.contains(KeyModifiers::CONTROL)
        || modifiers.contains(KeyModifiers::META)
}

fn is_delete_word_modifier(modifiers: KeyModifiers) -> bool {
    has_word_modifier(modifiers)
}

fn is_plain_navigation(modifiers: KeyModifiers) -> bool {
    modifiers == KeyModifiers::NONE
}

/// Apply one prompt key binding. Returns `true` when the event was handled.
pub fn apply_textarea_key(state: &mut TextareaState, key: &KeyEvent) -> bool {
    if is_newline_key(key) {
        insert_newline(state);
        return true;
    }

    if key.kind != KeyEventKind::Press {
        return false;
    }

    match key.code {
        KeyCode::Left if is_super_only(key.modifiers) => {
            move_line_start(state);
            true
        }
        KeyCode::Right if is_super_only(key.modifiers) => {
            move_line_end(state);
            true
        }
        KeyCode::Left if is_word_nav_modifier(key.modifiers) => {
            move_word_left(state);
            true
        }
        KeyCode::Right if is_word_nav_modifier(key.modifiers) => {
            move_word_right(state);
            true
        }
        KeyCode::Left if is_plain_navigation(key.modifiers) => {
            move_char_left(state);
            true
        }
        KeyCode::Right if is_plain_navigation(key.modifiers) => {
            move_char_right(state);
            true
        }
        KeyCode::Up if is_plain_navigation(key.modifiers) => {
            move_line_up(state);
            true
        }
        KeyCode::Down if is_plain_navigation(key.modifiers) => {
            move_line_down(state);
            true
        }
        KeyCode::Home if is_plain_navigation(key.modifiers) || has_word_modifier(key.modifiers) => {
            move_line_start(state);
            true
        }
        KeyCode::End if is_plain_navigation(key.modifiers) || has_word_modifier(key.modifiers) => {
            move_line_end(state);
            true
        }
        KeyCode::Char('b') if is_word_nav_modifier(key.modifiers) => {
            move_word_left(state);
            true
        }
        KeyCode::Char('f') if is_word_nav_modifier(key.modifiers) => {
            move_word_right(state);
            true
        }
        KeyCode::Backspace if is_delete_word_modifier(key.modifiers) => {
            if is_super_only(key.modifiers) || key.modifiers.contains(KeyModifiers::CONTROL) {
                delete_to_line_start(state);
            } else {
                delete_word_backward(state);
            }
            true
        }
        KeyCode::Delete if is_delete_word_modifier(key.modifiers) => {
            if is_super_only(key.modifiers) {
                delete_to_line_end(state);
            } else {
                delete_word_forward(state);
            }
            true
        }
        KeyCode::Char('w') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            delete_word_backward(state);
            true
        }
        KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            delete_to_line_start(state);
            true
        }
        _ => false,
    }
}

/// Consume prompt navigation/delete keys before [`Context::textarea`] runs.
pub fn consume_prompt_textarea_keys(ui: &mut Context, state: &mut TextareaState, active: bool) {
    let mut consumed = Vec::new();
    for (index, key) in ui.key_presses_when(active) {
        if apply_textarea_key(state, key) {
            consumed.push(index);
        }
    }
    for index in consumed {
        ui.consume_event(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slt::{Event, KeyEvent};

    fn press(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        match Event::key_mod(code, modifiers) {
            Event::Key(key) => key,
            _ => panic!("expected key event"),
        }
    }

    fn textarea_with(text: &str) -> TextareaState {
        let mut state = TextareaState::new();
        state.set_value(text);
        state.cursor_col = grapheme_count(text);
        state
    }

    #[test]
    fn plain_left_moves_cursor() {
        let mut state = textarea_with("hello");
        state.cursor_col = grapheme_count("hello");
        assert!(apply_textarea_key(
            &mut state,
            &press(KeyCode::Left, KeyModifiers::NONE)
        ));
        assert_eq!(state.cursor_col, grapheme_count("hell"));
    }

    #[test]
    fn alt_left_jumps_to_previous_word() {
        let mut state = textarea_with("hello world");
        state.cursor_col = grapheme_count("hello world");
        assert!(apply_textarea_key(&mut state, &press(KeyCode::Left, KeyModifiers::ALT)));
        assert_eq!(state.cursor_col, grapheme_count("hello "));
    }

    #[test]
    fn alt_b_jumps_to_previous_word() {
        let mut state = textarea_with("hello world");
        state.cursor_col = grapheme_count("hello world");
        assert!(apply_textarea_key(
            &mut state,
            &press(KeyCode::Char('b'), KeyModifiers::ALT)
        ));
        assert_eq!(state.cursor_col, grapheme_count("hello "));
    }

    #[test]
    fn alt_backspace_deletes_previous_word() {
        let mut state = textarea_with("hello world");
        state.cursor_col = grapheme_count("hello world");
        assert!(apply_textarea_key(
            &mut state,
            &press(KeyCode::Backspace, KeyModifiers::ALT)
        ));
        assert_eq!(state.value(), "hello ");
    }

    #[test]
    fn super_left_moves_to_line_start() {
        let mut state = textarea_with("hello world");
        state.cursor_col = grapheme_count("hello world");
        assert!(apply_textarea_key(
            &mut state,
            &press(KeyCode::Left, KeyModifiers::SUPER)
        ));
        assert_eq!(state.cursor_col, 0);
    }

    #[test]
    fn shift_enter_inserts_newline() {
        let mut state = textarea_with("hello");
        assert!(apply_textarea_key(
            &mut state,
            &press(KeyCode::Enter, KeyModifiers::SHIFT)
        ));
        assert_eq!(state.lines.len(), 2);
        assert_eq!(state.lines[0], "hello");
    }

    #[test]
    fn ctrl_j_inserts_newline() {
        let mut state = textarea_with("hello");
        assert!(apply_textarea_key(
            &mut state,
            &press(KeyCode::Char('\x0a'), KeyModifiers::NONE)
        ));
        assert_eq!(state.lines.len(), 2);
    }

    #[test]
    fn ctrl_u_deletes_to_line_start() {
        let mut state = textarea_with("hello world");
        state.cursor_col = grapheme_count("hello ");
        assert!(apply_textarea_key(
            &mut state,
            &press(KeyCode::Char('u'), KeyModifiers::CONTROL)
        ));
        assert_eq!(state.value(), "world");
    }
}
