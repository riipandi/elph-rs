//! TUI demo - basic chat layout
//!
//! ```bash
//! cargo run -p elph-tui --example chat_layout
//! ```

use anyhow::Result;
use chrono::Local;
use iocraft::prelude::*;
use std::time::Duration;

#[component]
fn MainShell(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (width, height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut time = hooks.use_state(|| Local::now());
    let mut should_exit = hooks.use_state(|| false);

    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            time.set(Local::now());
        }
    });

    hooks.use_terminal_events({
        move |event| match event {
            TerminalEvent::Key(KeyEvent { code, kind, .. }) if kind != KeyEventKind::Release => match code {
                KeyCode::Char('q') => should_exit.set(true),
                _ => {}
            },
            _ => {}
        }
    });

    if should_exit.get() {
        system.exit();
    }

    element! {
        View(
            width,
            height,
            background_color: Color::Reset,
            border_style: BorderStyle::None,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexEnd,
            align_items: AlignItems::Center,
            margin: 0,
            padding: 0,
        ) {
            View(
                width,
                height,
                background_color: Color::Reset,
                border_style: BorderStyle::None,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Baseline,
                justify_content: JustifyContent::FlexEnd,
                padding: 1,
            ) {
                // Content stream (chat transcription)
                Text(content: "This is a content stream placeholder...", color: Color::White)
            }

            View(
                width,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding_left: 1,
                padding_right: 1,
            ) {
                Text(color: Color::DarkGrey, content: format!("1 Current Time: {}", time.get().format("%r")))
                Text(color: Color::DarkGrey, content: "Press \"q\" to quit.")
             }

             View(
                width,
                border_style: BorderStyle::None,
                align_items: AlignItems::Baseline,
                flex_direction: FlexDirection::Column,
                margin_bottom: 0,
                padding_top: 0,
                padding_bottom: 0,
                padding_left: 0,
                padding_right: 0,
             ) {
                View(
                    width,
                    min_height: 3,
                    border_style: BorderStyle::Round,
                    border_color: Color::DarkGrey,
                    align_items: AlignItems::Baseline,
                    margin_bottom: 0,
                    padding_top: 0,
                    padding_bottom: 0,
                    padding_left: 1,
                    padding_right: 1,
                ) {
                    // View(width, margin_top: -1, margin_left: 1, position: Position::Absolute) {
                    //     Text(content: "Overlap Example", wrap: TextWrap::NoWrap)
                    // }
                    TextInput(has_focus: true, multiline: true, cursor_color: Color::Grey)
                }
                View(
                    width,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding_left: 1,
                    padding_right: 1,
                ) {
                    Text(color: Color::DarkGrey, content: "STATUSBAR_TOP_LEFT")
                    Text(color: Color::DarkGrey, content: "STATUSBAR_TOP_RIGHT")
                }
                View(
                    width,
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::SpaceBetween,
                    padding_left: 1,
                    padding_right: 1,
                ) {
                    Text(color: Color::DarkGrey, content: "STATUSBAR_BOTTOM_LEFT")
                    Text(color: Color::DarkGrey, content: "STATUSBAR_BOTTOM_RIGHT")
                }
             }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    element!(MainShell).fullscreen().await?;
    Ok(())
}
