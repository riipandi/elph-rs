//! Free-text user answer dialog body.

use super::layout::dialog_body_section_gap;
use crate::components::Input;
use crate::components::theme::{UiTheme, resolve_ui_theme};
use iocraft::prelude::*;

/// Props for [`DialogUserInputContent`].
#[derive(Clone, Props)]
pub struct DialogUserInputContentProps {
    pub width: u16,
    pub question: String,
    pub placeholder: String,
    pub value: Option<State<String>>,
    pub has_focus: bool,
    pub theme: Option<UiTheme>,
}

impl Default for DialogUserInputContentProps {
    fn default() -> Self {
        Self {
            width: 40,
            question: String::new(),
            placeholder: String::new(),
            value: None,
            has_focus: true,
            theme: None,
        }
    }
}

/// Ask-user body with a prompt line and single-line text field.
#[component]
pub fn DialogUserInputContent(props: &DialogUserInputContentProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = resolve_ui_theme(&hooks, props.theme);
    let internal = hooks.use_state(String::new);
    let value = props.value.unwrap_or(internal);
    let placeholder = if props.placeholder.is_empty() {
        "Type your answer…".to_string()
    } else {
        props.placeholder.clone()
    };

    let section_gap = dialog_body_section_gap(theme);

    element! {
        View(
            width: props.width,
            flex_direction: FlexDirection::Column,
            gap: section_gap,
            flex_shrink: 0f32,
        ) {
            Text(
                content: props.question.clone(),
                color: theme.text_secondary,
                wrap: TextWrap::Wrap,
            )
            Input(
                width: props.width,
                value: Some(value),
                has_focus: props.has_focus,
                cursor_color: Some(theme.input_cursor_color()),
                focused_border_color: Some(theme.border_focus),
                theme: Some(theme),
            )
            Text(
                content: format!("{placeholder} · Enter submit"),
                color: theme.text_muted,
                wrap: TextWrap::NoWrap,
            )
        }
    }
}
