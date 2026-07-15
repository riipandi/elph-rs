//! Bordered panel container (OpenTUI Box analogue).

use iocraft::prelude::*;

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
    fn to_iocraft(self) -> BorderStyle {
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
    pub children: Vec<AnyElement<'a>>,
}

/// Bordered container with optional overlapping title label.
#[component]
pub fn Card<'a>(props: &mut CardProps<'a>) -> impl Into<AnyElement<'a>> {
    let border = props.border_style.to_iocraft();
    let show_title = !props.title.is_empty() && props.border_style != CardBorderStyle::None;
    let children = std::mem::take(&mut props.children);
    let title = props.title.clone();
    let border_color = props.border_color.unwrap_or(Color::DarkGrey);
    let background_color = props.background_color.unwrap_or(Color::Reset);

    element! {
        View(
            width: props.width,
            min_height: props.min_height,
            border_style: border,
            border_color: border_color,
            background_color: background_color,
            padding: props.padding,
            gap: props.gap,
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
                            color: border_color,
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
