//! Environment handling for Owly.
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki)
//! `src/env.ts`. Original MIT License, Copyright (c) 2026 LangChain.

use anyhow::Result;

use crate::config::Config;
use crate::constants::{provider_config, resolve_configured_provider};

/// Load and validate environment for running Owly
pub fn setup_environment(config: &Config) -> Result<()> {
    // Load environment from ~/.owly/.env
    crate::credentials::load_env()?;

    // Validate API key is available
    let api_key_env = config.api_key_env_key();
    if std::env::var(api_key_env).is_err() {
        let label = provider_config(&config.provider).map(|c| c.label).unwrap_or("Unknown");
        anyhow::bail!(
            "{} is required to run Owly with {}. Set it in your environment or ~/.owly/.env",
            api_key_env,
            label
        );
    }

    Ok(())
}

/// Get environment debug info (redacted)
pub fn get_debug_env() -> Vec<(String, String)> {
    let keys = vec![
        "OWLY_PROVIDER",
        "OWLY_MODEL_ID",
        "OPENCODE_API_KEY",
        "OPENROUTER_API_KEY",
        "ANTHROPIC_API_KEY",
        "OPENAI_API_KEY",
        "GOOGLE_API_KEY",
        "DEEPSEEK_API_KEY",
    ];

    keys.into_iter()
        .map(|key| {
            let value = std::env::var(key).ok();
            let display = match value {
                None => "unset".to_string(),
                Some(v) if key.contains("KEY") => format!("set(length={})", v.len()),
                Some(v) => format!("set(value={:?})", v),
            };
            (key.to_string(), display)
        })
        .collect()
}

/// Resolve the provider from environment or config
pub fn resolve_provider() -> String {
    resolve_configured_provider().to_string()
}
