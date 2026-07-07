//! Jina AI search — `JINA_API_KEY` optional.

use reqwest::Client;
use serde_json::Value;

use super::super::common::do_get;
use super::super::ranking::SearchResult;

pub async fn search(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
    let url = format!("https://s.jina.ai/{}", urlencoding::encode(query));
    let mut headers = vec![("Accept", "application/json")];
    let auth = if api_key.is_empty() {
        String::new()
    } else {
        format!("Bearer {}", api_key)
    };
    if !auth.is_empty() {
        headers.push(("Authorization", &auth));
    }
    let data = do_get(client, &url, &headers).await?;
    let val: Value = serde_json::from_str(&data)?;
    Ok(val["data"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|r| SearchResult {
                    title: r["title"].as_str().unwrap_or("").to_string(),
                    url: r["url"].as_str().unwrap_or("").to_string(),
                    snippet: r["snippet"].as_str().unwrap_or("").to_string(),
                    content: None,
                })
                .collect()
        })
        .unwrap_or_default())
}
