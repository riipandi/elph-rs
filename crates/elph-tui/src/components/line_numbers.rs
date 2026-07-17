//! Line number gutter for code blocks.

use iocraft::prelude::*;

use super::theme::{UiTheme, resolve_ui_theme};

/// Props for [`LineNumbers`].
#[derive(Clone, Default, Props)]
pub struct LineNumbersProps {
    pub line_count: usize,
    pub start_line: usize,
    pub gutter_width: u16,
    pub color: Option<Color>,
    pub theme: Option<UiTheme>,
}

/// Right-aligned line numbers column.
#[component]
pub fn LineNumbers(props: &LineNumbersProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = resolve_ui_theme(&hooks, props.theme);
    let color = props.color.unwrap_or(theme.text_hint);
    let mut lines = Vec::new();
    for i in 0..props.line_count {
        let num = props.start_line + i + 1;
        lines.push(element! {
            Text(content: format!("{num:>width$}", width = props.gutter_width as usize - 1), color, wrap: TextWrap::NoWrap)
        });
    }

    let gutter_gap = theme.gutter_gap();
    element! {
        View(
            width: props.gutter_width,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::End,
            padding_right: gutter_gap,
        ) {
            #(lines)
        }
    }
}
