//! Terminal UI layer: stdout/stderr formatting and interactive prompt chrome.
//!
//! Application modules (`app`, `agent`, etc.) own orchestration and data;
//! this module owns how results are rendered to the user.

pub mod auth;
pub mod credentials;
pub mod dry_run;
pub mod ingest;
pub mod schedules;
pub mod spinner;
pub mod stream;
pub mod terminal;
pub mod wizard;

pub use terminal::{
    format_stream_footer, print_agent_status, print_assistant_response, print_banner, print_chat_header,
    print_command_header, print_completion, print_tool_call, print_tool_result, truncate_path_for_display,
};
