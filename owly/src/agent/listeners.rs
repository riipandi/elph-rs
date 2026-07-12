//! Checkpoint persistence listeners (data layer).

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use elph_agent::AgentEvent;
use elph_ai::AssistantMessageEvent;

use crate::runtime::session::{TurnWriteContext, is_ask_tool};
use crate::ui::terminal::print_warning;

use super::tools::{summarize_tool_args, summarize_tool_result};

pub(super) fn create_checkpoint_write_subscriber(write_ctx: TurnWriteContext) -> elph_agent::AgentListener {
    let tool_args = Arc::new(Mutex::new(HashMap::<String, String>::new()));
    Arc::new(move |event, _token| {
        let write_ctx = write_ctx.clone();
        let tool_args = tool_args.clone();
        Box::pin(async move {
            match event {
                AgentEvent::MessageUpdate {
                    assistant_message_event,
                    ..
                } => {
                    if let AssistantMessageEvent::TextDelta { delta, .. } = &*assistant_message_event
                        && let Err(err) = write_ctx.record_assistant_delta(delta).await
                    {
                        tracing::warn!(error = %err, "failed to persist assistant draft");
                        print_warning(format!("Warning: checkpoint draft write failed: {err:#}"));
                    }
                }
                AgentEvent::ToolExecutionStart {
                    tool_call_id,
                    tool_name,
                    args,
                    ..
                } => {
                    let args_summary = summarize_tool_args(&tool_name, &args);
                    tool_args
                        .lock()
                        .await
                        .insert(tool_call_id.clone(), args_summary.clone());
                    if is_ask_tool(&tool_name)
                        && let Err(err) = write_ctx
                            .record_interrupt(&tool_call_id, &tool_name, &args_summary)
                            .await
                    {
                        tracing::warn!(error = %err, tool = %tool_name, "failed to persist interrupt");
                        print_warning(format!("Warning: checkpoint interrupt write failed ({tool_name}): {err:#}"));
                    }
                }
                AgentEvent::ToolExecutionUpdate {
                    tool_call_id,
                    tool_name,
                    partial_result,
                    ..
                } => {
                    let args_summary = tool_args.lock().await.get(&tool_call_id).cloned().unwrap_or_default();
                    let output = summarize_tool_result(&partial_result);
                    if let Err(err) = write_ctx
                        .record_tool_partial(&tool_call_id, &tool_name, &args_summary, &output)
                        .await
                    {
                        tracing::warn!(error = %err, tool = %tool_name, "failed to persist tool partial");
                        print_warning(format!("Warning: checkpoint tool partial write failed ({tool_name}): {err:#}"));
                    }
                }
                AgentEvent::ToolExecutionEnd {
                    tool_call_id,
                    tool_name,
                    is_error,
                    result,
                    ..
                } => {
                    let args_summary = tool_args.lock().await.remove(&tool_call_id).unwrap_or_default();
                    let output = summarize_tool_result(&result);
                    if is_ask_tool(&tool_name)
                        && let Err(err) = write_ctx
                            .record_resume(&tool_call_id, &tool_name, &output, is_error)
                            .await
                    {
                        tracing::warn!(error = %err, tool = %tool_name, "failed to persist resume");
                        print_warning(format!("Warning: checkpoint resume write failed ({tool_name}): {err:#}"));
                    }
                    if let Err(err) = write_ctx
                        .record_tool_result(&tool_call_id, &tool_name, &args_summary, is_error, &output)
                        .await
                    {
                        tracing::warn!(error = %err, tool = %tool_name, "failed to persist tool write");
                        print_warning(format!("Warning: checkpoint tool write failed ({tool_name}): {err:#}"));
                    }
                }
                _ => {}
            }
        })
    })
}
