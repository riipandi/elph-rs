//! Demo: Input and Textarea components.
//!
//! ```sh
//! cargo run -p elph-tui --example demo_input
//! ```

use anyhow::Result;
use elph_tui::prelude::*;

#[component]
fn Demo(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let name = hooks.use_state(String::new);
    let notes = hooks.use_state(String::new);
    let mut focus = hooks.use_state(|| 0u8);
    let mut exit = hooks.use_state(|| false);

    hooks.use_terminal_events(move |event| {
        let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event else {
            return;
        };
        if kind == KeyEventKind::Release {
            return;
        }
        match code {
            KeyCode::Char('q') => exit.set(true),
            KeyCode::Tab => focus.set((focus.get() + 1) % 2),
            KeyCode::BackTab => focus.set(focus.get().saturating_sub(1)),
            _ => {}
        }
    });

    if exit.get() {
        system.exit();
    }

    element! {
        View(padding: 2u16, flex_direction: FlexDirection::Column, gap: 1u16) {
            StyledText(content: "Input demo — Tab to switch fields, q to quit".to_string(), color: Color::DarkGrey)
            Input(width: 40u16, initial_value: String::new(), has_focus: focus.get() == 0, value: name)
            Textarea(width: 40u16, min_height: 4u16, has_focus: focus.get() == 1, value: notes)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    element!(Demo).render_loop().await?;
    Ok(())
}
