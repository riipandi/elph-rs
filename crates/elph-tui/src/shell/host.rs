//! Object-safe host trait backing the tuie agent shell.

use crate::diff::SlashCommand;
use crate::keymap::ShellAction;
use crate::prompt::{AgentMode, PromptAction};
use crate::theme::Theme;

/// Owned chrome snapshot consumed by tuie footer / activity widgets.
#[derive(Debug, Clone, Default)]
pub struct ShellChromeData {
    pub running: bool,
    pub sidebar_open: bool,
    pub palette_open: bool,
    pub activity_visible: bool,
    pub activity_label: String,
    pub activity_cancel_requested: bool,
    pub model_name: String,
    pub provider: String,
    pub thinking_level: String,
    pub supports_images: bool,
    pub cost_usd: f64,
    pub tokens_used: u64,
    pub context_pct: f64,
    pub context_limit: u64,
    pub project_dir: String,
    pub session_id: String,
    pub mode: AgentMode,
    pub turn: u32,
    pub branch: String,
    pub git_additions: u32,
    pub git_deletions: u32,
}

/// Application state and callbacks surfaced to [`crate::shell::tuie_shell::AgentShell`].
pub trait ShellHost {
    fn poll(&mut self);
    fn should_exit(&self) -> bool;
    fn chrome(&self) -> ShellChromeData;
    fn commands(&self) -> Vec<SlashCommand>;
    fn transcript_lines(&self) -> Vec<String>;
    fn on_shell_action(&mut self, action: ShellAction);
    fn on_prompt_action(&mut self, action: PromptAction);
    fn running(&self) -> bool;
    fn sidebar_open(&self) -> bool;
    fn set_sidebar_open(&mut self, open: bool);
    fn palette_open(&self) -> bool;
    fn set_palette_open(&mut self, open: bool);
    fn theme(&self) -> Theme;
    fn set_theme(&mut self, theme: Theme);
    fn prompt_text(&self) -> String;
    fn set_prompt_text(&mut self, text: String);
    fn clear_prompt(&mut self);
}
