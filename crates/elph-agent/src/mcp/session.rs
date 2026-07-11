//! Long-lived MCP client sessions with reconnect-on-failure.
//!
//! Production tool calls reuse these sessions instead of spawning a new
//! stdio process (or HTTP handshake) for every invocation.

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result, bail};
use rmcp::model::{CallToolResult, Tool};
use serde_json::Value;
use tokio::sync::Mutex;
use tokio::time::timeout;
use tracing::{debug, warn};

use super::client::{McpClient, call_tool_on_client, connect, list_tools_on_client, shutdown_client};
use super::config::McpServerConfig;

/// One connected MCP server with exclusive client access.
pub struct McpServerSession {
    name: String,
    config: McpServerConfig,
    client: Mutex<Option<McpClient>>,
}

impl McpServerSession {
    pub fn new(name: impl Into<String>, config: McpServerConfig) -> Self {
        Self {
            name: name.into(),
            config,
            client: Mutex::new(None),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn config(&self) -> &McpServerConfig {
        &self.config
    }

    async fn ensure_client_locked(guard: &mut Option<McpClient>, name: &str, config: &McpServerConfig) -> Result<()> {
        if guard.is_some() {
            return Ok(());
        }
        debug!(server = %name, "opening MCP session");
        let client = connect(config)
            .await
            .with_context(|| format!("connect MCP server \"{name}\""))?;
        *guard = Some(client);
        Ok(())
    }

    async fn drop_client_locked(guard: &mut Option<McpClient>) {
        if let Some(client) = guard.take() {
            shutdown_client(client).await;
        }
    }

    /// Drop the current client (if any) and shut it down.
    pub async fn drop_client(&self) {
        let mut guard = self.client.lock().await;
        Self::drop_client_locked(&mut guard).await;
    }

    pub async fn list_tools(&self) -> Result<Vec<Tool>> {
        let op_timeout = self.config.operation_timeout();
        timeout(op_timeout, self.list_tools_inner())
            .await
            .with_context(|| format!("list tools on \"{}\" timed out after {op_timeout:?}", self.name))?
    }

    async fn list_tools_inner(&self) -> Result<Vec<Tool>> {
        let mut guard = self.client.lock().await;
        Self::ensure_client_locked(&mut guard, &self.name, &self.config).await?;
        let client = guard.as_ref().context("MCP client missing")?;
        match list_tools_on_client(client).await {
            Ok(tools) => Ok(tools),
            Err(first_error) => {
                warn!(
                    server = %self.name,
                    error = %first_error,
                    "MCP list_tools failed; reconnecting once"
                );
                Self::drop_client_locked(&mut guard).await;
                Self::ensure_client_locked(&mut guard, &self.name, &self.config).await?;
                let client = guard.as_ref().context("MCP client missing after reconnect")?;
                list_tools_on_client(client)
                    .await
                    .with_context(|| format!("MCP server \"{}\" after reconnect: {first_error}", self.name))
            }
        }
    }

    pub async fn call_tool(&self, tool_name: &str, args: Value) -> Result<CallToolResult> {
        let op_timeout = self.config.operation_timeout();
        timeout(op_timeout, self.call_tool_inner(tool_name, args))
            .await
            .with_context(|| format!("call tool on \"{}\" timed out after {op_timeout:?}", self.name))?
    }

    async fn call_tool_inner(&self, tool_name: &str, args: Value) -> Result<CallToolResult> {
        let mut guard = self.client.lock().await;
        Self::ensure_client_locked(&mut guard, &self.name, &self.config).await?;
        let client = guard.as_ref().context("MCP client missing")?;
        match call_tool_on_client(client, tool_name, args.clone()).await {
            Ok(result) => Ok(result),
            Err(first_error) => {
                warn!(
                    server = %self.name,
                    tool = %tool_name,
                    error = %first_error,
                    "MCP call_tool failed; reconnecting once"
                );
                Self::drop_client_locked(&mut guard).await;
                Self::ensure_client_locked(&mut guard, &self.name, &self.config).await?;
                let client = guard.as_ref().context("MCP client missing after reconnect")?;
                call_tool_on_client(client, tool_name, args)
                    .await
                    .with_context(|| format!("MCP server \"{}\" after reconnect: {first_error}", self.name))
            }
        }
    }

    /// Close the session (best-effort).
    pub async fn close(&self) {
        self.drop_client().await;
    }
}

impl Drop for McpServerSession {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.client.try_lock()
            && let Some(client) = guard.take()
        {
            tokio::spawn(async move {
                shutdown_client(client).await;
            });
        }
    }
}

/// Pool of named MCP server sessions.
pub struct McpSessionPool {
    sessions: Mutex<HashMap<String, Arc<McpServerSession>>>,
}

impl Default for McpSessionPool {
    fn default() -> Self {
        Self::new()
    }
}

impl McpSessionPool {
    pub fn new() -> Self {
        Self {
            sessions: Mutex::new(HashMap::new()),
        }
    }

    /// Get or create a session for `name` with the given config.
    pub async fn get_or_insert(&self, name: &str, config: McpServerConfig) -> Arc<McpServerSession> {
        let old = {
            let mut sessions = self.sessions.lock().await;
            if let Some(existing) = sessions.get(name)
                && existing.config() == &config
            {
                return Arc::clone(existing);
            }
            sessions.remove(name)
        };
        if let Some(old) = old {
            old.close().await;
        }
        let mut sessions = self.sessions.lock().await;
        if let Some(existing) = sessions.get(name)
            && existing.config() == &config
        {
            return Arc::clone(existing);
        }
        let session = Arc::new(McpServerSession::new(name, config));
        sessions.insert(name.to_string(), Arc::clone(&session));
        session
    }

    pub async fn get(&self, name: &str) -> Option<Arc<McpServerSession>> {
        self.sessions.lock().await.get(name).cloned()
    }

    pub async fn remove(&self, name: &str) -> bool {
        let session = self.sessions.lock().await.remove(name);
        if let Some(session) = session {
            session.close().await;
            true
        } else {
            false
        }
    }

    pub async fn close_all(&self) {
        let sessions: Vec<_> = {
            let mut guard = self.sessions.lock().await;
            guard.drain().map(|(_, s)| s).collect()
        };
        for session in sessions {
            session.close().await;
        }
    }

    pub async fn len(&self) -> usize {
        self.sessions.lock().await.len()
    }

    pub async fn is_empty(&self) -> bool {
        self.sessions.lock().await.is_empty()
    }

    pub async fn list_tools(&self, name: &str, config: McpServerConfig) -> Result<Vec<Tool>> {
        let session = self.get_or_insert(name, config).await;
        session.list_tools().await
    }

    pub async fn call_tool(
        &self,
        name: &str,
        config: McpServerConfig,
        tool_name: &str,
        args: Value,
    ) -> Result<CallToolResult> {
        if config.is_disabled() {
            bail!("MCP server \"{name}\" is disabled");
        }
        let session = self.get_or_insert(name, config).await;
        session.call_tool(tool_name, args).await
    }
}

impl Drop for McpSessionPool {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.sessions.try_lock() {
            guard.clear();
        }
    }
}
