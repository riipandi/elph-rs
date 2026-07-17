//! Status row between transcript and editor.

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use elph_tui::rgb;
use iocraft::prelude::*;

const IDLE_ACTION_HINT: &str = "Enter to send · Ctrl+D exit";

const TIPS: &[&str] = &[
    "Tab toggles prompt / transcript focus",
    "Shift+Tab cycles agent mode (Build · Plan · Ask · Brave)",
    "Ctrl+~ cycles thinking level (Ctrl+` works too)",
    "Ctrl+L opens the model picker · /model filters too",
    "Shift+↑↓ scrolls the transcript",
    "Esc returns to the prompt · type while reading to jump back",
    "Shift+Enter or Ctrl+J inserts a newline",
    "Type / for commands · /help lists them all",
    "/compact shortens long conversation history",
    "! runs a shell command with session context",
    "!! runs a shell command without context",
    "@ opens the file picker to insert paths",
    "Ctrl+V pastes an image when the model supports vision",
    "Brave mode skips tool-approval prompts",
    "Plan mode is for read-only exploration and planning",
    "Enter sends · Ctrl+D exits · Ctrl+C cancels a busy turn",
];

use crate::tui::activity::{
    braille_spinner_glyph, format_activity_busy_line, format_session_busy_right_line, format_session_idle_right_line,
};

/// Props for [`StatusRow`].
#[derive(Props)]
pub struct StatusRowProps {
    pub screen_width: u16,
    pub busy: bool,
    pub activity_label: String,
    pub accent: Color,
    /// Drives braille spinner animation from the shell tick (no local timer).
    pub spinner_tick: u32,
    /// Elapsed seconds for the current activity phase (left segment).
    pub activity_elapsed_secs: f64,
    /// Elapsed seconds for the in-flight turn (added to session total on the right).
    pub turn_elapsed_secs: f64,
    /// Accumulated elapsed seconds across completed turns (right segment adds in-flight turn).
    pub session_elapsed_secs: f64,
    /// Replaces the idle tip briefly after a turn completes (e.g. `Turn complete · 1.2s`).
    pub idle_notice: Option<String>,
    /// When true, append quit-confirm keys to the busy right segment.
    pub quit_confirm_pending: bool,
}

impl Default for StatusRowProps {
    fn default() -> Self {
        Self {
            screen_width: 80,
            busy: false,
            activity_label: String::new(),
            accent: default_spinner_accent(),
            spinner_tick: 0,
            activity_elapsed_secs: 0.0,
            turn_elapsed_secs: 0.0,
            session_elapsed_secs: 0.0,
            idle_notice: None,
            quit_confirm_pending: false,
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

#[component]
pub fn StatusRow(props: &StatusRowProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut tip_index = hooks.use_ref(initial_tip_index);
    let mut was_busy = hooks.use_ref(|| false);

    if props.busy && !was_busy.get() {
        was_busy.set(true);
    } else if !props.busy && was_busy.get() {
        was_busy.set(false);
        tip_index.set(random_tip_index(tip_index.get(), TIPS.len()));
    }

    let right_half = props.screen_width / 2;
    let idle_line = props
        .idle_notice
        .clone()
        .unwrap_or_else(|| TIPS[tip_index.get() % TIPS.len()].to_string());
    let activity_line = format_activity_busy_line(&props.activity_label, props.activity_elapsed_secs);
    let busy_right_line =
        format_session_busy_right_line(props.session_elapsed_secs, props.turn_elapsed_secs, props.quit_confirm_pending);
    let idle_right_line = format_session_idle_right_line(IDLE_ACTION_HINT);
    let _spinner_frame = props.spinner_tick;
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
                                color: props.accent,
                                wrap: TextWrap::NoWrap,
                                content: spinner_glyph.to_string(),
                            )
                            Text(
                                color: Color::DarkGrey,
                                wrap: TextWrap::NoWrap,
                                content: activity_line,
                            )
                        }
                    }
                } else {
                    element! {
                        View(align_items: AlignItems::Center, justify_content: JustifyContent::Start) {
                            Text(color: Color::DarkGrey, wrap: TextWrap::NoWrap, content: idle_line)
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
                            Text(color: Color::DarkGrey, wrap: TextWrap::NoWrap, content: busy_right_line)
                        }
                    }
                } else {
                    element! {
                        View(align_items: AlignItems::Center, justify_content: JustifyContent::End) {
                            Text(color: Color::DarkGrey, wrap: TextWrap::NoWrap, content: idle_right_line)
                        }
                    }
                })
            }
        }
    }
}

/// Default accent for the braille spinner (opencode theme).
pub fn default_spinner_accent() -> Color {
    rgb(0xfa, 0xb2, 0x83)
}

/// Elapsed seconds rounded to one decimal (50ms tick granularity).
pub fn format_elapsed_secs(started: Instant) -> f64 {
    let tenths = started.elapsed().as_millis() / 100;
    tenths as f64 / 10.0
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
        std::thread::sleep(std::time::Duration::from_millis(250));
        let elapsed = format_elapsed_secs(started);
        assert!((0.2..=0.4).contains(&elapsed));
    }
}
