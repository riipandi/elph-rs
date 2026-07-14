//! Built-in agent tools and tool registry helpers.

mod common;
pub mod types;

#[cfg(feature = "tools-bash")]
mod bash;
#[cfg(feature = "tools-edit")]
mod edit;
#[cfg(any(feature = "tools-grep", feature = "tools-find", feature = "tools-ls"))]
mod fff_picker;
#[cfg(feature = "tools-find")]
mod find;
#[cfg(feature = "tools-grep")]
mod grep;
#[cfg(feature = "tools-ls")]
mod ls;
#[cfg(feature = "tools-multi-agent")]
mod multi_agent;
#[cfg(feature = "tools-read")]
mod read;
#[cfg(feature = "tools-web")]
pub mod web;
#[cfg(feature = "tools-write")]
mod write;

#[cfg(feature = "mcp")]
pub mod mcp;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use elph_ai::Tool;
use serde_json::Value;

use crate::runtime::local_env::LocalExecutionEnv;
use crate::types::{AgentTool, AgentToolResult, ToolExecuteFn};

#[cfg(feature = "tools-bash")]
pub use bash::create_bash_tool;
#[cfg(feature = "tools-edit")]
pub use edit::create_edit_tool;
#[cfg(feature = "tools-find")]
pub use find::create_find_tool;
#[cfg(feature = "tools-grep")]
pub use grep::create_grep_tool;
#[cfg(feature = "tools-ls")]
pub use ls::create_ls_tool;
#[cfg(feature = "tools-multi-agent")]
pub use multi_agent::create_multi_agent_tools;
#[cfg(feature = "tools-read")]
pub use read::create_read_tool;
#[cfg(feature = "tools-web")]
pub use web::{
    Engine as WebSearchEngine, SearchResult as WebSearchResult, create_web_tools, create_webfetch_tool,
    create_websearch_tool,
};
#[cfg(feature = "tools-write")]
pub use write::create_write_tool;

pub fn simple_tool(
    tool: Tool,
    label: impl Into<String>,
    execute: impl Fn(String, Value) -> Pin<Box<dyn Future<Output = anyhow::Result<AgentToolResult>> + Send>>
    + Send
    + Sync
    + 'static,
) -> AgentTool {
    let execute_fn: ToolExecuteFn = Arc::new(move |id, args, _signal, _on_update| execute(id, args));
    AgentTool {
        tool,
        label: label.into(),
        execution_mode: None,
        prepare_arguments: None,
        execute: execute_fn,
    }
}

pub fn echo_tool() -> AgentTool {
    simple_tool(
        Tool {
            name: "echo".into(),
            description: "Echo text".into(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": { "text": { "type": "string" } },
                "required": ["text"]
            }),
        },
        "Echo",
        |_, args| {
            let text = args.get("text").and_then(|v| v.as_str()).unwrap_or("").to_string();
            Box::pin(async move { Ok(AgentToolResult::text(text)) })
        },
    )
}

/// Core filesystem and shell tools: read, bash, edit, write.
#[cfg(feature = "tools-core")]
pub fn create_core_tools(env: Arc<LocalExecutionEnv>) -> Vec<AgentTool> {
    let tools = vec![
        #[cfg(feature = "tools-read")]
        create_read_tool(env.clone()),
        #[cfg(feature = "tools-bash")]
        create_bash_tool(env.clone()),
        #[cfg(feature = "tools-edit")]
        create_edit_tool(env.clone()),
        #[cfg(feature = "tools-write")]
        create_write_tool(env),
    ];
    tools
}

/// Read-only exploration tools.
#[cfg(feature = "tools-explore")]
pub fn create_read_only_tools(env: Arc<LocalExecutionEnv>) -> Vec<AgentTool> {
    let tools = vec![
        #[cfg(feature = "tools-read")]
        create_read_tool(env.clone()),
        #[cfg(feature = "tools-grep")]
        create_grep_tool(env.clone()),
        #[cfg(feature = "tools-find")]
        create_find_tool(env.clone()),
        #[cfg(feature = "tools-ls")]
        create_ls_tool(env),
    ];
    tools
}

/// All enabled filesystem built-in tools.
#[cfg(any(feature = "tools-core", feature = "tools-explore"))]
pub fn create_all_tools(env: Arc<LocalExecutionEnv>) -> Vec<AgentTool> {
    crate::builder::BuiltinToolsBuilder::new(env).without_web().build()
}

/// All enabled built-in tools including web tools when compiled in.
#[cfg(any(feature = "tools-core", feature = "tools-explore", feature = "tools-web"))]
pub fn create_all_tools_with_web(env: Arc<LocalExecutionEnv>) -> Vec<AgentTool> {
    crate::builder::BuiltinToolsBuilder::all(env).build()
}
