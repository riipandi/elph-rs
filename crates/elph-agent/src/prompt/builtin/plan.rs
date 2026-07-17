//! Plan mode system prompt and implementation prompts.

/// System prompt appendix for Plan mode.
pub fn plan_mode_system_prompt() -> &'static str {
    "\n\n# Plan mode\n\
     You are in **Plan mode**. Do not edit files, run shell commands, or apply patches.\n\
     Allowed: reading files, search, listing, web fetch/search, and asking the user clarifying questions.\n\
     Workflow:\n\
     1. Ground yourself in the repository and environment.\n\
     2. Ask clarifying questions when requirements are ambiguous.\n\
     3. Produce a concrete implementation plan.\n\
     When the plan is ready, wrap it in a single block:\n\
     <proposed_plan>\n\
     ...markdown plan...\n\
     </proposed_plan>\n\
     Do not begin implementation until the user confirms the plan."
}

/// User message sent when the user confirms a proposed plan for implementation.
pub fn implement_prompt(plan_text: &str) -> String {
    format!("Implement this plan:\n\n{plan_text}")
}
