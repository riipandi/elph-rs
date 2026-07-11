//! MCP tool allow/deny/approval policy.

use serde::{Deserialize, Serialize};

/// Action applied when no allow/deny/require rule matches.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum McpPolicyAction {
    /// Expose the tool and run without approval.
    Allow,
    /// Expose the tool but require user approval (default).
    #[default]
    RequireApproval,
    /// Do not expose the tool to the model.
    Deny,
}

/// Glob-like patterns for tool names (exact, `prefix*`, or `*`).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McpPolicyConfig {
    /// Default when no pattern matches.
    #[serde(default)]
    pub default: McpPolicyAction,
    /// Tools always allowed without approval (and exposed).
    #[serde(default)]
    pub allow: Vec<String>,
    /// Tools never exposed.
    #[serde(default)]
    pub deny: Vec<String>,
    /// Tools exposed but requiring approval.
    #[serde(default)]
    pub require_approval: Vec<String>,
}

impl McpPolicyConfig {
    pub fn is_empty(&self) -> bool {
        self.allow.is_empty()
            && self.deny.is_empty()
            && self.require_approval.is_empty()
            && self.default == McpPolicyAction::RequireApproval
    }

    /// Merge server-level policy over base (server patterns prepended; default overrides if set via Some).
    pub fn merge(&self, overlay: &McpPolicyConfig) -> McpPolicyConfig {
        let mut merged = self.clone();
        // Overlay default always wins when overlay is non-default or has rules —
        // simpler: overlay.default always replaces.
        merged.default = overlay.default;
        // Prepend overlay lists so they take precedence in first-match evaluation.
        let mut allow = overlay.allow.clone();
        allow.extend(self.allow.iter().cloned());
        merged.allow = allow;
        let mut deny = overlay.deny.clone();
        deny.extend(self.deny.iter().cloned());
        merged.deny = deny;
        let mut require_approval = overlay.require_approval.clone();
        require_approval.extend(self.require_approval.iter().cloned());
        merged.require_approval = require_approval;
        merged
    }

    /// Whether the tool should be exposed to the model at all.
    pub fn is_exposed(&self, tool_name: &str) -> bool {
        !matches!(self.resolve(tool_name), McpPolicyAction::Deny)
    }

    /// Whether the tool requires user approval before execution.
    pub fn requires_approval(&self, tool_name: &str) -> bool {
        matches!(self.resolve(tool_name), McpPolicyAction::RequireApproval)
    }

    pub fn resolve(&self, tool_name: &str) -> McpPolicyAction {
        // First match wins: deny > allow > require_approval > default
        if pattern_list_matches(&self.deny, tool_name) {
            return McpPolicyAction::Deny;
        }
        if pattern_list_matches(&self.allow, tool_name) {
            return McpPolicyAction::Allow;
        }
        if pattern_list_matches(&self.require_approval, tool_name) {
            return McpPolicyAction::RequireApproval;
        }
        self.default
    }
}

fn pattern_list_matches(patterns: &[String], name: &str) -> bool {
    patterns.iter().any(|p| pattern_matches(p, name))
}

/// Simple glob: `*` (any), `prefix*`, `*suffix`, exact.
pub fn pattern_matches(pattern: &str, name: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(prefix) = pattern.strip_suffix('*')
        && !prefix.contains('*')
    {
        return name.starts_with(prefix);
    }
    if let Some(suffix) = pattern.strip_prefix('*')
        && !suffix.contains('*')
    {
        return name.ends_with(suffix);
    }
    pattern == name
}

/// Whether an MCP tool name should be treated as requiring approval under `policy`.
///
/// Built-in meta tools (`list_resources`, `list_prompts`) default to allow when
/// policy is empty; all other `mcp_*` tools follow policy (default require approval).
pub fn mcp_tool_requires_approval(policy: &McpPolicyConfig, tool_name: &str) -> bool {
    if !tool_name.starts_with("mcp_") {
        return false;
    }
    // Built-in bridge tools that only inspect state.
    if tool_name.ends_with("__list_resources")
        || tool_name.ends_with("__list_prompts")
        || tool_name.ends_with("__read_resource")
    {
        // Still honor explicit deny/require/allow.
        if !policy.allow.is_empty()
            || !policy.deny.is_empty()
            || !policy.require_approval.is_empty()
            || policy.default != McpPolicyAction::RequireApproval
        {
            return policy.requires_approval(tool_name);
        }
        return false;
    }
    policy.requires_approval(tool_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deny_wins_over_allow() {
        let policy = McpPolicyConfig {
            default: McpPolicyAction::Allow,
            allow: vec!["mcp_fs__*".into()],
            deny: vec!["mcp_fs__delete".into()],
            require_approval: vec![],
        };
        assert!(!policy.is_exposed("mcp_fs__delete"));
        assert!(policy.is_exposed("mcp_fs__read"));
        assert!(!policy.requires_approval("mcp_fs__read"));
    }

    #[test]
    fn default_require_approval() {
        let policy = McpPolicyConfig::default();
        assert!(policy.requires_approval("mcp_x__write"));
        assert!(policy.is_exposed("mcp_x__write"));
    }

    #[test]
    fn pattern_prefix_suffix() {
        assert!(pattern_matches("mcp_fs__*", "mcp_fs__read"));
        assert!(pattern_matches("*__read", "mcp_fs__read"));
        assert!(!pattern_matches("mcp_other__*", "mcp_fs__read"));
    }
}
