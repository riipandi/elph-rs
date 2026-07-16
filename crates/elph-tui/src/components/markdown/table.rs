//! GFM table layout and iocraft flex rendering (bordered, themed).

use iocraft::prelude::*;

use crate::text_input_layout::WrappedTextLayout;

use super::model::MarkdownTable;
use super::theme::MarkdownTheme;

const MIN_COL_WIDTH: u16 = 3;
/// Horizontal space consumed by a round border (left + right edges).
const TABLE_BORDER_COLS: u16 = 2;
/// Top and bottom border rows around the table body.
const TABLE_BORDER_ROWS: u16 = 2;

/// Measured column widths and per-row terminal heights for one table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableLayout {
    pub col_widths: Vec<u16>,
    pub row_heights: Vec<u16>,
}

fn column_count(table: &MarkdownTable) -> usize {
    table.rows.iter().map(|row| row.len()).max().unwrap_or(0)
}

fn cell_display_width(text: &str) -> u16 {
    use unicode_width::UnicodeWidthStr;
    text.width().max(1) as u16
}

fn cell_wrap_rows(text: &str, col_width: u16) -> u16 {
    if text.is_empty() {
        return 1;
    }
    WrappedTextLayout::new_for_overlay_editor(text, col_width.max(1)).row_count()
}

fn normalize_row(row: &[String], columns: usize) -> Vec<String> {
    let mut cells = row.to_vec();
    cells.resize(columns, String::new());
    cells
}

fn compute_column_widths(table: &MarkdownTable, max_width: u16) -> Vec<u16> {
    let columns = column_count(table);
    if columns == 0 {
        return Vec::new();
    }

    let inner_width = max_width.max(MIN_COL_WIDTH + TABLE_BORDER_COLS).saturating_sub(TABLE_BORDER_COLS);
    let mut natural = vec![MIN_COL_WIDTH; columns];
    for row in &table.rows {
        let cells = normalize_row(row, columns);
        for (index, cell) in cells.iter().enumerate() {
            natural[index] = natural[index].max(cell_display_width(cell));
        }
    }

    let total: u16 = natural.iter().sum();
    if total <= inner_width {
        distribute_extra_width(&mut natural, inner_width.saturating_sub(total));
        return natural;
    }

    shrink_columns_to_fit(&mut natural, inner_width);
    natural
}

fn distribute_extra_width(widths: &mut [u16], extra: u16) {
    if extra == 0 || widths.is_empty() {
        return;
    }
    let total: u32 = widths.iter().map(|&w| u32::from(w)).sum();
    if total == 0 {
        return;
    }
    let mut remaining = extra;
    let last = widths.len().saturating_sub(1);
    for (index, width) in widths.iter_mut().enumerate() {
        if index == last {
            *width = width.saturating_add(remaining);
            break;
        }
        let share = (u32::from(extra) * u32::from(*width) / total) as u16;
        *width = width.saturating_add(share);
        remaining = remaining.saturating_sub(share);
    }
}

fn shrink_columns_to_fit(widths: &mut [u16], inner_width: u16) {
    let columns = widths.len();
    if columns == 0 {
        return;
    }

    let min_total = MIN_COL_WIDTH.saturating_mul(columns as u16);
    if inner_width <= min_total {
        widths.fill(MIN_COL_WIDTH);
        return;
    }

    let mut surplus: i32 = i32::from(widths.iter().sum::<u16>()) - i32::from(inner_width);
    while surplus > 0 {
        let mut shrinkable: Vec<usize> = widths
            .iter()
            .enumerate()
            .filter_map(|(index, &width)| (width > MIN_COL_WIDTH).then_some(index))
            .collect();
        if shrinkable.is_empty() {
            break;
        }
        shrinkable.sort_by(|left, right| widths[*right].cmp(&widths[*left]));
        for index in shrinkable {
            if surplus <= 0 {
                break;
            }
            widths[index] -= 1;
            surplus -= 1;
        }
    }
}

/// Compute wrapped column widths and row heights for a markdown table matrix.
pub fn layout_markdown_table(table: &MarkdownTable, max_width: u16) -> Option<TableLayout> {
    if table.rows.is_empty() {
        return None;
    }
    let columns = column_count(table);
    if columns == 0 {
        return None;
    }

    let col_widths = compute_column_widths(table, max_width);
    let row_heights = table
        .rows
        .iter()
        .map(|row| {
            let cells = normalize_row(row, columns);
            cells
                .iter()
                .enumerate()
                .map(|(index, cell)| cell_wrap_rows(cell, col_widths[index]))
                .max()
                .unwrap_or(1)
        })
        .collect();

    Some(TableLayout { col_widths, row_heights })
}

/// Terminal row budget for scroll layout (borders + wrapped body rows).
pub fn markdown_table_row_count(table: &MarkdownTable, max_width: u16) -> u16 {
    let Some(layout) = layout_markdown_table(table, max_width) else {
        return 0;
    };
    layout
        .row_heights
        .iter()
        .copied()
        .fold(TABLE_BORDER_ROWS, u16::saturating_add)
        .max(1)
}

