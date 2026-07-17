//! Tool exposure policy per collaboration mode.

use super::CollaborationMode;

/// Built-in tools for read-only exploration (Plan + Ask modes).
pub const EXPLORATION_BUILTIN_TOOLS: &[&str] = &[
    "read_file",
    "grep",
    "find_path",
    "list_dir",
    "web_fetch",
    "web_search",
    "diagnostics",
    "ask_user_question",
    "list_available_tools",
];

/// Tools that mutate workspace state or spawn work — blocked in Plan mode.
const MUTATING_TOOLS: &[&str] = &[
    "write_file",
    "edit_file",
    "shell_exec",
    "create_dir",
    "copy_path",
    "delete_path",
    "move_path",
    "spawn_agent",
    "send_message",
    "followup_task",
    "wait_agent",
];

const COLLABORATION_TOOLS: &[&str] = &[
    "ask_user_question",
    "spawn_agent",
    "send_message",
    "followup_task",
    "wait_agent",
    "list_agents",
];

pub fn is_mcp_tool(name: &str) -> bool {
    name.starts_with("mcp_")
}

pub fn is_goal_tool(name: &str) -> bool {
    matches!(name, "create_goal" | "get_goal" | "update_goal" | "set_goal_budget")
}

pub fn is_exploration_builtin_tool(name: &str) -> bool {
    EXPLORATION_BUILTIN_TOOLS.contains(&name)
}

/// MCP tools that only read or list remote state (safe in Plan / Ask).
pub fn is_read_only_mcp_tool(name: &str) -> bool {
    if !is_mcp_tool(name) {
        return false;
    }
    if is_mcp_read_only_bridge_tool(name) {
        return true;
    }
    let lower = name.to_ascii_lowercase();
    lower.contains("__read")
        || lower.contains("__list")
        || lower.contains("__get")
        || lower.contains("__search")
        || lower.contains("__fetch")
        || lower.contains("__browse")
        || lower.ends_with("_read")
}

pub fn is_plan_mode_tool(name: &str) -> bool {
    is_exploration_builtin_tool(name) || is_goal_tool(name) || is_read_only_mcp_tool(name)
}

/// Filter tool names for Ask mode (read-only; optional MCP registry for approval hints).
pub fn filter_ask_mode_tools(
    all_names: &[String],
    mcp_registry: Option<&crate::tools::mcp::McpToolRegistry>,
) -> Vec<String> {
    all_names
        .iter()
        .filter(|name| is_ask_mode_tool(name, mcp_registry))
        .cloned()
        .collect()
}

pub fn is_ask_mode_tool(name: &str, mcp_registry: Option<&crate::tools::mcp::McpToolRegistry>) -> bool {
    if is_exploration_builtin_tool(name) {
        return true;
    }
    if matches!(name, "get_goal") {
        return true;
    }
    if is_read_only_mcp_tool(name) {
        return true;
    }
    if let Some(reg) = mcp_registry {
        return is_mcp_tool(name) && !reg.tool_requires_approval(name);
    }
    false
}

pub fn is_mutating_tool(name: &str) -> bool {
    if MUTATING_TOOLS.contains(&name) {
        return true;
    }
    // MCP tools are treated as potentially mutating unless they are read-only bridge tools.
    // Product-level policy may refine this via `McpToolRegistry::tool_requires_approval`.
    if is_mcp_tool(name) {
        return !is_mcp_read_only_bridge_tool(name);
    }
    false
}

/// MCP bridge tools that only inspect server state (safe without approval by default).
pub fn is_mcp_read_only_bridge_tool(name: &str) -> bool {
    name.ends_with("__list_resources") || name.ends_with("__list_prompts") || name.ends_with("__read_resource")
}

pub fn is_collaboration_tool(name: &str) -> bool {
    COLLABORATION_TOOLS.contains(&name)
}

/// Filter active tool names for the given collaboration mode.
pub fn filter_active_tools(mode: CollaborationMode, all_names: &[String]) -> Vec<String> {
    match mode {
        CollaborationMode::Default => all_names.to_vec(),
        CollaborationMode::Plan => all_names
            .iter()
            .filter(|name| is_plan_mode_tool(name))
            .cloned()
            .collect(),
    }
}

/// Whether a tool call should be blocked in Plan mode.
pub fn plan_mode_blocks_tool(mode: CollaborationMode, tool_name: &str) -> bool {
    mode == CollaborationMode::Plan && (is_mutating_tool(tool_name) || !is_plan_mode_tool(tool_name))
}

pub fn plan_mode_block_reason(tool_name: &str) -> String {
    format!(
        "Tool \"{tool_name}\" is not available in Plan mode. Use read-only tools to explore, \
         then wrap your implementation plan in <proposed_plan>...</proposed_plan> for user confirmation."
    )
}

pub use crate::prompt::builtin::plan::plan_mode_system_prompt;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_mode_filters_mutating_tools() {
        let all = vec![
            "read_file".into(),
            "shell_exec".into(),
            "write_file".into(),
            "grep".into(),
        ];
        let filtered = filter_active_tools(CollaborationMode::Plan, &all);
        assert_eq!(filtered, vec!["read_file".to_string(), "grep".to_string()]);
    }

    #[test]
    fn blocks_shell_exec_in_plan_mode() {
        assert!(plan_mode_blocks_tool(CollaborationMode::Plan, "shell_exec"));
        assert!(!plan_mode_blocks_tool(CollaborationMode::Default, "shell_exec"));
    }

    #[test]
    fn plan_mode_includes_list_available_tools() {
        assert!(is_plan_mode_tool("list_available_tools"));
    }

    #[test]
    fn plan_mode_excludes_mutating_mcp_by_default() {
        assert!(!is_plan_mode_tool("mcp_fs__write_file"));
        assert!(is_plan_mode_tool("mcp_wiki__read_wiki"));
    }

    #[test]
    fn ask_mode_includes_exploration_and_excludes_write() {
        let all = vec![
            "read_file".into(),
            "write_file".into(),
            "list_available_tools".into(),
            "mcp_x__write".into(),
        ];
        let filtered = filter_ask_mode_tools(&all, None);
        assert_eq!(filtered, vec!["read_file".to_string(), "list_available_tools".to_string()]);
    }
}
