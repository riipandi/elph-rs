//! Demo: SelectList, TabSelect, and Slider.
//!
//! ```bash
//! cargo run -p elph-tui --example demo_select
//! ```

use anyhow::Result;
use elph_tui::prelude::*;

#[component]
fn Demo(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut exit = hooks.use_state(|| false);
    let selected = hooks.use_state(|| 0usize);
    let tab = hooks.use_state(|| 0usize);
    let volume = hooks.use_state(|| 40f32);
    let mut focus_pane = hooks.use_state(|| 0u8);

    let options = vec![
        SelectOption::new("New", "Create a new file"),
        SelectOption::new("Open", "Open existing file"),
        SelectOption::new("Save", "Save current file"),
        SelectOption::new("Exit", "Quit application"),
    ];

    let tabs = vec![
        TabItem::new("List", "Use ↑/↓ on the select list."),
        TabItem::new("Tabs", "Use ←/→ to switch tabs."),
        TabItem::new("Slider", "Use ←/→ on the slider when focused."),
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
            KeyCode::Char('f') => focus_pane.set((focus_pane.get() + 1) % 3),
            _ => {}
        }
    });

    if exit.get() {
        system.exit();
    }

    let pane = focus_pane.get();

    element! {
        View(padding: 2u16, flex_direction: FlexDirection::Column, gap: 1u16) {
            StyledText(content: "f cycles focus (list/tabs/slider) — q to quit".to_string(), color: Color::DarkGrey)
            SelectList(
                width: 40u16,
                height: 8u16,
                options: options,
                selected_index: selected,
                has_focus: pane == 0,
                show_description: true,
            )
            TabSelect(width: 60u16, tabs: tabs, selected_index: tab, has_focus: pane == 1)
            Slider(width: 40u16, min: 0.0_f32, max: 100.0_f32, step: 5.0_f32, value: volume, has_focus: pane == 2, label: "Volume".to_string())
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    element!(Demo).render_loop().await?;
    Ok(())
}
