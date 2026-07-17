//! Pure derivations from editor draft and command registry.

use crate::types::SelectOption;

use super::SlashCommand;
use super::fuzzy::filter_commands;

pub const MAX_VISIBLE_ROWS: u16 = 8;
pub const FAST_SCROLL_STEP: usize = 5;

/// Render-ready snapshot derived from the editor draft and command list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaletteSnapshot {
    pub visible: bool,
    pub query: String,
    pub filtered: Vec<SlashCommand>,
    pub options: Vec<SelectOption>,
    pub list_height: u16,
    pub match_count: usize,
}

impl Default for PaletteSnapshot {
    fn default() -> Self {
        Self::hidden()
    }
}

impl PaletteSnapshot {
    pub fn hidden() -> Self {
        Self {
            visible: false,
            query: String::new(),
            filtered: Vec::new(),
            options: Vec::new(),
            list_height: 0,
            match_count: 0,
        }
    }

    pub fn should_render(&self) -> bool {
        self.visible
    }

    pub fn has_matches(&self) -> bool {
        !self.options.is_empty()
    }

    pub fn from_draft(draft: &str, commands: &[SlashCommand], screen_height: u16) -> Self {
        build_snapshot(draft, commands, screen_height)
    }
}

/// Palette is shown only while the user is typing a command name (before the first space).
pub fn palette_visible(draft: &str) -> bool {
    let trimmed = draft.trim_start();
    if !trimmed.starts_with('/') {
        return false;
    }
    let body = trimmed.trim_start_matches('/').trim_start();
    !body.contains(' ')
}

/// Filter query from the current draft (command name only, lowercased).
pub fn palette_query(draft: &str) -> String {
    let trimmed = draft.trim_start();
    let body = trimmed.trim_start_matches('/').trim_start();
    body.split_once(' ').map_or(body, |(name, _)| name).to_ascii_lowercase()
}

/// Map filtered commands to select rows.
pub fn commands_to_options(commands: &[SlashCommand]) -> Vec<SelectOption> {
    commands
        .iter()
        .map(|cmd| SelectOption::new(format!("/{}", cmd.name), cmd.description.clone()))
        .collect()
}

pub fn list_viewport_cap(screen_height: u16) -> usize {
    if screen_height < 24 {
        4
    } else if screen_height < 36 {
        6
    } else {
        MAX_VISIBLE_ROWS as usize
    }
}

/// List body height: all matches when few, capped at viewport when many.
pub fn list_height(option_count: usize, screen_height: u16) -> u16 {
    if option_count == 0 {
        return 1;
    }
    option_count.min(list_viewport_cap(screen_height)).max(1) as u16
}

pub fn palette_window_start(selected: usize, height: usize, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    let viewport = height.max(1).min(len);
    let max_start = len.saturating_sub(viewport);
    selected.saturating_sub(viewport / 2).min(max_start)
}

pub fn clamp_index(index: usize, len: usize) -> usize {
    if len == 0 { 0 } else { index.min(len - 1) }
}

/// Replace the command name in a draft and preserve trailing args.
pub fn complete_command(draft: &str, command_name: &str) -> String {
    let trimmed = draft.trim_start();
    let body = trimmed.trim_start_matches('/').trim_start();
    let args = body.split_once(' ').map(|(_, args)| args.trim()).unwrap_or("");
    if args.is_empty() {
        format!("/{command_name} ")
    } else {
        format!("/{command_name} {args}")
    }
}

pub fn selected_command_name(filtered: &[SlashCommand], index: usize) -> Option<&str> {
    filtered
        .get(clamp_index(index, filtered.len()))
        .map(|cmd| cmd.name.as_str())
}

pub fn build_snapshot(draft: &str, commands: &[SlashCommand], screen_height: u16) -> PaletteSnapshot {
    if !palette_visible(draft) {
        return PaletteSnapshot::hidden();
    }
    let query = palette_query(draft);
    let filtered = filter_commands(commands, &query);
    let options = commands_to_options(&filtered);
    let match_count = options.len();
    PaletteSnapshot {
        visible: true,
        query,
        filtered,
        options,
        list_height: list_height(match_count, screen_height),
        match_count,
    }
}

/// Seed the prompt with `/` to open the palette (idempotent when already visible).
pub fn open_palette_draft(existing: &str) -> Option<String> {
    if palette_visible(existing) || !existing.is_empty() {
        return None;
    }
    Some("/".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bare_slash_opens_palette() {
        assert!(palette_visible("/"));
        let snap = build_snapshot("/", &[SlashCommand::new("help", "Help")], 40);
        assert!(snap.should_render());
        assert_eq!(snap.match_count, 1);
    }

    #[test]
    fn palette_hides_after_first_space() {
        assert!(!palette_visible("/help args"));
    }

    #[test]
    fn snapshot_visible_without_matches() {
        let snap = build_snapshot("/zzz", &[SlashCommand::new("help", "Help")], 40);
        assert!(snap.should_render());
        assert!(!snap.has_matches());
        assert_eq!(snap.list_height, 1);
    }
}
