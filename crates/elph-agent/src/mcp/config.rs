//! MCP server configuration types.
//!
//! Supports:
//! - **stdio** — spawn a local MCP server process
//! - **http** / **streamableHttp** — connect to a streamable HTTP MCP endpoint

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Default per-operation timeout when a server does not override it.
pub const DEFAULT_OPERATION_TIMEOUT_SECS: u64 = 60;

/// Root MCP configuration (typically `~/.elph/mcp.json`).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpConfig {
    /// Named server definitions.
    #[serde(default)]
    pub servers: BTreeMap<String, McpServerConfig>,
}

impl McpConfig {
    /// Servers that are not explicitly disabled.
    pub fn enabled_servers(&self) -> impl Iterator<Item = (&str, &McpServerConfig)> {
        self.servers
            .iter()
            .filter(|(_, cfg)| !cfg.is_disabled())
            .map(|(name, cfg)| (name.as_str(), cfg))
    }

    pub fn is_empty(&self) -> bool {
        self.servers.is_empty()
    }

    pub fn server_count(&self) -> usize {
        self.servers.len()
    }

    pub fn enabled_count(&self) -> usize {
        self.enabled_servers().count()
    }
}

/// One MCP server endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum McpServerConfig {
    /// Local process speaking MCP over stdio.
    #[serde(rename = "stdio")]
    Stdio(McpStdioConfig),
    /// Remote MCP server over streamable HTTP (`type: "http"` or `"streamableHttp"`).
    #[serde(rename = "http", alias = "streamableHttp", alias = "streamable-http")]
    Http(McpHttpConfig),
}

impl McpServerConfig {
    pub fn stdio(command: impl Into<String>, args: Vec<String>) -> Self {
        Self::Stdio(McpStdioConfig {
            command: command.into(),
            args,
            env: BTreeMap::new(),
            cwd: None,
            timeout_ms: None,
            disabled: false,
        })
    }

    pub fn http(url: impl Into<String>) -> Self {
        Self::Http(McpHttpConfig {
            url: url.into(),
            headers: BTreeMap::new(),
            auth_token: None,
            auth_token_env: None,
            timeout_ms: None,
            disabled: false,
        })
    }

    pub fn is_disabled(&self) -> bool {
        match self {
            Self::Stdio(c) => c.disabled,
            Self::Http(c) => c.disabled,
        }
    }

    pub fn operation_timeout(&self) -> Duration {
        let ms = match self {
            Self::Stdio(c) => c.timeout_ms,
            Self::Http(c) => c.timeout_ms,
        };
        Duration::from_millis(ms.unwrap_or(DEFAULT_OPERATION_TIMEOUT_SECS * 1000))
    }

    pub fn kind_label(&self) -> &'static str {
        match self {
            Self::Stdio(_) => "stdio",
            Self::Http(_) => "http",
        }
    }
}

/// Stdio (child process) server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpStdioConfig {
    /// Executable to spawn (absolute path or PATH lookup).
    pub command: String,
    /// Command arguments.
    #[serde(default)]
    pub args: Vec<String>,
    /// Extra environment variables for the child process.
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// Optional working directory for the child process.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cwd: Option<PathBuf>,
    /// Per-operation timeout in milliseconds (list tools, call tool).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// When true, the server is skipped during discovery and tool calls.
    #[serde(default)]
    pub disabled: bool,
}

/// Streamable HTTP MCP server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct McpHttpConfig {
    /// Base URL of the MCP HTTP endpoint (e.g. `https://host/mcp`).
    pub url: String,
    /// Extra HTTP headers sent with every request.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,
    /// Bearer token value (without `Bearer ` prefix). Prefer `auth_token_env` for secrets.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_token: Option<String>,
    /// Environment variable name holding a bearer token.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_token_env: Option<String>,
    /// Per-operation timeout in milliseconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// When true, the server is skipped during discovery and tool calls.
    #[serde(default)]
    pub disabled: bool,
}

impl McpHttpConfig {
    /// Resolve bearer token from inline value or environment.
    pub fn resolve_auth_token(&self) -> Option<String> {
        if let Some(token) = &self.auth_token
            && !token.is_empty()
        {
            return Some(token.clone());
        }
        self.auth_token_env
            .as_ref()
            .and_then(|name| std::env::var(name).ok())
            .filter(|v| !v.is_empty())
    }
}

/// Options controlling how a registry discovers tools.
#[derive(Debug, Clone)]
pub struct McpLoadOptions {
    /// When true (default), a failing server is recorded and skipped instead of failing the whole load.
    pub continue_on_error: bool,
    /// Max concurrent server discovery tasks (default 4).
    pub max_concurrency: usize,
    /// Override global discovery timeout per server (defaults to each server's timeout).
    pub discovery_timeout: Option<Duration>,
}

impl Default for McpLoadOptions {
    fn default() -> Self {
        Self {
            continue_on_error: true,
            max_concurrency: 4,
            discovery_timeout: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_stdio_and_http() {
        let json = r#"{
            "servers": {
                "local": {
                    "type": "stdio",
                    "command": "npx",
                    "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
                },
                "remote": {
                    "type": "http",
                    "url": "https://example.com/mcp",
                    "authTokenEnv": "MCP_TOKEN"
                }
            }
        }"#;
        let cfg: McpConfig = serde_json::from_str(json).expect("parse");
        assert_eq!(cfg.server_count(), 2);
        assert_eq!(cfg.enabled_count(), 2);
        assert!(matches!(cfg.servers.get("local"), Some(McpServerConfig::Stdio(_))));
        assert!(matches!(cfg.servers.get("remote"), Some(McpServerConfig::Http(_))));
    }

    #[test]
    fn disabled_servers_filtered() {
        let mut cfg = McpConfig::default();
        cfg.servers.insert(
            "a".into(),
            McpServerConfig::Stdio(McpStdioConfig {
                command: "true".into(),
                args: vec![],
                env: BTreeMap::new(),
                cwd: None,
                timeout_ms: None,
                disabled: true,
            }),
        );
        assert_eq!(cfg.enabled_count(), 0);
    }

    #[test]
    fn streamable_http_alias() {
        let json = r#"{"type":"streamableHttp","url":"http://localhost:8080/mcp"}"#;
        let cfg: McpServerConfig = serde_json::from_str(json).expect("parse");
        assert!(matches!(cfg, McpServerConfig::Http(_)));
    }
}
