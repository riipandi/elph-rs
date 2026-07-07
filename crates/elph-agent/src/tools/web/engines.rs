//! Search engine HTTP backends.

use regex::Regex;
use reqwest::Client;
use serde_json::{Value, json};
use std::sync::OnceLock;

use super::common::{do_get, do_post_json, strip_html};
use super::ranking::{Engine, SearchResult};

pub async fn search_engine(
    client: &Client,
    engine: Engine,
    query: &str,
    api_key: &str,
) -> anyhow::Result<Vec<SearchResult>> {
    match engine {
        Engine::DuckDuckGo => search_duckduckgo(client, query).await,
        Engine::Brave => search_brave(client, query, api_key).await,
        Engine::Exa => search_exa(client, query, api_key).await,
        Engine::Firecrawl => search_firecrawl(client, query, api_key).await,
        Engine::Jina => search_jina(client, query, api_key).await,
        Engine::Perplexity => search_perplexity(client, query, api_key).await,
        Engine::Tavily => search_tavily(client, query, api_key).await,
        Engine::Serpapi => search_serpapi(client, query, api_key).await,
    }
}

pub fn parse_ddg_html(html: &str) -> Vec<SearchResult> {
    static LINK_RE: OnceLock<Regex> = OnceLock::new();
    static SNIPPET_RE: OnceLock<Regex> = OnceLock::new();
    let link_re = LINK_RE.get_or_init(|| {
        Regex::new(r#"<a[^>]*class="result__a"[^>]*href="([^"]*)"[^>]*>([\s\S]*?)</a>"#).expect("ddg link regex")
    });
    let snippet_re = SNIPPET_RE.get_or_init(|| {
        Regex::new(r#"<a[^>]*class="result__snippet"[^>]*>([\s\S]*?)</a>"#).expect("ddg snippet regex")
    });

    let links: Vec<_> = link_re.captures_iter(html).collect();
    let snippets: Vec<_> = snippet_re.captures_iter(html).collect();
    let n = links.len().min(snippets.len());
    let mut results = Vec::with_capacity(n);
    for i in 0..n {
        let url = links[i].get(1).map(|m| m.as_str().to_string()).unwrap_or_default();
        let title = strip_html(links[i].get(2).map(|m| m.as_str()).unwrap_or(""));
        let snippet = strip_html(snippets[i].get(1).map(|m| m.as_str()).unwrap_or(""));
        if !url.is_empty() && !title.is_empty() {
            results.push(SearchResult {
                title,
                url,
                snippet,
                content: None,
            });
        }
    }
    results
}

async fn search_duckduckgo(client: &Client, query: &str) -> anyhow::Result<Vec<SearchResult>> {
    let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
    let html = do_get(
        client,
        &url,
        &[(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36",
        )],
    )
    .await?;
    Ok(parse_ddg_html(&html))
}

async fn search_brave(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
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

async fn search_exa(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
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
                    let snippet = if snippet.len() > 300 {
                        snippet[..300].to_string()
                    } else {
                        snippet.to_string()
                    };
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

async fn search_firecrawl(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
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

async fn search_jina(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
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

async fn search_perplexity(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
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
    if let Some(m) = json_re.find(content) {
        if let Ok(parsed) = serde_json::from_str::<Vec<SearchResult>>(m.as_str()) {
            if !parsed.is_empty() {
                return Ok(parsed);
            }
        }
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

async fn search_tavily(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
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

async fn search_serpapi(client: &Client, query: &str, api_key: &str) -> anyhow::Result<Vec<SearchResult>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ddg_results() {
        let html = r#"<a class="result__a" href="https://example.com">Example <b>Site</b></a>
<a class="result__snippet">A short <i>snippet</i> here</a>"#;
        let results = parse_ddg_html(html);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "Example Site");
        assert_eq!(results[0].url, "https://example.com");
        assert_eq!(results[0].snippet, "A short snippet here");
    }
}
