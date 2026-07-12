//! Ensure provider credentials are configured before a run.

use anyhow::{Context, Result};

use crate::runtime::config::Config;
use crate::runtime::startup::stdin_is_tty;
use crate::setup::onboarding;

/// Run the dialoguer onboarding wizard when credentials are missing (TTY only).
pub async fn ensure_provider_setup(config: Config) -> Result<Config> {
    if !onboarding::needs_setup(&config) {
        return Ok(config);
    }
    if !stdin_is_tty() {
        anyhow::bail!(
            "Provider credentials are not configured. Run Owly in an interactive terminal to complete setup, \
             or set API keys / OAuth tokens in ~/.owly/.env."
        );
    }

    tokio::task::spawn_blocking(move || {
        let mut config = config;
        onboarding::run_wizard(&mut config).context("credential setup cancelled")?;
        Ok(config)
    })
    .await
    .context("credential setup interrupted")?
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::config::Config;
    use std::path::PathBuf;

    #[test]
    fn ensure_setup_skips_when_configured() {
        let config = Config {
            provider: "opencode".to_string(),
            model_id: "big-pickle".to_string(),
            cwd: PathBuf::from("/tmp"),
        };
        if onboarding::needs_setup(&config) {
            return;
        }
        let rt = tokio::runtime::Runtime::new().unwrap();
        let out = rt.block_on(ensure_provider_setup(config)).unwrap();
        assert_eq!(out.provider, "opencode");
    }
}
