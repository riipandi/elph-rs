//! Core agent types — elph-agent module.
//!
//! Global enums live here; domain-specific types have been distributed:
//! - loop config types → `crate::runtime::loop_config`
//! - message types → `crate::messages::types`
//! - tool types → `crate::tools::types`

pub mod enums;

pub use enums::{AgentThinkingLevel, QueueMode, ToolExecutionMode};

// Re-export from domain modules for backward compatibility.
pub use crate::messages::types::{
    AgentMessage, CustomAgentMessage, assistant_message_to_agent, extract_tool_calls, llm_message_to_agent,
    tool_result_to_agent,
};
pub use crate::runtime::loop_config::{
    AfterToolCallContext, AfterToolCallFn, AfterToolCallResult, AgentContext, AgentEvent, AgentLoopConfig,
    AgentLoopTurnUpdate, AgentState, BeforeToolCallContext, BeforeToolCallFn, BeforeToolCallResult, ConvertToLlmFn,
    GetApiKeyFn, GetQueuedMessagesFn, PrepareNextTurnContext, PrepareNextTurnFn, PrepareNextTurnLegacyFn,
    ShouldStopAfterTurnContext, ShouldStopAfterTurnFn, StreamFn, TransformContextFn,
};
pub use crate::tools::types::{
    AgentTool, AgentToolCall, AgentToolResult, ToolExecuteFn, ToolResultContent, ToolUpdateCallback,
};
