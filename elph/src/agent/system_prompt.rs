//! System prompt assembly for coding sessions.

use crate::types::AgentMode;
use elph_agent::AgentHarnessResources;
use elph_agent::{format_skills_for_system_prompt, now_iso_timestamp};
use std::path::Path;

use super::tool_policy::mode_tool_guidance;

pub fn build_system_prompt(
    cwd: &Path,
    resources: &AgentHarnessResources,
    tool_names: &[String],
    agents_md: Option<&str>,
    mode: AgentMode,
) -> String {
    let date = now_iso_timestamp().chars().take(10).collect::<String>();
    let mut parts = vec![
        "You are Elph, a helpful AI coding agent.".to_string(),
        format!("Working directory: {}", cwd.display()),
        format!("Current date: {date}"),
        mode_tool_guidance(mode).to_string(),
    ];

    if !tool_names.is_empty() {
        parts.push(format!(
            "Tools available in this turn (only call names from this list): {}",
            tool_names.join(", ")
        ));
    }

    if let Some(agents_md) = agents_md.filter(|s| !s.trim().is_empty()) {
        parts.push(format!("# Project context\n{agents_md}"));
    }

    if !resources.skills.is_empty() {
        parts.push(format_skills_for_system_prompt(&resources.skills));
    }

    parts.join("\n\n")
}

pub fn agents_md_for_cwd(cwd: &Path) -> Option<String> {
    find_agents_md(cwd)
}

fn find_agents_md(mut dir: &Path) -> Option<String> {
    for _ in 0..8 {
        let candidate = dir.join("AGENTS.md");
        if candidate.is_file() {
            return std::fs::read_to_string(candidate).ok();
        }
        dir = dir.parent()?;
    }
    None
}
