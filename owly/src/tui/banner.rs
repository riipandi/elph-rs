//! Owly session chrome (fixed status line above the transcript).

use std::path::Path;

use elph_tui::Theme;
use slt::Context;

use crate::cli::truncate_path_for_display;

pub fn directory_display(cwd: &Path) -> String {
    truncate_path_for_display(cwd, 48)
}

/// Session metadata for the fixed status line.
#[derive(Debug, Clone, Copy)]
pub struct OwlyBannerInfo<'a> {
    pub provider: &'a str,
    pub model: &'a str,
    pub directory: &'a str,
    pub version: &'a str,
}

/// One-line session context pinned above the scrollable transcript.
pub fn render_status_line(ui: &mut Context, banner: OwlyBannerInfo<'_>, theme: Theme) {
    let line = format!(
        "owly v{} · {} · {} · {}",
        banner.version, banner.model, banner.provider, banner.directory
    );
    let _ = ui.text(line).fg(theme.muted);
}
