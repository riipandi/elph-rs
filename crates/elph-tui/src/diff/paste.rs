use std::ops::Range;

/// Collapse pastes with at least this many logical lines.
pub const PASTE_COLLAPSE_MIN_LINES: usize = 2;

/// Collapse pastes with at least this many bytes (single-line pastes stay expanded longer).
pub const PASTE_COLLAPSE_MIN_CHARS: usize = 256;

const MARKER_PREFIX: &str = "[Pasted: ";
const MARKER_SUFFIX: &str = " lines] ";

/// Normalizes clipboard text for insertion.
pub fn normalize_paste_text(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                if chars.peek() != Some(&'\n') {
                    out.push('\n');
                }
            }
            ch => out.push(ch),
        }
    }
    out
}

/// Collapsed paste stored in the prompt field.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollapsedPaste {
    pub full: String,
    pub summary: String,
    /// Byte offset of `summary` in the display text at collapse time.
    pub offset: usize,
}

impl CollapsedPaste {
    pub fn new(full: String, preview_width: usize, offset: usize) -> Self {
        let summary = format_paste_summary(&full, preview_width);
        Self { full, summary, offset }
    }
}

/// Counts logical lines (minimum 1 for non-empty text).
pub fn line_count(text: &str) -> usize {
    if text.is_empty() {
        return 0;
    }
    text.chars().filter(|&c| c == '\n').count() + 1
}

/// Returns `true` when pasted text should collapse to a summary chip.
pub fn should_collapse_paste(text: &str) -> bool {
    line_count(text) >= PASTE_COLLAPSE_MIN_LINES || text.len() >= PASTE_COLLAPSE_MIN_CHARS
}

/// Builds `[Pasted: NN lines] preview` for display in the prompt field.
pub fn format_paste_summary(full: &str, preview_width: usize) -> String {
    let lines = line_count(full);
    let marker = format!("{MARKER_PREFIX}{lines:02}{MARKER_SUFFIX}");
    let preview_budget = preview_width.saturating_sub(marker.chars().count());
    let preview = first_line_preview(full, preview_budget);
    if preview.is_empty() {
        marker
    } else {
        format!("{marker}{preview}")
    }
}

/// Shifts stored offsets at or after `from` by `delta` bytes.
pub fn shift_paste_offsets(pastes: &mut [CollapsedPaste], from: usize, delta: isize) {
    if delta == 0 {
        return;
    }
    for paste in pastes.iter_mut() {
        if paste.offset >= from {
            paste.offset = ((paste.offset as isize) + delta).max(0) as usize;
        }
    }
}

/// Shifts offsets for an insertion at `at` with byte length `len`.
pub fn shift_paste_offsets_for_insert(pastes: &mut [CollapsedPaste], at: usize, len: usize) {
    if len > 0 {
        shift_paste_offsets(pastes, at, len as isize);
    }
}

/// Removes pastes wholly inside `range` and shifts later markers.
pub fn adjust_pastes_for_delete(pastes: &mut Vec<CollapsedPaste>, range: Range<usize>) {
    let len = range.end.saturating_sub(range.start);
    if len == 0 {
        return;
    }
    pastes.retain(|paste| {
        let end = paste.offset.saturating_add(paste.summary.len());
        end <= range.start || paste.offset >= range.end
    });
    shift_paste_offsets(pastes, range.end, -(len as isize));
}

/// Re-syncs stored offsets against the current display text (handles preceding edits).
pub fn reconcile_paste_offsets(display: &str, pastes: &mut [CollapsedPaste]) {
    let mut used: Vec<Range<usize>> = Vec::new();
    let mut order: Vec<usize> = (0..pastes.len()).collect();
    order.sort_by_key(|idx| pastes[*idx].offset);

    for idx in order {
        let summary = pastes[idx].summary.clone();
        let start = pastes[idx].offset;
        let end = start.saturating_add(summary.len());
        if end <= display.len()
            && display.get(start..end) == Some(summary.as_str())
            && !range_overlaps_used(start..end, &used)
        {
            used.push(start..end);
            continue;
        }

        let mut search = 0usize;
        let mut matched = false;
        while search < display.len() {
            let Some(rel) = display[search..].find(&summary) else {
                break;
            };
            let at = search + rel;
            let end = at.saturating_add(summary.len());
            if end <= display.len() && !range_overlaps_used(at..end, &used) {
                pastes[idx].offset = at;
                used.push(at..end);
                matched = true;
                break;
            }
            search = at.saturating_add(1);
        }
        if !matched {
            pastes[idx].offset = display.len();
        }
    }
}

