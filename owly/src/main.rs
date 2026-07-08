//! Owly - Agent docs for codebases
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki) (`openwiki`).
//! The original code is licensed under the [MIT License](https://opensource.org/licenses/MIT).
//! Copyright (c) 2026 LangChain.
//!
//! This Rust port preserves the core design (agent-driven documentation
//! generation with filesystem tools, git integration, and update metadata)
//! with platform-specific adaptations for the Elph agent runtime.
//!
//! Uses `elph-agent` and `elph-ai` for agent execution and LLM provider
//! integration instead of the original LangChain/LangGraph implementation.
//!
//! Configuration is explicit: use CLI flags or environment variables.
//! No hidden state is maintained outside the working directory.

use clap::Parser;
use tracing_subscriber::{EnvFilter, fmt};

use owly::cli::Cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let cli = Cli::parse();

    cli.execute().await
}
