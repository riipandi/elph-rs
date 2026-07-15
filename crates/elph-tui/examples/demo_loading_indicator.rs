//! Demo: loading indicators — braille spinner and KITT scanner widgets.
//!
//! ```bash
//! cargo run -p elph-tui --example demo_loading_indicator
//! ```
//!
//! Keys: `Space` toggle active · `Tab` cycle accent · `q` quit

use anyhow::Result;
use elph_tui::prelude::*;

const ACCENTS: &[Color] = &[
    rgb(0xfa, 0xb2, 0x83),
    rgb(0x7d, 0xce, 0xa0),
    rgb(0x89, 0xb4, 0xfa),
    rgb(0xf3, 0x8b, 0xa8),
];

#[component]
fn Demo(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut exit = hooks.use_state(|| false);
    let mut active = hooks.use_state(|| true);
    let mut accent_index = hooks.use_state(|| 0usize);

    hooks.use_terminal_events(move |event| {
        let TerminalEvent::Key(KeyEvent {
            code, kind, modifiers, ..
        }) = event
        else {
            return;
        };
        if kind == KeyEventKind::Release {
            return;
        }
        match code {
            KeyCode::Char('q') => exit.set(true),
            KeyCode::Char(' ') => active.set(!active.get()),
            KeyCode::Tab => {
                let next = (accent_index.get()
                    + if modifiers.contains(KeyModifiers::SHIFT) {
                        ACCENTS.len().saturating_sub(1)
                    } else {
                        1
                    })
                    % ACCENTS.len();
                accent_index.set(next);
            }
            _ => {}
        }
    });

    if exit.get() {
        system.exit();
    }

    let accent = ACCENTS[accent_index.get() % ACCENTS.len()];
    let state_label = if active.get() { "running" } else { "idle" };
    let state_color = if active.get() { accent } else { Color::DarkGrey };

    element! {
        View(padding: 2u16, flex_direction: FlexDirection::Column, gap: 1u16) {
            StyledText(
                content: "Loading indicators".to_string(),
                color: Color::Cyan,
                weight: Weight::Bold,
            )
            StyledText(
                content: "Space toggle · Tab / Shift+Tab accent · q quit".to_string(),
                color: Color::DarkGrey,
            )
            Card(
                width: 56u16,
                title: "Braille spinner".to_string(),
                border_style: CardBorderStyle::Round,
                padding: 1u16,
            ) {
                StyledText(
                    content: "Classic activity glyph for status rows and tool cards.".to_string(),
                    color: Color::Grey,
                )
                View(flex_direction: FlexDirection::Row, align_items: AlignItems::Center, gap: 1u16) {
                    SpinnerLoaderView(color: accent, active: active.get())
                    StyledText(content: state_label.to_string(), color: state_color)
                }
            }
            Card(
                width: 56u16,
                title: "KITT scanner".to_string(),
                border_style: CardBorderStyle::Round,
                padding: 1u16,
            ) {
                StyledText(
                    content: "Knight-Rider trail — used for longer-running operations.".to_string(),
                    color: Color::Grey,
                )
                View(flex_direction: FlexDirection::Row, align_items: AlignItems::Center, gap: 1u16) {
                    KittScannerView(width: 10u16, accent: accent, active: active.get())
                    StyledText(content: state_label.to_string(), color: state_color)
                }
            }
            Card(
                width: 56u16,
                title: "Status row preview".to_string(),
                border_style: CardBorderStyle::Single,
                padding: 1u16,
            ) {
                View(flex_direction: FlexDirection::Row, align_items: AlignItems::Center, gap: 1u16) {
                    SpinnerLoaderView(color: accent, active: active.get())
                    StyledText(
                        content: "Reading src/tui/shell.rs · 12s".to_string(),
                        color: Color::DarkGrey,
                    )
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    element!(Demo).render_loop().await?;
    Ok(())
}
