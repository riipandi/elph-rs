//! Agent integration using elph-agent and elph-ai.
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki)
//! `src/agent/index.ts`. Original MIT License, Copyright (c) 2026 LangChain.
//!
//! This module uses the Elph agent runtime instead of LangChain/LangGraph.
//! The core agent loop and tool execution are delegated to `elph-agent`,
//! while LLM provider integration uses `elph-ai`.

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::Arc;

use elph_agent::{Agent, AgentEvent, AgentOptions, PartialAgentState};
use elph_ai::{builtin_models, get_builtin_model};

use crate::config::Config;
use crate::constants::provider_config;
use crate::metadata::UpdateMetadata;
use crate::prompts::{create_chat_prompt, create_init_prompt, create_update_prompt};

/// Run the agent with the given command
pub async fn run_agent(
    _command: &str,
    system_prompt: &str,
    user_prompt: &str,
    config: &Config,
    _cwd: &Path,
    print_mode: bool,
) -> Result<String> {
    // Get the model - try direct lookup first, then provider/model format
    let model = get_builtin_model(&config.provider, &config.model_id)
        .or_else(|| {
            // Try parsing as provider/model format (e.g., "opencode/big-pickle")
            let parts: Vec<&str> = config.model_id.splitn(2, '/').collect();
            if parts.len() == 2 {
                get_builtin_model(parts[0], parts[1])
            } else {
                None
            }
        })
        .or_else(|| {
            // Try with the configured provider prefix
            get_builtin_model(&config.provider, &config.model_id)
        })
        .context(format!(
            "Model not found: {}/{}. Use provider/model format (e.g., opencode/big-pickle)",
            config.provider, config.model_id
        ))?;

    // Create progress bar
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
            .template("{spinner:.cyan} {msg}")
            .unwrap(),
    );
    pb.set_message("Starting agent...");
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Get models and auth
    let models = builtin_models(None);
    let auth = models.get_auth(&model).await?;
    if auth.is_none() {
        let provider_cfg =
            provider_config(&config.provider).context(format!("Unknown provider: {}", config.provider))?;
        anyhow::bail!(
            "No API key configured for {}. Set {} environment variable.",
            provider_cfg.label,
            provider_cfg.api_key_env_key
        );
    }

    let models: Arc<elph_ai::Models> = models.into_arc();
    let stream_fn: elph_agent::StreamFn = {
        let models = models.clone();
        Arc::new(move |m, ctx, opts| models.stream_simple(m, ctx, opts))
    };

    // Create the agent
    // TODO: Add tools support (requires implementing ExecutionEnv trait)
    let agent = Agent::new(AgentOptions {
        initial_state: Some(PartialAgentState {
            system_prompt: Some(system_prompt.to_string()),
            model: Some(model),
            ..Default::default()
        }),
        stream_fn: Some(stream_fn),
        ..Default::default()
    });

    // Subscribe to events for progress display
    let pb_clone = pb.clone();

    agent
        .subscribe(Arc::new(move |event, _token| {
            let pb = pb_clone.clone();
            Box::pin(async move {
                match event {
                    AgentEvent::ToolExecutionStart { tool_name, .. } => {
                        pb.set_message(format!("Using tool: {}", tool_name));
                    }
                    AgentEvent::AgentEnd { .. } => {
                        pb.finish_and_clear();
                    }
                    _ => {}
                }
            })
        }))
        .await;

    pb.set_message("Agent running...");

    // Send the user prompt
    agent.prompt_text(user_prompt, None).await?;

    // Wait for completion
    agent.wait_for_idle().await;

    // Get the final state
    let state = agent.state().await;

    if print_mode {
        // Extract the final assistant message
        if let Some(elph_ai::Message::Assistant(assistant)) = state.messages.last().and_then(|m| m.as_llm()) {
            let mut text = String::new();
            for block in &assistant.content {
                if let elph_ai::AssistantContentBlock::Text(t) = block {
                    text.push_str(&t.text);
                }
            }
            Ok(text)
        } else {
            Ok(String::new())
        }
    } else {
        Ok(format!(
            "Agent completed successfully.\nTranscript messages: {}",
            state.messages.len()
        ))
    }
}

/// Prepare the init command
pub fn prepare_init_command(_cwd: &Path, user_message: Option<&str>, _model: &str) -> (String, String) {
    let system_prompt = create_system_prompt_for_init();
    let user_prompt = create_init_prompt("", user_message);

    (system_prompt, user_prompt)
}

/// Prepare the update command
pub fn prepare_update_command(
    cwd: &Path,
    user_message: Option<&str>,
    _model: &str,
    last_update: Option<&UpdateMetadata>,
) -> (String, String) {
    let system_prompt = create_system_prompt_for_update();
    let git_summary = crate::docs::get_git_summary(cwd);
    let user_prompt = create_update_prompt(last_update, &git_summary, user_message);

    (system_prompt, user_prompt)
}

/// Prepare the chat command
pub fn prepare_chat_command(message: &str) -> (String, String) {
    let system_prompt = create_system_prompt_for_chat();
    let user_prompt = create_chat_prompt(message);

    (system_prompt, user_prompt)
}

fn create_system_prompt_for_init() -> String {
    let base = crate::prompts::create_system_prompt();
    format!(
        "{base}\n\n- This is an initial documentation run.\n- Assume {OWLY_DIR}/ does not yet contain useful documentation.\n- Build the documentation structure from scratch.\n- First build a repository inventory: existing docs, graph/app entrypoints, package/config files, major domain folders, tests/evals, data/schema files, skill/playbook files, and operational scripts.\n- Use git evidence during init to understand how important files and workflows came to be.\n- Create {OWLY_DIR}/quickstart.md first, then the linked section pages.\n- Use at most 8 documentation pages on the initial run unless the repository is clearly tiny.\n- Do not try to document every source file. Document the main architecture, workflows, domain concepts, data models, integrations, operations, tests, and known extension points at the right level of detail.\n- The CLI will record successful run metadata after you finish.",
        OWLY_DIR = crate::constants::OWLY_DIR
    )
}

fn create_system_prompt_for_update() -> String {
    let base = crate::prompts::create_system_prompt();
    format!(
        "{base}\n\n- This is a maintenance update run.\n- Inspect the existing {OWLY_DIR}/ documentation before editing.\n- Always use git-oriented repository evidence to understand recent changes.\n- Before editing, build a docs impact plan from the changed source files.\n- Update runs must be surgical. Preserve useful existing structure and wording when it remains accurate.\n- Only edit pages whose current content is inaccurate, incomplete, or misleading because of the recent changes.\n- Keep each concept in one canonical page.\n- Do not make formatting-only edits.\n- Use a soft diff budget: if fewer than about 5 source files changed, update at most 1-2 wiki pages.\n- Updates may be a no-op. If there are no relevant changes, do not edit files.\n- The CLI will record successful run metadata after you finish.",
        OWLY_DIR = crate::constants::OWLY_DIR
    )
}

fn create_system_prompt_for_chat() -> String {
    let base = crate::prompts::create_system_prompt();
    format!(
        "{base}\n\n- This is an interactive chat turn.\n- Answer the user's message directly.\n- Do not create or update Owly documentation unless the user explicitly asks you to modify documentation.\n- If the user asks to initialize or update the wiki, explain that they can run owly --init or owly --update."
    )
}
