//! Credential management for Owly.
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki)
//! `src/env.ts`. Original MIT License, Copyright (c) 2026 LangChain.

use anyhow::{Context, Result};
use memchr::memchr;
use std::collections::HashMap;
use std::path::PathBuf;

/// Managed environment keys
pub const MANAGED_ENV_KEYS: &[&str] = &[
    "OPENCODE_API_KEY",
    "OPENROUTER_API_KEY",
    "ANTHROPIC_API_KEY",
    "OPENAI_API_KEY",
    "GOOGLE_API_KEY",
    "DEEPSEEK_API_KEY",
    "GROQ_API_KEY",
    "FIREWORKS_API_KEY",
    "TOGETHER_API_KEY",
    "MISTRAL_API_KEY",
    "OWLY_PROVIDER",
    "OWLY_MODEL_ID",
];

/// Load environment from ~/.owly/.env
pub fn load_env() -> Result<HashMap<String, String>> {
    let env_path = env_path();
    if !env_path.exists() {
        return Ok(HashMap::new());
    }

    let content = std::fs::read_to_string(&env_path).context("Failed to read environment file")?;

    let mut env = HashMap::new();
    let mut start = 0usize;
    while start <= content.len() {
        let remaining = &content[start..];
        if remaining.is_empty() {
            break;
        }
        let (line, next_start) = match memchr(b'\n', remaining.as_bytes()) {
            Some(end) => (&remaining[..end], start + end + 1),
            None => (remaining, content.len() + 1),
        };
        let line = line.trim();
        if !line.is_empty()
            && !line.starts_with('#')
            && let Some((key, value)) = line.split_once('=')
        {
            env.insert(key.trim().to_string(), parse_env_value(value.trim()));
        }
        if next_start > content.len() {
            break;
        }
        start = next_start;
    }

    // Apply to process env
    for (key, value) in &env {
        if std::env::var(key).is_err() {
            // SAFETY: We are setting environment variables before any threads are spawned
            unsafe {
                std::env::set_var(key, value);
            }
        }
    }

    Ok(env)
}

/// Save environment updates to ~/.owly/.env
pub fn save_env(updates: &HashMap<String, String>) -> Result<()> {
    let mut current = load_env().unwrap_or_default();

    // Merge updates
    for (key, value) in updates {
        current.insert(key.clone(), value.clone());
    }

    let dir = env_dir();
    std::fs::create_dir_all(&dir).context("Failed to create env directory")?;

    // Write env file
    let lines: Vec<String> = current
        .iter()
        .map(|(k, v)| format!("{}={}", k, format_env_value(v)))
        .collect();

    std::fs::write(env_path(), lines.join("\n") + "\n")?;

    // Apply to process env
    for (key, value) in updates {
        // SAFETY: We are setting environment variables before any threads are spawned
        unsafe {
            std::env::set_var(key, value);
        }
    }

    Ok(())
}

/// Check if any API key is configured
pub fn has_any_api_key() -> bool {
    MANAGED_ENV_KEYS
        .iter()
        .filter(|k| k.ends_with("_API_KEY"))
        .any(|k| std::env::var(k).is_ok())
}

/// Parse environment value, handling quoted strings
pub fn parse_env_value(value: &str) -> String {
    if value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1]
            .replace("\\n", "\n")
            .replace("\\\"", "\"")
            .replace("\\\\", "\\")
    } else {
        value.to_string()
    }
}

/// Format environment value for storage
pub fn format_env_value(value: &str) -> String {
    format!(
        "\"{}\"",
        value.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n")
    )
}

fn env_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".owly")
}

fn env_path() -> PathBuf {
    env_dir().join(".env")
}
