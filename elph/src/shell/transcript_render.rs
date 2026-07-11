//! Formats [`TranscriptEntry`] rows for the tuie transcript pane.

use elph_tui::{CollapseState, ToolExecutionState, TranscriptEntry, TranscriptRole};

const USER_PREFIX: &str = "› ";
const DIAMOND: &str = "♦ ";

/// Flatten typed entries into scrollable transcript lines (Composer layout).
pub fn entries_to_lines(
    entries: &[TranscriptEntry],
    show_thinking: bool,
    agent_running: bool,
    collapse: &CollapseState,
) -> Vec<String> {
    let mut lines = Vec::new();
    let mut prev_role: Option<TranscriptRole> = None;

    for (index, entry) in entries.iter().enumerate() {
        if agent_running && entry.role == TranscriptRole::Assistant && entry.is_streaming {
            break;
        }
        let gap = section_gap(prev_role, entry.role);
        for _ in 0..gap {
            lines.push(String::new());
        }
        prev_role = Some(entry.role);
        lines.extend(entry_lines(entry, index, show_thinking, collapse));
    }
    lines
}

fn entry_lines(entry: &TranscriptEntry, index: usize, show_thinking: bool, collapse: &CollapseState) -> Vec<String> {
    match entry.role {
        TranscriptRole::User => format_user(&entry.content),
        TranscriptRole::Assistant => entry.content.lines().map(str::to_string).collect(),
        TranscriptRole::Thinking if show_thinking => {
            let duration = entry.timestamp.as_deref().map(|_| "0.2").unwrap_or("…");
            let header = format!("{DIAMOND}Thought for {duration}s");
            if collapse.is_expanded(index) && !entry.content.trim().is_empty() {
                std::iter::once(header)
                    .chain(entry.content.lines().map(str::to_string))
                    .collect()
            } else {
                vec![header]
            }
        }
        TranscriptRole::Thinking => Vec::new(),
        TranscriptRole::Tool => entry
            .tool
            .as_ref()
            .map(|tool| vec![format_tool_line(tool)])
            .unwrap_or_default(),
        TranscriptRole::System => vec![format!("{DIAMOND}{}", entry.content)],
    }
}

fn section_gap(prev: Option<TranscriptRole>, current: TranscriptRole) -> u32 {
    let Some(prev) = prev else {
        return 0;
    };
    match (prev, current) {
        (TranscriptRole::User, _) | (_, TranscriptRole::User) => 1,
        (TranscriptRole::Assistant, TranscriptRole::Assistant) => 0,
        _ => 1,
    }
}

fn format_user(message: &str) -> Vec<String> {
    let trimmed = message.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let mut lines_iter = trimmed.lines();
    let Some(first) = lines_iter.next() else {
        return Vec::new();
    };
    let mut out = vec![format!("{USER_PREFIX}{first}")];
    for line in lines_iter {
        out.push(format!("  {line}"));
    }
    out
}

fn format_tool_line(tool: &ToolExecutionState) -> String {
    let label = tool_block_label(tool);
    let detail = tool_detail_line(tool);
    if detail.is_empty() {
        format!("{DIAMOND}{label}")
    } else {
        format!("{DIAMOND}{label}  {detail}")
    }
}

fn tool_block_label(tool: &ToolExecutionState) -> &'static str {
    let name = tool.name.to_ascii_lowercase();
    if name.contains("edit") || name.contains("write") || name.contains("str_replace") || name.contains("patch") {
        "Edit"
    } else if name.contains("shell") || name.contains("bash") || name.contains("run") || name.contains("command") {
        "Run"
    } else {
        "Tool"
    }
}

fn tool_detail_line(tool: &ToolExecutionState) -> String {
    let args = tool.args_summary.trim();
    if args.is_empty() {
        return String::new();
    }
    if args.contains('/') || args.contains('.') {
        args.lines().next().unwrap_or(args).to_string()
    } else if args.chars().count() > 72 {
        format!("{}…", args.chars().take(69).collect::<String>())
    } else {
        args.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elph_tui::ToolExecutionStatus;

    #[test]
    fn user_entry_uses_composer_prefix() {
        let lines = entries_to_lines(
            &[TranscriptEntry::user("hello")],
            true,
            false,
            &CollapseState::default(),
        );
        assert_eq!(lines, vec!["› hello".to_string()]);
    }

    #[test]
    fn streaming_assistant_truncates_while_running() {
        let entries = vec![
            TranscriptEntry::user("go"),
            TranscriptEntry::assistant_streaming("partial"),
            TranscriptEntry::assistant("done"),
        ];
        let lines = entries_to_lines(&entries, true, true, &CollapseState::default());
        assert!(lines.iter().any(|l| l == "› go"));
        assert!(!lines.iter().any(|l| l == "done"));
    }

    #[test]
    fn tool_line_includes_detail() {
        let tool = ToolExecutionState::new("1", "edit")
            .with_args("/src/main.rs")
            .with_status(ToolExecutionStatus::Success);
        let lines = entries_to_lines(&[TranscriptEntry::tool(tool)], true, false, &CollapseState::default());
        assert_eq!(lines, vec!["♦ Edit  /src/main.rs".to_string()]);
    }

    #[test]
    fn multiline_user_indents_continuation() {
        let lines = entries_to_lines(
            &[TranscriptEntry::user("first\nsecond")],
            true,
            false,
            &CollapseState::default(),
        );
        assert_eq!(lines, vec!["› first".to_string(), "  second".to_string()]);
    }

    #[test]
    fn thinking_collapsed_by_default() {
        let entry = TranscriptEntry::thinking("internal plan", false);
        let lines = entries_to_lines(&[entry], true, false, &CollapseState::default());
        assert_eq!(lines.len(), 1);
        assert!(lines[0].starts_with('♦'));
        assert!(!lines[0].contains("internal"));
    }

    #[test]
    fn system_entry_uses_diamond_prefix() {
        let lines = entries_to_lines(
            &[TranscriptEntry::system("ready")],
            true,
            false,
            &CollapseState::default(),
        );
        assert_eq!(lines, vec!["♦ ready".to_string()]);
    }
}
