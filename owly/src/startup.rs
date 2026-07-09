//! Startup command resolution and TTY validation.

use std::io::IsTerminal;
use std::path::Path;

use crate::commands::Command;
use crate::constants::provider_config;
use crate::credentials;
use crate::metadata;

/// Resolved startup mode for the CLI.
#[derive(Debug, Clone)]
pub enum StartupMode {
    /// Run once and exit (`--print` or piped input).
    NonInteractive,
    /// Stay open for follow-up messages (OpenWiki default).
    Interactive { initial: Option<InitialRun> },
}

#[derive(Debug, Clone)]
pub enum InitialRun {
    Init,
    Update,
    Chat { message: String },
}

/// Decide whether Owly should stay interactive after the first command.
pub fn resolve_startup_mode(command: &Command, print_mode: bool) -> StartupMode {
    if print_mode {
        return StartupMode::NonInteractive;
    }

    let initial = match command {
        Command::Init => Some(InitialRun::Init),
        Command::Update => Some(InitialRun::Update),
        Command::Chat { message: Some(msg) } => Some(InitialRun::Chat { message: msg.clone() }),
        Command::Chat { message: None } => None,
    };

    StartupMode::Interactive { initial }
}

/// Validate that non-interactive invocations have credentials and input.
pub fn validate_non_interactive(command: &Command, cwd: &Path) -> anyhow::Result<()> {
    if std::io::stdin().is_terminal() {
        return Ok(());
    }

    match command {
        Command::Chat { message: None } => {
            anyhow::bail!(
                "Interactive chat requires a terminal. Pass a message or use --init or --update for non-interactive runs."
            );
        }
        Command::Init | Command::Update => {}
        Command::Chat { message: Some(msg) } if msg.trim().is_empty() => {
            anyhow::bail!("User message cannot be empty.");
        }
        Command::Chat { message: Some(_) } => {}
    }

    if can_skip_credentials_for_clean_update(command, cwd) {
        return Ok(());
    }

    if !credentials::has_provider_api_key_for_any() {
        anyhow::bail!(
            "An API key is required for non-interactive runs. Run owly in an interactive terminal to save credentials."
        );
    }

    Ok(())
}

fn can_skip_credentials_for_clean_update(command: &Command, cwd: &Path) -> bool {
    matches!(command, Command::Update) && metadata::is_update_noop(cwd) && !credentials::has_provider_api_key_for_any()
}

/// Returns `true` when stdin is a TTY (safe for dialoguer prompts).
pub fn stdin_is_tty() -> bool {
    std::io::stdin().is_terminal()
}

/// Check whether the configured provider has an API key in the environment.
pub fn provider_has_api_key(provider: &str) -> bool {
    provider_config(provider)
        .and_then(|cfg| std::env::var(cfg.api_key_env_key).ok())
        .filter(|v| !v.trim().is_empty())
        .is_some()
}
