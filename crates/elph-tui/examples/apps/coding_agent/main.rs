//! Coding agent — full AI chat TUI simulator.
//!
//! Four-zone shell matching production `chat_layout` / elph agent (header · transcript ·
//! status · prompt), slash command palette, and dialog overlays (mode, model, confirm, goals,
//! progress). Uses placeholder transcript data and simulated turns.
//!
//! ```bash
//! cargo run -p elph-tui --example coding_agent
//! ```
//!
//! Type `/` for the slash palette (`/demo-mode`, `/demo-multi`, `/demo-todo`, `/demo-tool`, …)
//! `Ctrl+M` mode · `Ctrl+L` model · `Ctrl+G` goals · `Ctrl+P` progress · `Esc` dismiss · `Ctrl+D` quit

mod app;
mod overlays;
mod seed;
mod shell;

#[path = "../../common/mod.rs"]
mod common;

use anyhow::Result;
use app::CodingAgent;
use common::run::run_fullscreen;
use iocraft::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    run_fullscreen(element!(CodingAgent)).await
}
