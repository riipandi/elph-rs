//! Free-text user answer dialog body.

use super::layout::dialog_body_section_gap;
use crate::components::Input;
use crate::components::theme::{UiTheme, resolve_ui_theme};
use iocraft::prelude::*;

/// Props for [`DialogUserInputContent`].
#[derive(Props)]
pub struct DialogUserInputContentProps {
    pub width: u16,
    pub question: String,
    pub placeholder: String,
    pub value: Option<State<String>>,
    pub has_focus: bool,
    pub theme: Option<UiTheme>,
    pub section_gap: Option<u16>,
    /// When false, the prompt is shown only in the dialog header (compact inline layout).
    pub show_prompt: bool,
    pub on_submit: HandlerMut<'static, ()>,
    pub on_cancel: HandlerMut<'static, ()>,
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
            section_gap: None,
            show_prompt: true,
            on_submit: HandlerMut::default(),
            on_cancel: HandlerMut::default(),
        }
    }
}

/// Ask-user body with a prompt line and single-line text field.
#[component]
pub fn DialogUserInputContent(
    props: &mut DialogUserInputContentProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = resolve_ui_theme(&hooks, props.theme);
    let internal = hooks.use_state(String::new);
    let value = props.value.unwrap_or(internal);
    let placeholder = if props.placeholder.is_empty() {
        "Type your answer…".to_string()
    } else {
        props.placeholder.clone()
    };

    let section_gap = props.section_gap.unwrap_or_else(|| dialog_body_section_gap(theme));
    let has_focus = props.has_focus;
    let input_border = theme.shell_zone_border_color(has_focus);

    hooks.use_terminal_events({
        let mut on_submit = props.on_submit.take();
        let mut on_cancel = props.on_cancel.take();
        move |event| {
            if !has_focus {
                return;
            }
            let TerminalEvent::Key(KeyEvent {
                code, kind, modifiers, ..
            }) = event
            else {
                return;
            };
            if kind == KeyEventKind::Release {
                return;
            }
            if !modifiers.is_empty() {
                return;
            }
            match code {
                KeyCode::Enter if !on_submit.is_default() => on_submit(()),
                KeyCode::Esc if !on_cancel.is_default() => on_cancel(()),
                _ => {}
            }
        }
    });

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
                        color: theme.text_secondary,
                        wrap: TextWrap::Wrap,
                    )
                })
            } else {
                None
            })
            View(
                width: props.width,
                border_style: BorderStyle::Single,
                border_edges: Edges::Bottom,
                border_color: input_border,
                padding_bottom: 0,
                flex_shrink: 0f32,
            ) {
                Input(
                    width: props.width,
                    value: Some(value),
                    has_focus: has_focus,
                    cursor_color: Some(theme.input_cursor_color()),
                    theme: Some(theme),
                )
            }
            Text(
                content: format!("{placeholder} · Enter submit · Esc cancel"),
                color: theme.text_muted,
                wrap: TextWrap::NoWrap,
            )
        }
    }
}
