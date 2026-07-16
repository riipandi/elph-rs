//! Full-width inline dialog shell (matches prompt editor chrome).

use elph_tui::components::{UiTheme, dialog_header_title_fit};
use iocraft::prelude::*;

/// Gap between sections inside inline dialog bodies (tighter than modal dialogs).
pub const INLINE_SECTION_GAP: u16 = 0;

/// Inner content width inside the round border and zone padding.
pub fn inline_body_width(screen_width: u16) -> u16 {
    UiTheme::default().shell_editor_inner_width(screen_width)
}

/// Props for [`InlineDialogShell`].
#[derive(Props)]
pub struct InlineDialogShellProps<'a> {
    pub screen_width: u16,
    pub title: String,
    pub has_focus: bool,
    pub children: Vec<AnyElement<'a>>,
}

impl<'a> Default for InlineDialogShellProps<'a> {
    fn default() -> Self {
        Self {
            screen_width: 80,
            title: String::new(),
            has_focus: false,
            children: Vec::new(),
        }
    }
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
            View(width: inner, flex_shrink: 0f32) {
                Text(
                    content: title,
                    color: theme.text_primary,
                    weight: Weight::Bold,
                    wrap: TextWrap::NoWrap,
                )
            }
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
        }
    }
}
