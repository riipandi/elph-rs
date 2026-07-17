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
    pub section_gap: Option<u16>,
    pub compact: bool,
    /// When false, the prompt is shown only in the dialog header (compact inline layout).
    pub show_prompt: bool,
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
            section_gap: None,
            compact: false,
            show_prompt: true,
        }
    }
}

/// Ask-user body with a prompt line and keyboard-navigable options.
#[component]
pub fn DialogQuestionContent(props: &DialogQuestionContentProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = resolve_ui_theme(&hooks, props.theme);
    let section_gap = props.section_gap.unwrap_or_else(|| dialog_body_section_gap(theme));

    element! {
        View(
            width: props.width,
            flex_direction: FlexDirection::Column,
            gap: section_gap,
            flex_shrink: 0f32,
        ) {
            #(if props.show_prompt {
                Some(element! {
                    Text(
                        content: props.question.clone(),
                        color: props.question_color,
                        wrap: TextWrap::Wrap,
                    )
                })
            } else {
                None
            })
            SelectList(
                width: props.width,
                height: props.height,
                options: props.options.clone(),
                selected_index: props.selected_index,
                has_focus: props.has_focus,
                show_description: props.show_description,
                compact: props.compact,
                theme: Some(theme),
            )
        }
    }
}
