//! Single-line text input (OpenTUI Input analogue).

use crate::text_editing::wire_input_shortcuts;
use iocraft::prelude::*;

use super::theme::{UiTheme, resolve_ui_theme};

/// Props for [`Input`].
#[derive(Default, Props)]
pub struct InputProps {
    pub width: u16,
    pub initial_value: String,
    pub has_focus: bool,
    pub text_color: Option<Color>,
    pub cursor_color: Option<Color>,
    pub focused_border_color: Option<Color>,
    pub value: Option<State<String>>,
    pub theme: Option<UiTheme>,
    pub on_change: HandlerMut<'static, String>,
}

/// Single-line text input with optional external state.
#[component]
pub fn Input(props: &mut InputProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let external = props.value;
    let mut internal = hooks.use_state(|| {
        external
            .map(|state| state.read().clone())
            .unwrap_or_else(|| props.initial_value.clone())
    });

    if let Some(parent) = external {
        let parent_text = parent.read().clone();
        if parent_text != internal.read().clone() {
            internal.set(parent_text);
        }
    }

    let mut value = internal;
    let has_focus = props.has_focus;
    let input_handle = hooks.use_ref_default::<TextInputHandle>();
    let theme = resolve_ui_theme(&hooks, props.theme);
    let inset = theme.input_inset();
    let mut on_change = props.on_change.take();

    wire_input_shortcuts(&mut hooks, has_focus, value, input_handle);

    let display = value.read().clone();
    element! {
        View(
            width: props.width,
            border_style: theme.focus_border(has_focus),
            border_color: props.focused_border_color.unwrap_or_else(|| theme.input_border_color(has_focus)),
            padding_left: inset,
            padding_right: inset,
        ) {
            TextInput(
                handle: Some(input_handle),
                has_focus: has_focus,
                multiline: false,
                color: props.text_color.unwrap_or_else(|| theme.input_text_color(has_focus)),
                cursor_color: props.cursor_color.unwrap_or_else(|| theme.input_cursor_color()),
                value: display,
                on_change: move |new_value: String| {
                    let prev = value.read().clone();
                    value.set(new_value.clone());
                    if let Some(mut parent) = external {
                        parent.set(new_value.clone());
                    }
                    if new_value != prev && !on_change.is_default() {
                        on_change(new_value);
                    }
                },
            )
        }
    }
}
