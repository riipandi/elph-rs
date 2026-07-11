use crate::diff::SlashCommand;

/// Built-in slash commands for the Elph TUI palette.
pub fn elph_builtin_commands() -> Vec<SlashCommand> {
    vec![
        SlashCommand::new("help", "List all commands"),
        SlashCommand::new("model", "Open model selector"),
        SlashCommand::new("exit", "Quit"),
        SlashCommand::new("quit", "Quit"),
        SlashCommand::new("changelog", "Show version history"),
        SlashCommand::new("compact", "Compact conversation history"),
        SlashCommand::new("goal", "Manage session goals"),
        SlashCommand::new("settings", "Open settings"),
    ]
}

/// Built-in slash commands for the Owly documentation shell.
pub fn owly_builtin_commands() -> Vec<SlashCommand> {
    vec![
        SlashCommand::new("help", "List commands"),
        SlashCommand::new("init", "Initialize openwiki"),
        SlashCommand::new("update", "Refresh documentation"),
        SlashCommand::new("history", "List recent checkpoints"),
        SlashCommand::new("restore", "Restore checkpoint (# or id)"),
        SlashCommand::new("clear", "Reset thread"),
        SlashCommand::new("name", "Show or set session title"),
        SlashCommand::new("exit", "Quit"),
        SlashCommand::new("quit", "Quit"),
    ]
}
