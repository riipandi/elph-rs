//! Shared formatting for tool execution output in the Owly TUI.

use elph_tui::ToolExecutionState;

/// First-line preview of tool output for compact UI rows.
pub fn tool_output_preview(output: &str, max_chars: usize) -> Option<String> {
    let max_chars = max_chars.max(8);
    let line = output.trim().lines().find(|line| !line.trim().is_empty())?;
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(truncate_chars(trimmed, max_chars))
}

/// Compact chip/summary label: `name args` with optional output preview.
pub fn tool_chip_label(tool: &ToolExecutionState, args_max: usize, preview_max: usize) -> String {
    let args = tool.args_summary.trim();
    let base = if args.is_empty() {
        tool.name.clone()
    } else {
        format!("{} {}", tool.name, truncate_chars(args, args_max))
    };
    if let Some(preview) = tool_output_preview(&tool.output, preview_max) {
        format!("{base} → {preview}")
    } else {
        base
    }
}

pub fn truncate_chars(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        value.to_string()
    } else {
        let end = value
            .char_indices()
            .nth(max.saturating_sub(1))
            .map(|(i, _)| i)
            .unwrap_or(value.len());
        format!("{}…", &value[..end])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use elph_tui::{ToolExecutionState, ToolExecutionStatus};

    #[test]
    fn preview_uses_first_non_empty_line() {
        let preview = tool_output_preview("\n\nWrote 42 bytes\n", 20).expect("preview");
        assert_eq!(preview, "Wrote 42 bytes");
    }

    #[test]
    fn chip_includes_output_arrow() {
        let tool = ToolExecutionState::new("1", "write")
            .with_args(r#"{"path":"a.md"}"#)
            .with_output("Wrote 120 bytes to a.md")
            .with_status(ToolExecutionStatus::Success);
        let label = tool_chip_label(&tool, 24, 32);
        assert!(label.contains("write"));
        assert!(label.contains("→ Wrote 120 bytes"));
    }
}
