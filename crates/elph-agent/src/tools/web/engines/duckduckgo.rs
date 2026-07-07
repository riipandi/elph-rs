//! DuckDuckGo HTML search — no API key required.

use std::sync::OnceLock;

use regex::Regex;
use reqwest::Client;

use super::super::common::{do_get, strip_html};
use super::super::ranking::SearchResult;

const USER_AGENT: &str =
    "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub async fn search(client: &Client, query: &str) -> anyhow::Result<Vec<SearchResult>> {
    let url = format!("https://html.duckduckgo.com/html/?q={}", urlencoding::encode(query));
    let html = do_get(client, &url, &[("User-Agent", USER_AGENT)]).await?;
    Ok(parse_ddg_html(&html))
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
