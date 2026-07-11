//! MCP client integration via [rmcp](https://crates.io/crates/rmcp).
//!
//! # Features
//!
//! - **stdio** and **streamable HTTP** transports
//! - **Session pool** — reuse connections across tool calls (no process-per-call)
//! - **Resilient load** — one failing server does not block the rest
//! - **Agent tools** — `mcp_{server}__{tool}` naming for the model tool surface
//!
//! Enable with Cargo feature `mcp` (on by default).
//!
//! See crate docs: `docs/mcp.md`.

#[cfg(feature = "mcp")]
mod client;
#[cfg(feature = "mcp")]
mod config;
#[cfg(feature = "mcp")]
mod registry;
#[cfg(feature = "mcp")]
mod session;

#[cfg(feature = "mcp")]
pub use client::{
    McpClient, McpProbeResult, PROBE_TIMEOUT, call_stdio_tool, call_tool_for_server, connect, connect_http,
    connect_stdio, list_tools, list_tools_for_server, parse_stdio_config, probe_server, probe_stdio_server,
    shutdown_client, validate_server_config,
};
#[cfg(feature = "mcp")]
pub use config::{
    DEFAULT_OPERATION_TIMEOUT_SECS, McpConfig, McpHttpConfig, McpLoadOptions, McpServerConfig, McpStdioConfig,
};
#[cfg(feature = "mcp")]
pub use registry::{
    McpLoadReport, McpServerLoadReport, McpToolDescriptor, McpToolRegistry, expose_tool_name, mcp_result_to_agent,
    parse_exposed_tool_name,
};
#[cfg(feature = "mcp")]
pub use session::{McpServerSession, McpSessionPool};
