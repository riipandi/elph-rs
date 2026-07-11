//! MCP client integration via [rmcp](https://crates.io/crates/rmcp).
//!
//! # Features
//!
//! - **stdio**, **streamable HTTP**, and legacy **SSE** transports
//! - **OAuth 2.1** (PKCE) for remote servers via `mcp auth`
//! - **AES-256-GCM** credential encryption (`enc:` prefix) in shared `auth.json`
//! - **JSON Schema + semantic** config validation
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
mod auth_resolve;
#[cfg(feature = "mcp")]
mod client;
#[cfg(feature = "mcp")]
mod config;
#[cfg(feature = "mcp")]
mod crypto;
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
mod store_lock;
#[cfg(feature = "mcp")]
mod truncate;
#[cfg(feature = "mcp")]
mod validate;

#[cfg(feature = "mcp")]
pub use auth::{
    AuthStoreFile, AuthStorePathBuilder, DEFAULT_AUTH_FILE_NAME, DEFAULT_OAUTH_SCOPES, FileCredentialStore,
    FileCredentialStoreBuilder, McpOAuthFlowOptions, McpOAuthFlowResult, auth_store_path, clear_credentials,
    has_stored_credentials, resolve_oauth_access_token, run_oauth_flow, run_oauth_flow_with_scopes,
};
#[cfg(feature = "mcp")]
pub use auth_resolve::{McpAuthSource, McpAuthSourceReport, ResolvedMcpAuth, resolve_remote_auth};
#[cfg(feature = "mcp")]
pub use client::{
    McpClient, McpConnectContext, McpProbeResult, PROBE_TIMEOUT, call_stdio_tool, call_tool_for_server, connect,
    connect_http, connect_stdio, connect_with_context, list_tools, list_tools_for_server, parse_stdio_config,
    probe_server, probe_server_with_auth, probe_stdio_server, shutdown_client, validate_server_config,
};
#[cfg(feature = "mcp")]
pub use config::{
    DEFAULT_OPERATION_TIMEOUT_SECS, McpAuthConflictPolicy, McpConfig, McpHttpConfig, McpLoadOptions,
    McpOAuthClientMeta, McpServerConfig, McpStdioConfig,
};
#[cfg(feature = "mcp")]
pub use crypto::{
    Aes256Key, DEFAULT_AUTH_KEY_FILE_NAME, ENC_PREFIX, decrypt_async, decrypt_json_async, decrypt_string_async,
    decrypt_string_sync, default_auth_key_path, encrypt_async, encrypt_json_async, encrypt_string_async,
    encrypt_string_sync, is_encrypted_value,
};
#[cfg(feature = "mcp")]
pub use events::{McpClientService, McpEventBus, McpServerEvent};
#[cfg(feature = "mcp")]
pub use policy::{McpPolicyAction, McpPolicyConfig, mcp_tool_requires_approval, pattern_matches};
#[cfg(feature = "mcp")]
pub use registry::{
    McpLoadReport, McpPromptDescriptor, McpResourceDescriptor, McpServerLoadReport, McpToolDescriptor, McpToolRegistry,
    expose_tool_name, mcp_result_to_agent, mcp_result_to_agent_with_limit, parse_exposed_tool_name,
};
#[cfg(feature = "mcp")]
pub use session::{McpServerSession, McpSessionPool};
#[cfg(feature = "mcp")]
pub use truncate::{DEFAULT_MAX_STRUCTURED_DETAIL_CHARS, DEFAULT_MAX_TOOL_RESULT_CHARS, truncate_chars};
#[cfg(feature = "mcp")]
pub use validate::{
    McpConfigValidationError, parse_and_validate_mcp_config, parse_and_validate_mcp_config_async,
    parse_and_validate_server_config_json, validate_mcp_config, validate_mcp_config_semantic,
    validate_mcp_config_value,
};
