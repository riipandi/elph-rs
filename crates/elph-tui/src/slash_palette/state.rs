//! Selection state kept by the shell while the palette is active.

use super::model::{PaletteSnapshot, clamp_index};

/// Reset and clamp palette selection when the filter query or match list changes.
pub fn sync_selection(tracked_query: &mut String, selected_index: &mut usize, snapshot: &PaletteSnapshot) {
    if *tracked_query != snapshot.query {
        *tracked_query = snapshot.query.clone();
        *selected_index = 0;
    }
    if snapshot.has_matches() {
        *selected_index = clamp_index(*selected_index, snapshot.options.len());
    } else {
        *selected_index = 0;
    }
}
