//! Default system prompt constants available without template features.

/// Default persona when the host does not supply a custom system prompt.
pub const DEFAULT_SYSTEM_PROMPT: &str =
    "You are an efficient, creative, and high performance AI assistant. Be helpful, concise, and direct.";

/// Resolve an optional prompt string, falling back to [`DEFAULT_SYSTEM_PROMPT`].
pub fn resolve_system_prompt_text(prompt: Option<&str>) -> String {
    match prompt.filter(|value| !value.trim().is_empty()) {
        Some(value) => value.to_string(),
        None => DEFAULT_SYSTEM_PROMPT.to_string(),
    }
}
