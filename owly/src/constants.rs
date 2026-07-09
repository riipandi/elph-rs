//! Constants for Owly.
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki)
//! `src/constants.ts`. Original MIT License, Copyright (c) 2026 LangChain.
//!
//! Extended to support all providers available in `elph-ai`.

/// The directory where documentation is stored
pub const OWLY_DIR: &str = "openwiki";

/// Path to the last update metadata file
pub const UPDATE_METADATA_PATH: &str = "openwiki/.last-update.json";

/// Owly version
pub const OWLY_VERSION: &str = "0.0.1";

/// Environment variable keys
pub const OWLY_PROVIDER_ENV_KEY: &str = "OWLY_PROVIDER";
pub const OWLY_MODEL_ID_ENV_KEY: &str = "OWLY_MODEL_ID";

/// Default provider
pub const DEFAULT_PROVIDER: &str = "opencode";

/// Default model ID (OpenCode Zen big-pickle)
pub const DEFAULT_MODEL_ID: &str = "big-pickle";

/// Environment variable for optional provider base URL override.
pub const ANTHROPIC_BASE_URL_ENV_KEY: &str = "ANTHROPIC_BASE_URL";
pub const OPENAI_BASE_URL_ENV_KEY: &str = "OPENAI_BASE_URL";
pub const OPENROUTER_BASE_URL_ENV_KEY: &str = "OPENROUTER_BASE_URL";
pub const OPENAI_COMPATIBLE_API_KEY_ENV_KEY: &str = "OPENAI_COMPATIBLE_API_KEY";
pub const OPENAI_COMPATIBLE_BASE_URL_ENV_KEY: &str = "OPENAI_COMPATIBLE_BASE_URL";

/// Providers offered in the first-run onboarding wizard.
pub const ONBOARDING_PROVIDERS: &[&str] = &[
    "opencode",
    "openrouter",
    "anthropic",
    "openai",
    "google",
    "deepseek",
    "groq",
    "fireworks",
];

/// Provider configuration with API key environment variable
#[derive(Debug, Clone)]
pub struct ProviderConfig {
    pub label: &'static str,
    pub api_key_env_key: &'static str,
    pub default_model: &'static str,
    pub base_url_env_key: Option<&'static str>,
    pub requires_base_url: bool,
}

const fn provider_defaults(
    label: &'static str,
    api_key_env_key: &'static str,
    default_model: &'static str,
) -> ProviderConfig {
    ProviderConfig {
        label,
        api_key_env_key,
        default_model,
        base_url_env_key: None,
        requires_base_url: false,
    }
}

