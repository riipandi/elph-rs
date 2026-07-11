//! Web search and fetch tools with multi-engine ranking and Obscura fallback.

mod common;
pub mod engines;
mod fetch;
pub mod ranking;
mod search;

#[cfg(feature = "obscura")]
mod obscura;

pub use fetch::create_webfetch_tool;
pub use ranking::{Engine, SearchResult};
pub use search::create_websearch_tool;

/// Web tools that do not require an [`ExecutionEnv`].
pub fn create_web_tools() -> Vec<crate::types::AgentTool> {
    vec![create_websearch_tool(), create_webfetch_tool()]
}
