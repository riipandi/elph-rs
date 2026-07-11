//! MCP tool registry — discover remote tools and bridge them into the agent harness.
//!
//! Production path:
//! 1. [`McpToolRegistry::load`] / [`load_with_options`] discovers tools (fail-open by default).
//! 2. Sessions are pooled so tool calls reuse stdio processes / HTTP sessions.
//! 3. [`McpToolRegistry::create_agent_tools`] exposes `mcp_{server}__{tool}` agent tools.

use std::sync::Arc;

use anyhow::{Context, Result};
use elph_ai::Tool;
use futures::stream::{self, StreamExt};
use rmcp::model::{CallToolResult, ContentBlock, Tool as McpTool};
use serde_json::{Value, json};
use tracing::{info, warn};

use crate::tools::simple_tool;
use crate::types::{AgentTool, AgentToolResult, ToolResultContent};

use super::client::{call_tool_for_server, probe_server, validate_server_config};
use super::config::{McpConfig, McpLoadOptions, McpServerConfig};
use super::session::McpSessionPool;

/// A discovered MCP tool ready for exposure to the model.
#[derive(Debug, Clone)]
pub struct McpToolDescriptor {
    pub server_name: String,
    pub tool_name: String,
    pub exposed_name: String,
    pub description: String,
    pub parameters: Value,
}

/// Per-server discovery outcome.
#[derive(Debug, Clone)]
pub struct McpServerLoadReport {
    pub name: String,
    pub ok: bool,
    pub transport: String,
    pub tool_count: usize,
    pub message: String,
}

/// Full registry load report (for doctor / logging).
#[derive(Debug, Clone, Default)]
pub struct McpLoadReport {
    pub servers: Vec<McpServerLoadReport>,
    pub tools_loaded: usize,
    pub servers_ok: usize,
    pub servers_failed: usize,
    pub servers_skipped: usize,
}

/// Registry of MCP servers, pooled sessions, and discovered tools.
pub struct McpToolRegistry {
    config: McpConfig,
    tools: Vec<McpToolDescriptor>,
    pool: Arc<McpSessionPool>,
    report: McpLoadReport,
}

impl McpToolRegistry {
    pub fn empty() -> Self {
        Self {
            config: McpConfig::default(),
            tools: Vec::new(),
            pool: Arc::new(McpSessionPool::new()),
            report: McpLoadReport::default(),
        }
    }

    /// Load with default options (continue on server errors, concurrency 4).
    pub async fn load(config: McpConfig) -> Result<Self> {
        Self::load_with_options(config, McpLoadOptions::default()).await
    }

    /// Discover tools from all enabled servers.
    pub async fn load_with_options(config: McpConfig, options: McpLoadOptions) -> Result<Self> {
        let pool = Arc::new(McpSessionPool::new());
        let enabled: Vec<(String, McpServerConfig)> = config
            .enabled_servers()
            .map(|(n, c)| (n.to_string(), c.clone()))
            .collect();
        let skipped = config.server_count().saturating_sub(enabled.len());

        let concurrency = options.max_concurrency.max(1);
        let pool_for_discovery = Arc::clone(&pool);
        let results: Vec<ServerDiscovery> = stream::iter(enabled)
            .map(|(name, server_config)| {
                let discovery_timeout = options.discovery_timeout;
                let pool = Arc::clone(&pool_for_discovery);
                async move { discover_one(&pool, &name, server_config, discovery_timeout).await }
            })
            .buffer_unordered(concurrency)
            .collect()
            .await;

        let mut tools = Vec::new();
        let mut report = McpLoadReport {
            servers_skipped: skipped,
            ..Default::default()
        };

        for result in results {
            match result {
                ServerDiscovery::Ok {
                    name,
                    transport,
                    descriptors,
                    message,
                } => {
                    report.servers_ok += 1;
                    report.tools_loaded += descriptors.len();
                    report.servers.push(McpServerLoadReport {
                        name: name.clone(),
                        ok: true,
                        transport,
                        tool_count: descriptors.len(),
                        message,
                    });
                    tools.extend(descriptors);
                }
                ServerDiscovery::Failed { name, transport, error } => {
                    report.servers_failed += 1;
                    report.servers.push(McpServerLoadReport {
                        name: name.clone(),
                        ok: false,
                        transport,
                        tool_count: 0,
                        message: error.clone(),
                    });
                    if options.continue_on_error {
                        warn!(server = %name, error = %error, "MCP server discovery failed; continuing");
                    } else {
                        anyhow::bail!("MCP server \"{name}\" discovery failed: {error}");
                    }
                }
            }
        }

        info!(
            tools = report.tools_loaded,
            ok = report.servers_ok,
            failed = report.servers_failed,
            skipped = report.servers_skipped,
            "MCP registry loaded"
        );

        Ok(Self {
            config,
            tools,
            pool,
            report,
        })
    }