/// Expands collapsed paste summaries back to full text for submit.
pub fn expand_paste_markers(display: &str, pastes: &[CollapsedPaste]) -> String {
    if pastes.is_empty() {
        return display.to_string();
    }

    let mut resolved = pastes.to_vec();
    reconcile_paste_offsets(display, &mut resolved);

    let mut ordered: Vec<&CollapsedPaste> = resolved.iter().collect();
    ordered.sort_by_key(|paste| paste.offset);

    let mut out = String::new();
    let mut cursor = 0usize;
    for paste in ordered {
        if paste.offset < cursor || paste.offset > display.len() {
            continue;
        }
        let end = paste.offset.saturating_add(paste.summary.len());
        if end > display.len() || display[paste.offset..end] != paste.summary {
            continue;
        }
        out.push_str(&display[cursor..paste.offset]);
        out.push_str(&paste.full);
        cursor = end;
    }
    out.push_str(&display[cursor..]);
    out
}

/// Finds the byte range of a collapsed paste summary covering `cursor`, if any.
pub fn paste_block_range(value: &str, cursor: usize, pastes: &[CollapsedPaste]) -> Option<Range<usize>> {
    let cursor = cursor.min(value.len());
    let mut resolved = pastes.to_vec();
    reconcile_paste_offsets(value, &mut resolved);

    for paste in &resolved {
        let start = paste.offset;
        let end = start.saturating_add(paste.summary.len());
        if end <= value.len() && value[start..end] == paste.summary && cursor >= start && cursor <= end {
            return Some(start..end);
        }
    }
    None
}

/// Removes the collapsed paste block at `range` and returns the removed paste index (0-based).
pub fn remove_paste_block(value: &str, range: Range<usize>, pastes: &[CollapsedPaste]) -> (String, Option<usize>) {
    if range.start > range.end || range.end > value.len() {
        return (value.to_string(), None);
    }
    let matched = &value[range.start..range.end];
    let mut resolved = pastes.to_vec();
    reconcile_paste_offsets(value, &mut resolved);
    let Some(idx) = resolved
        .iter()
        .position(|paste| paste.offset == range.start && paste.summary == matched)
    else {
        return (value.to_string(), None);
    };

    let mut next = String::new();
    next.push_str(&value[..range.start]);
    next.push_str(&value[range.end..]);
    (next, Some(idx))
}

/// Removes a paste block, updates `pastes` offsets, and returns the new display text.
pub fn remove_paste_block_and_adjust(
    value: &str,
    range: Range<usize>,
    pastes: &mut Vec<CollapsedPaste>,
) -> Option<String> {
    let (next, idx) = remove_paste_block(value, range.clone(), pastes);
    let idx = idx?;
    reconcile_paste_offsets(value, pastes);
    pastes.remove(idx);
    adjust_pastes_for_delete(pastes, range);
    Some(next)
}

fn range_overlaps_used(range: Range<usize>, used: &[Range<usize>]) -> bool {
    used.iter()
        .any(|other| range.start < other.end && other.start < range.end)
}

fn first_line_preview(full: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let line = full
        .lines()
        .find(|line| !line.trim().is_empty())
        .or_else(|| full.lines().next())
        .unwrap_or("");
    let line = line.trim_end();
    if line.is_empty() {
        return String::new();
    }
    truncate_to_char_boundary(line, max_chars)
}

