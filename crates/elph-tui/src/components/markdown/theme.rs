//! Pi-aligned markdown colors for transcript rendering.

use crate::components::theme::UiTheme;
use iocraft::prelude::{Color, Weight};

/// Semantic markdown palette (Pi `dark` theme).
#[derive(Clone, Copy, Debug)]
pub struct MarkdownTheme {
    pub ui: UiTheme,
    pub body: Color,
    pub heading: Color,
    pub heading_weight: Weight,
    pub strong: Color,
    pub emphasis: Color,
    pub inline_code: Color,
    pub link: Color,
    pub code_bg: Color,
    pub code_inset: u16,
    pub blockquote: Color,
    pub list_marker: Color,
}

impl MarkdownTheme {
    pub fn from_ui_theme(theme: UiTheme) -> Self {
        Self {
            ui: theme,
            body: theme.text_primary,
            heading: theme.warning,
            heading_weight: Weight::Bold,
            strong: theme.text_primary,
            emphasis: theme.text_secondary,
            inline_code: theme.success,
            link: theme.accent,
            code_bg: theme.selection_bg,
            code_inset: theme.container_inset(),
            blockquote: theme.text_muted,
            list_marker: theme.accent_soft,
        }
    }
}

impl Default for MarkdownTheme {
    fn default() -> Self {
        Self::from_ui_theme(UiTheme::default())
    }
}
