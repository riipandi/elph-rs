//! Transcript message types and per-kind card rendering.

use iocraft::prelude::*;

use crate::tui::theme::{
    BUBBLE_BG, META_BG, META_FG, SKILL_BG, SKILL_FG, TEXT_FG, THINKING_BG, THINKING_FG, TOOL_ARGS_FG, TOOL_FAILED_BG,
    TOOL_FAILED_FG, TOOL_OUTPUT_FG, TOOL_RUNNING_BG, TOOL_RUNNING_FG, TOOL_SUCCESS_BG, TOOL_SUCCESS_FG,
};

const COLORED_CARD_PAD: u16 = 1;
const COLORED_CARD_GAP: u16 = 1;
const FLUSH_CARD_PAD: u16 = 0;
const FLUSH_CARD_GAP: u16 = 0;
/// Rows between a thinking block and the following assistant reply in a flush pair.
const THINKING_RESPONSE_GAP: u16 = 1;
/// Rows between tool header/args and the output body.
const TOOL_OUTPUT_SECTION_GAP: u16 = 1;
const TOOL_OUTPUT_MAX_LINES: usize = 12;
const TOOL_OUTPUT_MAX_CHARS: usize = 1_500;

const LOREM_IPSUM: &str = "Lorem ipsum odor amet, consectetuer adipiscing elit. \
Lobortis hendrerit nec ipsum dapibus quam. Donec malesuada tincidunt elementum \
mollis vehicula quisque purus. Est volutpat integer, donec sagittis placerat \
fermentum phasellus ipsum sollicitudin. Tempus laoreet ad tempus aptent proin \
per donec lectus. Quisque auctor urna; phasellus urna tortor ligula. Class \
pharetra bibendum tristique, quisque consectetur placerat potenti. Imperdiet ut \
torquent vestibulum eleifend bibendum et. Dictumst vulputate interdum iaculis \
at conubia venenatis.";

/// Structured payload for tool invocation cards in the transcript.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolCardDetail {
    pub name: String,
    pub args_summary: String,
    pub output: String,
}

#[derive(Clone)]
pub struct TranscriptMessage {
    pub content: String,
    pub style: TranscriptStyle,
    pub tool: Option<ToolCardDetail>,
}

impl TranscriptMessage {
    pub fn text(content: impl Into<String>, style: TranscriptStyle) -> Self {
        Self {
            content: content.into(),
            style,
            tool: None,
        }
    }

    pub fn tool_call(name: impl Into<String>, args_summary: impl Into<String>, style: TranscriptStyle) -> Self {
        Self {
            content: String::new(),
            style,
            tool: Some(ToolCardDetail {
                name: name.into(),
                args_summary: args_summary.into(),
                output: String::new(),
            }),
        }
    }

    /// Flattened text for scroll row layout (matches rendered line breaks).
    pub fn layout_text(&self) -> String {
        if let Some(tool) = &self.tool {
            tool.layout_text(self.style)
        } else {
            self.content.clone()
        }
    }
}

impl ToolCardDetail {
    pub fn layout_text(&self, style: TranscriptStyle) -> String {
        let mut lines = vec![format!("{} {}", tool_status_marker(style), self.name)];
        let args = format_tool_args_display(&self.args_summary);
        if !args.is_empty() {
            for arg_line in args.lines() {
                lines.push(format!("  {arg_line}"));
            }
        }
        let output = format_tool_output_display(&self.output);
        if !output.is_empty() {
            lines.push(String::new());
            lines.extend(output.lines().map(str::to_string));
        }
        lines.join("\n")
    }
}

/// Visual card kind for one transcript entry.
#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TranscriptCardKind {
    UserPrompt,
    SkillPrompt,
    Thinking,
    ChatResponse,
    ToolCall,
    Error,
    Meta,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TranscriptStyle {
    /// Submitted user prompt — tinted card, eligible for sticky scroll.
    User,
    /// Model thinking stream — flush dim text (no tinted background).
    Thinking,
    /// Assistant reply — flush text (no tinted background).
    Assistant,
    /// Slash command / skill / prompt-template invocation.
    SkillPrompt,
    /// System meta line (steering, goals, subagent status).
    Meta,
    Error,
    /// Tool invoked — soft gray card.
    ToolRunning,
    /// Tool finished OK — soft green card.
    ToolSuccess,
    /// Tool failed — soft red card.
    ToolFailed,
}

