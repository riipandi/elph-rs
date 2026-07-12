//! Owly - Agent docs for codebases
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki)
//! (`openwiki`). The original code is licensed under the
//! [MIT License](https://opensource.org/licenses/MIT).
//! Copyright (c) 2026 LangChain.
//!
//! ## Layout
//!
//! - [`cli`] — argument parsing and routing
//! - [`ui`] — terminal output and stream rendering
//! - [`app`] — use-case orchestration (init, update, chat, ingest, cron)
//! - [`wiki`] — documentation domain (prompts, metadata, mode context)
//! - [`agent`] — elph-agent integration
//! - [`connectors`] — ingestion source connectors
//! - [`setup`] — first-run wizard and connector auth
//! - [`runtime`] — config, credentials, checkpoint/session persistence

pub mod agent;
pub mod app;
pub mod cli;
pub mod connectors;
pub mod runtime;
pub mod setup;
pub mod ui;
pub mod wiki;

// Stable re-exports for integration tests and downstream callers.
pub use runtime::ask_user;
pub use runtime::checkpoint;
pub use runtime::config;
pub use runtime::constants;
pub use runtime::credentials;
pub use runtime::diagnostics;
pub use runtime::env;
pub use runtime::session;
pub use runtime::startup;
pub use runtime::utils;
pub use setup::auth;
pub use setup::onboarding;
pub use setup::onboarding_config;
pub use wiki::code_mode;
pub use wiki::docs;
pub use wiki::ecosystem;
pub use wiki::frontmatter;
pub use wiki::instructions;
pub use wiki::metadata;
pub use wiki::mode;
pub use wiki::prompts;
