//! GFM table formatting for terminal markdown via [`tabled`].

use tabled::builder::Builder;
use tabled::settings::{Style, Width};

use super::model::MarkdownTable;

/// Render a markdown table matrix into terminal lines at `max_width` display columns.
pub fn format_markdown_table(table: &MarkdownTable, max_width: u16) -> Vec<String> {
    if table.rows.is_empty() {
        return Vec::new();
    }

    let mut builder = Builder::default();
    for row in &table.rows {
        builder.push_record(row.iter().map(String::as_str));
    }

    let width = usize::from(max_width.max(1));
    builder
        .build()
        .with(Style::markdown())
        .with(Width::wrap(width).keep_words(true))
        .to_string()
        .lines()
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_pipe_table_with_headers() {
        let table = MarkdownTable {
            rows: vec![vec!["Name".into(), "Status".into()], vec!["Ada".into(), "✅".into()]],
        };
        let lines = format_markdown_table(&table, 40);
        assert!(!lines.is_empty());
        let joined = lines.join("\n");
        assert!(joined.contains('|'));
        assert!(joined.contains("Ada"));
        assert!(joined.contains('✅'));
    }
}
