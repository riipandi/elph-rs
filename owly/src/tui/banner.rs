//! Owly startup banner.

use std::path::Path;

use elph_tui::Theme;
use slt::{Border, Color, Context};

use crate::cli::truncate_path_for_display;

use super::chrome::subtle_border;

pub fn directory_display(cwd: &Path) -> String {
    truncate_path_for_display(cwd, 48)
}

pub fn render_banner(ui: &mut Context, provider: &str, model: &str, directory: &str, version: &str, theme: Theme) {
    let title = format!(">_ Owly v{version} agent docs for codebases");
    let _ = ui
        .bordered(Border::Single)
        .border_fg(subtle_border(theme))
        .p(1)
        .gap(1)
        .col(|ui| {
            let _ = ui.text(title).bold().fg(Color::Cyan);
            let _ = ui.text(format!("provider: {provider}")).fg(Color::Green);
            let _ = ui.text(format!("model: {model}")).fg(Color::Green);
            let _ = ui.text(format!("directory: {directory}")).fg(theme.muted);
        });
}
