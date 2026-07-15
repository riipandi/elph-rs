//! Markdown renderer (pulldown-cmark → styled lines).

use super::scroll_box::ScrollBox;
use iocraft::prelude::*;
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// Props for [`MarkdownView`].
#[derive(Clone, Default, Props)]
pub struct MarkdownViewProps {
    pub width: u16,
    pub height: u16,
    pub source: String,
}

pub fn render_markdown_lines(source: &str) -> Vec<AnyElement<'static>> {
    let parser = Parser::new_ext(source, Options::all());
    let mut lines: Vec<AnyElement<'static>> = Vec::new();
    let mut current = String::new();
    let mut style = MarkdownLineStyle::Body;

    let flush = |text: &str, style: MarkdownLineStyle, out: &mut Vec<AnyElement<'static>>| {
        if text.is_empty() {
            return;
        }
        out.push(
            element! {
                Text(
                    content: text.to_string(),
                    color: style.color(),
                    weight: style.weight(),
                    wrap: TextWrap::Wrap,
                )
            }
            .into(),
        );
    };

    for event in parser {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                flush(&current, style, &mut lines);
                current.clear();
                let _ = level;
                style = MarkdownLineStyle::Heading;
            }
            Event::End(TagEnd::Heading(_)) => {
                flush(&current, style, &mut lines);
                current.clear();
                style = MarkdownLineStyle::Body;
            }
            Event::Start(Tag::CodeBlock(_)) => {
                flush(&current, style, &mut lines);
                current.clear();
                style = MarkdownLineStyle::Code;
            }
            Event::End(TagEnd::CodeBlock) => {
                flush(&current, style, &mut lines);
                current.clear();
                style = MarkdownLineStyle::Body;
            }
            Event::Start(Tag::List(_)) | Event::End(TagEnd::Item) => {
                flush(&current, style, &mut lines);
                current.clear();
            }
            Event::Start(Tag::Item) => {
                current.push_str("• ");
            }
            Event::Text(text) | Event::Code(text) => {
                current.push_str(&text);
            }
            Event::SoftBreak | Event::HardBreak => {
                flush(&current, style, &mut lines);
                current.clear();
            }
            _ => {}
        }
    }
    flush(&current, style, &mut lines);

    if lines.is_empty() {
        lines.push(element! { Text(content: "", color: Color::Grey) }.into());
    }

    lines
}

#[derive(Clone, Copy)]
enum MarkdownLineStyle {
    Body,
    Heading,
    Code,
}

impl MarkdownLineStyle {
    fn color(self) -> Color {
        match self {
            Self::Body => Color::Grey,
            Self::Heading => Color::Cyan,
            Self::Code => Color::Green,
        }
    }

    fn weight(self) -> Weight {
        match self {
            Self::Heading => Weight::Bold,
            _ => Weight::Normal,
        }
    }
}

/// Scrollable markdown document.
#[component]
pub fn MarkdownView(props: &MarkdownViewProps) -> impl Into<AnyElement<'static>> {
    let children = render_markdown_lines(&props.source);

    element! {
        ScrollBox(
            width: props.width,
            height: props.height,
            children: children,
        )
    }
}
