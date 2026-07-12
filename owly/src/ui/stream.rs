//! Live agent event stream rendering (stdout/stderr during LLM runs).

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use elph_agent::AgentEvent;
use elph_ai::AssistantMessageEvent;
use indicatif::ProgressBar;

use crate::runtime::env;

use super::terminal::{print_tool_call, print_tool_result};

pub fn create_event_subscriber(
    stream: bool,
    verbose: bool,
    generating: ProgressBar,
    saw_any_delta: Arc<AtomicBool>,
    stream_ends_with_newline: Arc<AtomicBool>,
) -> elph_agent::AgentListener {
    let verbose_clone = verbose;
    let stream_clone = stream;
    Arc::new(move |event, _token| {
        let generating = generating.clone();
        let saw_any_delta = saw_any_delta.clone();
        let stream_ends_with_newline = stream_ends_with_newline.clone();
        let verbose = verbose_clone;
        let stream = stream_clone;
        Box::pin(async move {
            match event {
                AgentEvent::MessageUpdate {
                    assistant_message_event,
                    ..
                } => match &*assistant_message_event {
                    AssistantMessageEvent::TextDelta { delta, .. } => {
                        if !saw_any_delta.swap(true, Ordering::SeqCst) {
                            generating.finish_and_clear();
                        }
                        if stream {
                            stream_ends_with_newline.store(delta.ends_with('\n'), Ordering::SeqCst);
                            print!("{delta}");
                            let _ = std::io::stdout().flush();
                        }
                    }
                    AssistantMessageEvent::ThinkingDelta { delta, .. } => {
                        if !saw_any_delta.swap(true, Ordering::SeqCst) {
                            generating.finish_and_clear();
                        }
                        if verbose {
                            eprint!("\x1b[2m{delta}\x1b[0m");
                            let _ = std::io::stderr().flush();
                        }
                    }
                    _ => {}
                },
                AgentEvent::ToolExecutionStart { tool_name, .. } => {
                    if !saw_any_delta.load(Ordering::SeqCst) {
                        generating.finish_and_clear();
                    }
                    env::debug_log(format!("tool start: {tool_name}"));
                    print_tool_call(&tool_name, verbose);
                }
                AgentEvent::ToolExecutionEnd {
                    tool_name, is_error, ..
                } => {
                    env::debug_log(format!("tool end: {tool_name} error={is_error}"));
                    print_tool_result(&tool_name, !is_error, verbose);
                }
                AgentEvent::AgentEnd { .. } if !saw_any_delta.load(Ordering::SeqCst) => {
                    generating.finish_and_clear();
                }
                _ => {}
            }
        })
    })
}
