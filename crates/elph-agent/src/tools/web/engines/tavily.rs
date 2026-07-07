//! Tavily search API — requires `TAVILY_API_KEY`.

use reqwest::Client;
use serde_json::json;

use super::super::common::do_post_json;
use super::super::ranking::SearchResult;

pub async fn search(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
    if api_key.is_empty() {
        return Err(anyhow::anyhow!("missing TAVILY_API_KEY"));
    }
    let body = json!({
        "api_key": api_key,
        "query": query,
        "search_depth": "basic",
        "include_answer": false,
        "include_raw_content": false
    });
    let val = do_post_json(client, "https://api.tavily.com/search", &[], &body).await?;
    Ok(val["results"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|r| SearchResult {
                    title: r["title"].as_str().unwrap_or("").to_string(),
                    url: r["url"].as_str().unwrap_or("").to_string(),
                    snippet: r["content"].as_str().unwrap_or("").to_string(),
                    content: None,
                })
                .collect()
        })
        .unwrap_or_default())
}