impl TranscriptStyle {
    #[cfg_attr(not(test), allow(dead_code))]
    pub fn card_kind(self) -> TranscriptCardKind {
        match self {
            Self::User => TranscriptCardKind::UserPrompt,
            Self::SkillPrompt => TranscriptCardKind::SkillPrompt,
            Self::Thinking => TranscriptCardKind::Thinking,
            Self::Assistant => TranscriptCardKind::ChatResponse,
            Self::ToolRunning | Self::ToolSuccess | Self::ToolFailed => TranscriptCardKind::ToolCall,
            Self::Error => TranscriptCardKind::Error,
            Self::Meta => TranscriptCardKind::Meta,
        }
    }

    /// Map a submitted editor line to its transcript card style.
    pub fn for_user_submit(text: &str) -> Self {
        let trimmed = text.trim_start();
        if trimmed.starts_with('/') {
            Self::SkillPrompt
        } else {
            Self::User
        }
    }

    /// Submitted editor prompt — the only transcript entry eligible for sticky scroll.
    pub fn is_sticky_prompt(self) -> bool {
        matches!(self, Self::User)
    }

    /// Whether the card paints a tinted background (vs terminal default).
    pub fn has_tinted_background(self) -> bool {
        !matches!(self.background_color(), Color::Reset)
    }

    fn is_flush_text(self) -> bool {
        matches!(self, Self::Thinking | Self::Assistant)
    }

    /// Rows of vertical gap after this entry in scroll layout and between rendered cards.
    pub fn entry_gap_after(self, next: Option<TranscriptStyle>) -> u16 {
        match (self, next) {
            (Self::Thinking, Some(Self::Assistant)) => THINKING_RESPONSE_GAP,
            (Self::Assistant, Some(Self::Thinking)) => 0,
            (prev, Some(next)) if prev.is_flush_text() && next.has_tinted_background() => COLORED_CARD_GAP,
            _ if self.has_tinted_background() => COLORED_CARD_GAP,
            _ => FLUSH_CARD_GAP,
        }
    }

    /// Adjacent thinking + chat response blocks render as one flush group with internal spacing.
    pub fn forms_flush_pair_with(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Self::Thinking, Self::Assistant) | (Self::Assistant, Self::Thinking)
        )
    }

    pub fn sticky_padding_top(self) -> u16 {
        self.padding()
    }

    pub fn sticky_padding_bottom(self) -> u16 {
        self.padding()
    }

    pub fn sticky_bubble_padding_rows(self) -> u16 {
        self.sticky_padding_top().saturating_add(self.sticky_padding_bottom())
    }

    /// Horizontal inset per side for wrap width and flush-text alignment with tinted cards.
    pub fn horizontal_padding(self) -> u16 {
        if self.is_flush_text() || self.has_tinted_background() {
            COLORED_CARD_PAD
        } else {
            FLUSH_CARD_PAD
        }
    }

    fn text_color(self) -> Color {
        match self {
            Self::Thinking => THINKING_FG,
            Self::SkillPrompt => SKILL_FG,
            Self::Meta => META_FG,
            Self::User | Self::Assistant => TEXT_FG,
            Self::Error => TOOL_FAILED_FG,
            Self::ToolRunning => TOOL_RUNNING_FG,
            Self::ToolSuccess => TOOL_SUCCESS_FG,
            Self::ToolFailed => TOOL_FAILED_FG,
        }
    }

    fn background_color(self) -> Color {
        match self {
            Self::Assistant => Color::Reset,
            Self::User => BUBBLE_BG,
            Self::Error => TOOL_FAILED_BG,
            Self::SkillPrompt => SKILL_BG,
            Self::Meta => META_BG,
            Self::Thinking => THINKING_BG,
            Self::ToolRunning => TOOL_RUNNING_BG,
            Self::ToolSuccess => TOOL_SUCCESS_BG,
            Self::ToolFailed => TOOL_FAILED_BG,
        }
    }

    fn padding(self) -> u16 {
        if self.has_tinted_background() {
            COLORED_CARD_PAD
        } else {
            FLUSH_CARD_PAD
        }
    }
}

