//! CLI argument parsing and command routing.
//!
//! Terminal output lives in [`crate::ui`]; this module only parses flags and dispatches work.

mod help;
mod product;

use clap::{CommandFactory, Parser};
use std::path::PathBuf;

use crate::app::{Command, run_command};
use crate::wiki::mode::{RunMode, WikiContext};

pub use help::get_help_text;
pub use product::{CronAction, ProductCommand, execute as execute_product, parse_product_command};

pub const INTERACTIVE_NOT_YET_MESSAGE: &str = "Interactive mode not yet implemented";

/// Owly agent docs for codebases and personal knowledge wikis (OpenWiki port on elph-agent).
#[derive(Parser)]
#[command(
    name = "owly",
    about = "Owly — agent docs for codebases and personal wikis (elph-agent / elph-ai)",
    long_about = None,
    disable_help_flag = true,
    disable_help_subcommand = true,
)]
pub struct Cli {
    /// Run once and print the final assistant output
    #[arg(short, long)]
    pub print: bool,

    /// Use a model ID for this run (providerId/modelId); alias: --modelId
    #[arg(long, alias = "modelId", alias = "model-id")]
    pub model: Option<String>,

    /// Generate initial documentation
    #[arg(long)]
    pub init: bool,

    /// Update existing documentation
    #[arg(long)]
    pub update: bool,

    /// Product mode: code (repository openwiki/) or personal (~/.owly/wiki)
    #[arg(long, value_name = "MODE")]
    pub mode: Option<String>,

    /// Plan only: no LLM run and no wiki writes (prints what would happen)
    #[arg(long)]
    pub dry_run: bool,

    /// Print credential diagnostics (managed keys; secrets masked) and exit
    #[arg(long)]
    pub credentials: bool,

    /// Show stream response from LLM (without thinking)
    #[arg(short, long)]
    pub stream: bool,

    /// Show stream response and thinking from LLM
    #[arg(short, long)]
    pub verbose: bool,

    /// Message to send to the agent
    #[arg(trailing_var_arg = true)]
    pub message: Option<Vec<String>>,

    /// Working directory (defaults to current directory; code mode repository root)
    #[arg(short, long)]
    pub directory: Option<PathBuf>,

    /// Print help
    #[arg(short = 'h', long)]
    pub help: bool,
}

impl Cli {
    pub fn command() -> clap::Command {
        <Self as CommandFactory>::command().long_about(get_help_text())
    }

    pub async fn execute(mut self) -> anyhow::Result<()> {
        let mut message_parts = self.message.clone().unwrap_or_default();
        self.extract_trailing_flags(&mut message_parts)?;

        if self.help {
            println!("{}", get_help_text());
            return Ok(());
        }

        if self.is_bare_invocation() {
            println!("{INTERACTIVE_NOT_YET_MESSAGE}");
            return Ok(());
        }

        let cwd = self
            .directory
            .clone()
            .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

        if self.credentials {
            return crate::ui::credentials::print_credentials_diagnostics();
        }

        if let Some(product) = parse_product_command(&message_parts)? {
            return execute_product(product, self.effective_stream(), self.verbose).await;
        }

        let (run_mode, mode_source) = resolve_run_mode(self.mode.as_deref(), &mut message_parts)?;
        let ctx = match run_mode {
            RunMode::Code => WikiContext::code(&cwd),
            RunMode::Personal => WikiContext::personal(&cwd),
        };

        let command = resolve_command(self.init, self.update, self.print, &message_parts, run_mode, mode_source)?;

        run_command(
            command,
            &ctx,
            self.model.as_deref(),
            self.print,
            self.effective_stream(),
            self.verbose,
            self.dry_run,
        )
        .await
    }

    /// Stream LLM output to the terminal by default; `--print` disables unless `--stream` is set.
    pub fn effective_stream(&self) -> bool {
        if self.stream {
            return true;
        }
        !self.print
    }

