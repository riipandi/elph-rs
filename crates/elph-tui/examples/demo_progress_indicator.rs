//! Demo: progress indicators — CLI stepped bar and fullscreen init simulation.
//!
//! ```sh
//! # Fullscreen stepped progress (auto-advance + manual)
//! cargo run -p elph-tui --example demo_progress_indicator
//!
//! # CLI-only: stderr spinner + init progress bar (no fullscreen)
//! cargo run -p elph-tui --example demo_progress_indicator -- --cli
//! ```

use std::time::Duration;

use anyhow::Result;
use elph_tui::CliProgress;
use elph_tui::prelude::*;
use elph_tui::progress_spinner;
use tokio::time::sleep;

const STEPS: &[&str] = &[
    "Resolving paths",
    "Ensuring directories",
    "Opening datastore",
    "Loading configuration",
    "Ready",
];

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::args().any(|arg| arg == "--cli") {
        run_cli_demo().await?;
        return Ok(());
    }
    element!(ProgressDemo).render_loop().await?;
    Ok(())
}

async fn run_cli_demo() -> Result<()> {
    println!("CLI progress demo (iocraft-rendered stderr lines)\n");

    let init = CliProgress::new(STEPS.len() as u64);
    for step in STEPS {
        init.advance(*step);
        sleep(Duration::from_millis(500)).await;
    }
    init.finish();
    println!("Init complete.\n");

    let spinner = progress_spinner("Waiting for model response...");
    sleep(Duration::from_millis(1800)).await;
    spinner.finish_and_clear();
    println!("Response received.");

    Ok(())
}

fn format_bar(pos: u64, len: u64, width: usize) -> (String, String, String) {
    if len == 0 {
        return (String::new(), String::new(), "─".repeat(width));
    }
    if pos >= len {
        return ("━".repeat(width), String::new(), String::new());
    }

    let mut solid = ((pos as usize) * width / len as usize).min(width);
    if pos > 0 && solid == 0 {
        solid = 1;
    }

    let with_head = solid.min(width.saturating_sub(1));
    let head = if with_head < width {
        "╸".to_string()
    } else {
        String::new()
    };
    let empty = width.saturating_sub(with_head + head.chars().count());

    ("━".repeat(with_head), head, "─".repeat(empty))
}

#[derive(Clone, Copy, Default, Props)]
struct StepListProps {
    current: usize,
}

#[component]
fn StepList(props: &StepListProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let _ = hooks;
    let rows: Vec<_> = STEPS
        .iter()
        .enumerate()
        .map(|(index, label)| {
            let (marker, color) = if index < props.current {
                ("✓", Color::Green)
            } else if index == props.current {
                ("●", Color::Cyan)
            } else {
                ("○", Color::DarkGrey)
            };
            element! {
                View(flex_direction: FlexDirection::Row, gap: 1u16) {
                    Text(color: color, wrap: TextWrap::NoWrap, content: marker.to_string())
                    Text(
                        color: if index <= props.current { Color::Grey } else { Color::DarkGrey },
                        wrap: TextWrap::NoWrap,
                        content: label.to_string(),
                    )
                }
            }
        })
        .collect();

    element! {
        View(flex_direction: FlexDirection::Column, gap: 0u16) {
            #(rows)
        }
    }
}

#[derive(Clone, Default, Props)]
struct ProgressBarLineProps {
    pos: u64,
    len: u64,
    message: String,
}

#[component]
fn ProgressBarLine(props: &ProgressBarLineProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let _ = hooks;
    let (filled, head, empty) = format_bar(props.pos, props.len, 24);

    element! {
        View(flex_direction: FlexDirection::Row, align_items: AlignItems::Center, gap: 1u16) {
            SpinnerLoaderView(color: Some(rgb(0x7d, 0xce, 0xa0)), active: props.pos < props.len)
            Text(color: Color::Cyan, wrap: TextWrap::NoWrap, content: format!("{} ", props.message))
            Text(color: Color::Cyan, wrap: TextWrap::NoWrap, content: "[".to_string())
            Text(color: Color::Cyan, wrap: TextWrap::NoWrap, content: filled)
            Text(color: Color::Blue, wrap: TextWrap::NoWrap, content: head)
            Text(color: Color::Blue, wrap: TextWrap::NoWrap, content: empty)
            Text(
                color: Color::Cyan,
                wrap: TextWrap::NoWrap,
                content: format!("] {}/{}", props.pos, props.len),
            )
        }
    }
}

#[component]
fn ProgressDemo(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut exit = hooks.use_state(|| false);
    let mut step = hooks.use_state(|| 0usize);
    let mut auto = hooks.use_state(|| true);
    let total = STEPS.len() as u64;

    hooks.use_terminal_events(move |event| {
        let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event else {
            return;
        };
        if kind == KeyEventKind::Release {
            return;
        }
        match code {
            KeyCode::Char('q') => exit.set(true),
            KeyCode::Char(' ') => auto.set(!auto.get()),
            KeyCode::Right | KeyCode::Char('n') => {
                let next = (step.get() + 1).min(STEPS.len().saturating_sub(1));
                step.set(next);
            }
            KeyCode::Left | KeyCode::Char('p') => {
                step.set(step.get().saturating_sub(1));
            }
            KeyCode::Char('r') => step.set(0),
            _ => {}
        }
    });

    hooks.use_future(async move {
        loop {
            sleep(Duration::from_millis(700)).await;
            if !auto.get() {
                continue;
            }
            let current = step.get();
            if current + 1 < STEPS.len() {
                step.set(current + 1);
            } else {
                sleep(Duration::from_millis(900)).await;
                step.set(0);
            }
        }
    });

    if exit.get() {
        system.exit();
    }

    let pos = (step.get() as u64).saturating_add(1).min(total);
    let message = STEPS[step.get().min(STEPS.len().saturating_sub(1))];

    element! {
        View(padding: 2u16, flex_direction: FlexDirection::Column, gap: 1u16) {
            StyledText(
                content: "Progress indicator".to_string(),
                color: Color::Cyan,
                weight: Weight::Bold,
            )
            StyledText(
                content: "Auto-advance · Space pause · ←/→ step · r reset · q quit · --cli for stderr demo".to_string(),
                color: Color::DarkGrey,
            )
            Card(
                width: 62u16,
                title: "Init progress".to_string(),
                border_style: CardBorderStyle::Round,
                padding: 1u16,
            ) {
                ProgressBarLine(pos: pos, len: total, message: message.to_string())
            }
            Card(
                width: 62u16,
                title: "Steps".to_string(),
                border_style: CardBorderStyle::Single,
                padding: 1u16,
            ) {
                StepList(current: step.get())
            }
            StyledText(
                content: if auto.get() {
                    "Auto-advance on"
                } else {
                    "Auto-advance paused"
                }
                .to_string(),
                color: Color::DarkGrey,
            )
        }
    }
}