pub fn seed_transcript_messages() -> Vec<TranscriptMessage> {
    vec![
        TranscriptMessage::text("Explain how sticky scroll works in this layout.", TranscriptStyle::User),
        TranscriptMessage::text(
            "/tui-design sync chat_layout with production shell",
            TranscriptStyle::SkillPrompt,
        ),
        TranscriptMessage {
            content: String::new(),
            style: TranscriptStyle::ToolSuccess,
            tool: Some(ToolCardDetail {
                name: "read_file".to_string(),
                args_summary: r#"{"path":"elph/src/tui/transcript/mod.rs"}"#.to_string(),
                output: "//! Scrollable transcript panel with sticky user prompts.\n\nmod message;".to_string(),
            }),
        },
        TranscriptMessage::text(
            "Need to check scroll offset and clamp sticky height…",
            TranscriptStyle::Thinking,
        ),
        TranscriptMessage::text(LOREM_IPSUM, TranscriptStyle::Assistant),
        TranscriptMessage {
            content: String::new(),
            style: TranscriptStyle::ToolFailed,
            tool: Some(ToolCardDetail {
                name: "bash".to_string(),
                args_summary: r#"{"command":"npm test"}"#.to_string(),
                output: "Error: command exited with code 1\n\nFAIL tests/agent.rs".to_string(),
            }),
        },
        TranscriptMessage::text("request failed: connection reset", TranscriptStyle::Error),
        TranscriptMessage::text("Steering queued — will run after current turn", TranscriptStyle::Meta),
    ]
}

pub fn build_transcript_bubbles(screen_width: u16, messages: &[TranscriptMessage]) -> Vec<AnyElement<'static>> {
    let mut bubbles = Vec::with_capacity(messages.len());
    let mut index = 0;
    while index < messages.len() {
        let message = &messages[index];
        let next_style = messages.get(index + 1).map(|m| m.style);
        if let Some(next) = messages.get(index + 1)
            && message.style.forms_flush_pair_with(next.style)
        {
            let after_pair = messages.get(index + 2).map(|m| m.style);
            let margin_bottom = TranscriptStyle::Assistant.entry_gap_after(after_pair);
            bubbles.push(thinking_response_pair_card(screen_width, message, next, margin_bottom));
            index += 2;
            continue;
        }
        let margin_bottom = message.style.entry_gap_after(next_style);
        bubbles.push(transcript_message_bubble(screen_width, message, margin_bottom));
        index += 1;
    }
    bubbles
}

pub fn transcript_message_bubble(
    screen_width: u16,
    message: &TranscriptMessage,
    margin_bottom: u16,
) -> AnyElement<'static> {
    match message.style {
        TranscriptStyle::User => user_prompt_card(screen_width, message, margin_bottom),
        TranscriptStyle::SkillPrompt => skill_prompt_card(screen_width, message, margin_bottom),
        TranscriptStyle::Thinking => thinking_card(screen_width, message, margin_bottom),
        TranscriptStyle::Assistant => chat_response_card(screen_width, message, margin_bottom),
        TranscriptStyle::ToolRunning | TranscriptStyle::ToolSuccess | TranscriptStyle::ToolFailed => {
            tool_call_card(screen_width, message, margin_bottom)
        }
        TranscriptStyle::Error => error_card(screen_width, message, margin_bottom),
        TranscriptStyle::Meta => meta_card(screen_width, message, margin_bottom),
    }
}

fn thinking_response_pair_card(
    screen_width: u16,
    first: &TranscriptMessage,
    second: &TranscriptMessage,
    margin_bottom: u16,
) -> AnyElement<'static> {
    let (thinking, assistant) = if first.style == TranscriptStyle::Thinking {
        (first, second)
    } else {
        (second, first)
    };
    element! {
        View(
            width: screen_width - 3,
            background_color: Color::Reset,
            border_style: BorderStyle::None,
            margin_bottom: margin_bottom,
            padding_top: FLUSH_CARD_PAD,
            padding_bottom: FLUSH_CARD_PAD,
            padding_left: COLORED_CARD_PAD,
            padding_right: COLORED_CARD_PAD,
            flex_direction: FlexDirection::Column,
            gap: THINKING_RESPONSE_GAP,
        ) {
            Text(color: THINKING_FG, wrap: TextWrap::Wrap, content: thinking.content.as_str())
            Text(color: TEXT_FG, wrap: TextWrap::Wrap, content: assistant.content.as_str())
        }
    }
    .into()
}

