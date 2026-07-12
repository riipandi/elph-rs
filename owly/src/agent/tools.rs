use elph_agent::{AgentToolResult, ToolResultContent};

fn truncate_with_ellipsis(value: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    if value.chars().count() <= max_chars {
        return value.to_string();
    }
    let cut: String = value.chars().take(max_chars).collect();
    format!("{cut}...")
}

pub(super) fn summarize_tool_args(tool_name: &str, args: &serde_json::Value) -> String {
    if crate::runtime::ask_user::ASK_TOOL_NAMES.contains(&tool_name) {
        crate::runtime::ask_user::format_args_summary(tool_name, args)
    } else {
        args.to_string()
    }
}

pub(super) fn summarize_tool_result(result: &AgentToolResult) -> String {
    const MAX: usize = 4_096;
    let mut out = String::new();
    for block in &result.content {
        match block {
            ToolResultContent::Text(text) => {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(&text.text);
            }
            ToolResultContent::Image(_) => {
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str("[image output]");
            }
        }
        if out.chars().count() >= MAX {
            return truncate_with_ellipsis(&out, MAX);
        }
    }
    if out.is_empty() {
        let details = serde_json::to_string(&result.details).unwrap_or_default();
        if !details.is_empty() && details != "{}" && details != "null" {
            if details.chars().count() > MAX {
                return truncate_with_ellipsis(&details, MAX);
            }
            return details;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use elph_agent::AgentToolResult;
    #[test]
    fn truncate_with_ellipsis_respects_multibyte_characters() {
        let input = "日本語".repeat(2_000);
        let truncated = truncate_with_ellipsis(&input, 100);
        assert!(truncated.ends_with("..."));
        assert!(truncated.is_char_boundary(truncated.len()));
        assert!(truncated.chars().count() <= 103);
    }

    #[test]
    fn summarize_tool_result_truncates_utf8_safely() {
        let text = "é".repeat(5_000);
        let result = AgentToolResult::text(text);
        let summary = summarize_tool_result(&result);
        assert!(summary.ends_with("..."));
        assert!(summary.is_char_boundary(summary.len()));
    }
}
