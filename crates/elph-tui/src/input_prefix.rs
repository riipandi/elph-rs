//! Dynamic prompt prefix detection for the shell editor.

/// Default chat prompt chevron: U+276F MEDIUM RIGHT-POINTING ANGLE BRACKET ORNAMENT (`❯`).
pub const DEFAULT_PROMPT_PREFIX_GLYPH: &str = "\u{276F}";

/// Selection marker for list rows (same glyph as [`DEFAULT_PROMPT_PREFIX_GLYPH`]).
pub const LIST_SELECTION_MARKER: &str = DEFAULT_PROMPT_PREFIX_GLYPH;

/// Two-column row prefix for palette-style lists (`❯ ` when selected, `  ` otherwise).
pub const LIST_SELECTION_ROW_PREFIX_SELECTED: &str = "\u{276F} ";
pub const LIST_SELECTION_ROW_PREFIX_IDLE: &str = "  ";

/// Row prefix before list labels in the question dialog and slash palette.
pub fn list_selection_row_prefix(selected: bool) -> &'static str {
    if selected {
        LIST_SELECTION_ROW_PREFIX_SELECTED
    } else {
        LIST_SELECTION_ROW_PREFIX_IDLE
    }
}

/// Horizontal space reserved for the prefix glyph and trailing gap.
pub const PREFIX_COLUMN_WIDTH: u16 = 2;

/// Input kind inferred from the draft (after leading whitespace is trimmed).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InputPrefixKind {
    #[default]
    Default,
    Slash,
    ShellWithContext,
    ShellNoContext,
}

/// Controls whether the prefix column is rendered and which glyph represents normal chat.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromptPrefixConfig {
    pub enabled: bool,
    pub default_glyph: &'static str,
}

impl Default for PromptPrefixConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_glyph: DEFAULT_PROMPT_PREFIX_GLYPH,
        }
    }
}

/// Detect prefix kind from draft text. Leading spaces are ignored.
pub fn detect_input_prefix(draft: &str, _config: &PromptPrefixConfig) -> InputPrefixKind {
    let trimmed = draft.trim_start();
    if trimmed.starts_with("!!") {
        InputPrefixKind::ShellNoContext
    } else if trimmed.starts_with('!') {
        InputPrefixKind::ShellWithContext
    } else if trimmed.starts_with('/') {
        InputPrefixKind::Slash
    } else {
        InputPrefixKind::Default
    }
}

/// Glyph shown in the prefix column for the given kind.
pub fn prefix_symbol(kind: InputPrefixKind, config: &PromptPrefixConfig) -> &'static str {
    match kind {
        InputPrefixKind::Default => config.default_glyph,
        InputPrefixKind::Slash => "/",
        InputPrefixKind::ShellWithContext => "$",
        InputPrefixKind::ShellNoContext => "#",
    }
}

/// Strip the trigger prefix on submit. Normal chat returns the original text unchanged.
pub fn strip_submit_trigger(raw: &str) -> (InputPrefixKind, String) {
    let kind = detect_input_prefix(raw, &PromptPrefixConfig::default());
    strip_body_triggers(kind, raw)
}

/// Remove redundant trigger characters from the body when the prefix column already shows them.
pub fn strip_body_triggers(kind: InputPrefixKind, raw: &str) -> (InputPrefixKind, String) {
    if kind == InputPrefixKind::Default {
        return (kind, raw.to_string());
    }
    let (leading, trimmed) = split_leading_whitespace(raw);
    let stripped = match kind {
        InputPrefixKind::ShellNoContext => trimmed.strip_prefix("!!").unwrap_or(trimmed),
        InputPrefixKind::ShellWithContext => {
            if trimmed.starts_with("!!") {
                trimmed.strip_prefix("!!").unwrap_or(trimmed)
            } else {
                trimmed.strip_prefix('!').unwrap_or(trimmed)
            }
        }
        InputPrefixKind::Slash => trimmed.strip_prefix('/').unwrap_or(trimmed),
        InputPrefixKind::Default => trimmed,
    };
    let body = format!("{leading}{}", stripped.trim_start());
    (kind, body)
}

/// Compose the logical draft string used by slash palette filtering.
pub fn compose_palette_draft(kind: InputPrefixKind, body: &str) -> String {
    match kind {
        InputPrefixKind::Slash => {
            let trimmed = body.trim_start();
            if trimmed.starts_with('/') {
                trimmed.to_string()
            } else {
                format!("/{trimmed}")
            }
        }
        _ => body.to_string(),
    }
}

