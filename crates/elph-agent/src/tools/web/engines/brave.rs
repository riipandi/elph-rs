//! Brave Search API — requires `BRAVE_SEARCH_API_KEY`.

use reqwest::Client;
use serde_json::Value;

use super::super::common::do_get;
use super::super::ranking::SearchResult;

pub async fn search(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
    if api_key.is_empty() {
        return Err(anyhow::anyhow!("missing BRAVE_SEARCH_API_KEY"));
    }
    let url = format!(
        "https://api.search.brave.com/res/v1/web/search?q={}",
        urlencoding::encode(query)
    );
    let data = do_get(
        client,
        &url,
        &[("Accept", "application/json"), ("X-Subscription-Token", api_key)],
    )
    .await?;
    let val: Value = serde_json::from_str(&data)?;
    Ok(val["web"]["results"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|r| SearchResult {
                    title: r["title"].as_str().unwrap_or("").to_string(),
                    url: r["url"].as_str().unwrap_or("").to_string(),
                    snippet: r["description"].as_str().unwrap_or("").to_string(),
                    content: None,
                })
                .collect()
        })
        .unwrap_or_default())
}
