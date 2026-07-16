//! Inline ask-user dialog above the status row.

use elph_tui::components::{
    ConfirmButtonFocus, DIALOG_SELECT_AUTO_HEIGHT, DialogConfirmButtonsContent, DialogMultiChoiceContent,
    DialogQuestionContent, DialogUserInputContent, UiTheme, dialog_body_min_height, dialog_header_title_fit,
    dialog_max_content_height, dialog_select_body_plan,
};
use iocraft::prelude::*;

use crate::agent::UserQuestionOption;
use crate::tui::inline_dialog::{INLINE_SECTION_GAP, InlineDialogShell, inline_body_width};
use crate::tui::user_question::{PendingUserQuestion, QuestionInputFocus, user_question_select_options};

/// Snapshot for rendering the active ask-user step.
#[derive(Debug, Clone, Default)]
pub struct UserQuestionView {
    pub step_index: usize,
    pub step_count: usize,
    pub question: String,
    pub is_confirm: bool,
    pub is_multi_select: bool,
    pub allow_custom: bool,
    pub custom_label: String,
    pub options: Option<Vec<UserQuestionOption>>,
}

impl UserQuestionView {
    pub fn from_pending(pending: &PendingUserQuestion) -> Self {
        Self {
            step_index: pending.step_index(),
            step_count: pending.step_count(),
            question: pending.question().to_string(),
            is_confirm: pending.is_confirm(),
            is_multi_select: pending.is_multi_select(),
            allow_custom: pending.allow_custom(),
            custom_label: pending.custom_label().to_string(),
            options: pending.options().map(|options| options.to_vec()),
        }
    }
}

/// Props for [`UserQuestionBar`].
#[derive(Props)]
pub struct UserQuestionBarProps {
    pub screen_width: u16,
    pub screen_height: u16,
    pub view: UserQuestionView,
    pub selected_index: Option<State<usize>>,
    pub multi_checked: Option<State<Vec<bool>>>,
    pub confirm_focus: Option<State<ConfirmButtonFocus>>,
    pub answer: Option<State<String>>,
    pub input_focus: QuestionInputFocus,
    pub has_focus: bool,
    pub on_confirm_yes: HandlerMut<'static, ()>,
    pub on_confirm_no: HandlerMut<'static, ()>,
    pub on_text_submit: HandlerMut<'static, ()>,
    pub on_text_cancel: HandlerMut<'static, ()>,
}

impl Default for UserQuestionBarProps {
    fn default() -> Self {
        Self {
            screen_width: 80,
            screen_height: 24,
            view: UserQuestionView::default(),
            selected_index: None,
            multi_checked: None,
            confirm_focus: None,
            answer: None,
            input_focus: QuestionInputFocus::Choices,
            has_focus: false,
            on_confirm_yes: HandlerMut::default(),
            on_confirm_no: HandlerMut::default(),
            on_text_submit: HandlerMut::default(),
            on_text_cancel: HandlerMut::default(),
        }
    }
}

fn question_title(view: &UserQuestionView, body_width: u16) -> String {
    if view.is_confirm {
        if view.step_count > 1 {
            return format!("Confirm ({}/{})", view.step_index + 1, view.step_count);
        }
        return "Confirm".to_string();
    }
    if view.step_count > 1 {
        let prompt = view.question.lines().next().unwrap_or(&view.question);
        let fit = dialog_header_title_fit(prompt, body_width.saturating_sub(8), "");
        return format!("{}/{} · {fit}", view.step_index + 1, view.step_count);
    }
    let prompt = view.question.lines().next().unwrap_or(&view.question);
    dialog_header_title_fit(prompt, body_width, "")
}

fn select_intro(view: &UserQuestionView) -> &str {
    if view.is_multi_select || view.question.contains('\n') {
        view.question.as_str()
    } else {
        ""
    }
}

fn select_trailing_rows(view: &UserQuestionView) -> u16 {
    let mut trailing = 0u16;
    if view.is_multi_select {
        trailing = trailing.saturating_add(1);
    }
    if view.allow_custom {
        trailing = trailing.saturating_add(3);
    }
    trailing
}

fn question_list_height(screen_width: u16, screen_height: u16, view: &UserQuestionView, body_width: u16) -> u16 {
    let theme = UiTheme::default();
    let chrome = elph_tui::components::DialogChrome::from_theme(theme, screen_width);
    let max_body = dialog_max_content_height(screen_height, &chrome, 16);

    if view.is_confirm {
        let rows = view.question.lines().count() as u16;
        let _ = dialog_body_min_height(7u16.saturating_add(rows).min(max_body));
        return DIALOG_SELECT_AUTO_HEIGHT;
    }

    if let Some(options) = view.options.as_ref() {
        let select_options = user_question_select_options(options);
        let (_, list_h) = dialog_select_body_plan(
            &select_options,
            true,
            body_width,
            theme,
            select_intro(view),
            select_trailing_rows(view),
            Some(max_body),
            true,
        );
        return list_h;
    }

    DIALOG_SELECT_AUTO_HEIGHT
}

