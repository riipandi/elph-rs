//! Slim bordered palette shell with a single top-left label.

use elph_tui::prelude::*;

use super::chrome::PaletteCardChrome;

#[derive(Default, Props)]
pub struct PaletteCardFrameProps {
    pub chrome: PaletteCardChrome,
    pub children: Vec<AnyElement<'static>>,
}

#[component]
pub fn PaletteCardFrame(props: &mut PaletteCardFrameProps) -> impl Into<AnyElement<'static>> {
    let chrome = &props.chrome;
    let children = std::mem::take(&mut props.children);
    let theme = UiTheme::default();
    let inset = theme.padding_sm;
    let title = format!(" {} ", chrome.title);

    element! {
        View(
            width: chrome.card_width,
            border_style: BorderStyle::Round,
            border_color: chrome.border_color,
            background_color: chrome.background,
            padding_top: 0,
            padding_bottom: 0,
            padding_left: inset,
            padding_right: inset,
            gap: 0,
            flex_direction: FlexDirection::Column,
            position: Position::Relative,
        ) {
            View(
                position: Position::Absolute,
                top: 0,
                left: inset,
                margin_top: -1,
                background_color: chrome.background,
            ) {
                Text(
                    content: title,
                    color: chrome.title_color,
                    weight: Weight::Bold,
                    wrap: TextWrap::NoWrap,
                )
            }
            #(children)
        }
    }
}