fn render_cell(
    text: &str,
    col_width: u16,
    color: Color,
    weight: Weight,
    decoration: TextDecoration,
) -> AnyElement<'static> {
    element! {
        View(width: col_width, flex_shrink: 0f32) {
            Text(
                content: text.to_string(),
                color: color,
                weight: weight,
                decoration: decoration,
                wrap: TextWrap::Wrap,
            )
        }
    }
    .into()
}

fn render_table_row(
    cells: &[String],
    col_widths: &[u16],
    theme: &MarkdownTheme,
    header: bool,
    zebra: bool,
) -> AnyElement<'static> {
    let columns = col_widths.len();
    let normalized = normalize_row(cells, columns);
    let bg = if header {
        None
    } else if zebra {
        Some(theme.table_zebra)
    } else {
        None
    };
    let cell_elements: Vec<AnyElement<'static>> = normalized
        .iter()
        .enumerate()
        .map(|(index, cell)| {
            let width = col_widths[index];
            if header {
                render_cell(
                    cell,
                    width,
                    theme.table_header,
                    Weight::Bold,
                    TextDecoration::Underline,
                )
            } else {
                render_cell(cell, width, theme.body, Weight::Normal, TextDecoration::None)
            }
        })
        .collect();

    if header {
        element! {
            View(
                border_style: BorderStyle::Single,
                border_edges: Edges::Bottom,
                border_color: theme.table_border,
                flex_shrink: 0f32,
            ) {
                #(cell_elements)
            }
        }
        .into()
    } else {
        element! {
            View(background_color: bg, flex_shrink: 0f32) {
                #(cell_elements)
            }
        }
        .into()
    }
}

/// Render a markdown table as a bordered iocraft flex table.
pub fn render_markdown_table(
    table: &MarkdownTable,
    width: u16,
    theme: &MarkdownTheme,
    margin_bottom: u16,
) -> Option<AnyElement<'static>> {
    let layout = layout_markdown_table(table, width)?;
    if table.rows.is_empty() {
        return None;
    }

    let header = &table.rows[0];
    let body = &table.rows[1..];
    let header_row = render_table_row(header, &layout.col_widths, theme, true, false);
    let body_rows: Vec<AnyElement<'static>> = body
        .iter()
        .enumerate()
        .map(|(index, row)| render_table_row(row, &layout.col_widths, theme, false, index % 2 == 1))
        .collect();

    Some(
        element! {
            View(
                width: width.max(1),
                margin_bottom: margin_bottom,
                border_style: BorderStyle::Round,
                border_color: theme.table_border,
                flex_direction: FlexDirection::Column,
                gap: 0,
                flex_shrink: 0f32,
            ) {
                #(header_row)
                #(body_rows)
            }
        }
        .into(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::markdown::parse_markdown_document;

    #[test]
    fn layout_respects_emoji_display_width() {
        let table = MarkdownTable {
            rows: vec![vec!["Name".into(), "Status".into()], vec!["Ada".into(), "✅".into()]],
        };
        let layout = layout_markdown_table(&table, 40).expect("layout");
        assert_eq!(layout.col_widths.len(), 2);
        assert!(layout.col_widths[0] >= 4);
        assert!(layout.col_widths[1] >= 2);
    }

    #[test]
    fn narrow_width_wraps_cells_and_increases_row_count() {
        let table = MarkdownTable {
            rows: vec![
                vec!["Header".into()],
                vec!["abcdefghijklmnopqrstuvwxyz".into()],
            ],
        };
        let rows = markdown_table_row_count(&table, 12);
        assert!(rows > 3, "wrapped cell should add terminal rows, got {rows}");
    }

    #[test]
    fn row_count_matches_layout_engine() {
        let table = MarkdownTable {
            rows: vec![
                vec!["A".into(), "B".into()],
                vec!["one".into(), "two".into()],
            ],
        };
        let layout = layout_markdown_table(&table, 30).expect("layout");
        let counted = markdown_table_row_count(&table, 30);
        let expected = layout.row_heights.iter().copied().fold(TABLE_BORDER_ROWS, u16::saturating_add);
        assert_eq!(counted, expected);
    }

    #[test]
    fn parsed_gfm_table_renders_bordered_block() {
        let doc = parse_markdown_document("| Name | Status |\n| --- | --- |\n| Ada | ✅ |");
        let table = doc
            .lines
            .iter()
            .find_map(|line| line.table.as_ref())
            .expect("table");
        let block = render_markdown_table(table, 40, &MarkdownTheme::default(), 0).expect("render");
        let rendered = element! { View(width: 40) { #(vec![block]) } }.to_string();
        assert!(rendered.contains("Ada"));
        assert!(rendered.contains('✅'));
    }
}