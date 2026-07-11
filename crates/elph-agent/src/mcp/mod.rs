//! MCP client integration via [rmcp](https://crates.io/crates/rmcp).
//!
//! # Features
//!
//! - **stdio**, **streamable HTTP**, and legacy **SSE** transports
//! - **OAuth 2.1** (PKCE) for remote servers via `mcp auth`
//! - **Resources** and **prompts** bridge tools
//! - **Tool policy** (allow / deny / requireApproval)
//! - **Session pool** — reuse connections across tool calls
//! - **Hot reload** on `tools/list_changed` (and resource/prompt variants)
//! - **Agent tools** — `mcp_{server}__{tool}` naming for the model tool surface
//!
//! Enable with Cargo feature `mcp` (on by default).

#[cfg(feature = "mcp")]
mod auth;
#[cfg(feature = "mcp")]
mod client;
#[cfg(feature = "mcp")]
mod config;
#[cfg(feature = "mcp")]
mod events;
#[cfg(feature = "mcp")]
mod policy;
#[cfg(feature = "mcp")]
mod registry;
#[cfg(feature = "mcp")]
mod session;
#[cfg(feature = "mcp")]
mod sse;

#[cfg(feature = "mcp")]
#[allow(deprecated)]
pub use auth::mcp_auth_dir;
#[cfg(feature = "mcp")]
pub use auth::{
    AuthStoreFile, AuthStorePathBuilder, DEFAULT_AUTH_FILE_NAME, DEFAULT_OAUTH_SCOPES, FileCredentialStore,
    FileCredentialStoreBuilder, McpOAuthFlowResult, auth_store_path, clear_credentials, has_stored_credentials,
    run_oauth_flow,
};
#[cfg(feature = "mcp")]
pub use client::{
    McpClient, McpConnectContext, McpProbeResult, PROBE_TIMEOUT, call_stdio_tool, call_tool_for_server, connect,
    connect_http, connect_stdio, connect_with_context, list_tools, list_tools_for_server, parse_stdio_config,
    probe_server, probe_server_with_auth, probe_stdio_server, shutdown_client, validate_server_config,
};
#[cfg(feature = "mcp")]
pub use config::{
    DEFAULT_OPERATION_TIMEOUT_SECS, McpConfig, McpHttpConfig, McpLoadOptions, McpServerConfig, McpStdioConfig,
};
#[cfg(feature = "mcp")]
pub use events::{McpClientService, McpEventBus, McpServerEvent};
#[cfg(feature = "mcp")]
pub use policy::{McpPolicyAction, McpPolicyConfig, mcp_tool_requires_approval, pattern_matches};
#[cfg(feature = "mcp")]
pub use registry::{
    McpLoadReport, McpPromptDescriptor, McpResourceDescriptor, McpServerLoadReport, McpToolDescriptor, McpToolRegistry,
    expose_tool_name, mcp_result_to_agent, parse_exposed_tool_name,
};
#[cfg(feature = "mcp")]
pub use session::{McpServerSession, McpSessionPool};
