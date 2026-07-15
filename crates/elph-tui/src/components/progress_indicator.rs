//! Progress indicator widgets for iocraft (KITT scanner + braille spinner).

use std::time::Duration;

use iocraft::prelude::*;

use crate::color::rgb;
use crate::loader::{KittScanner, KittScannerConfig, LoaderCell, SpinnerLoader};

/// Props for [`KittScannerView`].
#[derive(Clone, Copy, Props)]
pub struct KittScannerViewProps {
    pub width: u16,
    pub accent: Color,
    pub active: bool,
}

impl Default for KittScannerViewProps {
    fn default() -> Self {
        Self {
            width: 8,
            accent: rgb(0xfa, 0xb2, 0x83),
            active: true,
        }
    }
}

/// Props for [`SpinnerLoaderView`].
#[derive(Clone, Copy, Props)]
pub struct SpinnerLoaderViewProps {
    pub color: Color,
    pub active: bool,
}

impl Default for SpinnerLoaderViewProps {
    fn default() -> Self {
        Self {
            color: rgb(0xfa, 0xb2, 0x83),
            active: true,
        }
    }
}

/// Renders a KITT-style scanner (`■` head + fading trail, `⬝` inactive dots).
#[component]
pub fn KittScannerView(props: &KittScannerViewProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut scanner = hooks.use_ref(|| {
        let config = KittScannerConfig {
            accent: props.accent,
            width: if props.width > 0 {
                props.width as usize
            } else {
                KittScannerConfig::default().width
            },
            ..Default::default()
        };
        KittScanner::with_config(config)
    });
    let mut frame_tick = hooks.use_state(|| 0u32);

    let mut active = hooks.use_ref(|| props.active);
    active.set(props.active);

    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(40)).await;
            if !active.get() {
                continue;
            }
            scanner.write().tick();
            frame_tick.set(frame_tick.get().wrapping_add(1));
        }
    });

    if props.accent != scanner.read().accent() {
        scanner.write().set_accent(props.accent);
    }

    let _tick = frame_tick.get();
    let render_width = props.width.max(1) as usize;
    let cells: Vec<LoaderCell> = if props.active {
        scanner.read().into_cells(render_width)
    } else {
        idle_cells(render_width, props.accent)
    };

    let cell_elements: Vec<_> = cells
        .into_iter()
        .map(|cell| {
            element! {
                Text(
                    color: cell.color,
                    wrap: TextWrap::NoWrap,
                    content: cell.ch.to_string(),
                )
            }
        })
        .collect();

    element! {
        View(
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            flex_shrink: 0f32,
        ) {
            #(cell_elements)
        }
    }
}

/// Renders a cycling braille spinner (`⠋⠙⠹…`).
#[component]
pub fn SpinnerLoaderView(props: &SpinnerLoaderViewProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut spinner = hooks.use_state(SpinnerLoader::new);

    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_millis(80)).await;
            let mut state = spinner.get();
            state.tick();
            spinner.set(state);
        }
    });

    let glyph = if props.active { spinner.get().glyph() } else { " " };

    element! {
        Text(color: props.color, wrap: TextWrap::NoWrap, content: glyph.to_string())
    }
}

fn idle_cells(width: usize, accent: Color) -> Vec<LoaderCell> {
    let (r, g, b) = rgb_components(accent);
    let faded = Color::Rgb {
        r: (r as f64 * 0.25) as u8,
        g: (g as f64 * 0.25) as u8,
        b: (b as f64 * 0.25) as u8,
    };
    (0..width)
        .map(|_| LoaderCell {
            ch: '⬝', color: faded
        })
        .collect()
}

fn rgb_components(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => (128, 128, 128),
    }
}