    /// Recover flags captured into `message` after `personal`/`code` (clap `trailing_var_arg`).
    pub fn extract_trailing_flags(&mut self, message_parts: &mut Vec<String>) -> anyhow::Result<()> {
        let mut i = 0;
        while i < message_parts.len() {
            match message_parts[i].as_str() {
                "--init" => {
                    if self.init || self.update {
                        anyhow::bail!("Use either --init or --update, not both.");
                    }
                    self.init = true;
                    message_parts.remove(i);
                }
                "--update" => {
                    if self.init || self.update {
                        anyhow::bail!("Use either --init or --update, not both.");
                    }
                    self.update = true;
                    message_parts.remove(i);
                }
                "--dry-run" => {
                    self.dry_run = true;
                    message_parts.remove(i);
                }
                "--print" | "-p" => {
                    self.print = true;
                    message_parts.remove(i);
                }
                "--stream" | "-s" => {
                    self.stream = true;
                    message_parts.remove(i);
                }
                "--verbose" | "-v" => {
                    self.verbose = true;
                    message_parts.remove(i);
                }
                "--credentials" => {
                    self.credentials = true;
                    message_parts.remove(i);
                }
                "--help" | "-h" => {
                    self.help = true;
                    message_parts.remove(i);
                }
                "--mode" => {
                    let value = message_parts
                        .get(i + 1)
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!("--mode requires a value"))?;
                    self.mode = Some(value);
                    message_parts.remove(i + 1);
                    message_parts.remove(i);
                }
                "--model" | "--modelId" | "--model-id" => {
                    let value = message_parts
                        .get(i + 1)
                        .cloned()
                        .ok_or_else(|| anyhow::anyhow!("--model requires a value"))?;
                    self.model = Some(value);
                    message_parts.remove(i + 1);
                    message_parts.remove(i);
                }
                token if token.starts_with("--mode=") => {
                    self.mode = Some(
                        token
                            .split_once('=')
                            .map(|(_, value)| value.to_string())
                            .unwrap_or_default(),
                    );
                    message_parts.remove(i);
                }
                token
                    if {
                        token.starts_with("--model=")
                            || token.starts_with("--modelId=")
                            || token.starts_with("--model-id=")
                    } =>
                {
                    self.model = Some(
                        token
                            .split_once('=')
                            .map(|(_, value)| value.to_string())
                            .unwrap_or_default(),
                    );
                    message_parts.remove(i);
                }
                _ => i += 1,
            }
        }
        Ok(())
    }

    /// True only for `owly` with no flags and no trailing subcommands/args.
    pub fn is_bare_invocation(&self) -> bool {
        !self.print
            && self.model.is_none()
            && !self.init
            && !self.update
            && self.mode.is_none()
            && !self.dry_run
            && !self.credentials
            && !self.stream
            && !self.verbose
            && self.directory.is_none()
            && !self.help
            && self.message.as_ref().is_none_or(|parts| parts.is_empty())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModeSource {
    Default,
    Option,
    Positional,
}

fn resolve_run_mode(mode_flag: Option<&str>, message_parts: &mut Vec<String>) -> anyhow::Result<(RunMode, ModeSource)> {
    if let Some(flag) = mode_flag {
        let mode = RunMode::parse(flag)
            .ok_or_else(|| anyhow::anyhow!("Invalid --mode value `{flag}`. Expected `personal` or `code`."))?;
        return Ok((mode, ModeSource::Option));
    }

    if message_parts.first().is_some_and(|p| p.eq_ignore_ascii_case("code")) {
        message_parts.remove(0);
        return Ok((RunMode::Code, ModeSource::Positional));
    }
    if message_parts
        .first()
        .is_some_and(|p| p.eq_ignore_ascii_case("personal"))
    {
        message_parts.remove(0);
        return Ok((RunMode::Personal, ModeSource::Positional));
    }

    Ok((RunMode::Personal, ModeSource::Default))
}

fn resolve_command(
    init: bool,
    update: bool,
    print_mode: bool,
    message_parts: &[String],
    run_mode: RunMode,
    mode_source: ModeSource,
) -> anyhow::Result<Command> {
    if init && update {
        anyhow::bail!("Use either --init or --update, not both.");
    }

    if init && mode_source == ModeSource::Default {
        anyhow::bail!(
            "owly --init requires a mode.\n\nRun one of:\n  \
             owly personal --init  Build your local personal brain wiki in ~/.owly/wiki.\n  \
             owly code --init      Build repository documentation in ./openwiki."
        );
    }

    if init {
        return Ok(Command::Init);
    }
    if update {
        return Ok(Command::Update);
    }

    if !message_parts.is_empty() {
        let msg = message_parts.join(" ");
        return Ok(Command::Chat { message: Some(msg) });
    }

    if print_mode {
        anyhow::bail!("-p, --print requires a message, --init, or --update.");
    }

    if run_mode == RunMode::Personal || mode_source == ModeSource::Default {
        return Ok(Command::Chat { message: None });
    }

    Ok(Command::Chat { message: None })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_mode_is_personal() {
        let mut parts = vec!["hello".to_string()];
        let (mode, _) = resolve_run_mode(None, &mut parts).unwrap();
        assert_eq!(mode, RunMode::Personal);
    }

    #[test]
    fn init_without_mode_errors() {
        let err = resolve_command(true, false, false, &[], RunMode::Personal, ModeSource::Default);
        assert!(err.is_err());
    }

    #[test]
    fn code_positional_sets_mode() {
        let mut parts = vec!["code".to_string()];
        let (mode, src) = resolve_run_mode(None, &mut parts).unwrap();
        assert_eq!(mode, RunMode::Code);
        assert_eq!(src, ModeSource::Positional);
        assert!(parts.is_empty());
    }

    fn bare_cli() -> Cli {
        Cli {
            print: false,
            model: None,
            init: false,
            update: false,
            mode: None,
            dry_run: false,
            credentials: false,
            stream: false,
            verbose: false,
            message: None,
            directory: None,
            help: false,
        }
    }

    #[test]
    fn effective_stream_defaults_on_without_print() {
        assert!(bare_cli().effective_stream());
        assert!(
            !Cli {
                print: true,
                ..bare_cli()
            }
            .effective_stream()
        );
        assert!(
            Cli {
                print: true,
                stream: true,
                ..bare_cli()
            }
            .effective_stream()
        );
    }

    #[test]
    fn trailing_flags_after_personal_are_recovered() {
        let mut cli = Cli {
            message: Some(vec!["personal".into(), "--init".into(), "--dry-run".into()]),
            ..bare_cli()
        };
        let mut parts = cli.message.clone().unwrap();
        cli.extract_trailing_flags(&mut parts).unwrap();
        assert!(cli.init);
        assert!(cli.dry_run);
        assert_eq!(parts, vec!["personal"]);
    }

    #[test]
    fn trailing_flags_after_code_update() {
        let mut cli = Cli {
            message: Some(vec!["code".into(), "--update".into(), "-p".into()]),
            ..bare_cli()
        };
        let mut parts = cli.message.clone().unwrap();
        cli.extract_trailing_flags(&mut parts).unwrap();
        assert!(cli.update);
        assert!(cli.print);
        assert_eq!(parts, vec!["code"]);
    }

    #[test]
    fn trailing_init_and_update_conflict() {
        let mut cli = Cli {
            message: Some(vec!["--init".into(), "--update".into()]),
            ..bare_cli()
        };
        let mut parts = cli.message.clone().unwrap();
        assert!(cli.extract_trailing_flags(&mut parts).is_err());
    }

    #[test]
    fn bare_invocation_requires_no_flags_or_args() {
        assert_eq!(INTERACTIVE_NOT_YET_MESSAGE, "Interactive mode not yet implemented");
        assert!(bare_cli().is_bare_invocation());
        assert!(
            !Cli {
                stream: true,
                ..bare_cli()
            }
            .is_bare_invocation()
        );
        assert!(
            !Cli {
                message: Some(vec!["hi".into()]),
                ..bare_cli()
            }
            .is_bare_invocation()
        );
        assert!(
            !Cli {
                init: true,
                ..bare_cli()
            }
            .is_bare_invocation()
        );
    }
}
