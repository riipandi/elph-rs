//! Horizontal tab selector with content panel.

use crate::types::TabItem;
use iocraft::prelude::*;

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
        match code {
            KeyCode::Left | KeyCode::Char('h') => {
                selected.set(selected.get().saturating_sub(1));
            }
            KeyCode::Right | KeyCode::Char('l') | KeyCode::Tab => {
                selected.set((selected.get() + 1).min(len - 1));
            }
            KeyCode::BackTab => {
                selected.set(selected.get().saturating_sub(1));
            }
            _ => {}
        }
    });

    let index = if len == 0 { 0 } else { selected.get().min(len - 1) };
    let active_content = tabs.get(index).map(|t| t.content.clone()).unwrap_or_default();

    let tab_labels: Vec<_> = tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let active_tab = i == index;
            element! {
                View(
                    border_style: if active_tab { BorderStyle::Round } else { BorderStyle::None },
                    border_color: if active_tab { Color::Cyan } else { Color::DarkGrey },
                    padding_left: 1,
                    padding_right: 1,
                ) {
                    Text(
                        content: tab.label.clone(),
                        color: if active_tab { Color::Cyan } else { Color::DarkGrey },
                        weight: if active_tab { Weight::Bold } else { Weight::Normal },
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
