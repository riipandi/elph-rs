mod app;
mod cmd;

use clap::Parser;

fn main() {
    let cli = cmd::Cli::parse();

    if cli.version {
        std::process::exit(cmd::version::handle());
    }

    std::process::exit(cmd::run(&cli));
}