/// All supported providers from elph-ai
pub fn provider_config(provider: &str) -> Option<ProviderConfig> {
    match provider {
        "opencode" => Some(ProviderConfig {
            label: "OpenCode Zen",
            api_key_env_key: "OPENCODE_API_KEY",
            default_model: "big-pickle",
            base_url_env_key: None,
            requires_base_url: false,
        }),
        "opencode-go" => Some(ProviderConfig {
            label: "OpenCode Go",
            api_key_env_key: "OPENCODE_API_KEY",
            default_model: "big-pickle",
            base_url_env_key: None,
            requires_base_url: false,
        }),
        "anthropic" => Some(ProviderConfig {
            label: "Anthropic",
            api_key_env_key: "ANTHROPIC_API_KEY",
            default_model: "claude-sonnet-5",
            base_url_env_key: Some(ANTHROPIC_BASE_URL_ENV_KEY),
            requires_base_url: false,
        }),
        "openai" => Some(ProviderConfig {
            label: "OpenAI",
            api_key_env_key: "OPENAI_API_KEY",
            default_model: "gpt-5.4-mini",
            base_url_env_key: Some(OPENAI_BASE_URL_ENV_KEY),
            requires_base_url: false,
        }),
        "openai-compatible" => Some(ProviderConfig {
            label: "OpenAI-compatible",
            api_key_env_key: OPENAI_COMPATIBLE_API_KEY_ENV_KEY,
            default_model: "gpt-4o-mini",
            base_url_env_key: Some(OPENAI_COMPATIBLE_BASE_URL_ENV_KEY),
            requires_base_url: true,
        }),
        "openrouter" => Some(ProviderConfig {
            label: "OpenRouter",
            api_key_env_key: "OPENROUTER_API_KEY",
            default_model: "z-ai/glm-5.2",
            base_url_env_key: Some(OPENROUTER_BASE_URL_ENV_KEY),
            requires_base_url: false,
        }),
        "google" => Some(ProviderConfig {
            label: "Google",
            api_key_env_key: "GOOGLE_API_KEY",
            default_model: "gemini-2.5-flash",
            base_url_env_key: None,
            requires_base_url: false,
        }),
        "google-vertex" => Some(provider_defaults(
            "Google Vertex",
            "GOOGLE_APPLICATION_CREDENTIALS",
            "gemini-2.5-flash",
        )),
        "deepseek" => Some(provider_defaults("DeepSeek", "DEEPSEEK_API_KEY", "deepseek-chat")),
        "xai" => Some(provider_defaults("xAI", "XAI_API_KEY", "grok-2")),
        "groq" => Some(provider_defaults("Groq", "GROQ_API_KEY", "llama-3.3-70b-versatile")),
        "fireworks" => Some(provider_defaults(
            "Fireworks",
            "FIREWORKS_API_KEY",
            "accounts/fireworks/models/glm-5p2",
        )),
        "together" => Some(provider_defaults(
            "Together",
            "TOGETHER_API_KEY",
            "meta-llama/Llama-3.3-70B-Instruct-Turbo",
        )),
        "mistral" => Some(provider_defaults("Mistral", "MISTRAL_API_KEY", "mistral-large-latest")),
        "nvidia" => Some(provider_defaults(
            "NVIDIA",
            "NVIDIA_API_KEY",
            "meta/llama-3.3-70b-instruct",
        )),
        "cerebras" => Some(provider_defaults("Cerebras", "CEREBRAS_API_KEY", "llama-3.3-70b")),
        "amazon-bedrock" => Some(provider_defaults(
            "Amazon Bedrock",
            "AWS_ACCESS_KEY_ID",
            "anthropic.claude-3-5-sonnet-20241022-v2:0",
        )),
        "github-copilot" => Some(provider_defaults("GitHub Copilot", "GITHUB_TOKEN", "gpt-4o")),
        "cloudflare-workers-ai" => Some(provider_defaults(
            "Cloudflare Workers AI",
            "CLOUDFLARE_API_TOKEN",
            "@cf/meta/llama-3.3-70b-instruct-fp8",
        )),
        "cloudflare-ai-gateway" => Some(provider_defaults(
            "Cloudflare AI Gateway",
            "CLOUDFLARE_API_TOKEN",
            "@cf/meta/llama-3.3-70b-instruct-fp8",
        )),
        "huggingface" => Some(provider_defaults(
            "Hugging Face",
            "HF_TOKEN",
            "meta-llama/Llama-3.3-70B-Instruct",
        )),
        "moonshotai" => Some(provider_defaults("MoonshotAI", "MOONSHOT_API_KEY", "moonshot-v1-auto")),
        "zai" => Some(provider_defaults("Z.AI", "ZAI_API_KEY", "glm-5.2")),
        "xiaomi" => Some(provider_defaults("Xiaomi", "XIAOMI_API_KEY", "MiLM-7B-Chat")),
        "minimax" => Some(provider_defaults("MiniMax", "MINIMAX_API_KEY", "abab6.5s-chat")),
        "ant-ling" => Some(provider_defaults("Ant Ling", "ANT_LING_API_KEY", "qwen-72b-chat")),
        _ => None,
    }
}

