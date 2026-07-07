use std::collections::HashMap;

use crate::types::ProviderHeaders;

pub fn headers_to_record(headers: &reqwest::header::HeaderMap) -> HashMap<String, String> {
    let mut result = HashMap::new();
    for (key, value) in headers.iter() {
        if let Ok(v) = value.to_str() {
            result.insert(key.to_string(), v.to_string());
        }
    }
    result
}

pub fn provider_headers_to_record(headers: Option<&ProviderHeaders>) -> HashMap<String, String> {
    let mut result = HashMap::new();
    if let Some(headers) = headers {
        for (key, value) in headers {
            if let Some(v) = value {
                result.insert(key.clone(), v.clone());
            }
        }
    }
    result
}

pub fn merge_provider_headers(
    base: &HashMap<String, String>,
    extra: Option<&ProviderHeaders>,
) -> HashMap<String, String> {
    let mut result = base.clone();
    if let Some(extra) = extra {
        for (key, value) in extra {
            match value {
                Some(v) => {
                    result.insert(key.clone(), v.clone());
                }
                None => {
                    result.remove(key);
                }
            }
        }
    }
    result
}

pub fn has_header(headers: &HashMap<String, String>, name: &str) -> bool {
    let expected = name.to_lowercase();
    headers
        .iter()
        .any(|(k, v)| k.to_lowercase() == expected && !v.trim().is_empty())
}
