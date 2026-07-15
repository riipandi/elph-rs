//! Animated loader state machines (KITT scanner + braille spinner).

use iocraft::prelude::Color;

// ---------------------------------------------------------------------------
// KITT-style (Knight Rider) bidirectional scanner state machine.
// Ported from https://github.com/penso/ratatui-opentui-loader
// ---------------------------------------------------------------------------

/// One rendered scanner cell (glyph + foreground color).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoaderCell {
    pub ch: char,
    pub color: Color,
}

/// Configuration for [`KittScanner`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct KittScannerConfig {
    pub accent: Color,
    pub width: usize,
    pub trail_steps: usize,
    pub hold_start: usize,
    pub hold_end: usize,
    pub inactive_factor: f64,
    pub min_fade: f64,
    pub inverted: bool,
}

impl Default for KittScannerConfig {
    fn default() -> Self {
        Self {
            accent: Color::Rgb {
                r: 0xfa,
                g: 0xb2,
                b: 0x83,
            },
            width: 8,
            trail_steps: 6,
            hold_start: 9,
            hold_end: 30,
            inactive_factor: 0.25,
            min_fade: 0.55,
            inverted: false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ScannerState {
    active_pos: usize,
    is_forward: bool,
    is_holding: bool,
    hold_progress: f64,
    hold_frame: usize,
}

/// KITT-style scanner animation (call [`tick`](Self::tick) ~every 40ms).
#[derive(Debug, Clone)]
pub struct KittScanner {
    config: KittScannerConfig,
    trail_colors: Vec<Color>,
    inactive_color: Color,
    frame_index: usize,
    total_frames: usize,
}

impl Default for KittScanner {
    fn default() -> Self {
        Self::new()
    }
}

impl KittScanner {
    pub fn new() -> Self {
        Self::with_config(KittScannerConfig::default())
    }

    pub fn with_config(config: KittScannerConfig) -> Self {
        let trail_colors = derive_trail(config.accent, config.trail_steps);
        let inactive_color = derive_inactive(config.accent, config.inactive_factor);
        let total_frames = config.width + config.hold_end + (config.width - 1) + config.hold_start;
        Self {
            config,
            trail_colors,
            inactive_color,
            frame_index: 0,
            total_frames,
        }
    }

    pub fn set_accent(&mut self, accent: Color) {
        self.config.accent = accent;
        self.trail_colors = derive_trail(accent, self.config.trail_steps);
        self.inactive_color = derive_inactive(accent, self.config.inactive_factor);
    }

    pub fn tick(&mut self) {
        self.frame_index = (self.frame_index + 1) % self.total_frames;
    }

    pub fn width(&self) -> usize {
        self.config.width
    }

    pub fn accent(&self) -> Color {
        self.config.accent
    }

    pub fn total_frames(&self) -> usize {
        self.total_frames
    }

    pub fn frame_index(&self) -> usize {
        self.frame_index
    }

    pub fn into_cells(&self, render_width: usize) -> Vec<LoaderCell> {
        let w = self.config.width.min(render_width);
        if w == 0 {
            return Vec::new();
        }

        let state = self.scanner_state();
        let fade = if state.is_holding {
            let p = state.hold_progress.min(1.0);
            1.0 - p * (1.0 - self.config.min_fade)
        } else {
            1.0
        };
        let faded_inactive = apply_fade(self.inactive_color, fade);

        (0..w)
            .map(|i| {
                let dist = if state.is_forward {
                    state.active_pos as i32 - i as i32
                } else {
                    i as i32 - state.active_pos as i32
                };
                let effective_dist = if state.is_holding {
                    dist + state.hold_frame as i32
                } else {
                    dist
                };

                if effective_dist >= 0 && (effective_dist as usize) < self.trail_colors.len() {
                    let idx = if self.config.inverted {
                        self.trail_colors.len() - 1 - effective_dist as usize
                    } else {
                        effective_dist as usize
                    };
                    LoaderCell {
                        ch: '■',
                        color: self.trail_colors[idx],
                    }
                } else {
                    LoaderCell {
                        ch: '⬝',
                        color: faded_inactive,
                    }
                }
            })
            .collect()
    }

    fn scanner_state(&self) -> ScannerState {
        let fi = self.frame_index;
        let w = self.config.width;
        let he = self.config.hold_end;
        let hs = self.config.hold_start;
        let backward_frames = w - 1;

        if fi < w {
            ScannerState {
                active_pos: fi,
                is_forward: true,
                is_holding: false,
                hold_progress: 0.0,
                hold_frame: 0,
            }
        } else if fi < w + he {
            let p = fi - w;
            ScannerState {
                active_pos: w - 1,
                is_forward: true,
                is_holding: true,
                hold_progress: if he > 0 { p as f64 / he as f64 } else { 1.0 },
                hold_frame: p,
            }
        } else if fi < w + he + backward_frames {
            let back_i = fi - w - he;
            ScannerState {
                active_pos: w - 2 - back_i,
                is_forward: false,
                is_holding: false,
                hold_progress: 0.0,
                hold_frame: 0,
            }
        } else {
            let p = fi - w - he - backward_frames;
            ScannerState {
                active_pos: 0,
                is_forward: false,
                is_holding: true,
                hold_progress: if hs > 0 { p as f64 / hs as f64 } else { 1.0 },
                hold_frame: p,
            }
        }
    }
}

fn rgb_components(c: Color) -> (u8, u8, u8) {
    match c {
        Color::Rgb { r, g, b } => (r, g, b),
        _ => (255, 0, 0),
    }
}

fn derive_trail(accent: Color, steps: usize) -> Vec<Color> {
    let (r, g, b) = rgb_components(accent);
    (0..steps)
        .map(|i| {
            if i == 0 {
                accent
            } else {
                let factor = 0.65_f64.powi(i as i32);
                Color::Rgb {
                    r: (r as f64 * factor) as u8,
                    g: (g as f64 * factor) as u8,
                    b: (b as f64 * factor) as u8,
                }
            }
        })
        .collect()
}

fn derive_inactive(accent: Color, factor: f64) -> Color {
    let (r, g, b) = rgb_components(accent);
    Color::Rgb {
        r: (r as f64 * factor) as u8,
        g: (g as f64 * factor) as u8,
        b: (b as f64 * factor) as u8,
    }
}

fn apply_fade(color: Color, fade: f64) -> Color {
    let (r, g, b) = rgb_components(color);
    Color::Rgb {
        r: (r as f64 * fade) as u8,
        g: (g as f64 * fade) as u8,
        b: (b as f64 * fade) as u8,
    }
}

// ---------------------------------------------------------------------------
// Braille spinner frames (classic TUI activity indicator).
// ---------------------------------------------------------------------------
const FRAMES: &[&str] = &["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

/// Cycling braille spinner (one glyph per tick).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SpinnerLoader {
    frame_index: usize,
}

impl SpinnerLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tick(&mut self) {
        self.frame_index = (self.frame_index + 1) % FRAMES.len();
    }

    pub fn glyph(&self) -> &'static str {
        FRAMES[self.frame_index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_eight_wide() {
        let scanner = KittScanner::new();
        assert_eq!(scanner.width(), 8);
        assert_eq!(scanner.into_cells(8).len(), 8);
    }

    #[test]
    fn tick_wraps() {
        let mut scanner = KittScanner::new();
        let total = scanner.total_frames();
        for _ in 0..total {
            scanner.tick();
        }
        assert_eq!(scanner.frame_index(), 0);
    }

    #[test]
    fn into_cells_width() {
        let scanner = KittScanner::new();
        assert_eq!(scanner.into_cells(8).len(), 8);
        assert!(scanner.into_cells(0).is_empty());
    }

    #[test]
    fn hold_at_start_produces_inactive_tail() {
        let mut scanner = KittScanner::new();
        let ticks = scanner.width() + 30 + (scanner.width() - 1);
        for _ in 0..ticks {
            scanner.tick();
        }
        let cells = scanner.into_cells(8);
        assert_eq!(cells.len(), 8);
        assert!(cells.iter().any(|c| c.ch == '■'));
    }

    #[test]
    fn tick_wraps_frames() {
        let mut spinner = SpinnerLoader::new();
        for _ in 0..FRAMES.len() {
            spinner.tick();
        }
        assert_eq!(spinner.frame_index, 0);
    }
}
