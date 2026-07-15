//! Bounded scrollable region (OpenTUI scroll container analogue).

use super::scroll_bar::ScrollbarStyle;
use iocraft::prelude::*;

/// Props for [`ScrollBox`].
#[derive(Default, Props)]
pub struct ScrollBoxProps<'a> {
    pub width: u16,
    pub height: u16,
    pub auto_scroll: bool,
    pub keyboard_scroll: bool,
    pub scroll_step: u16,
    pub scrollbar: bool,
    pub scrollbar_style: Option<ScrollbarStyle>,
    pub children: Vec<AnyElement<'a>>,
}

/// Clipped viewport with an inner [`ScrollView`].
#[component]
pub fn ScrollBox<'a>(props: &mut ScrollBoxProps<'a>) -> impl Into<AnyElement<'a>> {
    let style = props.scrollbar_style.unwrap_or_else(ScrollbarStyle::dark);
    let children = std::mem::take(&mut props.children);

    element! {
        View(
            width: props.width,
            height: props.height,
            overflow: Overflow::Hidden,
            border_style: BorderStyle::Single,
            border_color: Color::DarkGrey,
        ) {
            View(width: 100pct, height: 100pct, overflow: Overflow::Hidden) {
                ScrollView(
                    auto_scroll: props.auto_scroll,
                    keyboard_scroll: Some(props.keyboard_scroll),
                    scroll_step: if props.scroll_step == 0 {
                        None
                    } else {
                        Some(props.scroll_step)
                    },
                    scrollbar: Some(props.scrollbar),
                    scrollbar_thumb_color: style.thumb_color,
                    scrollbar_track_color: style.track_color,
                ) {
                    #(children)
                }
            }
        }
    }
}
