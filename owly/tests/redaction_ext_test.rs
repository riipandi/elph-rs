//! Extended tests for Owly redaction module.

use owly::diagnostics::*;

#[test]
fn test_sanitize_various_api_key_patterns() {
    let test_cases = vec![
        ("token: sk-abc123def456", "[REDACTED:API_KEY]"),
        ("key: sk-or-v1-xyz789", "[REDACTED:OPENROUTER_API_KEY]"),
        ("bearer: Bearer abc123def", "Bearer [REDACTED]"),
        ("langsmith: lsv2_abc123", "[REDACTED:LANGSMITH_API_KEY]"),
    ];

    for (input, expected) in test_cases {
        let result = sanitize_diagnostic_text(input);
        assert!(
            result.contains(expected),
            "Expected '{}' in result for input: {}",
            expected,
            input
        );
    }
}

#[test]
fn test_sanitize_multiple_secrets() {
    let input = "sk-abc123 and sk-or-v1-xyz789 and Bearer token123";
    let result = sanitize_diagnostic_text(input);

    assert!(!result.contains("sk-abc123"));
    assert!(!result.contains("sk-or-v1-xyz789"));
    assert!(!result.contains("Bearer token123"));
    assert!(result.contains("[REDACTED:API_KEY]"));
    assert!(result.contains("[REDACTED:OPENROUTER_API_KEY]"));
    assert!(result.contains("Bearer [REDACTED]"));
}

#[test]
fn test_sanitize_preserves_structure() {
    let input = "Error: Authentication failed with sk-abc123";
    let result = sanitize_diagnostic_text(input);

    assert!(result.contains("Error: Authentication failed with"));
    assert!(!result.contains("sk-abc123"));
}

#[test]
fn test_is_provider_server_error_variants() {
    let error500 = anyhow::anyhow!("500 Internal Server Error");
    let error400 = anyhow::anyhow!("400 Bad Request");

    assert!(is_provider_server_error(&error500, &format!("{error500}")));
    assert!(!is_provider_server_error(&error400, &format!("{error400}")));
}

#[test]
fn test_is_provider_server_error_message_check() {
    let error = anyhow::anyhow!("Provider error");
    let msg_with_500 = "500 Internal Server Error";
    let msg_without_500 = "400 Bad Request";

    assert!(is_provider_server_error(&error, msg_with_500));
    assert!(!is_provider_server_error(&error, msg_without_500));
}

#[test]
fn test_get_error_message_for_various_errors() {
    let error500 = anyhow::anyhow!("500 Internal Server Error");
    let msg500 = get_error_message(&error500);
    assert!(msg500.contains("500 Internal Server Error"));
    assert!(msg500.contains("--model"));

    let error_normal = anyhow::anyhow!("File not found");
    let msg_normal = get_error_message(&error_normal);
    assert!(msg_normal.contains("File not found"));
}

#[test]
fn test_sanitize_empty_string() {
    let result = sanitize_diagnostic_text("");
    assert_eq!(result, "");
}

#[test]
fn test_sanitize_long_text() {
    let input = "a".repeat(10000);
    let result = sanitize_diagnostic_text(&input);
    assert_eq!(result.len(), 10000);
}