/// Resolve prefix kind for display when the prefix column is enabled.
pub fn effective_prefix_kind(stored_kind: InputPrefixKind, body: &str, config: &PromptPrefixConfig) -> InputPrefixKind {
    if config.enabled {
        stored_kind
    } else {
        detect_input_prefix(body, config)
    }
}

/// Consume a trigger key typed at an empty body; returns the new kind when the char is not inserted.
pub fn try_consume_trigger(kind: InputPrefixKind, body: &str, ch: char, enabled: bool) -> Option<InputPrefixKind> {
    if !enabled || !body.trim().is_empty() {
        return None;
    }
    match (kind, ch) {
        (InputPrefixKind::Default, '/') => Some(InputPrefixKind::Slash),
        (InputPrefixKind::Default, '!') => Some(InputPrefixKind::ShellWithContext),
        (InputPrefixKind::ShellWithContext, '!') => Some(InputPrefixKind::ShellNoContext),
        _ => None,
    }
}

/// Step down the stored prefix kind on backspace when the body is empty.
pub fn backspace_trigger_kind(kind: InputPrefixKind, body: &str, enabled: bool) -> Option<InputPrefixKind> {
    if !enabled || !body.is_empty() {
        return None;
    }
    match kind {
        InputPrefixKind::ShellNoContext => Some(InputPrefixKind::ShellWithContext),
        InputPrefixKind::ShellWithContext | InputPrefixKind::Slash => Some(InputPrefixKind::Default),
        InputPrefixKind::Default => None,
    }
}

/// Merge inline trigger characters from pasted or completed text into stored prefix state.
pub fn absorb_inline_triggers(
    kind: InputPrefixKind,
    body: &str,
    config: &PromptPrefixConfig,
) -> (InputPrefixKind, String) {
    if !config.enabled {
        return strip_submit_trigger(body);
    }
    let (detected, stripped) = strip_submit_trigger(body);
    let kind = if kind == InputPrefixKind::Default && detected != InputPrefixKind::Default {
        detected
    } else {
        kind
    };
    strip_body_triggers(kind, &stripped)
}

/// Resolve submit payload from stored prefix state and textarea body.
pub fn resolve_submit_draft(
    stored_kind: InputPrefixKind,
    body: &str,
    config: &PromptPrefixConfig,
) -> (InputPrefixKind, String) {
    if config.enabled {
        strip_body_triggers(stored_kind, body)
    } else {
        strip_submit_trigger(body)
    }
}