fn tinted_card(
    screen_width: u16,
    message: &TranscriptMessage,
    background: Color,
    text: Color,
    margin_bottom: u16,
) -> AnyElement<'static> {
    element! {
        View(
            width: screen_width - 3,
            background_color: background,
            border_style: BorderStyle::None,
            margin_bottom: margin_bottom,
            padding: COLORED_CARD_PAD,
        ) {
            Text(color: text, wrap: TextWrap::Wrap, content: message.content.as_str())
        }
    }
    .into()
}

fn flush_card(screen_width: u16, message: &TranscriptMessage, text: Color, margin_bottom: u16) -> AnyElement<'static> {
    element! {
        View(
            width: screen_width - 3,
            background_color: Color::Reset,
            border_style: BorderStyle::None,
            margin_bottom: margin_bottom,
            padding_top: FLUSH_CARD_PAD,
            padding_bottom: FLUSH_CARD_PAD,
            padding_left: COLORED_CARD_PAD,
            padding_right: COLORED_CARD_PAD,
        ) {
            Text(color: text, wrap: TextWrap::Wrap, content: message.content.as_str())
        }
    }
    .into()
}

fn user_prompt_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    tinted_card(screen_width, message, BUBBLE_BG, TEXT_FG, margin_bottom)
}

fn skill_prompt_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    tinted_card(screen_width, message, SKILL_BG, SKILL_FG, margin_bottom)
}

fn thinking_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    flush_card(screen_width, message, THINKING_FG, margin_bottom)
}

fn chat_response_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    flush_card(screen_width, message, TEXT_FG, margin_bottom)
}

fn tool_call_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    let style = message.style;
    let background = style.background_color();
    let header_fg = style.text_color();

    if let Some(tool) = &message.tool {
        let header = format!("{} {}", tool_status_marker(style), tool.name);
        let args = format_tool_args_display(&tool.args_summary);
        let output = format_tool_output_display(&tool.output);
        return element! {
            View(
                width: screen_width - 3,
                background_color: background,
                border_style: BorderStyle::None,
                margin_bottom: margin_bottom,
                padding: COLORED_CARD_PAD,
                flex_direction: FlexDirection::Column,
                gap: 0,
            ) {
                Text(color: header_fg, wrap: TextWrap::NoWrap, content: header)
                #(if !args.is_empty() {
                    Some(element! {
                        Text(color: TOOL_ARGS_FG, wrap: TextWrap::Wrap, content: format!("  {args}"))
                    })
                } else {
                    None
                })
                #(if !output.is_empty() {
                    Some(element! {
                        View(
                            width: 100pct,
                            padding_top: TOOL_OUTPUT_SECTION_GAP,
                            flex_direction: FlexDirection::Column,
                            gap: 0,
                        ) {
                            Text(color: TOOL_OUTPUT_FG, wrap: TextWrap::Wrap, content: output)
                        }
                    })
                } else {
                    None
                })
            }
        }
        .into();
    }

    tinted_card(screen_width, message, background, header_fg, margin_bottom)
}

fn error_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    tinted_card(screen_width, message, TOOL_FAILED_BG, TOOL_FAILED_FG, margin_bottom)
}

fn meta_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    tinted_card(screen_width, message, META_BG, META_FG, margin_bottom)
}

fn tool_status_marker(style: TranscriptStyle) -> &'static str {
    match style {
        TranscriptStyle::ToolRunning => "○",
        TranscriptStyle::ToolSuccess => "●",
        TranscriptStyle::ToolFailed => "✕",
        _ => "○",
    }
}

fn format_tool_args_display(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) else {
        return trimmed.to_string();
    };
    format_tool_args_json(&value)
}

