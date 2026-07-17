//! `/system-prompt` slash command — show the compiled system prompt in a dialog.

use anyhow::Result;

use super::CodingAgentSession;

pub async fn compiled_system_prompt_message(session: &CodingAgentSession) -> Result<String> {
    session.compiled_system_prompt().await
}

/// Resolve compiled system prompt text for the TUI slash handler (sync).
pub fn system_prompt_slash_message(session: Option<&CodingAgentSession>) -> Result<String, String> {
    let Some(session) = session else {
        return Err("Agent session required for this command.".into());
    };
    match elph_agent::try_block_on(compiled_system_prompt_message(session)) {
        Ok(Ok(text)) => Ok(text),
        Ok(Err(err)) => Err(format!("Failed to compile system prompt: {err}")),
        Err(_) => Err("Failed to compile system prompt.".into()),
    }
}
