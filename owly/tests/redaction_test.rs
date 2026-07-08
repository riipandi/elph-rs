//! Tests for Owly diagnostics module.
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki)
//! `test/redaction.test.ts`. Original MIT License, Copyright (c) 2026 LangChain.

use owly::diagnostics::{get_error_message, sanitize_diagnostic_text};

#[test]
fn test_sanitize_redacts_exact_env_value() {
    // Set a test API key in the environment using a real API key env var
    // SAFETY: We're setting env vars in a single-threaded test context
    unsafe {
        std::env::set_var("OPENAI_API_KEY", "super-secret-value-12345");
    }

    let input = "request failed with key super-secret-value-12345 attached";
    let result = sanitize_diagnostic_text(input);

    // The result should not contain the secret
    assert!(!result.contains("super-secret-value-12345"));
    assert!(result.contains("[REDACTED:OPENAI_API_KEY]"));

    // Clean up
    // SAFETY: We're removing env vars in a single-threaded test context
    unsafe {
        std::env::remove_var("OPENAI_API_KEY");
    }
}

#[test]
fn test_sanitize_redacts_sk_tokens() {
    let result = sanitize_diagnostic_text("token sk-abcDEF123_456 rejected");
    assert!(!result.contains("sk-abcDEF123_456"));
    assert!(result.contains("[REDACTED:API_KEY]"));
}

#[test]
fn test_sanitize_redacts_openrouter_tokens() {
    let result = sanitize_diagnostic_text("using sk-or-v1-deadbeef00 now");
    assert!(!result.contains("sk-or-v1-deadbeef00"));
    assert!(result.contains("[REDACTED:OPENROUTER_API_KEY]"));
}

#[test]
fn test_sanitize_redacts_bearer_tokens() {
    let result = sanitize_diagnostic_text("Authorization: Bearer eyJhbGciOi.J9.abc-123");
    assert!(!result.contains("eyJhbGciOi.J9.abc-123"));
    assert!(result.contains("[REDACTED]"));
}

#[test]
fn test_sanitize_redacts_langsmith_keys() {
    let result = sanitize_diagnostic_text("langsmith lsv_1234abcd tracing");
    assert!(!result.contains("lsv_1234abcd"));
    assert!(result.contains("[REDACTED:LANGSMITH_API_KEY]"));
}

#[test]
fn test_sanitize_redacts_incorrect_api_key_phrase() {
    let result = sanitize_diagnostic_text("Incorrect API key provided: myLeakedKey. Check your account.");
    assert!(!result.contains("myLeakedKey"));
    assert!(result.contains("[REDACTED:API_KEY]"));
}

#[test]
fn test_sanitize_leaves_non_secret_text_untouched() {
    let message = "Repository has 12 files and the wiki is already current.";
    assert_eq!(sanitize_diagnostic_text(message), message);
}

#[test]
fn test_get_error_message_provider_500() {
    let error = anyhow::anyhow!("500 Internal Server Error");
    let message = get_error_message(&error);
    assert!(message.contains("500 Internal Server Error"));
    assert!(message.contains("--model"));
}

#[test]
fn test_get_error_message_redacts_secrets() {
    let error = anyhow::anyhow!("bad token sk-abcDEF123");
    let message = get_error_message(&error);
    assert!(message.contains("[REDACTED:API_KEY]"));
}
