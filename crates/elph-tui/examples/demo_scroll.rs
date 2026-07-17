//! Demo: ScrollBox, VerticalScrollbar, and ScrollIndicator (live-linked).
//!
//! ```sh
//! cargo run -p elph-tui --example demo_scroll
//! ```

use anyhow::Result;
use elph_tui::prelude::*;

const VIEWPORT_HEIGHT: u16 = 12;
const BOX_WIDTH: u16 = 56;
const LINE_COUNT: u32 = 30;

#[component]
fn Demo(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut exit = hooks.use_state(|| false);
    let scroll_handle = hooks.use_ref_default::<ScrollViewHandle>();
    let scroll_generation = hooks.use_state(|| 0u32);

    let mut text = String::new();
    for i in 1..=LINE_COUNT {
        text.push_str(&format!("Scroll line {i}\n"));
    }

    hooks.use_terminal_events({
        let mut scroll_generation = scroll_generation;
        move |event| {
            let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event else {
                return;
            };
            if kind == KeyEventKind::Release {
                return;
            }
            if matches!(code, KeyCode::Char('q')) {
                exit.set(true);
                return;
            }
            let scrolled = matches!(
                code,
                KeyCode::Up | KeyCode::Down | KeyCode::PageUp | KeyCode::PageDown | KeyCode::Home | KeyCode::End
            );
            if scrolled {
                scroll_generation.set(scroll_generation.get().wrapping_add(1));
                let _ = scroll_handle.read().scroll_offset();
            }
        }
    });

    if exit.get() {
        system.exit();
    }

    let _scroll_generation = scroll_generation.get();
    let handle = scroll_handle.read();
    let scroll_offset = handle.scroll_offset().max(0) as u16;
    let content_height = handle.content_height().max(VIEWPORT_HEIGHT);
    let viewport_height = handle.viewport_height().max(1);

    element! {
        View(padding: 2u16, flex_direction: FlexDirection::Column, gap: 1u16) {
            StyledText(
                content: "ScrollBox + VerticalScrollbar + ScrollIndicator — ↑/↓ scroll, q quit".to_string(),
                color: Color::DarkGrey,
            )
            View(flex_direction: FlexDirection::Row, gap: 1u16, align_items: AlignItems::FlexStart) {
                ScrollBox(
                    width: BOX_WIDTH,
                    height: VIEWPORT_HEIGHT,
                    auto_scroll: false,
                    keyboard_scroll: true,
                    scroll_step: 1u16,
                    scrollbar: false,
                    handle: Some(scroll_handle),
                ) {
                    StyledText(content: text.clone())
                }
                VerticalScrollbar(
                    viewport_height: viewport_height,
                    content_height: content_height,
                    scroll_offset: scroll_offset,
                )
            }
            ScrollIndicator(
                offset: scroll_offset as u32,
                total: LINE_COUNT,
                visible: viewport_height as u32,
                width: BOX_WIDTH.saturating_add(2),
            )
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    element!(Demo).render_loop().await?;
    Ok(())
}
