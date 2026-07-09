mod agent_mode;
mod chat_stream;
mod editing;
mod prompt_keys;
mod transcript_scroll;
mod widget;

pub use agent_mode::AgentMode;
pub use chat_stream::{
    ChatStreamState, DEFAULT_LINE_SCROLL_STEP, PAGE_SCROLL_VIEWPORT, render_chat_stream, render_chat_stream_with_agent,
};
pub use prompt_keys::{is_quit_command, should_cycle_agent_mode};
pub use transcript_scroll::{
    ScrollSnapshot, apply_transcript_auto_scroll, handle_transcript_scroll_keys, is_pinned_to_bottom,
    prepare_transcript_follow, scroll_to_bottom,
};
pub use widget::{PromptAction, PromptOpts, PromptState, handle_prompt_input, render_prompt, text_with_theme};
