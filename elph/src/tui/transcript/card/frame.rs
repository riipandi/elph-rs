//! Shared tinted and flush card frames for transcript entries.

use iocraft::prelude::*;

use super::super::markdown::render::render_markdown_part;
use super::super::types::TranscriptMessage;
use super::chrome::TranscriptCardChrome;

pub fn render_tinted_card(chrome: &TranscriptCardChrome, message: &TranscriptMessage) -> AnyElement<'static> {
    render_text_card(chrome, &message.content, chrome.background, chrome.foreground)
}

pub fn render_flush_card(chrome: &TranscriptCardChrome, message: &TranscriptMessage) -> AnyElement<'static> {
    render_text_card(chrome, &message.content, Color::Reset, chrome.foreground)
}

pub fn render_assistant_card(chrome: &TranscriptCardChrome, message: &TranscriptMessage) -> AnyElement<'static> {
    if message
        .markdown
        .as_ref()
        .is_none_or(|markdown| !markdown.stream_complete)
    {
        return render_flush_card(chrome, message);
    }
    let children = assistant_message_elements(message, chrome.foreground);
    element! {
        View(
            width: chrome.outer_width,
            background_color: Color::Reset,
            border_style: BorderStyle::None,
            margin_bottom: chrome.margin_bottom,
            padding_top: chrome.padding_top,
            padding_bottom: chrome.padding_bottom,
            padding_left: chrome.padding_h,
            padding_right: chrome.padding_h,
            flex_direction: FlexDirection::Column,
            gap: 0,
        ) {
            #(children)
        }
    }
    .into()
}

pub(crate) fn assistant_message_elements(message: &TranscriptMessage, foreground: Color) -> Vec<AnyElement<'static>> {
    let Some(markdown) = &message.markdown else {
        return Vec::new();
    };
    if !markdown.stream_complete {
        return vec![
            element! {
                Text(color: foreground, wrap: TextWrap::Wrap, content: message.content.clone())
            }
            .into(),
        ];
    }
    let raw = &message.content;
    let mut children = Vec::new();
    let mut source_start = 0usize;
    for part in &markdown.parts {
        children.extend(render_markdown_part(&raw[source_start..part.source_end]));
        source_start = part.source_end;
    }
    let tail = markdown.tail(raw);
    if !tail.is_empty() {
        children.push(
            element! {
                Text(color: foreground, wrap: TextWrap::Wrap, content: tail.to_string())
            }
            .into(),
        );
    }
    children
}

fn render_text_card(
    chrome: &TranscriptCardChrome,
    content: &str,
    background: Color,
    foreground: Color,
) -> AnyElement<'static> {
    element! {
        View(
            width: chrome.outer_width,
            background_color: background,
            border_style: BorderStyle::None,
            margin_bottom: chrome.margin_bottom,
            padding_top: chrome.padding_top,
            padding_bottom: chrome.padding_bottom,
            padding_left: chrome.padding_h,
            padding_right: chrome.padding_h,
        ) {
            Text(color: foreground, wrap: TextWrap::Wrap, content: content.to_string())
        }
    }
    .into()
}
