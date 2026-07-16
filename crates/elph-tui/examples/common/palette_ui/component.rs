//! Floating shell that anchors the palette card above the editor.

use elph_tui::prelude::*;
use elph_tui::slash_palette::PaletteSnapshot;

use super::card::SlashPaletteCard;

#[derive(Clone, Default, Props)]
pub struct SlashCommandPaletteProps {
    pub screen_width: u16,
    pub screen_height: u16,
    pub snapshot: PaletteSnapshot,
    pub anchor_bottom: u16,
    pub selected_index: Option<State<usize>>,
}

#[component]
pub fn SlashCommandPalette(props: &SlashCommandPaletteProps) -> impl Into<AnyElement<'static>> {
    if !props.snapshot.should_render() {
        return element! { View(width: 0u16, height: 0u16) {} };
    }

    element! {
        View(
            width: props.screen_width,
            position: Position::Absolute,
            left: 0,
            bottom: props.anchor_bottom,
            flex_shrink: 0f32,
            align_items: AlignItems::FlexStart,
        ) {
            SlashPaletteCard(
                screen_width: props.screen_width,
                screen_height: props.screen_height,
                snapshot: props.snapshot.clone(),
                selected_index: props.selected_index,
            )
        }
    }
}
