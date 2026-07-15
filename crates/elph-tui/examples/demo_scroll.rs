//! Demo: ScrollBox and ScrollIndicator.
//!
//! ```bash
//! cargo run -p elph-tui --example demo_scroll
//! ```

use anyhow::Result;
use elph_tui::prelude::*;

#[component]
fn Demo(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut exit = hooks.use_state(|| false);
    let mut text = String::new();
    for i in 1..=30 {
        text.push_str(&format!("Scroll line {i}\n"));
    }

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
            StyledText(content: "ScrollBox demo — arrow keys to scroll, q to quit".to_string(), color: Color::DarkGrey)
            ScrollBox(
                width: 60u16,
                height: 12u16,
                auto_scroll: false,
                keyboard_scroll: true,
                scroll_step: 2u16,
                scrollbar: true,
                scrollbar_style: ScrollbarStyle::dark(),
            ) {
                StyledText(content: text)
            }
            ScrollIndicator(offset: 0u32, total: 30u32, visible: 12u32, width: 60u16)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    element!(Demo).render_loop().await?;
    Ok(())
}
