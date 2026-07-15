//! Demo: DiffView component.
//!
//! ```bash
//! cargo run -p elph-tui --example demo_diff
//! ```

use anyhow::Result;
use elph_tui::prelude::*;

#[component]
fn Demo(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut exit = hooks.use_state(|| false);
    let mut mode = hooks.use_state(|| DiffMode::Unified);

    hooks.use_terminal_events(move |event| {
        let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event else {
            return;
        };
        if kind == KeyEventKind::Release {
            return;
        }
        match code {
            KeyCode::Char('q') => exit.set(true),
            KeyCode::Char('s') => mode.set(DiffMode::SideBySide),
            KeyCode::Char('u') => mode.set(DiffMode::Unified),
            _ => {}
        }
    });

    if exit.get() {
        system.exit();
    }

    let old = "alpha\nbeta\ngamma\n";
    let new = "alpha\nbravo\ngamma\ndelta\n";

    element! {
        View(padding: 2u16, flex_direction: FlexDirection::Column, gap: 1u16) {
            StyledText(content: "DiffView — u unified, s side-by-side, q quit".to_string(), color: Color::DarkGrey)
            DiffView(
                width: 72u16,
                height: 12u16,
                old_text: old.to_string(),
                new_text: new.to_string(),
                mode: mode.get(),
            )
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    element!(Demo).render_loop().await?;
    Ok(())
}
