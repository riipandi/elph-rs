//! Shared UI and session types for the Elph binary.

/// Agent permission / interaction mode.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AgentMode {
    #[default]
    Build,
    Plan,
    Ask,
    Brave,
}

impl AgentMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Build => "Build",
            Self::Plan => "Plan",
            Self::Ask => "Ask",
            Self::Brave => "Brave",
        }
    }

    pub fn footer_label(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Plan => "plan",
            Self::Ask => "ask",
            Self::Brave => "brave",
        }
    }

    /// Label / border accent color in the TUI.
    ///
    /// - **Build** white · **Plan** darker soft yellow · **Ask** blue · **Brave** orange
    pub const fn label_rgb(self) -> (u8, u8, u8) {
        match self {
            Self::Build => (236, 234, 228), // soft warm white (not pure white, not grey)
            Self::Plan => (204, 168, 52),   // darker soft yellow
            Self::Ask => (59, 130, 246),    // blue-500
            Self::Brave => (249, 115, 22),  // orange-500
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Build => Self::Plan,
            Self::Plan => Self::Ask,
            Self::Ask => Self::Brave,
            Self::Brave => Self::Build,
        }
    }
}

/// Reasoning / thinking level (aligned with `elph_ai::ThinkingLevel` + TUI-only `Off`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ThinkingLevel {
    #[default]
    Off,
    Minimal,
    Low,
    Medium,
    High,
    Xhigh,
    Max,
}

impl ThinkingLevel {
    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Xhigh => "xhigh",
            Self::Max => "max",
        }
    }

    /// Thinking-level color for footer model group and related chrome.
    ///
    /// Readable soft strata, kept clear of agent-mode green / yellow / blue / orange:
    /// grey → cyan → periwinkle → peach → rose → lavender → soft magenta.
    pub const fn border_rgb(self) -> (u8, u8, u8) {
        match self {
            Self::Off => (156, 163, 175),    // soft grey
            Self::Minimal => (94, 200, 212), // soft cyan
            Self::Low => (123, 159, 212),    // periwinkle (not Ask blue)
            Self::Medium => (212, 165, 116), // soft peach (not Brave orange)
            Self::High => (220, 110, 118),   // soft but clearer red/rose
            Self::Xhigh => (180, 154, 217),  // soft lavender
            Self::Max => (196, 138, 212),    // soft magenta-violet
        }
    }

    pub fn from_setting(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "minimal" => Self::Minimal,
            "low" => Self::Low,
            "medium" => Self::Medium,
            "high" => Self::High,
            "xhigh" | "x-high" => Self::Xhigh,
            "max" => Self::Max,
            _ => Self::Off,
        }
    }

    pub fn next(self) -> Self {
        match self {
            Self::Off => Self::Minimal,
            Self::Minimal => Self::Low,
            Self::Low => Self::Medium,
            Self::Medium => Self::High,
            Self::High => Self::Xhigh,
            Self::Xhigh => Self::Max,
            Self::Max => Self::Off,
        }
    }
}

/// Actions the prompt can signal to the parent app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptAction {
    None,
    Submit(String),
    Queue(String),
    Steer(String),
    Clear,
    CycleMode,
}

/// Returns true when submitted text is the Neovim-style quit command (`:q`).
pub fn is_quit_command(text: &str) -> bool {
    text.trim() == ":q"
}

/// Returns true for forced quit (`:q!`) — exits immediately, even during an active turn.
pub fn is_force_quit_command(text: &str) -> bool {
    text.trim() == ":q!"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn modes_cycle() {
        assert_eq!(AgentMode::Build.next(), AgentMode::Plan);
        assert_eq!(AgentMode::Brave.next(), AgentMode::Build);
    }

    #[test]
    fn thinking_levels_cycle() {
        assert_eq!(ThinkingLevel::High.next(), ThinkingLevel::Xhigh);
        assert_eq!(ThinkingLevel::Xhigh.next(), ThinkingLevel::Max);
        assert_eq!(ThinkingLevel::Max.next(), ThinkingLevel::Off);
    }

    #[test]
    fn thinking_level_from_setting_accepts_max_and_xhigh() {
        assert_eq!(ThinkingLevel::from_setting("max"), ThinkingLevel::Max);
        assert_eq!(ThinkingLevel::from_setting("xhigh"), ThinkingLevel::Xhigh);
        assert_eq!(ThinkingLevel::from_setting("x-high"), ThinkingLevel::Xhigh);
        assert_eq!(ThinkingLevel::Max.label(), "max");
    }

    #[test]
    fn detects_quit_command() {
        assert!(is_quit_command(":q"));
        assert!(!is_quit_command(":q!"));
        assert!(!is_quit_command("hello"));
    }

    #[test]
    fn detects_force_quit_command() {
        assert!(is_force_quit_command(":q!"));
        assert!(!is_force_quit_command(":q"));
    }
}

/// One selectable row (previously in elph-tui diff module).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectItem {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
}

impl SelectItem {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
        }
    }

    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Slash command entry for prompt autocomplete (previously in elph-tui diff module).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlashCommand {
    pub name: String,
    pub description: String,
    pub args_hint: Option<String>,
    /// When true, omitted from the slash palette but still dispatchable when typed.
    pub hidden: bool,
}

impl SlashCommand {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            args_hint: None,
            hidden: false,
        }
    }

    pub fn with_args_hint(mut self, hint: impl Into<String>) -> Self {
        self.args_hint = Some(hint.into());
        self
    }

    pub fn with_hidden(mut self, hidden: bool) -> Self {
        self.hidden = hidden;
        self
    }

    pub fn palette_command_name(&self) -> String {
        format!("/{}", self.name)
    }

    pub fn palette_command_label(&self) -> String {
        match &self.args_hint {
            Some(hint) => format!("{} {hint}", self.palette_command_name()),
            None => self.palette_command_name(),
        }
    }
}
