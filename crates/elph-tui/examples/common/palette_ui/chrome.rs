//! Resolved chrome tokens for one palette card render.

use elph_tui::components::theme::{UiTheme, list_row_desc_style, list_row_name_style};
use elph_tui::slash_palette::PaletteSnapshot;
use iocraft::prelude::Color;

use super::row_layout;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteCardChrome {
    pub card_width: u16,
    pub list_width: u16,
    pub command_column_width: u16,
    pub border_color: Color,
    pub background: Color,
    pub title_color: Color,
    pub name_idle_color: Color,
    pub name_active_color: Color,
    pub desc_active_color: Color,
    pub desc_idle_color: Color,
    pub title: String,
}

impl Default for PaletteCardChrome {
    fn default() -> Self {
        Self {
            card_width: 0,
            list_width: 0,
            command_column_width: 0,
            border_color: Color::Reset,
            background: Color::Reset,
            title_color: Color::Reset,
            name_idle_color: Color::Reset,
            name_active_color: Color::Reset,
            desc_active_color: Color::Reset,
            desc_idle_color: Color::Reset,
            title: String::new(),
        }
    }
}

impl PaletteCardChrome {
    pub fn from_snapshot(screen_width: u16, theme: UiTheme, snapshot: &PaletteSnapshot) -> Self {
        let card_width = row_layout::palette_card_width(screen_width);
        let list_width = row_layout::palette_list_width(screen_width);
        let command_column_width = row_layout::palette_command_column_width(&snapshot.options, list_width);
        let (name_active, _) = list_row_name_style(theme, true);
        let (name_idle, _) = list_row_name_style(theme, false);
        Self {
            card_width,
            list_width,
            command_column_width,
            border_color: theme.border,
            background: Color::Reset,
            title_color: theme.text_muted,
            name_idle_color: name_idle,
            name_active_color: name_active,
            desc_active_color: list_row_desc_style(theme, true),
            desc_idle_color: list_row_desc_style(theme, false),
            title: format!("{:02} Commands", snapshot.match_count),
        }
    }
}
