//! Multiline prompt editor (1-row default, grows with content).

use super::scroll_bar::{ScrollbarStyle, VerticalScrollbar};
use crate::text_editing::wire_editing_shortcuts;
use crate::text_input_layout::{WrappedTextLayout, update_scroll_offset};
use iocraft::prelude::*;

/// Props for [`Textarea`].
#[derive(Clone, Default, Props)]
pub struct TextareaProps {
    pub width: u16,
    /// Minimum visible rows. Defaults to 1 when unset or zero.
    pub min_height: u16,
    /// Maximum visible rows before clipping and showing a scrollbar. Unset = grow without limit.
    pub max_height: Option<u16>,
    pub initial_value: String,
    pub has_focus: bool,
    pub text_color: Option<Color>,
    pub cursor_color: Option<Color>,
    pub value: Option<State<String>>,
    /// When false, omits the inner border (for embedding in a parent chrome).
    pub show_border: Option<bool>,
    /// Set by the parent on submit so plain Enter's ghost `\n` is dropped, not the next keystroke.
    pub suppress_enter_newline: Option<Ref<bool>>,
    pub scrollbar_style: Option<ScrollbarStyle>,
}

/// Logical row count, including an empty row after a trailing `\n`.
pub fn logical_line_count(text: &str) -> u16 {
    let lines = text.chars().filter(|&c| c == '\n').count() + 1;
    lines.max(1) as u16
}

fn newline_count(text: &str) -> usize {
    text.chars().filter(|&c| c == '\n').count()
}

/// Display rows after soft-wrapping (matches multiline [`TextInput`] layout).
pub fn display_row_count(text: &str, viewport_width: u16) -> u16 {
    WrappedTextLayout::new(text, viewport_width).row_count()
}

/// Rows to allocate vertically: omit a trailing empty continuation row unless the cursor is on it.
pub fn visible_row_count(text: &str, cursor: usize, viewport_width: u16) -> u16 {
    let layout = WrappedTextLayout::new(text, viewport_width);
    let mut rows = layout.row_count();
    if rows > 1 && text.ends_with('\n') {
        let (cursor_row, _) = layout.row_column_for_offset(cursor.min(text.len()));
        let last_row = rows.saturating_sub(1);
        if cursor_row < last_row {
            rows -= 1;
        }
    }
    rows.max(1)
}

fn compute_viewport_height(content_rows: u16, min_height: u16, max_height: Option<u16>) -> u16 {
    let min_h = min_height.max(1);
    match max_height {
        None => content_rows.max(min_h),
        Some(max) => content_rows.min(max.max(min_h)).max(min_h),
    }
}

struct TextareaLayout {
    input_width: u16,
    content_rows: u16,
    viewport_height: u16,
    show_scrollbar: bool,
}

fn layout_textarea(
    text: &str,
    cursor: usize,
    outer_width: u16,
    min_height: u16,
    max_height: Option<u16>,
) -> TextareaLayout {
    let content_full = display_row_count(text, outer_width);
    let visible_full = visible_row_count(text, cursor, outer_width);
    let viewport_full = compute_viewport_height(visible_full, min_height, max_height);
    let mut show_scrollbar = max_height.is_some() && content_full > viewport_full;
    let mut input_width = outer_width.saturating_sub(if show_scrollbar { 1 } else { 0 });
    let mut content_rows = display_row_count(text, input_width);
    let visible_rows = visible_row_count(text, cursor, input_width);
    let mut viewport_height = compute_viewport_height(visible_rows, min_height, max_height);
    show_scrollbar = max_height.is_some() && content_rows > viewport_height;
    if show_scrollbar {
        input_width = outer_width.saturating_sub(1);
        content_rows = display_row_count(text, input_width);
        let visible_rows = visible_row_count(text, cursor, input_width);
        viewport_height = compute_viewport_height(visible_rows, min_height, max_height);
        show_scrollbar = content_rows > viewport_height;
    }
    TextareaLayout {
        input_width,
        content_rows,
        viewport_height,
        show_scrollbar,
    }
}

/// Remount key for iocraft [`TextInput`] when the clipped viewport geometry changes.
fn textarea_remount_key(layout: &TextareaLayout) -> u32 {
    (layout.viewport_height as u32) << 20 | (layout.show_scrollbar as u32) << 19 | layout.input_width as u32
}

