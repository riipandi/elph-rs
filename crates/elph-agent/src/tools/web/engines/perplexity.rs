//! Perplexity Sonar search — requires `PERPLEXITY_API_KEY`.

use std::sync::OnceLock;

use regex::Regex;
use reqwest::Client;
use serde_json::json;

use super::super::common::do_post_json;
use super::super::ranking::SearchResult;

pub async fn search(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
    if api_key.is_empty() {
        return Err(anyhow::anyhow!("missing PERPLEXITY_API_KEY"));
    }
    let body = json!({
        "model": "sonar",
        "messages": [
            {
                "role": "system",
                "content": "You are a search assistant. Return search results as a JSON array with objects containing title, url, and snippet fields."
            },
            { "role": "user", "content": query }
        ]
    });
    let auth = format!("Bearer {}", api_key);
    let val = do_post_json(
        client,
        "https://api.perplexity.ai/chat/completions",
        &[("Authorization", &auth)],
        &body,
    )
    .await?;

    let content = val["choices"][0]["message"]["content"].as_str().unwrap_or("");
    static JSON_RE: OnceLock<Regex> = OnceLock::new();
    let json_re = JSON_RE.get_or_init(|| Regex::new(r"\[[\s\S]*\]").expect("json array regex"));
    if let Some(m) = json_re.find(content)
        && let Ok(parsed) = serde_json::from_str::<Vec<SearchResult>>(m.as_str())
        && !parsed.is_empty()
    {
        return Ok(parsed);
    }

    if let Some(cits) = val["choices"][0]["citations"].as_array() {
        let results: Vec<SearchResult> = cits
            .iter()
            .enumerate()
            .map(|(i, c)| SearchResult {
                title: format!("Result {}", i + 1),
                url: c["url"].as_str().unwrap_or("").to_string(),
                snippet: content.trim().to_string(),
                content: None,
            })
            .collect();
        if !results.is_empty() {
            return Ok(results);
        }
    }
    Err(anyhow::anyhow!("perplexity: no parseable results"))
}
