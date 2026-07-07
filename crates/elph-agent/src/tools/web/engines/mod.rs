//! Search engine HTTP backends — one module per provider.

pub mod brave;
pub mod duckduckgo;
pub mod exa;
pub mod firecrawl;
pub mod jina;
pub mod perplexity;
pub mod serpapi;
pub mod tavily;

use reqwest::Client;

use super::ranking::{Engine, SearchResult};

pub use duckduckgo::parse_ddg_html;

/// Dispatch a search query to the appropriate engine backend.
pub async fn search_engine(
    client: &Client,
    engine: Engine,
    query: &str,
    api_key: &str,
) -> anyhow::Result<Vec<SearchResult>> {
    match engine {
        Engine::DuckDuckGo => duckduckgo::search(client, query).await,
        Engine::Brave => brave::search(client, query, api_key).await,
        Engine::Exa => exa::search(client, query, api_key).await,
        Engine::Firecrawl => firecrawl::search(client, query, api_key).await,
        Engine::Jina => jina::search(client, query, api_key).await,
        Engine::Perplexity => perplexity::search(client, query, api_key).await,
        Engine::Tavily => tavily::search(client, query, api_key).await,
        Engine::Serpapi => serpapi::search(client, query, api_key).await,
    }
}
