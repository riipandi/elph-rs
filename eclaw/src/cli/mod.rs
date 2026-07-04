mod default;
pub mod version;

use crate::runtime::ExitCode;
use clap::{Parser, Subcommand};

/// Personal AI assistant powered by Elph
#[derive(Parser)]
#[command(name = "eclaw", about, disable_version_flag = true)]
pub struct Cli {
    /// Print version information
    #[arg(short = 'V', long = "version", help = "Print version information")]
    pub version: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Print version information
    Version,
}

pub fn run(cli: &Cli) -> ExitCode {
    if let Err(err) = crate::layout::ensure_blocking(env!("CARGO_PKG_VERSION")) {
        eprintln!("failed to initialize eclaw home: {err}");
        return crate::runtime::EXIT_ERROR;
    }

    match &cli.command {
        None => default::handle(),
        Some(Commands::Version) => version::handle(),
    }
}