fn render_custom_input(
    props: &mut UserQuestionBarProps,
    body_width: u16,
    theme: UiTheme,
    custom_focused: bool,
) -> AnyElement<'static> {
    element! {
        DialogUserInputContent(
            width: body_width,
            question: String::new(),
            placeholder: props.view.custom_label.clone(),
            value: props.answer,
            has_focus: props.has_focus && custom_focused,
            theme: Some(theme),
            section_gap: Some(0),
            show_prompt: false,
            on_submit: props.on_text_submit.take(),
            on_cancel: props.on_text_cancel.take(),
        )
    }
    .into()
}

fn render_question_body(props: &mut UserQuestionBarProps, body_width: u16, list_height: u16) -> AnyElement<'static> {
    let theme = UiTheme::default();
    let has_focus = props.has_focus;
    let choices_focused = has_focus && props.input_focus.is_choices();
    let custom_focused = has_focus && props.input_focus.is_custom();
    let view = &props.view;

    if view.is_confirm {
        return element! {
            DialogConfirmButtonsContent(
                width: body_width,
                message: view.question.clone(),
                yes_label: "Yes".to_string(),
                no_label: "No".to_string(),
                focused_button: props.confirm_focus,
                has_focus: choices_focused,
                theme: Some(theme),
                section_gap: Some(INLINE_SECTION_GAP),
                on_yes: props.on_confirm_yes.take(),
                on_no: props.on_confirm_no.take(),
            )
        }
        .into();
    }

    if view.is_multi_select {
        let select_options = user_question_select_options(view.options.as_ref().unwrap_or(&vec![]));
        let multiline_prompt = view.question.contains('\n');
        return element! {
            View(
                width: body_width,
                flex_direction: FlexDirection::Column,
                gap: 0,
                flex_shrink: 0f32,
            ) {
                #(if multiline_prompt {
                    Some(element! {
                        View(width: body_width, padding_bottom: 1, flex_shrink: 0f32) {
                            Text(
                                content: view.question.clone(),
                                color: theme.text_secondary,
                                wrap: TextWrap::Wrap,
                            )
                        }
                    })
                } else {
                    None
                })
                DialogMultiChoiceContent(
                    width: body_width,
                    height: list_height,
                    question: if multiline_prompt { String::new() } else { view.question.clone() },
                    options: select_options,
                    cursor_index: props.selected_index,
                    checked: props.multi_checked,
                    has_focus: choices_focused,
                    show_description: true,
                    theme: Some(theme),
                    on_submit: HandlerMut::default(),
                )
                #(if view.allow_custom {
                    Some(render_custom_input(props, body_width, theme, custom_focused))
                } else {
                    None
                })
            }
        }
        .into();
    }

    if let Some(options) = view.options.as_ref() {
        let select_options = user_question_select_options(options);
        let multiline_prompt = view.question.contains('\n');
        return element! {
            View(
                width: body_width,
                flex_direction: FlexDirection::Column,
                gap: 0,
                flex_shrink: 0f32,
            ) {
                DialogQuestionContent(
                    width: body_width,
                    height: list_height,
                    question: view.question.clone(),
                    options: select_options,
                    selected_index: props.selected_index,
                    has_focus: choices_focused,
                    show_description: true,
                    question_color: theme.text_secondary,
                    theme: Some(theme),
                    section_gap: Some(0),
                    compact: true,
                    show_prompt: multiline_prompt,
                )
                #(if view.allow_custom {
                    Some(render_custom_input(props, body_width, theme, custom_focused))
                } else {
                    None
                })
            }
        }
        .into();
    }

    let multiline_prompt = view.question.contains('\n');
    element! {
        DialogUserInputContent(
            width: body_width,
            question: view.question.clone(),
            placeholder: "Type your answer…".to_string(),
            value: props.answer,
            has_focus: has_focus,
            theme: Some(theme),
            section_gap: Some(1),
            show_prompt: multiline_prompt,
            on_submit: props.on_text_submit.take(),
            on_cancel: props.on_text_cancel.take(),
        )
    }
    .into()
}

#[component]
pub fn UserQuestionBar(props: &mut UserQuestionBarProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let _ = hooks;
    let body_width = inline_body_width(props.screen_width);
    let list_height = question_list_height(props.screen_width, props.screen_height, &props.view, body_width);
    let title = question_title(&props.view, body_width);
    let body = render_question_body(props, body_width, list_height);

    element! {
        InlineDialogShell(
            screen_width: props.screen_width,
            title: title,
            has_focus: props.has_focus,
        ) {
            #(body)
        }
    }
}
