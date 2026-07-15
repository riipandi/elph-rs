//! Horizontal tab selector with content panel.

use crate::types::TabItem;
use iocraft::prelude::*;

/// Tab chrome for one tab label.
pub fn tab_select_tab_styles(active: bool) -> (BorderStyle, Color, Weight) {
    if active {
        (BorderStyle::Round, Color::Cyan, Weight::Bold)
    } else {
        (BorderStyle::None, Color::DarkGrey, Weight::Normal)
    }
}

/// Active tab index clamped to list bounds.
pub fn tab_select_clamped_index(selected: usize, len: usize) -> usize {
    if len == 0 { 0 } else { selected.min(len - 1) }
}

/// Next tab index after a key press.
pub fn tab_select_key_to_index(current: usize, len: usize, code: KeyCode) -> usize {
    if len == 0 {
        return 0;
    }
    match code {
        KeyCode::Left | KeyCode::Char('h') => current.saturating_sub(1),
        KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => (current + 1).min(len - 1),
        KeyCode::BackTab => current.saturating_sub(1),
        _ => current,
    }
}

/// Props for [`TabSelect`].
#[derive(Clone, Default, Props)]
pub struct TabSelectProps {
    pub width: u16,
    pub tabs: Vec<TabItem>,
    pub selected_index: Option<State<usize>>,
    pub has_focus: bool,
}

/// Tab row with a single active content panel below.
#[component]
pub fn TabSelect(props: &TabSelectProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let internal = hooks.use_state(|| 0usize);
    let mut selected = props.selected_index.unwrap_or(internal);
    let has_focus = props.has_focus;
    let tabs = props.tabs.clone();
    let len = tabs.len();

    hooks.use_terminal_events(move |event| {
        if !has_focus || len == 0 {
            return;
        }
        let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event else {
            return;
        };
        if kind == KeyEventKind::Release {
            return;
        }
        selected.set(tab_select_key_to_index(selected.get(), len, code));
    });

    let index = tab_select_clamped_index(selected.get(), len);
    let active_content = tabs.get(index).map(|t| t.content.clone()).unwrap_or_default();

    let tab_labels: Vec<_> = tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let active_tab = i == index;
            let (border_style, color, weight) = tab_select_tab_styles(active_tab);
            element! {
                View(
                    border_style: border_style,
                    border_color: color,
                    padding_left: 1,
                    padding_right: 1,
                ) {
                    Text(
                        content: tab.label.clone(),
                        color: color,
                        weight: weight,
                        wrap: TextWrap::NoWrap,
                    )
                }
            }
        })
        .collect();

    element! {
        View(width: props.width, flex_direction: FlexDirection::Column, gap: 1) {
            View(flex_direction: FlexDirection::Row, gap: 1, flex_wrap: FlexWrap::Wrap) {
                #(tab_labels)
            }
            View(
                width: props.width,
                border_style: BorderStyle::Single,
                border_color: Color::DarkGrey,
                padding: 1,
                min_height: 3,
            ) {
                Text(content: active_content, color: Color::Grey)
            }
        }
    }
}
