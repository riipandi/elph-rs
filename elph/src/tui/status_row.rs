//! Status row between transcript and editor.

use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use elph_tui::{KittScannerView, rgb};
use iocraft::prelude::*;

const IDLE_ACTION_HINT: &str = "Enter to send · Ctrl+Q exit";

const TIPS: &[&str] = &[
    "Shift+↑↓ scrolls the transcript",
    "Ctrl+A cycles agent mode",
    "Shift+Tab cycles thinking level",
    "Ctrl+L opens the model picker",
    "Shift+Enter inserts a newline",
    "Click footer labels to change mode",
];

const BUSY_CANCEL_HINT: &str = "Esc cancel";

const ELAPSED_TICK_MS: u64 = 200;

/// Props for [`StatusRow`].
#[derive(Props)]
pub struct StatusRowProps {
    pub screen_width: u16,
    pub busy: bool,
    pub activity_label: String,
    pub accent: Color,
}

impl Default for StatusRowProps {
    fn default() -> Self {
        Self {
            screen_width: 80,
            busy: false,
            activity_label: String::new(),
            accent: default_scanner_accent(),
        }
    }
}

fn initial_tip_index() -> usize {
    random_tip_index(0, TIPS.len())
}

/// Pick a pseudo-random tip index, avoiding `current` when possible.
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

/// Elapsed seconds rounded to one decimal (200ms tick granularity).
fn format_elapsed_secs(started: Instant) -> f64 {
    let tenths = started.elapsed().as_millis() / 100;
    tenths as f64 / 10.0
}

fn format_activity_line(label: &str, elapsed_secs: f64) -> String {
    format!("{label} · {elapsed_secs:.1}s")
}

#[component]
pub fn StatusRow(props: &StatusRowProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut tip_index = hooks.use_ref(initial_tip_index);
    let mut busy_started_at = hooks.use_ref(|| None::<Instant>);
    let mut is_busy = hooks.use_ref(|| false);
    let mut elapsed_secs = hooks.use_state(|| 0.0f64);

    let was_busy = is_busy.get();
    is_busy.set(props.busy);

    if props.busy && !was_busy {
        busy_started_at.set(Some(Instant::now()));
        elapsed_secs.set(0.0);
    } else if !props.busy && was_busy {
        busy_started_at.set(None);
        tip_index.set(random_tip_index(tip_index.get(), TIPS.len()));
        elapsed_secs.set(0.0);
    }

    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(ELAPSED_TICK_MS)).await;
            if !is_busy.get() {
                continue;
            }
            if let Some(started) = busy_started_at.read().as_ref() {
                let next = format_elapsed_secs(*started);
                if (elapsed_secs.get() - next).abs() > f64::EPSILON {
                    elapsed_secs.set(next);
                }
            }
        }
    });

    let right_half = props.screen_width / 2;
    let idle_tip = TIPS[tip_index.get() % TIPS.len()].to_string();
    let activity_line = format_activity_line(&props.activity_label, elapsed_secs.get());

    element! {
        View(
            width: props.screen_width,
            flex_shrink: 0f32,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding_left: 1,
            padding_right: 1,
        ) {
            View(
                width: right_half,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                padding: 0,
            ) {
                #(if props.busy {
                    element! {
                        View(
                            flex_direction: FlexDirection::Row,
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Start,
                            flex_shrink: 0f32,
                            gap: 1,
                            padding: 0,
                        ) {
                            Text(
                                color: Color::DarkGrey,
                                wrap: TextWrap::NoWrap,
                                content: activity_line,
                            )
                            View(padding: 0, margin: 0) {
                                KittScannerView(
                                    width: 8u16,
                                    accent: props.accent,
                                    active: true,
                                )
                            }
                        }
                    }
                } else {
                    element! {
                        View(align_items: AlignItems::Center, justify_content: JustifyContent::Start) {
                            Text(color: Color::DarkGrey, wrap: TextWrap::NoWrap, content: idle_tip)
                        }
                    }
                })
            }
            View(
                width: right_half,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::End,
                padding: 0,
            ) {
                #(if props.busy {
                    element! {
                        View(align_items: AlignItems::Center, justify_content: JustifyContent::End) {
                            Text(color: Color::DarkGrey, wrap: TextWrap::NoWrap, content: BUSY_CANCEL_HINT)
                        }
                    }
                } else {
                    element! {
                        View(align_items: AlignItems::Center, justify_content: JustifyContent::End) {
                            Text(color: Color::DarkGrey, wrap: TextWrap::NoWrap, content: IDLE_ACTION_HINT)
                        }
                    }
                })
            }
        }
    }
}

/// Default accent for the KITT scanner (opencode theme).
pub fn default_scanner_accent() -> Color {
    rgb(0xfa, 0xb2, 0x83)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_tip_index_is_in_range() {
        let idx = initial_tip_index();
        assert!(idx < TIPS.len());
    }

    #[test]
    fn random_tip_index_avoids_current_when_possible() {
        let next = random_tip_index(2, TIPS.len());
        assert_ne!(next, 2);
        assert!(next < TIPS.len());
    }

    #[test]
    fn format_elapsed_rounds_to_tenths() {
        let started = Instant::now();
        std::thread::sleep(Duration::from_millis(250));
        let elapsed = format_elapsed_secs(started);
        assert!((0.2..=0.4).contains(&elapsed));
    }
}
