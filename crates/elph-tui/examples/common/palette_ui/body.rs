//! Flat command list inside the palette card.

use elph_tui::prelude::*;
use elph_tui::slash_palette::{PaletteSnapshot, list_viewport_cap, palette_window_start};

use super::chrome::PaletteCardChrome;
use super::row_layout::{CMD_DESC_GAP_COLS, visible_terminal_rows, wrap_palette_description};

#[derive(Clone, Default, Props)]
pub struct PaletteCardBodyProps {
    pub chrome: PaletteCardChrome,
    pub snapshot: PaletteSnapshot,
    pub selected_index: Option<State<usize>>,
    pub screen_height: u16,
}

fn palette_row(chrome: &PaletteCardChrome, name: &str, description: &str, selected: bool) -> AnyElement<'static> {
    let prefix = elph_tui::list_selection_row_prefix(selected);
    let name_color = if selected {
        chrome.name_active_color
    } else {
        chrome.name_idle_color
    };
    let name_weight = if selected { Weight::Bold } else { Weight::Normal };
    let desc_color = if selected {
        chrome.desc_active_color
    } else {
        chrome.desc_idle_color
    };
    let cmd_col = chrome.command_column_width;
    let desc_width = chrome.list_width.saturating_sub(cmd_col + CMD_DESC_GAP_COLS).max(1);
    let desc_lines = wrap_palette_description(description, chrome.list_width, cmd_col);
    let row_height = desc_lines.len().max(1) as u16;
    let desc_text = desc_lines.join("\n");

    element! {
        View(
            width: chrome.list_width,
            height: row_height,
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::FlexStart,
            gap: CMD_DESC_GAP_COLS,
        ) {
            View(width: cmd_col, height: row_height) {
                Text(
                    content: format!("{prefix}{name}"),
                    color: name_color,
                    weight: name_weight,
                    wrap: TextWrap::NoWrap,
                )
            }
            View(width: desc_width, height: row_height) {
                Text(content: desc_text, color: desc_color, wrap: TextWrap::NoWrap)
            }
        }
    }
    .into()
}

#[component]
pub fn PaletteCardBody(props: &PaletteCardBodyProps) -> impl Into<AnyElement<'static>> {
    if props.snapshot.has_matches() {
        let options = &props.snapshot.options;
        let len = options.len();
        let viewport_cap = list_viewport_cap(props.screen_height).max(1);
        let index = props
            .selected_index
            .map(|state| state.get())
            .unwrap_or(0)
            .min(len.saturating_sub(1));
        let scroll_cap = viewport_cap.min(len.max(1));
        let window_start = palette_window_start(index, scroll_cap, len);
        let rows: Vec<_> = options
            .iter()
            .enumerate()
            .skip(window_start)
            .take(scroll_cap)
            .map(|(i, opt)| palette_row(&props.chrome, &opt.name, &opt.description, i == index))
            .collect();
        let body_height = visible_terminal_rows(
            options,
            window_start,
            scroll_cap,
            props.chrome.list_width,
            props.chrome.command_column_width,
            viewport_cap,
        );
        element! {
            View(
                width: props.chrome.list_width,
                height: body_height,
                flex_direction: FlexDirection::Column,
                gap: 0,
            ) {
                #(rows)
            }
        }
    } else {
        element! {
            View(
                width: props.chrome.list_width,
                height: props.snapshot.list_height,
                justify_content: JustifyContent::Center,
            ) {
                Text(
                    content: "No matches".to_string(),
                    color: props.chrome.desc_idle_color,
                    wrap: TextWrap::NoWrap,
                )
            }
        }
    }
}
