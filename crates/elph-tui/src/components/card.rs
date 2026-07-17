//! Bordered panel container (OpenTUI Box analogue).

use iocraft::prelude::*;

use super::theme::{UiTheme, resolve_ui_theme};

/// Border style alias matching OpenTUI naming.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CardBorderStyle {
    #[default]
    Single,
    Double,
    Round,
    Bold,
    None,
}

impl CardBorderStyle {
    pub fn to_iocraft(self) -> BorderStyle {
        match self {
            Self::Single => BorderStyle::Single,
            Self::Double => BorderStyle::Double,
            Self::Round => BorderStyle::Round,
            Self::Bold => BorderStyle::Bold,
            Self::None => BorderStyle::None,
        }
    }
}

/// Props for [`Card`].
#[derive(Default, Props)]
pub struct CardProps<'a> {
    pub width: u16,
    pub min_height: u16,
    pub title: String,
    pub border_style: CardBorderStyle,
    pub border_color: Option<Color>,
    pub background_color: Option<Color>,
    pub padding: u16,
    pub gap: u16,
    pub theme: Option<UiTheme>,
    pub children: Vec<AnyElement<'a>>,
}

/// Bordered container with optional overlapping title label.
#[component]
pub fn Card<'a>(props: &mut CardProps<'a>, hooks: Hooks) -> impl Into<AnyElement<'a>> {
    let border = props.border_style.to_iocraft();
    let show_title = !props.title.is_empty() && props.border_style != CardBorderStyle::None;
    let children = std::mem::take(&mut props.children);
    let title = props.title.clone();
    let theme = resolve_ui_theme(&hooks, props.theme);
    let border_color = props.border_color.unwrap_or(theme.border);
    let background_color = props.background_color.unwrap_or(theme.list_surface());
    let padding = if props.padding == 0 {
        theme.padding_md
    } else {
        props.padding
    };
    let gap = if props.gap == 0 { theme.gap_md } else { props.gap };

    element! {
        View(
            width: props.width,
            min_height: props.min_height,
            border_style: border,
            border_color: border_color,
            background_color: background_color,
            padding: padding,
            gap: gap,
            flex_direction: FlexDirection::Column,
            position: Position::Relative,
        ) {
            #(if show_title {
                element! {
                    View(
                        position: Position::Absolute,
                        top: 0,
                        left: 1,
                        margin_top: -1,
                        background_color: background_color,
                    ) {
                        Text(
                            content: format!(" {} ", title),
                            color: theme.text_primary,
                            weight: Weight::Bold,
                            wrap: TextWrap::NoWrap,
                        )
                    }
                }
            } else {
                element!(View)
            })
            #(children)
        }
    }
}
