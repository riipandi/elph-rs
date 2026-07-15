//! Multiline text input.

use iocraft::prelude::*;

/// Props for [`Textarea`].
#[derive(Clone, Default, Props)]
pub struct TextareaProps {
    pub width: u16,
    pub min_height: u16,
    pub initial_value: String,
    pub has_focus: bool,
    pub text_color: Option<Color>,
    pub cursor_color: Option<Color>,
    pub value: Option<State<String>>,
}

/// Multiline text input with optional external state.
#[component]
pub fn Textarea(props: &TextareaProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let internal = hooks.use_state(|| props.initial_value.clone());
    let mut value = props.value.unwrap_or(internal);

    element! {
        View(
            width: props.width,
            min_height: props.min_height.max(3),
            border_style: if props.has_focus { BorderStyle::Round } else { BorderStyle::Single },
            border_color: Color::DarkGrey,
            padding_left: 1,
            padding_right: 1,
        ) {
            TextInput(
                has_focus: props.has_focus,
                multiline: true,
                color: props.text_color.unwrap_or(Color::Grey),
                cursor_color: props.cursor_color.unwrap_or(Color::DarkGrey),
                value: value.to_string(),
                on_change: move |new_value| value.set(new_value),
            )
        }
    }
}
