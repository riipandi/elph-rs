//! Application use-cases: init, update, chat, ingest, and cron.

pub mod cron;
pub mod doc_run;
pub mod ingest;
mod non_interactive;
mod provider_setup;

pub use doc_run::{apply_doc_run_result, run_init_agent, run_update_agent, should_skip_update_noop};
pub use ingest::{IngestionTarget, parse_ingestion_target, run_ingestion};
pub use provider_setup::ensure_provider_setup;

use anyhow::Result;

use crate::runtime::config::Config;
use crate::runtime::credentials;
use crate::runtime::env;
use crate::runtime::startup;
use crate::wiki::mode::WikiContext;

/// Available commands
#[derive(Debug)]
pub enum Command {
    /// Initialize documentation
    Init,

    /// Update existing documentation
    Update,

    /// Chat message
    Chat { message: Option<String> },
}

/// Run a command
pub async fn run_command(
    command: Command,
    ctx: &WikiContext,
    model_override: Option<&str>,
    print_mode: bool,
    stream: bool,
    verbose: bool,
    dry_run: bool,
) -> Result<()> {
    ctx.ensure_layout()?;
    credentials::load_env()?;
    let config = Config::resolve(model_override, &ctx.repo_cwd)?;

    if dry_run {
        return non_interactive::run_dry_run(&config, ctx, &command);
    }

    let config = provider_setup::ensure_provider_setup(config).await?;
    startup::validate_non_interactive(&command, ctx)?;
    env::setup_environment(&config)?;
    non_interactive::run_non_interactive(&config, ctx, command, print_mode, stream, verbose).await
}
