//! Session header bar — session id + token stats.

use elph_tui::components::theme::UiTheme;
use iocraft::prelude::*;

#[derive(Default, Props)]
pub struct HeaderProps {
    pub screen_width: u16,
    pub session_label: String,
    pub stats_label: String,
}

#[component]
pub fn Header(props: &HeaderProps) -> impl Into<AnyElement<'static>> {
    let theme = UiTheme::default();
    let pad = theme.shell_zone_padding();

    element! {
        View(
            width: props.screen_width,
            flex_shrink: 0f32,
            background_color: theme.surface,
            border_style: BorderStyle::Single,
            border_edges: Edges::Top,
            border_color: theme.border,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding_left: pad,
            padding_right: pad,
        ) {
            Text(color: theme.text_hint, wrap: TextWrap::NoWrap, content: props.session_label.clone())
            Text(color: theme.text_hint, wrap: TextWrap::NoWrap, content: props.stats_label.clone())
        }
    }
}
