//! Tool exposure and approval policy for TUI agent modes.

use crate::types::AgentMode;
use elph_agent::{CollaborationMode, McpToolRegistry};
use elph_agent::{filter_active_tools, filter_ask_mode_tools, is_mcp_tool, is_mutating_tool};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::events::AgentUiEvent;
use super::events::{ToolApprovalChoice, ToolApprovalRequest};

pub struct AgentModePolicy {
    pub mode: AgentMode,
    brave: bool,
    session_allowed: Mutex<HashSet<String>>,
    /// Optional MCP registry for fine-grained MCP tool approval.
    mcp_registry: Option<Arc<McpToolRegistry>>,
}

impl AgentModePolicy {
    pub fn new(mode: AgentMode) -> Self {
        Self {
            mode,
            brave: mode == AgentMode::Brave,
            session_allowed: Mutex::new(HashSet::new()),
            mcp_registry: None,
        }
    }

    pub fn with_mcp_registry(mut self, registry: Arc<McpToolRegistry>) -> Self {
        self.mcp_registry = Some(registry);
        self
    }

    pub fn set_mcp_registry(&mut self, registry: Arc<McpToolRegistry>) {
        self.mcp_registry = Some(registry);
    }

    pub fn set_mode(&mut self, mode: AgentMode) {
        self.mode = mode;
        self.brave = mode == AgentMode::Brave;
    }

    /// Resolve which registered tools are exposed to the model for `mode`.
    pub fn active_tool_names_for_mode(
        mode: AgentMode,
        all_registered: &[String],
        mcp_registry: Option<&McpToolRegistry>,
    ) -> Vec<String> {
        let names = match mode {
            AgentMode::Build | AgentMode::Brave => all_registered.to_vec(),
            AgentMode::Plan => filter_active_tools(CollaborationMode::Plan, all_registered),
            AgentMode::Ask => filter_ask_mode_tools(all_registered, mcp_registry),
        };
        Self::ensure_list_available_tool(names)
    }

    fn ensure_list_available_tool(mut names: Vec<String>) -> Vec<String> {
        if !names.iter().any(|n| n == "list_available_tools") {
            names.push("list_available_tools".into());
        }
        names.sort();
        names.dedup();
        names
    }

    pub fn needs_approval(&self, tool_name: &str) -> bool {
        if self.brave || self.mode == AgentMode::Ask {
            return false;
        }
        if self.mode == AgentMode::Plan {
            return false;
        }
        if is_mcp_tool(tool_name) {
            if let Some(reg) = &self.mcp_registry {
                return reg.tool_requires_approval(tool_name);
            }
            return is_mutating_tool(tool_name);
        }
        is_mutating_tool(tool_name)
    }

    pub async fn request_approval(
        &self,
        tool_call_id: String,
        tool_name: String,
        args_summary: String,
        ui_tx: &tokio::sync::mpsc::UnboundedSender<AgentUiEvent>,
    ) -> Result<bool, String> {
        if self.session_allowed.lock().await.contains(&tool_name) {
            return Ok(true);
        }
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();
        let _ = ui_tx.send(AgentUiEvent::ToolApprovalRequired(ToolApprovalRequest {
            tool_call_id,
            tool_name: tool_name.clone(),
            args_summary,
            response_tx,
        }));
        match response_rx.await {
            Ok(ToolApprovalChoice::Approve) => Ok(true),
            Ok(ToolApprovalChoice::AllowSession) => {
                self.session_allowed.lock().await.insert(tool_name);
                Ok(true)
            }
            Ok(ToolApprovalChoice::Reject) => Ok(false),
            Err(_) => Err("Tool approval channel closed".into()),
        }
    }
}

pub fn agent_mode_from_setting(value: &str) -> AgentMode {
    match value.trim().to_ascii_lowercase().as_str() {
        "plan" => AgentMode::Plan,
        "ask" => AgentMode::Ask,
        "brave" => AgentMode::Brave,
        _ => AgentMode::Build,
    }
}

pub fn thinking_level_from_setting(value: &str) -> crate::types::ThinkingLevel {
    crate::types::ThinkingLevel::from_setting(value)
}

pub fn to_agent_thinking(level: crate::types::ThinkingLevel) -> elph_agent::AgentThinkingLevel {
    use crate::types::ThinkingLevel;
    use elph_agent::AgentThinkingLevel;
    match level {
        ThinkingLevel::Off => AgentThinkingLevel::Off,
        ThinkingLevel::Minimal => AgentThinkingLevel::Minimal,
        ThinkingLevel::Low => AgentThinkingLevel::Low,
        ThinkingLevel::Medium => AgentThinkingLevel::Medium,
        ThinkingLevel::High => AgentThinkingLevel::High,
        ThinkingLevel::Xhigh => AgentThinkingLevel::Xhigh,
    }
}

pub fn mode_tool_guidance(mode: AgentMode) -> &'static str {
    match mode {
        AgentMode::Build => {
            "Mode: Build — full tool access. Mutating tools (write, edit, bash, create_dir, etc.) may require user approval."
        }
        AgentMode::Brave => "Mode: Brave — full tool access without approval prompts. Use mutating tools responsibly.",
        AgentMode::Plan => {
            "Mode: Plan — read-only exploration only. Use web_search, read_file, grep, and similar tools to research. \
             Wrap your implementation plan in <proposed_plan>...</proposed_plan> for user confirmation before editing."
        }
        AgentMode::Ask => {
            "Mode: Ask — read-only exploration. Do not attempt write_file, edit_file, bash, create_dir, or other mutating tools; \
             they are not available in this mode."
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_mode_exposes_all_registered_tools() {
        let all = vec!["read_file".into(), "write_file".into(), "bash".into()];
        let active = AgentModePolicy::active_tool_names_for_mode(AgentMode::Build, &all, None);
        assert_eq!(active.len(), 4);
        assert!(active.contains(&"write_file".to_string()));
        assert!(active.contains(&"list_available_tools".to_string()));
    }

    #[test]
    fn ask_mode_hides_mutating_tools() {
        let all = vec![
            "read_file".into(),
            "write_file".into(),
            "web_search".into(),
            "create_dir".into(),
        ];
        let active = AgentModePolicy::active_tool_names_for_mode(AgentMode::Ask, &all, None);
        assert!(active.contains(&"read_file".to_string()));
        assert!(active.contains(&"web_search".to_string()));
        assert!(!active.contains(&"write_file".to_string()));
        assert!(!active.contains(&"create_dir".to_string()));
    }

    #[test]
    fn ask_mode_matches_exploration_surface() {
        let all: Vec<String> = elph_agent::EXPLORATION_BUILTIN_TOOLS
            .iter()
            .map(|name| (*name).to_string())
            .collect();
        let active = AgentModePolicy::active_tool_names_for_mode(AgentMode::Ask, &all, None);
        assert!(active.contains(&"read_file".to_string()));
        assert!(!active.contains(&"write_file".to_string()));
    }

    #[test]
    fn plan_mode_hides_edit_tools() {
        let all = vec!["read_file".into(), "edit_file".into(), "web_search".into()];
        let active = AgentModePolicy::active_tool_names_for_mode(AgentMode::Plan, &all, None);
        assert!(active.contains(&"web_search".to_string()));
        assert!(!active.contains(&"edit_file".to_string()));
    }
}
