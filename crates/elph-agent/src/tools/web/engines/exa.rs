//! Exa search API — requires `EXA_API_KEY`.

use reqwest::Client;
use serde_json::json;

use super::super::common::{do_post_json, truncate_at_chars};
use super::super::ranking::SearchResult;

pub async fn search(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
    if api_key.is_empty() {
        return Err(anyhow::anyhow!("missing EXA_API_KEY"));
    }
    let body = json!({
        "query": query,
        "numResults": 10,
        "contents": { "highlights": true }
    });
    let val = do_post_json(client, "https://api.exa.ai/search", &[("x-api-key", api_key)], &body).await?;
    Ok(val["results"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|r| {
                    let mut title = r["title"].as_str().unwrap_or("").to_string();
                    let mut meta = Vec::new();
                    if let Some(date) = r["publishedDate"].as_str() {
                        meta.push(&date[..date.len().min(10)]);
                    }
                    if let Some(author) = r["author"].as_str() {
                        meta.push(author);
                    }
                    if !meta.is_empty() {
                        title = format!("{} ({})", title, meta.join(" · "));
                    }
                    let snippet = r["highlights"]
                        .as_array()
                        .and_then(|h| h.first())
                        .and_then(|v| v.as_str())
                        .or_else(|| r["text"].as_str())
                        .unwrap_or("");
                    let snippet = truncate_at_chars(snippet, 300);
                    SearchResult {
                        title,
                        url: r["url"].as_str().unwrap_or("").to_string(),
                        snippet,
                        content: None,
                    }
                })
                .collect()
        })
        .unwrap_or_default())
}
