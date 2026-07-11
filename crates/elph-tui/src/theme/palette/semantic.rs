use tuie::prelude::Color;

use super::{Theme, ThemeMode};

impl Theme {
    /// Background color for views; `None` leaves the terminal background untouched.
    pub fn view_background(self) -> Option<Color> {
        match self.background {
            Color::Background => None,
            color => Some(color),
        }
    }

    /// Foreground color for text; `None` inherits the terminal foreground.
    pub fn text_color(self) -> Option<Color> {
        match self.foreground {
            Color::Foreground => None,
            color => Some(color),
        }
    }

    pub fn input_cursor(self) -> Color {
        self.dim_text()
    }

    pub fn input_placeholder(self) -> Color {
        self.dim_text()
    }

    pub fn paste_label(self) -> Color {
        self.dim_text()
    }

    pub fn blue_col(self) -> Color {
        match self.mode {
            ThemeMode::Dark => Color::CYAN,
            ThemeMode::Light => Color::BLUE,
        }
    }

    pub fn yellow_col(self) -> Color {
        Color::YELLOW
    }

    pub fn highlight(self) -> Color {
        Color::MAGENTA
    }

    pub fn special(self) -> Color {
        Color::GREEN
    }

    pub fn dim_text(self) -> Color {
        self.muted
    }

    pub fn bright_text(self) -> Color {
        Color::Foreground
    }

    pub fn user_pipe_col(self) -> Color {
        Color::MAGENTA
    }

    pub fn ai_pipe_col(self) -> Color {
        Color::grey256(8)
    }

    /// Primary emphasis — inherits terminal foreground.
    pub fn white_col(self) -> Color {
        Color::Foreground
    }

    pub fn thinking_color(self, level: &str) -> Color {
        match level.trim().to_ascii_lowercase().as_str() {
            "low" => Color::GREEN,
            "medium" => Color::YELLOW,
            "high" => Color::YELLOW,
            "xhigh" => Color::RED,
            _ => Color::grey256(8),
        }
    }

    pub fn context_usage_color(self, pct: f64) -> Color {
        if pct >= 90.0 {
            Color::RED
        } else if pct >= 80.0 {
            Color::BRIGHT_RED
        } else if pct >= 50.0 {
            Color::YELLOW
        } else {
            Color::Foreground
        }
    }

    pub fn git_status_color(self, additions: u32, deletions: u32) -> Color {
        if additions == 0 && deletions == 0 {
            Color::grey256(8)
        } else if additions > 0 && deletions == 0 {
            Color::GREEN
        } else if additions == 0 && deletions > 0 {
            Color::RED
        } else {
            Color::YELLOW
        }
    }
}