/// Byte offset after a single `\n` inserted at `prev_cursor`.
fn cursor_after_newline_insertion(prev: &str, new: &str, prev_cursor: usize) -> usize {
    if newline_count(new) <= newline_count(prev) {
        return prev_cursor.min(new.len());
    }
    (prev_cursor + '\n'.len_utf8()).min(new.len())
}

/// While suppression is active, keep real keystrokes and drop only ghost trailing newlines.
fn resolve_suppressed_change(new_value: String) -> String {
    if new_value.ends_with('\n') {
        String::new()
    } else {
        new_value
    }
}

fn apply_text_input_change(
    suppress_enter_newline: Option<Ref<bool>>,
    value: &mut State<String>,
    input_handle: &mut Ref<TextInputHandle>,
    mut cursor_snapshot: Ref<usize>,
    new_value: String,
) {
    let prev = value.read().clone();
    let prev_cursor = input_handle.read().cursor_offset();

    if let Some(mut suppress) = suppress_enter_newline
        && suppress.get()
    {
        suppress.set(false);
        let resolved = resolve_suppressed_change(new_value);
        if resolved.is_empty() {
            cursor_snapshot.set(0);
            input_handle.write().set_cursor_offset(0);
        }
        value.set(resolved);
        return;
    }

    value.set(new_value.clone());
    if newline_count(&new_value) > newline_count(&prev) {
        let next_cursor = cursor_after_newline_insertion(&prev, &new_value, prev_cursor);
        cursor_snapshot.set(next_cursor);
        input_handle.write().set_cursor_offset(next_cursor);
    }
}

