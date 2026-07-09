use crate::prompt::AgentMode;
use slt::{Color, Context, Theme as SltTheme};

/// Visual theme variant for the terminal UI.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeMode {
    #[default]
    Dark,
    Light,
}

/// Terminal-native palette — no custom RGB; defers to the emulator theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Theme {
    pub mode: ThemeMode,
    pub background: Color,
    pub foreground: Color,
    pub muted: Color,
    pub prompt_prefix: Color,
    pub scrollbar_thumb: Color,
    pub scrollbar_track: Color,
    pub frame_border: Color,
    mode_build: Color,
    mode_plan: Color,
    mode_ask: Color,
    mode_brave: Color,
}

impl Theme {
    /// Palette for dark terminal backgrounds (standard ANSI accents).
    pub fn dark() -> Self {
        Self::from_mode(ThemeMode::Dark)
    }

    /// Palette for light terminal backgrounds (standard ANSI accents).
    pub fn light() -> Self {
        Self::from_mode(ThemeMode::Light)
    }

    pub fn from_mode(mode: ThemeMode) -> Self {
        Self {
            mode,
            background: Color::Reset,
            foreground: Color::Reset,
            muted: Color::DarkGray,
            prompt_prefix: Color::Reset,
            scrollbar_thumb: Color::DarkGray,
            scrollbar_track: Color::DarkGray,
            frame_border: Color::Reset,
            mode_build: Color::DarkGray,
            mode_plan: Color::Cyan,
            mode_ask: Color::Blue,
            mode_brave: Color::Red,
        }
    }

    /// Resolves the active theme from `ELPH_THEME`, terminal `COLORFGBG`, or defaults to dark.
    pub fn detect() -> Self {
        if let Ok(value) = std::env::var("ELPH_THEME") {
            match value.trim().to_ascii_lowercase().as_str() {
                "light" => return Self::light(),
                "dark" => return Self::dark(),
                _ => {}
            }
        }

        if let Ok(fgbg) = std::env::var("COLORFGBG")
            && let Some(bg) = fgbg.split(';').nth(1).and_then(|part| part.trim().parse::<u8>().ok())
        {
            return if bg >= 8 { Self::light() } else { Self::dark() };
        }

        Self::dark()
    }

    pub fn toggle(self) -> Self {
        Self::from_mode(match self.mode {
            ThemeMode::Dark => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
        })
    }

    /// Sync SuperLightTUI with a terminal-respecting palette for the active mode.
    pub fn apply_to(self, ui: &mut Context) {
        ui.set_theme(self.slt_theme());
    }

    fn slt_theme(self) -> SltTheme {
        match self.mode {
            ThemeMode::Dark => SltTheme::builder()
                .is_dark(true)
                .text(Color::Reset)
                .text_dim(Color::DarkGray)
                .bg(Color::Reset)
                .border(Color::DarkGray)
                .primary(Color::Cyan)
                .secondary(Color::Blue)
                .accent(Color::Magenta)
                .success(Color::Green)
                .warning(Color::Yellow)
                .error(Color::Red)
                .selected_bg(Color::Blue)
                .selected_fg(Color::Reset)
                .surface(Color::Reset)
                .surface_hover(Color::Reset)
                .surface_text(Color::Reset)
                .build(),
            ThemeMode::Light => SltTheme::builder()
                .is_dark(false)
                .text(Color::Reset)
                .text_dim(Color::DarkGray)
                .bg(Color::Reset)
                .border(Color::DarkGray)
                .primary(Color::Blue)
                .secondary(Color::Cyan)
                .accent(Color::Magenta)
                .success(Color::Green)
                .warning(Color::Yellow)
                .error(Color::Red)
                .selected_bg(Color::Blue)
                .selected_fg(Color::Reset)
                .surface(Color::Reset)
                .surface_hover(Color::Reset)
                .surface_text(Color::Reset)
                .build(),
        }
    }

    pub fn mode_accent(self, mode: AgentMode) -> Color {
        self.mode_border_color(mode)
    }

    pub fn mode_border_color(self, mode: AgentMode) -> Color {
        match mode {
            AgentMode::Build => self.mode_build,
            AgentMode::Plan => self.mode_plan,
            AgentMode::Ask => self.mode_ask,
            AgentMode::Brave => self.mode_brave,
        }
    }