fn truncate_to_char_boundary(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let mut out = String::new();
    for ch in text.chars().take(max_chars.saturating_sub(1)) {
        out.push(ch);
    }
    out.push_str("...");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_windows_line_endings() {
        assert_eq!(normalize_paste_text("a\r\nb"), "a\nb");
        assert_eq!(normalize_paste_text("a\rb"), "a\nb");
    }

    #[test]
    fn collapses_multiline_paste() {
        assert!(should_collapse_paste("a\nb"));
        assert!(!should_collapse_paste("short"));
    }

    #[test]
    fn formats_zero_padded_line_count() {
        let summary = format_paste_summary("alpha\nbeta", 40);
        assert!(summary.starts_with("[Pasted: 02 lines] "));
        assert!(summary.contains("alpha"));
    }

    #[test]
    fn preview_preserves_leading_indentation() {
        let summary = format_paste_summary("    fn main() {\n        println!();\n    }", 50);
        assert!(summary.contains("    fn main()"));
    }

    #[test]
    fn expands_markers_on_submit() {
        let paste = CollapsedPaste::new("alpha\nbeta".into(), 40, 7);
        let display = format!("before {} after", paste.summary);
        assert_eq!(expand_paste_markers(&display, &[paste]), "before alpha\nbeta after");
    }

    #[test]
    fn expands_by_stored_offset_not_first_substring_match() {
        let summary = CollapsedPaste::new("real body".into(), 40, 0).summary;
        let offset = summary.len() + 1;
        let paste = CollapsedPaste::new("real body".into(), 40, offset);
        let display = format!("{summary} {summary}");
        assert_eq!(expand_paste_markers(&display, &[paste]), format!("{summary} real body"));
    }

    #[test]
    fn finds_paste_block_for_cursor() {
        let paste = CollapsedPaste::new("alpha\nbeta".into(), 40, 3);
        let value = format!("hi {}", paste.summary);
        let range = paste_block_range(&value, 4, std::slice::from_ref(&paste)).expect("cursor on marker");
        assert_eq!(&value[range], paste.summary);
    }

    #[test]
    fn removes_paste_block_and_returns_index() {
        let paste = CollapsedPaste::new("alpha\nbeta".into(), 40, 2);
        let value = format!("x {} y", paste.summary);
        let range = paste_block_range(&value, 3, std::slice::from_ref(&paste)).unwrap();
        let (next, idx) = remove_paste_block(&value, range, &[paste]);
        assert_eq!(next, "x  y");
        assert_eq!(idx, Some(0));
    }

    #[test]
    fn removes_correct_index_when_duplicate_summaries() {
        let summary = CollapsedPaste::new("first body".into(), 40, 0).summary;
        let offset1 = summary.len() + 1;
        let paste0 = CollapsedPaste::new("first body".into(), 40, 0);
        let paste1 = CollapsedPaste {
            full: "second body".into(),
            summary: summary.clone(),
            offset: offset1,
        };
        let value = format!("{summary} {summary}");
        let mut pastes = vec![paste0, paste1];

        let range = paste_block_range(&value, offset1 + 2, &pastes).expect("cursor on second block");
        assert_eq!(range.start, offset1);

        let next = remove_paste_block_and_adjust(&value, range, &mut pastes).expect("removed");
        assert_eq!(next, format!("{summary} "));
        assert_eq!(pastes.len(), 1);
        assert_eq!(pastes[0].full, "first body");
        assert_eq!(pastes[0].offset, 0);
    }

    #[test]
    fn delete_first_paste_then_expand_remaining() {
        let summary = CollapsedPaste::new("first body".into(), 40, 0).summary;
        let offset1 = summary.len() + 1;
        let paste0 = CollapsedPaste::new("first body".into(), 40, 0);
        let paste1 = CollapsedPaste {
            full: "second body".into(),
            summary: summary.clone(),
            offset: offset1,
        };
        let value = format!("{summary} {summary}");
        let mut pastes = vec![paste0, paste1];

        let range = paste_block_range(&value, 1, &pastes).expect("cursor on first block");
        let next = remove_paste_block_and_adjust(&value, range, &mut pastes).expect("removed first");
        assert_eq!(next, format!(" {summary}"));
        assert_eq!(expand_paste_markers(&next, &pastes), " second body");
    }

    #[test]
    fn pre_edit_before_marker_then_expand() {
        let paste = CollapsedPaste::new("alpha\nbeta".into(), 40, 0);
        let mut pastes = vec![paste.clone()];
        let mut display = paste.summary.clone();
        display.insert_str(0, "EDIT");
        shift_paste_offsets_for_insert(&mut pastes, 0, "EDIT".len());
        assert_eq!(expand_paste_markers(&display, &pastes), "EDITalpha\nbeta");
    }

    #[test]
    fn pre_edit_before_marker_then_block_delete() {
        let paste = CollapsedPaste::new("alpha\nbeta".into(), 40, 0);
        let mut pastes = vec![paste.clone()];
        let mut display = paste.summary.clone();
        display.insert(0, 'X');
        shift_paste_offsets_for_insert(&mut pastes, 0, 1);
        let range = paste_block_range(&display, pastes[0].offset + 1, &pastes).expect("find block");
        let next = remove_paste_block_and_adjust(&display, range, &mut pastes).expect("delete");
        assert_eq!(next, "X");
        assert!(pastes.is_empty());
    }

    #[test]
    fn adjust_pastes_for_delete_shifts_later_markers() {
        let first = CollapsedPaste::new("a\nb".into(), 40, 0);
        let gap = 1usize;
        let second = CollapsedPaste::new("c\nd".into(), 40, first.summary.len() + gap);
        let mut pastes = vec![first.clone(), second];
        adjust_pastes_for_delete(&mut pastes, 0..first.summary.len());
        assert_eq!(pastes.len(), 1);
        assert_eq!(pastes[0].offset, gap);
    }
}
