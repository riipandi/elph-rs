/// Actions the prompt layer can signal to the parent app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PromptAction {
    None,
    /// Submit immediately (agent idle).
    Submit(String),
    /// Queue until the current stream / turn finishes.
    Queue(String),
    /// Interrupt the in-flight response and steer with this message.
    Steer(String),
    Clear,
    CycleMode,
}

/// Resolves the prompt prefix from input content (`>`, `/`, `$`, `#`).
pub fn detect_prompt_prefix(text: &str) -> char {
    let trimmed = text.trim_start();
    if trimmed.starts_with("!!") {
        '#'
    } else if trimmed.starts_with('!') {
        '$'
    } else if trimmed.starts_with('/') {
        '/'
    } else {
        '>'
    }
}

/// Strips shell/slash trigger prefixes before submit (`/cmd` → `cmd`, `!!rpt` → `rpt`).
pub fn strip_submit_trigger(text: &str) -> String {
    let trimmed = text.trim();
    if let Some(rest) = trimmed.strip_prefix("!!") {
        rest.trim_start().to_string()
    } else if let Some(rest) = trimmed.strip_prefix('!') {
        rest.trim_start().to_string()
    } else if let Some(rest) = trimmed.strip_prefix('/') {
        rest.trim_start().to_string()
    } else {
        trimmed.to_string()
    }
}

/// Returns true when submitted text is the Neovim-style quit command (`:q`).
pub fn is_quit_command(text: &str) -> bool {
    matches!(text.trim(), ":q" | ":q!")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefix_detection_order() {
        assert_eq!(detect_prompt_prefix("!!git status"), '#');
        assert_eq!(detect_prompt_prefix("!ls"), '$');
        assert_eq!(detect_prompt_prefix("/help"), '/');
        assert_eq!(detect_prompt_prefix("hello"), '>');
    }

    #[test]
    fn strip_submit_triggers() {
        assert_eq!(strip_submit_trigger("/help"), "help");
        assert_eq!(strip_submit_trigger("!!rpt"), "rpt");
        assert_eq!(strip_submit_trigger("!ls -la"), "ls -la");
        assert_eq!(strip_submit_trigger("plain"), "plain");
    }

    #[test]
    fn detects_quit_command() {
        assert!(is_quit_command(":q"));
        assert!(!is_quit_command("hello"));
    }
}
