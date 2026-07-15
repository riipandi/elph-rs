//! Numeric range slider.

use iocraft::prelude::*;

/// Props for [`Slider`].
#[derive(Clone, Default, Props)]
pub struct SliderProps {
    pub width: u16,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub value: Option<State<f32>>,
    pub has_focus: bool,
    pub label: String,
    pub fill_color: Option<Color>,
    pub track_color: Option<Color>,
}

/// Horizontal slider adjusted with arrow keys when focused.
#[component]
pub fn Slider(props: &SliderProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let min = props.min;
    let max = props.max.max(min + props.step);
    let step = props.step.max(0.01);
    let internal = hooks.use_state(|| min);
    let mut value = props.value.unwrap_or(internal);
    let has_focus = props.has_focus;

    hooks.use_terminal_events(move |event| {
        if !has_focus {
            return;
        }
        let TerminalEvent::Key(KeyEvent { code, kind, .. }) = event else {
            return;
        };
        if kind == KeyEventKind::Release {
            return;
        }
        match code {
            KeyCode::Left | KeyCode::Char('h') => {
                value.set((value.get() - step).max(min));
            }
            KeyCode::Right | KeyCode::Char('l') => {
                value.set((value.get() + step).min(max));
            }
            _ => {}
        }
    });

    let current = value.get().clamp(min, max);
    let pct = if max <= min {
        0.0
    } else {
        ((current - min) / (max - min)) * 100.0
    };
    let fill = props.fill_color.unwrap_or(Color::DarkGreen);
    let track = props.track_color.unwrap_or(Color::DarkGrey);

    element! {
        View(width: props.width, flex_direction: FlexDirection::Column, gap: 0) {
            Text(
                content: if props.label.is_empty() {
                    String::new()
                } else {
                    format!("{}: {:.0}", props.label, current)
                },
                color: Color::Grey,
                wrap: TextWrap::NoWrap,
            )
            View(
                width: props.width,
                border_style: if has_focus { BorderStyle::Round } else { BorderStyle::Single },
                border_color: track,
            ) {
                View(width: Percent(pct), height: 1, background_color: fill)
            }
        }
    }
}
