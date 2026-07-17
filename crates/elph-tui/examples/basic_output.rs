//! TUI demo - basic use output
//!
//! ```bash
//! cargo run -p elph-tui --example basic_output
//! ```

use anyhow::Result;
use iocraft::prelude::*;
use std::time::Duration;

#[component]
fn Example(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (stdout, stderr) = hooks.use_output();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut should_exit = hooks.use_state(|| false);

    hooks.use_future(async move {
        stdout.println("Hello from iocraft to stdout!");
        stderr.println("  And hello to stderr too!");

        stdout.print("Working...");
        for _ in 0..10 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            stdout.print(".");
        }
        stdout.println("\nDone!");
        should_exit.set(true);
    });

    if *should_exit.read() {
        system.exit();
    }
    element! {
        View(border_style: BorderStyle::Round, border_color: Color::DarkBlue) {
            Text(content: "Hello, use_output!")
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    element!(Example).render_loop().await?;
    Ok(())
}
