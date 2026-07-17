//! Slash command palette model (demo-local mirror of elph slash_palette).

pub use elph_tui::slash_palette::SlashCommand;

/// Registry for the coding-agent simulator — each entry maps to a TUI component demo.
pub fn coding_agent_registry() -> Vec<SlashCommand> {
    vec![
        SlashCommand::new("help", "List demo slash commands and shortcuts"),
        SlashCommand::new("clear", "Reset transcript to seed data"),
        SlashCommand::new("compact", "Simulate history compaction notice"),
        SlashCommand::new("demo-mode", "DialogModeSelectContent — agent mode picker"),
        SlashCommand::new("demo-model", "DialogQuestionContent — single-choice list"),
        SlashCommand::new("demo-multi", "DialogMultiChoiceContent — multi-select list"),
        SlashCommand::new("demo-input", "DialogUserInputContent — free-text answer"),
        SlashCommand::new("demo-confirm", "DialogConfirmContent — y/n tool approval"),
        SlashCommand::new("demo-confirm-buttons", "DialogConfirmButtonsContent — Yes/No buttons"),
        SlashCommand::new("demo-todo", "DialogTodoListContent — session goals checklist"),
        SlashCommand::new("demo-progress", "DialogTodoProgressContent — spinner progress rows"),
        SlashCommand::new("demo-busy", "StatusRow — simulate thinking turn + progress overlay"),
        SlashCommand::new("demo-tool", "Transcript tool card — success"),
        SlashCommand::new("demo-tool-fail", "Transcript tool card — failure"),
        SlashCommand::new("demo-thinking", "Transcript thinking → assistant pair"),
        SlashCommand::new("demo-skill", "Transcript skill prompt card"),
        SlashCommand::new("demo-meta", "Transcript meta steering notice"),
    ]
}
