//! MCP server notification events (list_changed, etc.).

use std::sync::Arc;

use rmcp::handler::client::ClientHandler;
use rmcp::model::{CancelledNotificationParam, ProgressNotificationParam, ResourceUpdatedNotificationParam};
use rmcp::service::{NotificationContext, RoleClient};
use tokio::sync::mpsc;
use tracing::debug;

/// Events emitted when an MCP server notifies the client of catalog changes.
#[derive(Debug, Clone)]
pub enum McpServerEvent {
    ToolListChanged { server: String },
    ResourceListChanged { server: String },
    PromptListChanged { server: String },
    ResourceUpdated { server: String, uri: String },
}

/// Client-side handler that forwards list_changed notifications to a channel.
#[derive(Clone)]
pub struct McpClientService {
    server_name: String,
    events: Option<mpsc::UnboundedSender<McpServerEvent>>,
}

impl McpClientService {
    pub fn new(server_name: impl Into<String>, events: Option<mpsc::UnboundedSender<McpServerEvent>>) -> Self {
        Self {
            server_name: server_name.into(),
            events,
        }
    }

    pub fn noop() -> Self {
        Self {
            server_name: String::new(),
            events: None,
        }
    }
}

impl ClientHandler for McpClientService {
    fn on_tool_list_changed(
        &self,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        let server = self.server_name.clone();
        let events = self.events.clone();
        async move {
            debug!(%server, "MCP tools/list_changed");
            if let Some(tx) = events {
                let _ = tx.send(McpServerEvent::ToolListChanged { server });
            }
        }
    }

    fn on_resource_list_changed(
        &self,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        let server = self.server_name.clone();
        let events = self.events.clone();
        async move {
            debug!(%server, "MCP resources/list_changed");
            if let Some(tx) = events {
                let _ = tx.send(McpServerEvent::ResourceListChanged { server });
            }
        }
    }

    fn on_prompt_list_changed(
        &self,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        let server = self.server_name.clone();
        let events = self.events.clone();
        async move {
            debug!(%server, "MCP prompts/list_changed");
            if let Some(tx) = events {
                let _ = tx.send(McpServerEvent::PromptListChanged { server });
            }
        }
    }

    fn on_resource_updated(
        &self,
        params: ResourceUpdatedNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        let server = self.server_name.clone();
        let events = self.events.clone();
        async move {
            debug!(%server, uri = %params.uri, "MCP resources/updated");
            if let Some(tx) = events {
                let _ = tx.send(McpServerEvent::ResourceUpdated {
                    server,
                    uri: params.uri,
                });
            }
        }
    }

    fn on_cancelled(
        &self,
        _params: CancelledNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }

    fn on_progress(
        &self,
        _params: ProgressNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        std::future::ready(())
    }
}

/// Shared event bus for pooled MCP sessions.
#[derive(Clone, Default)]
pub struct McpEventBus {
    inner: Arc<std::sync::Mutex<Option<mpsc::UnboundedSender<McpServerEvent>>>>,
}

impl McpEventBus {
    pub fn new() -> Self {
        Self::default()
    }

    /// Install a sender (typically once at registry load).
    pub fn set_sender(&self, tx: mpsc::UnboundedSender<McpServerEvent>) {
        *self.inner.lock().unwrap_or_else(|e| e.into_inner()) = Some(tx);
    }

    pub fn sender(&self) -> Option<mpsc::UnboundedSender<McpServerEvent>> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }
}
