//! Firecrawl search API — keyless (`FIRECRAWL_API_KEY` optional).

use reqwest::Client;
use serde_json::json;

use super::super::common::do_post_json;
use super::super::ranking::SearchResult;

pub async fn search(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
    let body = json!({ "query": query, "limit": 10 });
    let mut headers = vec![("Content-Type", "application/json")];
    let auth = if api_key.is_empty() {
        String::new()
    } else {
        format!("Bearer {}", api_key)
    };
    if !auth.is_empty() {
        headers.push(("Authorization", &auth));
    }
    let val = do_post_json(client, "https://api.firecrawl.dev/v2/search", &headers, &body).await?;

    let success = val["success"].as_bool().unwrap_or(false);
    if !success {
        return Err(anyhow::anyhow!("firecrawl: search unsuccessful"));
    }

    Ok(val["data"]["web"]
        .as_array()
        .or_else(|| val["data"].as_array())
        .map(|arr| {
            arr.iter()
                .map(|r| SearchResult {
                    title: r["title"].as_str().unwrap_or("").to_string(),
                    url: r["url"].as_str().unwrap_or("").to_string(),
                    snippet: r["description"]
                        .as_str()
                        .or_else(|| r["snippet"].as_str())
                        .unwrap_or("")
                        .to_string(),
                    content: r["markdown"].as_str().map(str::to_string),
                })
                .collect()
        })
        .unwrap_or_default())
}