fn format_tool_args_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Object(map) if map.is_empty() => String::new(),
        serde_json::Value::Object(map) if map.len() == 1 => {
            map.values().next().map(format_json_scalar).unwrap_or_default()
        }
        serde_json::Value::Object(map) => map
            .iter()
            .map(|(key, val)| format!("{key}: {}", format_json_scalar(val)))
            .collect::<Vec<_>>()
            .join(", "),
        other => format_json_scalar(other),
    }
}

fn format_json_scalar(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Number(num) => num.to_string(),
        serde_json::Value::Bool(flag) => flag.to_string(),
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(format_json_scalar).collect();
            parts.join(", ")
        }
        serde_json::Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn format_tool_output_display(output: &str) -> String {
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

pub fn transcript_sticky_overlay(
    height: u16,
    message: &TranscriptMessage,
    display_content: &str,
) -> AnyElement<'static> {
    user_prompt_sticky_overlay(height, message, display_content)
}

fn user_prompt_sticky_overlay(height: u16, message: &TranscriptMessage, display_content: &str) -> AnyElement<'static> {
    let style = message.style;
    let pad_h = style.horizontal_padding();
    element! {
        View(
            position: Position::Absolute,
            top: 0,
            left: 0,
            right: 1,
            height: height,
            overflow: Overflow::Hidden,
            background_color: Color::Reset,
            border_style: BorderStyle::None,
            padding_left: 1,
            padding_right: 1,
            padding_bottom: 1,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Baseline,
        ) {
            View(
                width: 100pct,
                background_color: BUBBLE_BG,
                border_style: BorderStyle::None,
                padding_top: style.sticky_padding_top(),
                padding_bottom: style.sticky_padding_bottom(),
                padding_left: pad_h,
                padding_right: pad_h,
                flex_shrink: 0f32,
                margin_bottom: 0,
            ) {
                Text(
                    color: TEXT_FG,
                    wrap: TextWrap::NoWrap,
                    content: display_content.to_string(),
                )
            }
        }
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::TranscriptStyle;
    use iocraft::prelude::Color;

    use super::*;
    use crate::tui::theme::{META_BG, SKILL_BG, THINKING_BG, TOOL_FAILED_BG, TOOL_RUNNING_BG, TOOL_SUCCESS_BG};

    #[test]
    fn sticky_prompt_is_submitted_user_input_only() {
        assert!(TranscriptStyle::User.is_sticky_prompt());
        assert!(!TranscriptStyle::SkillPrompt.is_sticky_prompt());
        assert!(!TranscriptStyle::Assistant.is_sticky_prompt());
        assert!(!TranscriptStyle::ToolRunning.is_sticky_prompt());
        assert!(!TranscriptStyle::Thinking.is_sticky_prompt());
    }

    #[test]
    fn card_kinds_are_distinct_per_role() {
        assert_eq!(TranscriptStyle::User.card_kind(), TranscriptCardKind::UserPrompt);
        assert_eq!(TranscriptStyle::SkillPrompt.card_kind(), TranscriptCardKind::SkillPrompt);
        assert_eq!(TranscriptStyle::Meta.card_kind(), TranscriptCardKind::Meta);
        assert_eq!(TranscriptStyle::Thinking.card_kind(), TranscriptCardKind::Thinking);
        assert_eq!(TranscriptStyle::Assistant.card_kind(), TranscriptCardKind::ChatResponse);
        assert_eq!(TranscriptStyle::ToolRunning.card_kind(), TranscriptCardKind::ToolCall);
    }

    #[test]
    fn for_user_submit_detects_skill_and_chat_prompts() {
        assert_eq!(TranscriptStyle::for_user_submit("/tui-design"), TranscriptStyle::SkillPrompt);
        assert_eq!(TranscriptStyle::for_user_submit("  /help args"), TranscriptStyle::SkillPrompt);
        assert_eq!(TranscriptStyle::for_user_submit("hello"), TranscriptStyle::User);
    }

    #[test]
    fn skill_and_meta_cards_use_distinct_tints() {
        assert_eq!(TranscriptStyle::SkillPrompt.background_color(), SKILL_BG);
        assert_eq!(TranscriptStyle::Meta.background_color(), META_BG);
        assert_ne!(
            TranscriptStyle::SkillPrompt.background_color(),
            TranscriptStyle::Meta.background_color()
        );
    }

    #[test]
    fn tinted_cards_have_padding_and_gap_flush_cards_do_not() {
        assert!(TranscriptStyle::User.has_tinted_background());
        assert_eq!(TranscriptStyle::User.padding(), 1);
        assert_eq!(TranscriptStyle::User.entry_gap_after(None), 1);

        assert!(!TranscriptStyle::Assistant.has_tinted_background());
        assert_eq!(TranscriptStyle::Assistant.padding(), 0);
        assert_eq!(TranscriptStyle::Assistant.horizontal_padding(), 1);
        assert_eq!(TranscriptStyle::Assistant.entry_gap_after(None), 0);

        assert!(!TranscriptStyle::Thinking.has_tinted_background());
        assert_eq!(TranscriptStyle::Thinking.padding(), 0);
        assert_eq!(TranscriptStyle::Thinking.horizontal_padding(), 1);
        assert_eq!(TranscriptStyle::Thinking.entry_gap_after(None), 0);
    }

    #[test]
    fn thinking_and_assistant_pair_has_internal_gap() {
        assert_eq!(TranscriptStyle::Thinking.entry_gap_after(Some(TranscriptStyle::Assistant)), 1);
        assert_eq!(TranscriptStyle::Assistant.entry_gap_after(Some(TranscriptStyle::Thinking)), 0);
        assert!(TranscriptStyle::Thinking.forms_flush_pair_with(TranscriptStyle::Assistant));
    }

    #[test]
    fn assistant_inserts_gap_before_next_user_prompt() {
        assert_eq!(TranscriptStyle::Assistant.entry_gap_after(Some(TranscriptStyle::User)), 1);
    }

    #[test]
    fn flush_text_inserts_gap_before_tool_cards() {
        assert_eq!(
            TranscriptStyle::Assistant.entry_gap_after(Some(TranscriptStyle::ToolRunning)),
            1
        );
        assert_eq!(TranscriptStyle::Thinking.entry_gap_after(Some(TranscriptStyle::ToolSuccess)), 1);
        assert_eq!(TranscriptStyle::Assistant.entry_gap_after(Some(TranscriptStyle::ToolFailed)), 1);
    }

    #[test]
    fn sticky_user_bubble_has_symmetric_padding() {
        assert_eq!(TranscriptStyle::User.sticky_padding_top(), 1);
        assert_eq!(TranscriptStyle::User.sticky_padding_bottom(), 1);
        assert_eq!(TranscriptStyle::User.sticky_bubble_padding_rows(), 2);
    }

    #[test]
    fn thinking_and_response_transcript_colors() {
        assert_eq!(TranscriptStyle::Assistant.text_color(), TEXT_FG);
        assert_eq!(TranscriptStyle::Assistant.background_color(), Color::Reset);
        assert_eq!(TranscriptStyle::Thinking.background_color(), THINKING_BG);
        assert_eq!(TranscriptStyle::Thinking.background_color(), Color::Reset);
        assert_eq!(TranscriptStyle::Thinking.text_color(), THINKING_FG);
        assert_eq!(TranscriptStyle::Thinking.text_color(), Color::DarkGrey);
    }

    #[test]
    fn tool_card_status_colors_are_soft_and_distinct() {
        assert_eq!(TranscriptStyle::ToolRunning.background_color(), TOOL_RUNNING_BG);
        assert_eq!(TranscriptStyle::ToolSuccess.background_color(), TOOL_SUCCESS_BG);
        assert_eq!(TranscriptStyle::ToolFailed.background_color(), TOOL_FAILED_BG);
    }

    #[test]
    fn tool_card_layout_includes_header_args_and_output() {
        let tool = ToolCardDetail {
            name: "read_file".to_string(),
            args_summary: r#"{"path":"main.rs"}"#.to_string(),
            output: "fn main() {}".to_string(),
        };
        let layout = tool.layout_text(TranscriptStyle::ToolSuccess);
        assert!(layout.starts_with("● read_file"));
        assert!(layout.contains("  main.rs"));
        assert!(layout.contains("fn main()"));
    }

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