    /// Background color for views; `None` leaves the terminal background untouched.
    pub fn view_background(self) -> Option<Color> {
        match self.background {
            Color::Reset => None,
            color => Some(color),
        }
    }

    /// Foreground color for text; `None` inherits the terminal foreground.
    pub fn text_color(self) -> Option<Color> {
        match self.foreground {
            Color::Reset => None,
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
            ThemeMode::Dark => Color::Cyan,
            ThemeMode::Light => Color::Blue,
        }
    }

    pub fn yellow_col(self) -> Color {
        Color::Yellow
    }

    pub fn highlight(self) -> Color {
        Color::Magenta
    }

    pub fn special(self) -> Color {
        Color::Green
    }

    pub fn dim_text(self) -> Color {
        self.muted
    }

    pub fn bright_text(self) -> Color {
        Color::Reset
    }

    pub fn user_pipe_col(self) -> Color {
        Color::Magenta
    }

    pub fn ai_pipe_col(self) -> Color {
        Color::DarkGray
    }

    /// Primary emphasis — inherits terminal foreground.
    pub fn white_col(self) -> Color {
        Color::Reset
    }

    pub fn thinking_color(self, level: &str) -> Color {
        match level.trim().to_ascii_lowercase().as_str() {
            "low" => Color::Green,
            "medium" => Color::Yellow,
            "high" => Color::Yellow,
            "xhigh" => Color::Red,
            _ => Color::DarkGray,
        }
    }

    pub fn context_usage_color(self, pct: f64) -> Color {
        if pct >= 90.0 {
            Color::Red
        } else if pct >= 80.0 {
            Color::LightRed
        } else if pct >= 50.0 {
            Color::Yellow
        } else {
            Color::Reset
        }
    }

    pub fn git_status_color(self, additions: u32, deletions: u32) -> Color {
        if additions == 0 && deletions == 0 {
            Color::DarkGray
        } else if additions > 0 && deletions == 0 {
            Color::Green
        } else if additions == 0 && deletions > 0 {
            Color::Red
        } else {
            Color::Yellow
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::dark()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn palettes_use_terminal_reset() {
        let dark = Theme::dark();
        let light = Theme::light();
        assert_eq!(dark.background, Color::Reset);
        assert_eq!(light.background, Color::Reset);
        assert_eq!(dark.foreground, Color::Reset);
        assert_eq!(light.foreground, Color::Reset);
        assert_eq!(dark.mode, ThemeMode::Dark);
        assert_eq!(light.mode, ThemeMode::Light);
    }

    #[test]
    fn no_custom_rgb_in_semantic_tokens() {
        let theme = Theme::dark();
        for color in [
            theme.blue_col(),
            theme.yellow_col(),
            theme.highlight(),
            theme.special(),
            theme.dim_text(),
            theme.bright_text(),
            theme.user_pipe_col(),
            theme.ai_pipe_col(),
            theme.white_col(),
            theme.thinking_color("high"),
            theme.context_usage_color(95.0),
            theme.git_status_color(1, 0),
            theme.mode_border_color(AgentMode::Ask),
        ] {
            assert!(
                !matches!(color, Color::Rgb(_, _, _)),
                "expected terminal color, got {color:?}"
            );
        }
    }

    #[test]
    fn toggle_switches_mode() {
        let dark = Theme::dark();
        assert_eq!(dark.toggle().mode, ThemeMode::Light);
        assert_eq!(dark.toggle().toggle().mode, ThemeMode::Dark);
    }

    #[test]
    fn mode_border_uses_ansi() {
        let theme = Theme::dark();
        assert_eq!(theme.mode_border_color(AgentMode::Plan), Color::Cyan);
        assert_eq!(theme.mode_border_color(AgentMode::Ask), Color::Blue);
    }

    #[test]
    fn context_usage_thresholds() {
        let theme = Theme::dark();
        assert_eq!(theme.context_usage_color(30.0), Color::Reset);
        assert_eq!(theme.context_usage_color(60.0), Color::Yellow);
        assert_eq!(theme.context_usage_color(85.0), Color::LightRed);
        assert_eq!(theme.context_usage_color(95.0), Color::Red);
    }

    #[test]
    fn reset_colors_defer_to_terminal() {
        let theme = Theme::dark();
        assert_eq!(theme.view_background(), None);
        assert_eq!(theme.text_color(), None);
    }
}