fn split_leading_whitespace(raw: &str) -> (&str, &str) {
    let trimmed = raw.trim_start();
    let leading_len = raw.len().saturating_sub(trimmed.len());
    let (leading, _) = raw.split_at(leading_len);
    (leading, trimmed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_draft_is_default() {
        assert_eq!(
            detect_input_prefix("", &PromptPrefixConfig::default()),
            InputPrefixKind::Default
        );
    }

    #[test]
    fn leading_spaces_trimmed_before_detection() {
        assert_eq!(
            detect_input_prefix("  /help", &PromptPrefixConfig::default()),
            InputPrefixKind::Slash
        );
        assert_eq!(
            detect_input_prefix("  !ls", &PromptPrefixConfig::default()),
            InputPrefixKind::ShellWithContext
        );
        assert_eq!(
            detect_input_prefix("  !!pwd", &PromptPrefixConfig::default()),
            InputPrefixKind::ShellNoContext
        );
    }

    #[test]
    fn double_bang_checked_before_single() {
        assert_eq!(
            detect_input_prefix("!!cmd", &PromptPrefixConfig::default()),
            InputPrefixKind::ShellNoContext
        );
        assert_eq!(
            detect_input_prefix("!cmd", &PromptPrefixConfig::default()),
            InputPrefixKind::ShellWithContext
        );
    }

    #[test]
    fn prefix_symbols_match_kind() {
        let config = PromptPrefixConfig::default();
        assert_eq!(prefix_symbol(InputPrefixKind::Default, &config), DEFAULT_PROMPT_PREFIX_GLYPH);
        assert_eq!(prefix_symbol(InputPrefixKind::Slash, &config), "/");
        assert_eq!(prefix_symbol(InputPrefixKind::ShellWithContext, &config), "$");
        assert_eq!(prefix_symbol(InputPrefixKind::ShellNoContext, &config), "#");
    }

    #[test]
    fn list_selection_uses_prompt_glyph() {
        assert_eq!(LIST_SELECTION_MARKER, DEFAULT_PROMPT_PREFIX_GLYPH);
        assert_eq!(list_selection_row_prefix(true), LIST_SELECTION_ROW_PREFIX_SELECTED);
        assert_eq!(list_selection_row_prefix(false), LIST_SELECTION_ROW_PREFIX_IDLE);
    }

    #[test]
    fn custom_default_glyph() {
        let config = PromptPrefixConfig {
            enabled: true,
            default_glyph: ">",
        };
        assert_eq!(prefix_symbol(InputPrefixKind::Default, &config), ">");
    }

    #[test]
    fn strip_slash_trigger() {
        let (kind, body) = strip_submit_trigger("/help");
        assert_eq!(kind, InputPrefixKind::Slash);
        assert_eq!(body, "help");
    }

    #[test]
    fn strip_shell_triggers() {
        assert_eq!(strip_submit_trigger("!!rpt").1, "rpt");
        assert_eq!(strip_submit_trigger("! cargo test").1, "cargo test");
    }

    #[test]
    fn normal_chat_not_stripped() {
        let (kind, body) = strip_submit_trigger("hello world");
        assert_eq!(kind, InputPrefixKind::Default);
        assert_eq!(body, "hello world");
    }

    #[test]
    fn consume_slash_and_bang_triggers() {
        let config = PromptPrefixConfig::default();
        assert_eq!(
            try_consume_trigger(InputPrefixKind::Default, "", '/', config.enabled),
            Some(InputPrefixKind::Slash)
        );
        assert_eq!(
            try_consume_trigger(InputPrefixKind::Default, "", '!', config.enabled),
            Some(InputPrefixKind::ShellWithContext)
        );
        assert_eq!(
            try_consume_trigger(InputPrefixKind::ShellWithContext, "", '!', config.enabled),
            Some(InputPrefixKind::ShellNoContext)
        );
        assert!(try_consume_trigger(InputPrefixKind::Default, "ls", '/', config.enabled).is_none());
    }

    #[test]
    fn backspace_steps_down_prefix_kind() {
        let config = PromptPrefixConfig::default();
        assert_eq!(
            backspace_trigger_kind(InputPrefixKind::ShellNoContext, "", config.enabled),
            Some(InputPrefixKind::ShellWithContext)
        );
        assert_eq!(
            backspace_trigger_kind(InputPrefixKind::Slash, "", config.enabled),
            Some(InputPrefixKind::Default)
        );
        assert!(backspace_trigger_kind(InputPrefixKind::Default, "", config.enabled).is_none());
        assert!(backspace_trigger_kind(InputPrefixKind::Slash, "x", config.enabled).is_none());
    }

    #[test]
    fn strip_body_triggers_when_prefix_column_enabled() {
        let (kind, body) = strip_body_triggers(InputPrefixKind::Slash, "/help");
        assert_eq!(kind, InputPrefixKind::Slash);
        assert_eq!(body, "help");
        let (kind, body) = strip_body_triggers(InputPrefixKind::ShellWithContext, "!ls");
        assert_eq!(kind, InputPrefixKind::ShellWithContext);
        assert_eq!(body, "ls");
        let (kind, body) = strip_body_triggers(InputPrefixKind::ShellNoContext, "!!pwd");
        assert_eq!(kind, InputPrefixKind::ShellNoContext);
        assert_eq!(body, "pwd");
    }

    #[test]
    fn compose_palette_draft_for_slash_mode() {
        assert_eq!(compose_palette_draft(InputPrefixKind::Slash, ""), "/");
        assert_eq!(compose_palette_draft(InputPrefixKind::Slash, "hel"), "/hel");
        assert_eq!(compose_palette_draft(InputPrefixKind::Default, "hel"), "hel");
    }

    #[test]
    fn absorb_inline_triggers_from_paste() {
        let config = PromptPrefixConfig::default();
        let (kind, body) = absorb_inline_triggers(InputPrefixKind::Default, "/help args", &config);
        assert_eq!(kind, InputPrefixKind::Slash);
        assert_eq!(body, "help args");
    }

    #[test]
    fn resolve_submit_draft_uses_stored_kind() {
        let config = PromptPrefixConfig::default();
        let (kind, body) = resolve_submit_draft(InputPrefixKind::Slash, "help", &config);
        assert_eq!(kind, InputPrefixKind::Slash);
        assert_eq!(body, "help");
    }
}