/// Multiline text input with optional external state.
#[component]
pub fn Textarea(props: &TextareaProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let internal = hooks.use_state(|| props.initial_value.clone());
    let mut value = props.value.unwrap_or(internal);
    let suppress_enter_newline = props.suppress_enter_newline;
    let has_focus = props.has_focus;
    let min_height = props.min_height.max(1);
    let show_border = props.show_border.unwrap_or(true);
    let mut input_handle = hooks.use_ref_default::<TextInputHandle>();
    let cursor_snapshot = hooks.use_ref(|| 0usize);
    let scroll_offset = hooks.use_state(|| 0u16);

    wire_editing_shortcuts(&mut hooks, has_focus, true, value, input_handle, cursor_snapshot);

    let h_pad = if show_border { 2 } else { 0 };
    let inner_width = props.width.saturating_sub(h_pad);
    let text = value.read().clone();
    let cursor = input_handle.read().cursor_offset();
    let layout = layout_textarea(&text, cursor, inner_width, min_height, props.max_height);
    let wrapped = WrappedTextLayout::new(&text, layout.input_width);
    let (cursor_row, _) = wrapped.row_column_for_offset(cursor);

    let remount_key = textarea_remount_key(&layout);

    hooks.use_effect(
        {
            let mut cursor_snapshot = cursor_snapshot;
            move || {
                cursor_snapshot.set(cursor);
            }
        },
        cursor,
    );

    hooks.use_effect(
        {
            let mut scroll_offset = scroll_offset;
            move || {
                let next =
                    update_scroll_offset(scroll_offset.get(), cursor_row, layout.viewport_height, layout.content_rows);
                if scroll_offset.get() != next {
                    scroll_offset.set(next);
                }
            }
        },
        (cursor_row, layout.viewport_height, layout.content_rows),
    );

    // Remount clears iocraft's stale vertical scroll offset; restore cursor afterward.
    hooks.use_effect(
        {
            let mut input_handle = input_handle;
            let mut scroll_offset = scroll_offset;
            let cursor_snapshot = cursor_snapshot;
            move || {
                let next = update_scroll_offset(0, cursor_row, layout.viewport_height, layout.content_rows);
                scroll_offset.set(next);
                input_handle.write().set_cursor_offset(cursor_snapshot.get());
            }
        },
        remount_key,
    );

    let border_style = if show_border && has_focus {
        BorderStyle::Round
    } else if show_border {
        BorderStyle::Single
    } else {
        BorderStyle::None
    };

    let scrollbar_style = props.scrollbar_style.unwrap_or_else(ScrollbarStyle::dark);
    let viewport = layout.viewport_height;

    element! {
        View(
            width: props.width,
            height: viewport,
            flex_shrink: 0f32,
            position: Position::Relative,
            overflow: Overflow::Hidden,
            border_style: border_style,
            border_color: Color::DarkGrey,
            padding_left: if show_border { 1 } else { 0 },
            padding_right: if show_border { 1 } else { 0 },
        ) {
            View(
                position: Position::Absolute,
                top: 0,
                left: 0,
                width: layout.input_width,
                height: viewport,
                overflow: Overflow::Hidden,
            ) {
                TextInput(
                    key: remount_key,
                    handle: Some(input_handle),
                    has_focus: has_focus,
                    multiline: true,
                    color: props.text_color.unwrap_or(Color::Grey),
                    cursor_color: props.cursor_color.unwrap_or(Color::DarkGrey),
                    value: text,
                    on_change: move |new_value| {
                        apply_text_input_change(
                            suppress_enter_newline,
                            &mut value,
                            &mut input_handle,
                            cursor_snapshot,
                            new_value,
                        );
                    },
                )
            }
            #(if layout.show_scrollbar {
                Some(element! {
                    View(
                        position: Position::Absolute,
                        top: 0,
                        right: 0,
                        width: 1,
                        height: viewport,
                    ) {
                        VerticalScrollbar(
                            viewport_height: viewport,
                            content_height: layout.content_rows,
                            scroll_offset: scroll_offset.get(),
                            style: Some(scrollbar_style),
                        )
                    }
                })
            } else {
                None
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text_editing::insert_newline_at_cursor;
    use crate::text_input_layout::update_scroll_offset;

    #[test]
    fn insert_newline_at_cursor_appends() {
        let (text, next) = insert_newline_at_cursor("hi", 2);
        assert_eq!(text, "hi\n");
        assert_eq!(next, 3);
    }

    #[test]
    fn resolve_suppressed_change_keeps_first_typed_char() {
        assert_eq!(resolve_suppressed_change("a".into()), "a");
    }

    #[test]
    fn resolve_suppressed_change_drops_ghost_newlines() {
        assert_eq!(resolve_suppressed_change("\n".into()), "");
    }

    #[test]
    fn logical_line_count_includes_trailing_newline_row() {
        assert_eq!(logical_line_count("hello"), 1);
        assert_eq!(logical_line_count("hello\n"), 2);
        assert_eq!(logical_line_count("a\nb\n"), 3);
    }

    #[test]
    fn display_row_count_grows_with_newlines() {
        assert_eq!(display_row_count("one", 20), 1);
        assert_eq!(display_row_count("a\nb", 20), 2);
        assert_eq!(display_row_count("hello\n", 20), 2);
    }

    #[test]
    fn visible_row_count_omits_trailing_blank_unless_cursor_there() {
        let text = "hello\n";
        assert_eq!(visible_row_count(text, text.len(), 20), 2);
        assert_eq!(visible_row_count("line1\nline2\n", "line1\nline2".len(), 20), 2);
        assert_eq!(visible_row_count("line1\nline2\n", "line1\nline2\n".len(), 20), 3);
    }

    #[test]
    fn viewport_height_caps_at_max() {
        let layout = layout_textarea("a\nb\nc\nd\ne", 4, 20, 1, Some(3));
        assert_eq!(layout.viewport_height, 3);
        assert!(layout.show_scrollbar);
        assert_eq!(layout.content_rows, 5);
    }

    #[test]
    fn viewport_height_grows_without_max() {
        let layout = layout_textarea("a\nb\nc", 4, 20, 1, None);
        assert_eq!(layout.viewport_height, 3);
        assert!(!layout.show_scrollbar);
    }

    #[test]
    fn cursor_after_newline_insertion_advances_one_byte() {
        assert_eq!(cursor_after_newline_insertion("hi", "hi\n", 2), 3);
        assert_eq!(cursor_after_newline_insertion("hi", "h\ni", 1), 2);
    }

    #[test]
    fn update_scroll_offset_follows_cursor() {
        assert_eq!(update_scroll_offset(0, 4, 3, 8), 2);
        assert_eq!(update_scroll_offset(5, 2, 3, 8), 2);
    }

    #[test]
    fn remount_key_changes_with_viewport() {
        let a = layout_textarea("a", 0, 20, 1, None);
        let b = layout_textarea("a\nb", 2, 20, 1, None);
        assert_ne!(textarea_remount_key(&a), textarea_remount_key(&b));
    }
}
