use crate::diff::SelectItem;
use slt::{Align, Border, Color, Context, Justify, ListState};

pub fn select_item_label(item: &SelectItem) -> String {
    match &item.description {
        Some(desc) => format!("{} — {}", item.label, desc),
        None => item.label.clone(),
    }
}

/// Renders a centered modal list and returns the updated selection index.
pub fn render_select_modal(
    ui: &mut Context,
    title: &str,
    items: &[SelectItem],
    selected: usize,
    border_color: Color,
    width_pct: u8,
) -> usize {
    if items.is_empty() {
        return 0;
    }

    let labels: Vec<String> = items.iter().map(select_item_label).collect();
    let mut list = ListState::new(labels);
    list.selected = selected.min(items.len().saturating_sub(1));

    let _ = ui.modal(|ui| {
        let _ = ui
            .container()
            .justify(Justify::Center)
            .align(Align::Center)
            .grow(1)
            .col(|ui| {
                let _ = ui
                    .bordered(Border::Rounded)
                    .border_fg(border_color)
                    .p(1)
                    .w_pct(width_pct)
                    .col(|ui| {
                        let _ = ui.text(title).bold();
                        let _ = ui.list(&mut list);
                    });
            });
    });

    list.selected
}
