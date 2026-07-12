//! Core terminal formatting: headers, banners, completion footers, tool output.

use std::io::Write;

use elph_agent::AgentState;

const BANNER_INNER_WIDTH: usize = 52;

pub fn print_banner(provider: &str, model: &str, directory: &std::path::Path) {
    let version = env!("CARGO_PKG_VERSION");
    let border = "─".repeat(BANNER_INNER_WIDTH);

    println!();
    println!("  ┌{border}┐");
    println!("{}", banner_title(version));
    println!("{}", banner_field("provider", provider, "\x1b[32m"));
    println!("{}", banner_field("model", model, "\x1b[32m"));
    println!(
        "{}",
        banner_field(
            "directory",
            &truncate_path(directory, BANNER_INNER_WIDTH - "directory: ".len()),
            "",
        )
    );
    println!("  └{border}┘");
    println!();
}

fn banner_title(version: &str) -> String {
    let plain = format!(">_ Owly v{version} agent docs for codebases");
    let styled = format!("\x1b[36;1m>_ Owly\x1b[0m \x1b[2mv{version}\x1b[0m agent docs for codebases");
    banner_line(&plain, &styled)
}

fn banner_field(label: &str, value: &str, color: &str) -> String {
    let prefix = format!("{label}: ");
    let max_value = BANNER_INNER_WIDTH.saturating_sub(prefix.len());
    let value = truncate_display(value, max_value);
    let plain = format!("{prefix}{value}");
    let styled = if color.is_empty() {
        plain.clone()
    } else {
        format!("{prefix}{color}{value}\x1b[0m")
    };
    banner_line(&plain, &styled)
}

fn banner_line(plain: &str, styled: &str) -> String {
    let pad = BANNER_INNER_WIDTH.saturating_sub(plain.len());
    format!("  │ {styled}{}│", " ".repeat(pad))
}

fn truncate_display(value: &str, max_len: usize) -> String {
    if max_len == 0 {
        return String::new();
    }
    if value.len() <= max_len {
        return value.to_string();
    }
    if max_len <= 3 {
        return ".".repeat(max_len);
    }
    format!("...{}", &value[value.len() - max_len + 3..])
}

pub fn print_command_header(command: &str, provider: &str, model: &str) {
    println!();
    println!("\x1b[36;1m>_ Owly {command}\x1b[0m");
    println!("provider: \x1b[32m{provider}\x1b[0m");
    println!("model: \x1b[32m{model}\x1b[0m");
    println!();
}

/// Compact header for one-shot chat runs (streaming-friendly).
pub fn print_chat_header(provider: &str, model: &str) {
    println!("\x1b[36;1m>_ Owly Chat\x1b[0m \x1b[2m{provider} · {model}\x1b[0m");
    println!();
}

/// Dimmed timing footer after streamed or batch assistant output.
pub fn format_stream_footer(elapsed_secs: f64, streamed: bool, ends_with_newline: bool) -> String {
    let mut footer = String::new();
    if streamed && !ends_with_newline {
        footer.push('\n');
    }
    if streamed {
        footer.push('\n');
    }
    footer.push_str(&format!("\x1b[2mCompleted in {elapsed_secs:.1}s\x1b[0m"));
    footer
}

pub fn print_agent_status(message: &str) {
    println!("\x1b[2m[status]\x1b[0m {message}");
}

pub fn print_tool_call(name: &str, verbose: bool) {
    if verbose {
        eprintln!("  \x1b[36m> {name}\x1b[0m");
    }
}

pub fn print_tool_result(name: &str, success: bool, verbose: bool) {
    if verbose {
        let icon = if success {
            "\x1b[32m✓\x1b[0m"
        } else {
            "\x1b[31m✗\x1b[0m"
        };
        eprintln!("  {icon} {name}");
    }
}

pub fn print_completion(message: &str) {
    println!();
    println!("\x1b[32;1m✓\x1b[0m {message}");
    println!();
}

pub fn print_warning(message: impl AsRef<str>) {
    eprintln!("{}", message.as_ref());
}

pub fn print_assistant_response(state: &AgentState) -> bool {
    let Some(elph_ai::Message::Assistant(assistant)) = state.messages.last().and_then(|m| m.as_llm()) else {
        return false;
    };
    let mut wrote = false;
    for block in &assistant.content {
        if let elph_ai::AssistantContentBlock::Text(t) = block
            && !t.text.is_empty()
        {
            print!("{}", t.text);
            wrote = true;
        }
    }
    if wrote {
        println!();
        let _ = std::io::stdout().flush();
    }
    wrote
}

pub fn truncate_path_for_display(path: &std::path::Path, max_len: usize) -> String {
    truncate_display(&path.display().to_string(), max_len)
}

fn truncate_path(path: &std::path::Path, max_len: usize) -> String {
    truncate_path_for_display(path, max_len)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stream_footer_adds_spacing_after_stream() {
        let footer = format_stream_footer(2.5, true, true);
        assert!(footer.starts_with('\n'));
        assert!(footer.contains("Completed in 2.5s"));
    }

    #[test]
    fn stream_footer_appends_newline_when_stream_lacks_one() {
        let footer = format_stream_footer(1.0, true, false);
        assert!(footer.starts_with('\n'));
    }
}
