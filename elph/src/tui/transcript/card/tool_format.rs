//! Tool card argument and output formatting.

pub use crate::tui::tool_params::format_tool_params_display as format_tool_args_display;

pub const TOOL_OUTPUT_MAX_LINES: usize = 12;
pub const TOOL_OUTPUT_MAX_CHARS: usize = 1_500;

pub fn format_tool_output_display(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.chars().count() <= TOOL_OUTPUT_MAX_CHARS {
        let lines: Vec<&str> = trimmed.lines().collect();
        if lines.len() <= TOOL_OUTPUT_MAX_LINES {
            return trimmed.to_string();
        }
        let mut body = lines
            .iter()
            .take(TOOL_OUTPUT_MAX_LINES)
            .copied()
            .collect::<Vec<_>>()
            .join("\n");
        body.push_str(&format!("\n… ({line_count} lines total)", line_count = lines.len()));
        return body;
    }
    let truncated: String = trimmed.chars().take(TOOL_OUTPUT_MAX_CHARS.saturating_sub(1)).collect();
    format!("{truncated}…")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_args_json_single_key_shows_value_only() {
        assert_eq!(format_tool_args_display(r#"{"path":"src/lib.rs"}"#), "src/lib.rs");
    }

    #[test]
    fn tool_output_truncates_long_bodies() {
        let long = "line\n".repeat(20);
        let display = format_tool_output_display(&long);
        assert!(display.contains("lines total"));
    }
}
