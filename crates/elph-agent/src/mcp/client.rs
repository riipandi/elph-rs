//! MCP client connectivity via rmcp (stdio + streamable HTTP).

use std::collections::BTreeMap;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use http::{HeaderName, HeaderValue};
use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, CallToolResult, Tool};
use rmcp::service::RunningService;
use rmcp::transport::streamable_http_client::StreamableHttpClientTransportConfig;
use rmcp::transport::{ConfigureCommandExt, StreamableHttpClientTransport, TokioChildProcess};
use serde_json::Value;
use tokio::process::Command;
use tokio::time::timeout;
use tracing::{debug, warn};

use super::config::{McpHttpConfig, McpServerConfig, McpStdioConfig};

/// Live MCP client service (stdio process or HTTP session).
pub type McpClient = RunningService<rmcp::RoleClient, ()>;

/// Default probe/list timeout used by CLI doctor when no server timeout is set.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone)]
pub struct McpProbeResult {
    pub name: String,
    pub ok: bool,
    pub tool_count: usize,
    pub transport: String,
    pub message: String,
}

/// Open a long-lived connection (caller owns lifecycle / cancel).
pub async fn connect(config: &McpServerConfig) -> Result<McpClient> {
    match config {
        McpServerConfig::Stdio(cfg) => connect_stdio(cfg).await,
        McpServerConfig::Http(cfg) => connect_http(cfg).await,
    }
}

pub async fn connect_stdio(config: &McpStdioConfig) -> Result<McpClient> {
    let mut command = Command::new(&config.command);
    command.args(&config.args);
    command.envs(config.env.iter());
    if let Some(cwd) = &config.cwd {
        command.current_dir(cwd);
    }
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    // Capture stderr for debugging failed servers; do not inherit to avoid TUI noise.
    command.stderr(Stdio::piped());
    command.kill_on_drop(true);

    debug!(command = %config.command, args = ?config.args, "spawning MCP stdio server");
    let transport = TokioChildProcess::new(command.configure(|_| {})).context("spawn MCP stdio transport")?;
    let client = ().serve(transport).await.context("initialize MCP stdio client")?;
    Ok(client)
}

pub async fn connect_http(config: &McpHttpConfig) -> Result<McpClient> {
    let mut transport_config = StreamableHttpClientTransportConfig::with_uri(config.url.clone());
    if let Some(token) = config.resolve_auth_token() {
        transport_config = transport_config.auth_header(token);
    }
    if !config.headers.is_empty() {
        let mut headers = std::collections::HashMap::new();
        for (key, value) in &config.headers {
            let name =
                HeaderName::from_bytes(key.as_bytes()).with_context(|| format!("invalid HTTP header name: {key}"))?;
            let value = HeaderValue::from_str(value).with_context(|| format!("invalid HTTP header value for {key}"))?;
            headers.insert(name, value);
        }
        transport_config = transport_config.custom_headers(headers);
    }

    debug!(url = %config.url, "connecting MCP streamable HTTP server");
    let transport = StreamableHttpClientTransport::from_config(transport_config);
    let client = ().serve(transport).await.context("initialize MCP HTTP client")?;
    Ok(client)
}

pub async fn list_tools_on_client(client: &McpClient) -> Result<Vec<Tool>> {
    client.list_all_tools().await.context("list MCP tools")
}

pub async fn call_tool_on_client(client: &McpClient, tool_name: &str, args: Value) -> Result<CallToolResult> {
    let mut params = CallToolRequestParams::new(tool_name.to_string());
    if let Value::Object(map) = args {
        params = params.with_arguments(map);
    }
    client.call_tool(params).await.context("call MCP tool")
}

/// Gracefully cancel a client; errors are logged.
pub async fn shutdown_client(client: McpClient) {
    if let Err(error) = client.cancel().await {
        warn!("MCP client shutdown error: {error}");
    }
}

pub async fn probe_stdio_server(name: &str, config: &McpStdioConfig) -> McpProbeResult {
    probe_server(name, &McpServerConfig::Stdio(config.clone())).await
}

pub async fn probe_server(name: &str, config: &McpServerConfig) -> McpProbeResult {
    let transport = config.kind_label().to_string();
    let op_timeout = config.operation_timeout();
    match timeout(op_timeout, async {
        let client = connect(config).await?;
        let tools = list_tools_on_client(&client).await.unwrap_or_default();
        let count = tools.len();
        shutdown_client(client).await;
        Ok::<_, anyhow::Error>(count)
    })
    .await
    {
        Ok(Ok(count)) => McpProbeResult {
            name: name.to_string(),
            ok: true,
            tool_count: count,
            transport,
            message: format!("connected, {count} tools"),
        },
        Ok(Err(error)) => McpProbeResult {
            name: name.to_string(),
            ok: false,
            tool_count: 0,
            transport,
            message: error.to_string(),
        },
        Err(_) => McpProbeResult {
            name: name.to_string(),
            ok: false,
            tool_count: 0,
            transport,
            message: format!("timed out after {op_timeout:?}"),
        },
    }
}

/// One-shot list tools (connects, lists, disconnects). Prefer session pool for repeated calls.
pub async fn list_tools(config: &McpStdioConfig) -> Result<Vec<Tool>> {
    list_tools_for_server(&McpServerConfig::Stdio(config.clone())).await
}

pub async fn list_tools_for_server(config: &McpServerConfig) -> Result<Vec<Tool>> {
    let op_timeout = config.operation_timeout();
    timeout(op_timeout, async {
        let client = connect(config).await?;
        let tools = list_tools_on_client(&client).await?;
        shutdown_client(client).await;
        Ok(tools)
    })
    .await
    .context("MCP list tools timed out")?
}

/// One-shot tool call (connects, calls, disconnects). Prefer session pool for production.
pub async fn call_stdio_tool(config: &McpStdioConfig, tool_name: &str, args: Value) -> Result<CallToolResult> {
    call_tool_for_server(&McpServerConfig::Stdio(config.clone()), tool_name, args).await
}

pub async fn call_tool_for_server(config: &McpServerConfig, tool_name: &str, args: Value) -> Result<CallToolResult> {
    let op_timeout = config.operation_timeout();
    timeout(op_timeout, async {
        let client = connect(config).await?;
        let result = call_tool_on_client(&client, tool_name, args).await?;
        shutdown_client(client).await;
        Ok(result)
    })
    .await
    .context("MCP tool call timed out")?
}

/// Build a stdio config from CLI pieces.
pub fn parse_stdio_config(command: String, args: Vec<String>, env: BTreeMap<String, String>) -> McpStdioConfig {
    McpStdioConfig {
        command,
        args,
        env,
        cwd: None,
        timeout_ms: None,
        disabled: false,
    }
}

/// Validate a server config before connecting.
pub fn validate_server_config(name: &str, config: &McpServerConfig) -> Result<()> {
    if name.trim().is_empty() {
        bail!("MCP server name must not be empty");
    }
    match config {
        McpServerConfig::Stdio(cfg) => {
            if cfg.command.trim().is_empty() {
                bail!("MCP server \"{name}\" stdio command must not be empty");
            }
        }
        McpServerConfig::Http(cfg) => {
            if cfg.url.trim().is_empty() {
                bail!("MCP server \"{name}\" HTTP url must not be empty");
            }
            let parsed = url::Url::parse(&cfg.url).with_context(|| format!("MCP server \"{name}\" invalid url"))?;
            if parsed.scheme() != "http" && parsed.scheme() != "https" {
                bail!("MCP server \"{name}\" url must be http or https");
            }
        }
    }
    Ok(())
}
