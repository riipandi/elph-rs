//! Status row between transcript and editor.

use elph_tui::components::theme::UiTheme;
use elph_tui::loader::SpinnerLoader;
use iocraft::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};

const IDLE_ACTION_HINT: &str = "Enter to send · / search · Ctrl+D exit";
const BUSY_CANCEL_HINT: &str = "Ctrl+C cancel";

const TIPS: &[&str] = &[
    "Type / for demo commands — /demo-mode /demo-multi /demo-todo",
    "Shift+↑↓ scrolls the transcript",
    "/demo-tool and /demo-thinking add transcript cards",
    "Shift+Tab cycles thinking level · Tab cycles agent mode",
    "/demo-busy simulates a thinking turn",
    "Esc dismisses open dialogs",
];

fn initial_tip_index() -> usize {
    random_tip_index(0, TIPS.len())
}

fn random_tip_index(current: usize, tip_count: usize) -> usize {
    if tip_count <= 1 {
        return 0;
    }
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    let mut next = (seed.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(current as u64) as usize) % tip_count;
    if next == current {
        next = (current + 1) % tip_count;
    }
    next
}

fn braille_spinner_glyph(tick: u32) -> &'static str {
    let mut spinner = SpinnerLoader::new();
    for _ in 0..(tick as usize % 10) {
        spinner.tick();
    }
    spinner.glyph()
}

fn format_activity_line(label: &str, elapsed_secs: f64) -> String {
    format!("{label} · {elapsed_secs:.1}s")
}

#[derive(Props)]
pub struct StatusRowProps {
    pub screen_width: u16,
    pub busy: bool,
    pub activity_label: String,
    pub accent: Color,
    pub spinner_tick: u32,
    pub elapsed_secs: f64,
}

impl Default for StatusRowProps {
    fn default() -> Self {
        Self {
            screen_width: 80,
            busy: false,
            activity_label: String::new(),
            accent: Color::Rgb { r: 250, g: 178, b: 131 },
            spinner_tick: 0,
            elapsed_secs: 0.0,
        }
    }
}

#[component]
pub fn StatusRow(props: &StatusRowProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = UiTheme::default();
    let pad = theme.shell_zone_padding();
    let mut tip_index = hooks.use_ref(initial_tip_index);
    let mut was_busy = hooks.use_ref(|| false);

    if props.busy && !was_busy.get() {
        was_busy.set(true);
    } else if !props.busy && was_busy.get() {
        was_busy.set(false);
        tip_index.set(random_tip_index(tip_index.get(), TIPS.len()));
    }

    let right_half = props.screen_width / 2;
    let idle_tip = TIPS[tip_index.get() % TIPS.len()].to_string();
    let activity_line = format_activity_line(&props.activity_label, props.elapsed_secs);
    let spinner_glyph = if props.busy {
        braille_spinner_glyph(props.spinner_tick)
    } else {
        " "
    };

    element! {
        View(
            width: props.screen_width,
            flex_shrink: 0f32,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding_left: pad,
            padding_right: pad,
        ) {
            View(width: right_half, align_items: AlignItems::Center, justify_content: JustifyContent::FlexStart) {
                #(if props.busy {
                    element! {
                        View(flex_direction: FlexDirection::Row, align_items: AlignItems::Center, gap: theme.gap_md) {
                            Text(color: props.accent, wrap: TextWrap::NoWrap, content: spinner_glyph.to_string())
                            Text(color: theme.text_hint, wrap: TextWrap::NoWrap, content: activity_line)
                        }
                    }
                } else {
                    element! {
                        View(align_items: AlignItems::Center) {
                            Text(color: theme.text_hint, wrap: TextWrap::NoWrap, content: idle_tip)
                        }
                    }
                })
            }
            View(width: right_half, align_items: AlignItems::Center, justify_content: JustifyContent::FlexEnd) {
                Text(
                    color: theme.text_hint,
                    wrap: TextWrap::NoWrap,
                    content: if props.busy { BUSY_CANCEL_HINT } else { IDLE_ACTION_HINT },
                )
            }
        }
    }
}