    pub fn config(&self) -> &McpConfig {
        &self.config
    }

    pub fn descriptors(&self) -> &[McpToolDescriptor] {
        &self.tools
    }

    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    pub fn server_count(&self) -> usize {
        self.config.servers.len()
    }

    pub fn load_report(&self) -> &McpLoadReport {
        &self.report
    }

    pub fn session_pool(&self) -> &Arc<McpSessionPool> {
        &self.pool
    }

    /// Refresh tools for a single server (reconnect + re-list).
    pub async fn refresh_server(&mut self, server_name: &str) -> Result<usize> {
        let Some(server_config) = self.config.servers.get(server_name).cloned() else {
            anyhow::bail!("MCP server \"{server_name}\" not configured");
        };
        if server_config.is_disabled() {
            self.tools.retain(|t| t.server_name != server_name);
            let _ = self.pool.remove(server_name).await;
            return Ok(0);
        }

        let _ = self.pool.remove(server_name).await;
        let discovery = discover_one(&self.pool, server_name, server_config, None).await;
        match discovery {
            ServerDiscovery::Ok { descriptors, .. } => {
                let count = descriptors.len();
                self.tools.retain(|t| t.server_name != server_name);
                self.tools.extend(descriptors);
                Ok(count)
            }
            ServerDiscovery::Failed { error, .. } => Err(anyhow::anyhow!(error)),
        }
    }

    /// Convert discovered MCP tools into harness [`AgentTool`]s using the session pool.
    pub fn create_agent_tools(self: &Arc<Self>) -> Vec<AgentTool> {
        self.tools
            .iter()
            .map(|desc| {
                let registry = Arc::clone(self);
                let server = desc.server_name.clone();
                let tool_name = desc.tool_name.clone();
                simple_tool(
                    Tool {
                        name: desc.exposed_name.clone(),
                        description: desc.description.clone(),
                        parameters: desc.parameters.clone(),
                    },
                    format!("MCP:{}", desc.server_name),
                    move |_, args| {
                        let registry = registry.clone();
                        let server = server.clone();
                        let tool_name = tool_name.clone();
                        Box::pin(async move { registry.call_tool(&server, &tool_name, args).await })
                    },
                )
            })
            .collect()
    }

    /// Call a tool on a configured server (pooled connection).
    pub async fn call_tool(&self, server: &str, tool_name: &str, args: Value) -> Result<AgentToolResult> {
        let Some(server_config) = self.config.servers.get(server).cloned() else {
            anyhow::bail!("MCP server \"{server}\" not configured");
        };
        if server_config.is_disabled() {
            anyhow::bail!("MCP server \"{server}\" is disabled");
        }

        let result = self
            .pool
            .call_tool(server, server_config, tool_name, args)
            .await
            .with_context(|| format!("MCP tool {server}/{tool_name}"))?;
        Ok(mcp_result_to_agent(result))
    }

    /// One-shot call without using the pool (tests / doctor).
    pub async fn call_tool_ephemeral(&self, server: &str, tool_name: &str, args: Value) -> Result<AgentToolResult> {
        let Some(server_config) = self.config.servers.get(server) else {
            anyhow::bail!("MCP server \"{server}\" not configured");
        };
        let result = call_tool_for_server(server_config, tool_name, args).await?;
        Ok(mcp_result_to_agent(result))
    }

    /// Probe all configured servers (including disabled? no — enabled only).
    pub async fn probe_all(&self) -> Vec<super::client::McpProbeResult> {
        let mut out = Vec::new();
        for (name, config) in self.config.enabled_servers() {
            out.push(probe_server(name, config).await);
        }
        out
    }

    /// Shut down pooled sessions.
    pub async fn shutdown(&self) {
        self.pool.close_all().await;
    }
}

enum ServerDiscovery {
    Ok {
        name: String,
        transport: String,
        descriptors: Vec<McpToolDescriptor>,
        message: String,
    },
    Failed {
        name: String,
        transport: String,
        error: String,
    },
}

async fn discover_one(
    pool: &McpSessionPool,
    server_name: &str,
    config: McpServerConfig,
    override_timeout: Option<std::time::Duration>,
) -> ServerDiscovery {
    let transport = config.kind_label().to_string();
    if let Err(error) = validate_server_config(server_name, &config) {
        return ServerDiscovery::Failed {
            name: server_name.to_string(),
            transport,
            error: error.to_string(),
        };
    }

    let op = async {
        // Warm the session pool so subsequent tool calls reuse the connection.
        pool.list_tools(server_name, config)
            .await
            .with_context(|| format!("list tools from MCP server \"{server_name}\""))
    };

    let result = if let Some(t) = override_timeout {
        match tokio::time::timeout(t, op).await {
            Ok(inner) => inner,
            Err(_) => Err(anyhow::anyhow!("discovery timed out after {t:?}")),
        }
    } else {
        op.await
    };

    match result {
        Ok(remote_tools) => {
            let descriptors: Vec<_> = remote_tools
                .into_iter()
                .map(|tool| descriptor_from_mcp(server_name, &tool))
                .collect();
            let count = descriptors.len();
            ServerDiscovery::Ok {
                name: server_name.to_string(),
                transport,
                descriptors,
                message: format!("discovered {count} tools"),
            }
        }
        Err(error) => ServerDiscovery::Failed {
            name: server_name.to_string(),
            transport,
            error: error.to_string(),
        },
    }
}

