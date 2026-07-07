use serde_json::Value;

pub const MAX_PROVIDER_ERROR_BODY_CHARS: usize = 4000;

#[derive(Debug, Clone)]
pub struct NormalizedProviderError {
    pub status: Option<u16>,
    pub body: Option<String>,
    pub message: String,
    pub message_carries_body: bool,
}

pub fn normalize_provider_error(error: &anyhow::Error) -> NormalizedProviderError {
    let message = error.to_string();
    let status = error
        .downcast_ref::<reqwest::Error>()
        .and_then(|e| e.status())
        .map(|s| s.as_u16());

    NormalizedProviderError {
        status,
        body: None,
        message,
        message_carries_body: false,
    }
}

pub fn format_provider_error(norm: &NormalizedProviderError, prefix: Option<&str>) -> String {
    if norm.message_carries_body || norm.status.is_none() || norm.body.is_none() {
        if let (Some(prefix), Some(status)) = (prefix, norm.status) {
            return format!("{prefix} ({status}): {}", norm.message);
        }
        return norm.message.clone();
    }
    if let Some(prefix) = prefix {
        format!(
            "{prefix} ({}): {}",
            norm.status.unwrap_or(0),
            norm.body.as_deref().unwrap_or("")
        )
    } else {
        format!("{}: {}", norm.status.unwrap_or(0), norm.body.as_deref().unwrap_or(""))
    }
}

pub fn truncate_error_text(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }
    format!("{}... [truncated {} chars]", &text[..max_chars], text.len() - max_chars)
}

pub fn safe_json_stringify(value: &Value) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| value.to_string())
}

pub async fn error_body_from_response(response: reqwest::Response) -> String {
    let status = response.status();
    let text = response.text().await.unwrap_or_default();
    if text.trim().is_empty() {
        format!("{status}")
    } else {
        truncate_error_text(&text, MAX_PROVIDER_ERROR_BODY_CHARS)
    }
}
