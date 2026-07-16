//! Styled text display (OpenTUI Text analogue).

use iocraft::prelude::*;

use super::theme::{UiTheme, resolve_ui_theme};

/// Props for [`StyledText`].
#[derive(Clone, Default, Props)]
pub struct StyledTextProps {
    pub content: String,
    pub color: Option<Color>,
    pub weight: Weight,
    pub wrap: TextWrap,
    pub align: TextAlign,
    pub italic: bool,
    pub theme: Option<UiTheme>,
}

/// Display styled text content.
#[component]
pub fn StyledText(props: &StyledTextProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = resolve_ui_theme(&hooks, props.theme);
    element! {
        Text(
            content: props.content.clone(),
            color: props.color.unwrap_or(theme.text_primary),
            weight: props.weight,
            wrap: props.wrap,
            align: props.align,
            italic: props.italic,
        )
    }
}