fn descriptor_from_mcp(server_name: &str, tool: &McpTool) -> McpToolDescriptor {
    let tool_name = tool.name.to_string();
    let exposed_name = expose_tool_name(server_name, &tool_name);
    let description = tool
        .description
        .as_deref()
        .map(str::to_string)
        .unwrap_or_else(|| format!("MCP tool from server {server_name}"));
    let full_description = format!("[MCP:{server_name}] {description}");
    let parameters = {
        let schema = (*tool.input_schema).clone();
        if schema.is_empty() {
            json!({ "type": "object", "properties": {} })
        } else {
            Value::Object(schema)
        }
    };
    McpToolDescriptor {
        server_name: server_name.to_string(),
        tool_name,
        exposed_name,
        description: full_description,
        parameters,
    }
}

/// Public helper for stable tool naming: `mcp_{server}__{tool}`.
pub fn expose_tool_name(server: &str, tool: &str) -> String {
    format!("mcp_{}__{}", sanitize_identifier(server), sanitize_identifier(tool))
}

/// Parse `mcp_{server}__{tool}` back into components when possible.
pub fn parse_exposed_tool_name(exposed: &str) -> Option<(&str, &str)> {
    let rest = exposed.strip_prefix("mcp_")?;
    let (server, tool) = rest.split_once("__")?;
    if server.is_empty() || tool.is_empty() {
        return None;
    }
    Some((server, tool))
}

fn sanitize_identifier(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '_' })
        .collect()
}

pub fn mcp_result_to_agent(result: CallToolResult) -> AgentToolResult {
    let is_error = result.is_error.unwrap_or(false);
    let mut content = Vec::new();

    for block in &result.content {
        match block {
            ContentBlock::Text(text) => {
                content.push(ToolResultContent::Text(elph_ai::TextContent::new(&text.text)));
            }
            ContentBlock::Image(image) => {
                content.push(ToolResultContent::Image(elph_ai::ImageContent::new(
                    &image.data,
                    &image.mime_type,
                )));
            }
            other => {
                if let Ok(text) = serde_json::to_string(other) {
                    content.push(ToolResultContent::Text(elph_ai::TextContent::new(text)));
                }
            }
        }
    }

    if content.is_empty() {
        if let Some(structured) = &result.structured_content {
            content.push(ToolResultContent::Text(elph_ai::TextContent::new(
                structured.to_string(),
            )));
        } else if is_error {
            content.push(ToolResultContent::Text(elph_ai::TextContent::new(
                "MCP tool returned an error with no content",
            )));
        } else {
            content.push(ToolResultContent::Text(elph_ai::TextContent::new(
                "MCP tool completed with no output",
            )));
        }
    }

    let text_joined = content
        .iter()
        .filter_map(|c| match c {
            ToolResultContent::Text(t) => Some(t.text.clone()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut agent_result = if is_error {
        AgentToolResult::error(if text_joined.is_empty() {
            "MCP tool returned an error".to_string()
        } else {
            text_joined
        })
    } else {
        AgentToolResult {
            content,
            details: Value::Null,
            added_tool_names: None,
            terminate: None,
        }
    };

    agent_result.details = json!({
        "mcp": true,
        "structured_content": result.structured_content,
        "is_error": is_error,
    });
    agent_result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn expose_tool_name_sanitizes() {
        assert_eq!(expose_tool_name("my-server", "read/file"), "mcp_my_server__read_file");
    }

    #[test]
    fn parse_exposed_roundtrip() {
        let name = expose_tool_name("fs", "read_file");
        assert_eq!(parse_exposed_tool_name(&name), Some(("fs", "read_file")));
        assert_eq!(parse_exposed_tool_name("not_mcp"), None);
    }

    #[test]
    fn mcp_error_result_is_error_agent_tool() {
        let result = CallToolResult::error(vec![ContentBlock::text("boom")]);
        let agent = mcp_result_to_agent(result);
        let text = agent
            .content
            .iter()
            .filter_map(|c| match c {
                ToolResultContent::Text(t) => Some(t.text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        assert!(text.contains("boom"));
        assert_eq!(agent.details.get("is_error"), Some(&json!(true)));
    }
}
