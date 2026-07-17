//! Demo: CodeBlock and LineNumbers.
//!
//! ```bash
//! cargo run -p elph-tui --example demo_code
//! ```

use anyhow::Result;
use elph_tui::prelude::*;

const SAMPLE: &str = "fn main() {\n    let msg = \"hello\";\n    println!(\"{msg}\");\n}";

#[component]
fn Demo(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut exit = hooks.use_state(|| false);

    hooks.use_terminal_events(move |event| {
        let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event else {
            return;
        };
        if kind != KeyEventKind::Release && matches!(code, KeyCode::Char('q')) {
            exit.set(true);
        }
    });

    if exit.get() {
        system.exit();
    }

    element! {
        View(padding: 2u16, flex_direction: FlexDirection::Column, gap: 1u16) {
            StyledText(content: "CodeBlock demo — q to quit".to_string(), color: Color::DarkGrey)
            CodeBlock(width: 60u16, source: SAMPLE.to_string(), show_line_numbers: true, gutter_width: 4u16)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    element!(Demo).render_loop().await?;
    Ok(())
}
