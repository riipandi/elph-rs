//! SerpAPI Google search — requires `SERPAPI_KEY`.

use reqwest::Client;
use serde_json::Value;

use super::super::common::do_get;
use super::super::ranking::SearchResult;

pub async fn search(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
    if api_key.is_empty() {
        return Err(anyhow::anyhow!("missing SERPAPI_KEY"));
    }
    let url = format!(
        "https://serpapi.com/search?q={}&api_key={}&engine=google",
        urlencoding::encode(query),
        urlencoding::encode(api_key)
    );
    let data = do_get(client, &url, &[]).await?;
    let val: Value = serde_json::from_str(&data)?;
    Ok(val["organic_results"]
        .as_array()
        .map(|arr| {
            arr.iter()
                .map(|r| SearchResult {
                    title: r["title"].as_str().unwrap_or("").to_string(),
                    url: r["link"].as_str().unwrap_or("").to_string(),
                    snippet: r["snippet"].as_str().unwrap_or("").to_string(),
                    content: None,
                })
                .collect()
        })
        .unwrap_or_default())
}
