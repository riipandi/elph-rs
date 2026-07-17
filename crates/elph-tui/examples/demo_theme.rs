//! Demo: UiThemeProvider, per-component theme overrides, and on_change callbacks.
//!
//! Inactive inputs render in dimmed grey via [`UiTheme::input_text_color`].
//!
//! ```sh
//! cargo run -p elph-tui --example demo_theme
//! ```
//!
//! Keys: Tab focus · f cycles list/slider · q quit

use anyhow::Result;
use elph_tui::prelude::*;

fn accent_theme(base: UiTheme) -> UiTheme {
    UiTheme {
        accent: rgb(129, 161, 193),
        accent_soft: rgb(6, 182, 212),
        border_focus: rgb(129, 161, 193),
        ..base
    }
}

#[component]
fn ThemeDemo(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut exit = hooks.use_state(|| false);
    let mut focus = hooks.use_state(|| 0u8);
    let mut pane = hooks.use_state(|| 0u8);

    let name = hooks.use_state(|| "Focused field".to_string());
    let idle = hooks.use_state(|| "Inactive (dimmed)".to_string());
    let selected = hooks.use_state(|| 0usize);
    let volume = hooks.use_state(|| 55f32);
    let mut status = hooks.use_state(|| "—".to_string());

    let options = vec![
        SelectOption::new("Alpha", "First option"),
        SelectOption::new("Beta", "Second option"),
        SelectOption::new("Gamma", "Third option"),
    ];

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
            KeyCode::Char('f') => pane.set((pane.get() + 1) % 2),
            _ => {}
        }
    });

    if exit.get() {
        system.exit();
    }

    let base = UiTheme::default();
    let provider_theme = accent_theme(base);
    let list_theme = UiTheme {
        accent_soft: rgb(152, 195, 121),
        border_focus: rgb(152, 195, 121),
        ..base
    };

    let input_focused = focus.get() == 0;
    let list_focused = pane.get() == 0;

    element! {
        UiThemeProvider(theme: provider_theme) {
            View(padding: 2u16, flex_direction: FlexDirection::Column, gap: 1u16) {
                StyledText(
                    content: "UiThemeProvider + overrides — Tab: inputs · f: list/slider · q: quit".to_string(),
                    color: Color::DarkGrey,
                )
                StyledText(content: format!("Status: {}", status.read().clone()), color: base.text_hint)
                View(flex_direction: FlexDirection::Row, gap: 2u16) {
                    Input(
                        width: 28u16,
                        initial_value: "Focused field".to_string(),
                        has_focus: input_focused,
                        value: name,
                        on_change: move |text| {
                            status.set(format!("Input → \"{text}\""));
                        },
                    )
                    Input(
                        width: 28u16,
                        initial_value: "Inactive (dimmed)".to_string(),
                        has_focus: !input_focused,
                        value: idle,
                    )
                }
                SelectList(
                    width: 40u16,
                    height: 5u16,
                    options: options,
                    selected_index: selected,
                    has_focus: list_focused,
                    show_description: true,
                    theme: Some(list_theme),
                    on_change: move |idx| {
                        status.set(format!("SelectList → index {idx}"));
                    },
                )
                Slider(
                    width: 40u16,
                    min: 0.0_f32,
                    max: 100.0_f32,
                    step: 5.0_f32,
                    value: volume,
                    has_focus: !list_focused,
                    label: "Volume".to_string(),
                    on_change: move |v| {
                        status.set(format!("Slider → {v:.0}"));
                    },
                )
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    element!(ThemeDemo).render_loop().await?;
    Ok(())
}
