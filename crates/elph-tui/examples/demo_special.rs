//! Demo: AsciiText, QrCodeView, and FrameBufferView.
//!
//! ```sh
//! cargo run -p elph-tui --example demo_special
//! ```

use anyhow::Result;
use elph_tui::prelude::*;

#[component]
fn Demo(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut exit = hooks.use_state(|| false);

    let mut buffer = FrameBuffer::new(24, 4);
    buffer.set_text(1, 1, "FrameBuffer");
    buffer.set_text(1, 2, "custom cells");

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
            StyledText(content: "Special components — q to quit".to_string(), color: Color::DarkGrey)
            AsciiText(text: "ELPH".to_string(), use_figlet: false, color: Color::Cyan)
            QrCodeView(payload: "https://elph.space".to_string(), color: Color::White)
            FrameBufferView(buffer: buffer, color: Color::Grey)
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    element!(Demo).render_loop().await?;
    Ok(())
}
