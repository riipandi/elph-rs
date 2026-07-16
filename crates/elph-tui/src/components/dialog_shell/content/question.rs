//! Multi-option question dialog body.

use super::layout::dialog_body_section_gap;
use crate::components::SelectList;
use crate::components::select::SELECT_LIST_AUTO_HEIGHT;
use crate::components::theme::{UiTheme, resolve_ui_theme};
use crate::types::SelectOption;
use iocraft::prelude::*;

/// Props for [`DialogQuestionContent`].
#[derive(Clone, Props)]
pub struct DialogQuestionContentProps {
    pub width: u16,
    pub height: u16,
    pub question: String,
    pub options: Vec<SelectOption>,
    pub selected_index: Option<State<usize>>,
    pub has_focus: bool,
    pub show_description: bool,
    pub question_color: Color,
    pub theme: Option<UiTheme>,
}

impl Default for DialogQuestionContentProps {
    fn default() -> Self {
        let theme = UiTheme::default();
        Self {
            width: 40,
            height: SELECT_LIST_AUTO_HEIGHT,
            question: String::new(),
            options: Vec::new(),
            selected_index: None,
            has_focus: true,
            show_description: true,
            question_color: theme.text_secondary,
            theme: None,
        }
    }
}

/// Ask-user body with a prompt line and keyboard-navigable options.
#[component]
pub fn DialogQuestionContent(props: &DialogQuestionContentProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = resolve_ui_theme(&hooks, props.theme);

    element! {
        View(
            width: props.width,
            flex_direction: FlexDirection::Column,
            gap: dialog_body_section_gap(theme),
            flex_shrink: 0f32,
        ) {
            Text(
                content: props.question.clone(),
                color: props.question_color,
                wrap: TextWrap::Wrap,
            )
            SelectList(
                width: props.width,
                height: props.height,
                options: props.options.clone(),
                selected_index: props.selected_index,
                has_focus: props.has_focus,
                show_description: props.show_description,
                in_dialog: true,
                theme: Some(theme),
            )
        }
    }
}
