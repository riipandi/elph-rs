//! Confirmation dialog body.

use super::layout::dialog_body_section_gap;
use crate::components::theme::{UiTheme, resolve_ui_theme};
use iocraft::prelude::*;

/// Props for [`DialogConfirmContent`].
#[derive(Clone, Props)]
pub struct DialogConfirmContentProps {
    pub width: u16,
    pub message: String,
    pub action_hint: String,
    pub message_color: Color,
    pub hint_color: Color,
    pub theme: Option<UiTheme>,
}

impl Default for DialogConfirmContentProps {
    fn default() -> Self {
        let theme = UiTheme::default();
        Self {
            width: 40,
            message: String::new(),
            action_hint: String::new(),
            message_color: theme.text_secondary,
            hint_color: theme.text_muted,
            theme: None,
        }
    }
}

/// Yes/no confirmation body with a discoverable action line.
#[component]
pub fn DialogConfirmContent(props: &DialogConfirmContentProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = resolve_ui_theme(&hooks, props.theme);
    let hint = if props.action_hint.is_empty() {
        "y/Enter yes · n/Esc no".to_string()
    } else {
        props.action_hint.clone()
    };

    element! {
        View(
            width: props.width,
            flex_direction: FlexDirection::Column,
            gap: dialog_body_section_gap(theme),
            flex_shrink: 0f32,
        ) {
            Text(
                content: props.message.clone(),
                color: props.message_color,
                wrap: TextWrap::Wrap,
            )
            Text(
                content: hint,
                color: props.hint_color,
                wrap: TextWrap::NoWrap,
            )
        }
    }
}
