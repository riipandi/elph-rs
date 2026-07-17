//! Dialog overlay body presets.

use super::kinds::{DEMO_MULTI_OPTION_COUNT, OverlayKind, sample_progress, sample_todos};
use crate::common::lipsum_mock::{mock_paragraph, mock_select_options, mock_sentence, mock_title};
use elph_tui::prelude::*;

pub struct OverlayDemoBodyProps {
    pub kind: OverlayKind,
    pub width: u16,
    pub list_height: u16,
    pub selected: State<usize>,
    pub multi_cursor: State<usize>,
    pub multi_checked: State<Vec<bool>>,
    pub user_answer: State<String>,
    pub confirm_focus: State<ConfirmButtonFocus>,
    pub on_confirm_yes: HandlerMut<'static, ()>,
    pub on_confirm_no: HandlerMut<'static, ()>,
    pub on_multi_submit: HandlerMut<'static, Vec<usize>>,
}

pub fn overlay_demo_body(props: &mut OverlayDemoBodyProps) -> AnyElement<'static> {
    let theme = UiTheme::default();
    let options = mock_select_options(DEMO_MULTI_OPTION_COUNT);
    let question = mock_sentence();
    let confirm_message = format!("{}\n\n{}", mock_title(), mock_paragraph());

    match props.kind {
        OverlayKind::Mode => element! {
            DialogModeSelectContent(
                width: props.width,
                height: props.list_height,
                selected_index: props.selected,
                has_focus: true,
            )
        }
        .into(),
        OverlayKind::Question => element! {
            DialogQuestionContent(
                width: props.width,
                height: props.list_height,
                question: question.clone(),
                options: options.clone(),
                selected_index: props.selected,
                has_focus: true,
                show_description: true,
                question_color: theme.text_secondary,
            )
        }
        .into(),
        OverlayKind::MultiChoice => element! {
            DialogMultiChoiceContent(
                width: props.width,
                height: props.list_height,
                question: question.clone(),
                options: options,
                cursor_index: props.multi_cursor,
                checked: props.multi_checked,
                has_focus: true,
                show_description: true,
                theme: Some(theme),
                on_submit: props.on_multi_submit.take(),
            )
        }
        .into(),
        OverlayKind::UserInput => element! {
            DialogUserInputContent(
                width: props.width,
                question: question,
                placeholder: mock_title(),
                value: props.user_answer,
                has_focus: true,
                theme: Some(theme),
            )
        }
        .into(),
        OverlayKind::Confirm => element! {
            DialogConfirmContent(
                width: props.width,
                message: "Allow tool: Bash\n\nCommand: cargo test -p elph-tui".to_string(),
                action_hint: "y allow · n deny · Esc cancel".to_string(),
            )
        }
        .into(),
        OverlayKind::ConfirmButtons => element! {
            DialogConfirmButtonsContent(
                width: props.width,
                message: confirm_message,
                yes_label: "Yes".to_string(),
                no_label: "No".to_string(),
                focused_button: props.confirm_focus,
                has_focus: true,
                theme: Some(theme),
                on_yes: props.on_confirm_yes.take(),
                on_no: props.on_confirm_no.take(),
            )
        }
        .into(),
        OverlayKind::TodoList => element! {
            DialogTodoListContent(width: props.width, items: sample_todos())
        }
        .into(),
        OverlayKind::TodoProgress => element! {
            DialogTodoProgressContent(width: props.width, items: sample_progress())
        }
        .into(),
    }
}
