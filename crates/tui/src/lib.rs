mod agent_mode;
mod paste_guard;
mod prompt_edit;
mod prompt_input;
mod prompt_keys;
mod prompt_transcript;

use iocraft::prelude::*;

pub use agent_mode::AgentMode;
pub use prompt_input::{PromptInput, PromptInputProps};
pub use prompt_keys::{
    EditAction, MacEditAction, edit_action, is_interrupt_key, is_newline_key, is_prompt_newline_key, is_quit_command,
    is_submit_key, mac_edit_action,
};
pub use prompt_transcript::{PromptTranscript, PromptTranscriptProps};

#[derive(Default, Props)]
pub struct LabelProps {
    pub content: String,
    pub color: Option<Color>,
}

#[component]
pub fn Label(props: &LabelProps) -> impl Into<AnyElement<'static>> {
    element! {
        Text(
            color: props.color,
            content: &props.content,
        )
    }
}

pub fn frame<'a>(children: Vec<AnyElement<'a>>) -> Element<'a, View> {
    element! {
        View(
            border_style: BorderStyle::Round,
            border_color: Color::Blue,
            padding_top: 2,
            padding_bottom: 2,
            padding_left: 8,
            padding_right: 8,
        ) {
            #(children.into_iter())
        }
    }
}
