//! Vertical list selector (OpenTUI Select analogue).

use crate::types::SelectOption;
use iocraft::prelude::*;

/// Props for [`SelectList`].
#[derive(Clone, Default, Props)]
pub struct SelectListProps {
    pub width: u16,
    pub height: u16,
    pub options: Vec<SelectOption>,
    pub selected_index: Option<State<usize>>,
    pub has_focus: bool,
    pub show_description: bool,
    pub fast_scroll_step: usize,
}

/// Label text for one [`SelectList`] row.
pub fn select_option_line(prefix: &str, name: &str, description: &str, show_description: bool) -> String {
    let desc = if show_description && !description.is_empty() {
        format!("\n   {description}")
    } else {
        String::new()
    };
    format!("{prefix}{name}{desc}")
}

/// Row prefix for a [`SelectList`] option.
pub fn select_row_prefix(selected: bool) -> &'static str {
    if selected { "› " } else { "  " }
}

/// Row colors for a [`SelectList`] option.
pub fn select_row_colors(selected: bool) -> (Color, Weight) {
    if selected {
        (Color::Yellow, Weight::Bold)
    } else {
        (Color::Grey, Weight::Normal)
    }
}

/// Clamped selected index for a non-empty option list.
pub fn select_clamped_index(selected: usize, len: usize) -> usize {
    if len == 0 { 0 } else { selected.min(len - 1) }
}

/// Next selection index after a key press.
pub fn select_key_to_index(current: usize, len: usize, code: KeyCode, modifiers: KeyModifiers, step: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let fast = modifiers.contains(KeyModifiers::SHIFT);
    let Some(delta) = select_key_delta(code, fast, step) else {
        return current;
    };
    if delta < 0 {
        current.saturating_sub(delta.unsigned_abs())
    } else {
        (current + delta as usize).min(len - 1)
    }
}

/// Selection change from a key press (`None` = no change).
pub fn select_key_delta(code: KeyCode, fast: bool, step: usize) -> Option<isize> {
    let delta = if fast { step as isize } else { 1 };
    match code {
        KeyCode::Up | KeyCode::Char('k') => Some(-delta),
        KeyCode::Down | KeyCode::Char('j') => Some(delta),
        _ => None,
    }
}

/// First visible row index for a centered selection window.
pub fn select_window_start(selected: usize, height: usize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    selected.saturating_sub(height / 2).min(len.saturating_sub(1))
}

/// Keyboard-navigable option list.
#[component]
pub fn SelectList(props: &SelectListProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let internal_index = hooks.use_state(|| 0usize);
    let mut selected = props.selected_index.unwrap_or(internal_index);
    let step = props.fast_scroll_step.max(1);
    let has_focus = props.has_focus;
    let options = props.options.clone();
    let show_description = props.show_description;
    let height = props.height.max(1) as usize;

    let option_count = options.len();
    hooks.use_terminal_events(move |event| {
        if !has_focus {
            return;
        }
        let TerminalEvent::Key(KeyEvent {
            code, kind, modifiers, ..
        }) = event
        else {
            return;
        };
        if kind == KeyEventKind::Release {
            return;
        }

        let len = option_count;
        if len == 0 {
            return;
        }

        selected.set(select_key_to_index(selected.get(), len, code, modifiers, step));
    });

    let len = options.len();
    let index = select_clamped_index(selected.get(), len);
    let window_start = select_window_start(index, height, len);

    let mut rows: Vec<AnyElement<'static>> = Vec::new();
    if len == 0 {
        rows.push(element! { Text(content: "(no options)", color: Color::DarkGrey) }.into());
    } else {
        for (i, opt) in options.iter().enumerate().skip(window_start).take(height) {
            let selected_row = i == index;
            let prefix = select_row_prefix(selected_row);
            let (name_color, weight) = select_row_colors(selected_row);
            let line = select_option_line(prefix, &opt.name, &opt.description, show_description);
            rows.push(
                element! {
                    Text(
                        content: line,
                        color: name_color,
                        weight: weight,
                        wrap: TextWrap::NoWrap,
                    )
                }
                .into(),
            );
        }
    }

    element! {
        View(
            width: props.width,
            height: props.height,
            flex_direction: FlexDirection::Column,
            border_style: if has_focus { BorderStyle::Single } else { BorderStyle::None },
            border_color: Color::DarkGrey,
            padding_left: 1,
            padding_right: 1,
        ) {
            #(rows)
        }
    }
}
