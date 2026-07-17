//! Slim bordered palette: title chip + command list.

use elph_tui::components::theme::UiTheme;
use elph_tui::prelude::*;
use elph_tui::slash_palette::PaletteSnapshot;

use super::body::PaletteCardBody;
use super::chrome::PaletteCardChrome;
use super::frame::PaletteCardFrame;

#[derive(Clone, Default, Props)]
pub struct SlashPaletteCardProps {
    pub screen_width: u16,
    pub screen_height: u16,
    pub snapshot: PaletteSnapshot,
    pub selected_index: Option<State<usize>>,
}

#[component]
pub fn SlashPaletteCard(props: &SlashPaletteCardProps) -> impl Into<AnyElement<'static>> {
    let theme = UiTheme::default();
    let chrome = PaletteCardChrome::from_snapshot(props.screen_width, theme, &props.snapshot);

    element! {
        PaletteCardFrame(chrome: chrome.clone()) {
            PaletteCardBody(
                chrome: chrome,
                snapshot: props.snapshot.clone(),
                selected_index: props.selected_index,
                screen_height: props.screen_height,
            )
        }
    }
}
