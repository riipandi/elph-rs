//! Per-style transcript card renderers.

use elph_tui::components::{ProcessActivityTrail, ProcessStatus, ProcessStatusRow};
use iocraft::prelude::*;

use crate::tui::theme::{TEXT_FG, THINKING_FG, TOOL_ARGS_FG, TOOL_OUTPUT_FG};

use super::super::types::{TranscriptMessage, TranscriptStyle};
use super::chrome::{
    COLORED_CARD_PAD_H, FLUSH_CARD_PAD, THINKING_RESPONSE_GAP, TOOL_OUTPUT_SECTION_GAP, TranscriptCardChrome,
};
use super::frame::{assistant_message_elements, render_assistant_card, render_flush_card, render_tinted_card};
use super::tool_format::{format_tool_args_display, format_tool_output_display};

pub fn tool_status_marker(style: TranscriptStyle) -> &'static str {
    match style {
        TranscriptStyle::ToolRunning => "○",
        TranscriptStyle::ToolSuccess => "●",
        TranscriptStyle::ToolFailed => "✕",
        _ => "○",
    }
}

fn tool_process_status(style: TranscriptStyle) -> ProcessStatus {
    match style {
        TranscriptStyle::ToolRunning => ProcessStatus::Running,
        TranscriptStyle::ToolSuccess => ProcessStatus::Done,
        TranscriptStyle::ToolFailed => ProcessStatus::Failed,
        _ => ProcessStatus::Queued,
    }
}

pub fn user_prompt_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    let chrome = TranscriptCardChrome::tinted(screen_width, message.style, margin_bottom);
    render_tinted_card(&chrome, message)
}

pub fn skill_prompt_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    let chrome = TranscriptCardChrome::tinted(screen_width, message.style, margin_bottom);
    render_tinted_card(&chrome, message)
}

pub fn thinking_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    let chrome = TranscriptCardChrome::from_style(screen_width, message.style, margin_bottom);
    render_flush_card(&chrome, message)
}

pub fn chat_response_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    let chrome = TranscriptCardChrome::from_style(screen_width, message.style, margin_bottom);
    render_assistant_card(&chrome, message)
}

pub fn error_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    let chrome = TranscriptCardChrome::tinted(screen_width, message.style, margin_bottom);
    render_tinted_card(&chrome, message)
}

pub fn meta_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    let chrome = TranscriptCardChrome::tinted(screen_width, message.style, margin_bottom);
    render_tinted_card(&chrome, message)
}

pub fn tool_call_card(screen_width: u16, message: &TranscriptMessage, margin_bottom: u16) -> AnyElement<'static> {
    let style = message.style;
    let chrome = TranscriptCardChrome::tinted(screen_width, style, margin_bottom);

    if let Some(tool) = &message.tool {
        let args = format_tool_args_display(&tool.args_summary);
        let output = format_tool_output_display(&tool.output);
        let running = style == TranscriptStyle::ToolRunning;
        let status = tool_process_status(style);
        let inner_width = chrome
            .outer_width
            .saturating_sub(chrome.padding_h.saturating_mul(2))
            .max(8);
        return element! {
            View(
                width: chrome.outer_width,
                background_color: chrome.background,
                border_style: BorderStyle::None,
                margin_bottom: chrome.margin_bottom,
                padding_top: chrome.padding_top,
                padding_bottom: chrome.padding_bottom,
                padding_left: chrome.padding_h,
                padding_right: chrome.padding_h,
                flex_direction: FlexDirection::Column,
                gap: 0,
            ) {
                ProcessStatusRow(
                    status: status,
                    label: tool.name.clone(),
                    running_color: Some(chrome.foreground),
                    done_color: Some(chrome.foreground),
                    failed_color: Some(chrome.foreground),
                    emphasize_running: true,
                )
                #(if running && output.is_empty() {
                    Some(element! {
                        ProcessActivityTrail(
                            width: inner_width.min(28),
                            active: true,
                            accent: Some(chrome.foreground),
                        )
                    })
                } else {
                    None
                })
                #(if !args.is_empty() {
                    Some(element! {
                        Text(color: TOOL_ARGS_FG, wrap: TextWrap::Wrap, content: args)
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

    render_tinted_card(&chrome, message)
}

pub fn thinking_response_pair_card(
    screen_width: u16,
    first: &TranscriptMessage,
    second: &TranscriptMessage,
    margin_bottom: u16,
) -> AnyElement<'static> {
    let chrome = TranscriptCardChrome::from_style(screen_width, TranscriptStyle::Thinking, margin_bottom);
    let (thinking, assistant) = if first.style == TranscriptStyle::Thinking {
        (first, second)
    } else {
        (second, first)
    };
    let inner_width = chrome
        .outer_width
        .saturating_sub(chrome.padding_h.saturating_mul(2))
        .max(1);
    let assistant_body = if assistant.markdown.is_some() {
        assistant_message_elements(assistant, TEXT_FG, inner_width)
    } else {
        vec![
            element! {
                Text(color: TEXT_FG, wrap: TextWrap::Wrap, content: assistant.content.as_str())
            }
            .into(),
        ]
    };
    element! {
        View(
            width: chrome.outer_width,
            background_color: Color::Reset,
            border_style: BorderStyle::None,
            margin_bottom: margin_bottom,
            padding_top: FLUSH_CARD_PAD,
            padding_bottom: FLUSH_CARD_PAD,
            padding_left: COLORED_CARD_PAD_H,
            padding_right: COLORED_CARD_PAD_H,
            flex_direction: FlexDirection::Column,
            gap: THINKING_RESPONSE_GAP,
        ) {
            Text(color: THINKING_FG, wrap: TextWrap::Wrap, content: thinking.content.as_str())
            View(
                width: inner_width,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::FlexStart,
                gap: 0,
            ) {
                #(assistant_body)
            }
        }
    }
    .into()
}
