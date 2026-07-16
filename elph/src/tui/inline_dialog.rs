//! Full-width inline dialog shell (matches prompt editor chrome).

use elph_tui::components::{UiTheme, dialog_header_title_fit};
use iocraft::prelude::*;

use crate::tui::user_question::{QuestionStepTab, QuestionStepTabState};

/// Gap between sections inside inline dialog bodies (tighter than modal dialogs).
pub const INLINE_SECTION_GAP: u16 = 0;

/// Space above the first selectable answer row in inline dialogs.
pub const OPTIONS_LIST_TOP_GAP: u16 = 1;

/// Inner content width inside the round border and zone padding.
pub fn inline_body_width(screen_width: u16) -> u16 {
    UiTheme::default().shell_editor_inner_width(screen_width)
}

/// Visual state for a header tab chip.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InlineDialogTabState {
    Current,
    Answered,
    Upcoming,
}

impl From<QuestionStepTabState> for InlineDialogTabState {
    fn from(state: QuestionStepTabState) -> Self {
        match state {
            QuestionStepTabState::Current => Self::Current,
            QuestionStepTabState::Answered => Self::Answered,
            QuestionStepTabState::Upcoming => Self::Upcoming,
        }
    }
}

/// One navigable tab in the dialog header (not keyboard Tab).
#[derive(Clone, Debug)]
pub struct InlineDialogTab {
    pub index: usize,
    pub state: InlineDialogTabState,
}

impl From<QuestionStepTab> for InlineDialogTab {
    fn from(tab: QuestionStepTab) -> Self {
        Self {
            index: tab.index,
            state: tab.state.into(),
        }
    }
}

/// Props for [`InlineDialogShell`].
#[derive(Props)]
pub struct InlineDialogShellProps<'a> {
    pub screen_width: u16,
    pub title: String,
    pub has_focus: bool,
    pub tabs: Option<Vec<InlineDialogTab>>,
    pub footer_hint: Option<String>,
    pub children: Vec<AnyElement<'a>>,
}

impl<'a> Default for InlineDialogShellProps<'a> {
    fn default() -> Self {
        Self {
            screen_width: 80,
            title: String::new(),
            has_focus: false,
            tabs: None,
            footer_hint: None,
            children: Vec::new(),
        }
    }
}

fn step_tab_label(index: usize) -> String {
    format!("Step {}", index + 1)
}

fn tab_text_color(theme: UiTheme, state: InlineDialogTabState) -> Color {
    match state {
        InlineDialogTabState::Current => theme.warning,
        InlineDialogTabState::Answered => theme.text_secondary,
        InlineDialogTabState::Upcoming => theme.text_muted,
    }
}

fn render_tab_row(tabs: &[InlineDialogTab], inner: u16, theme: UiTheme) -> AnyElement<'static> {
    let mut segments: Vec<AnyElement<'static>> = Vec::new();
    for (i, tab) in tabs.iter().enumerate() {
        if i > 0 {
            segments.push(
                element! {
                    Text(content: " | ".to_string(), color: theme.text_muted, wrap: TextWrap::NoWrap)
                }
                .into(),
            );
        }
        let current = tab.state == InlineDialogTabState::Current;
        segments.push(
            element! {
                Text(
                    content: step_tab_label(tab.index),
                    color: tab_text_color(theme, tab.state),
                    weight: if current { Weight::Bold } else { Weight::Normal },
                    wrap: TextWrap::NoWrap,
                )
            }
            .into(),
        );
    }
    element! {
        View(width: inner, flex_direction: FlexDirection::Row, flex_wrap: FlexWrap::NoWrap, flex_shrink: 0f32) {
            #(segments)
        }
    }
    .into()
}

/// Single bordered frame for inline agent dialogs: full terminal width, reset background.
#[component]
pub fn InlineDialogShell<'a>(props: &mut InlineDialogShellProps<'a>, hooks: Hooks) -> impl Into<AnyElement<'a>> {
    let _ = hooks;
    let theme = UiTheme::default();
    let border_color = theme.shell_zone_border_color(props.has_focus);
    let inset = theme.shell_zone_padding();
    let inner = inline_body_width(props.screen_width);
    let title = dialog_header_title_fit(&props.title, inner, "");
    let divider = "─".repeat(inner.max(1) as usize);
    let children = std::mem::take(&mut props.children);
    let tabs = props.tabs.clone();
    let footer_hint = props.footer_hint.clone();

    let header = if let Some(ref tab_row) = tabs {
        if tab_row.is_empty() {
            element! {
                View(width: inner, flex_shrink: 0f32) {
                    Text(
                        content: title,
                        color: theme.text_primary,
                        weight: Weight::Bold,
                        wrap: TextWrap::NoWrap,
                    )
                }
            }
        } else {
            element! {
                View(width: inner, flex_shrink: 0f32) {
                    #(render_tab_row(tab_row, inner, theme))
                }
            }
        }
    } else {
        element! {
            View(width: inner, flex_shrink: 0f32) {
                Text(
                    content: title,
                    color: theme.text_primary,
                    weight: Weight::Bold,
                    wrap: TextWrap::NoWrap,
                )
            }
        }
    };

    element! {
        View(
            width: props.screen_width,
            flex_shrink: 0f32,
            border_style: BorderStyle::Round,
            border_color: border_color,
            background_color: Color::Reset,
            position: Position::Relative,
            padding_left: inset,
            padding_right: inset,
            flex_direction: FlexDirection::Column,
            gap: 0,
        ) {
            #(header)
            View(width: inner, flex_shrink: 0f32) {
                Text(
                    content: divider,
                    color: theme.text_muted,
                    wrap: TextWrap::NoWrap,
                )
            }
            View(
                width: inner,
                flex_direction: FlexDirection::Column,
                flex_shrink: 0f32,
            ) {
                #(children)
            }
            #(footer_hint.map(|hint| -> AnyElement<'static> {
                element! {
                    View(width: inner, padding_top: 1, flex_shrink: 0f32) {
                        Text(
                            content: hint,
                            color: theme.text_muted,
                            wrap: TextWrap::Wrap,
                        )
                    }
                }
                .into()
            }))
        }
    }
}