/// Get all supported provider IDs
pub fn all_providers() -> Vec<&'static str> {
    vec![
        "opencode",
        "opencode-go",
        "anthropic",
        "openai",
        "openai-compatible",
        "openrouter",
        "google",
        "google-vertex",
        "deepseek",
        "xai",
        "groq",
        "fireworks",
        "together",
        "mistral",
        "nvidia",
        "cerebras",
        "amazon-bedrock",
        "github-copilot",
        "cloudflare-workers-ai",
        "cloudflare-ai-gateway",
        "huggingface",
        "moonshotai",
        "zai",
        "xiaomi",
        "minimax",
        "ant-ling",
    ]
}

/// Resolve provider from environment
pub fn resolve_configured_provider() -> &'static str {
    // Check OWLY_PROVIDER env var first
    if let Ok(provider) = std::env::var(OWLY_PROVIDER_ENV_KEY)
        && provider_config(&provider).is_some()
    {
        return Box::leak(provider.into_boxed_str());
    }

    // Auto-detect based on available API keys
    // Check if env var is set AND non-empty
    if std::env::var("OPENCODE_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())
        .is_some()
    {
        return "opencode";
    }
    if std::env::var("ANTHROPIC_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())
        .is_some()
    {
        return "anthropic";
    }
    if std::env::var("OPENAI_API_KEY").ok().filter(|v| !v.is_empty()).is_some() {
        return "openai";
    }
    if std::env::var("OPENROUTER_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())
        .is_some()
    {
        return "openrouter";
    }
    if std::env::var("GOOGLE_API_KEY").ok().filter(|v| !v.is_empty()).is_some() {
        return "google";
    }
    if std::env::var("DEEPSEEK_API_KEY")
        .ok()
        .filter(|v| !v.is_empty())
        .is_some()
    {
        return "deepseek";
    }
    if std::env::var("GROQ_API_KEY").ok().filter(|v| !v.is_empty()).is_some() {
        return "groq";
    }

    DEFAULT_PROVIDER
}
/// Resolve model ID from environment or config
pub fn resolve_model_id(model_override: Option<&str>) -> String {
    // CLI override takes precedence
    if let Some(model) = model_override {
        return model.to_string();
    }

    // Check OWLY_MODEL_ID env var
    if let Ok(model) = std::env::var(OWLY_MODEL_ID_ENV_KEY)
        && !model.trim().is_empty()
    {
        return model;
    }

    // Use provider default
    let provider = resolve_configured_provider();
    provider_config(provider)
        .map(|c| c.default_model.to_string())
        .unwrap_or_else(|| DEFAULT_MODEL_ID.to_string())
}

/// Check if a provider needs an API key
pub fn provider_needs_api_key(provider: &str) -> bool {
    provider_config(provider)
        .map(|c| std::env::var(c.api_key_env_key).is_err())
        .unwrap_or(true)
}

/// Get API key for a provider
pub fn get_provider_api_key(provider: &str) -> Option<String> {
    provider_config(provider).and_then(|c| std::env::var(c.api_key_env_key).ok())
}

/// Normalize a model id from user input.
pub fn normalize_model_id(value: &str) -> String {
    value.trim().to_string()
}

/// Validate model id format (OpenWiki-compatible rules).
pub fn is_valid_model_id(value: &str) -> bool {
    let model_id = normalize_model_id(value);
    !model_id.is_empty()
        && model_id.len() <= 120
        && model_id.chars().next().is_some_and(|c| c.is_ascii_alphanumeric())
        && !model_id.contains("://")
        && model_id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '/' | '+' | '-'))
}

/// Resolve optional provider base URL from environment.
pub fn resolve_provider_base_url(provider: &str) -> Option<String> {
    let cfg = provider_config(provider)?;
    let key = cfg.base_url_env_key?;
    std::env::var(key).ok().filter(|v| !v.trim().is_empty())
}

pub fn provider_requires_base_url(provider: &str) -> bool {
    provider_config(provider).map(|c| c.requires_base_url).unwrap_or(false)
}
