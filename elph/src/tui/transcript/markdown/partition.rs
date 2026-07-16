//! Conservative stable-boundary detection for streaming markdown.

/// Byte index through which `raw` is safe to parse as markdown.
///
/// When `force_flush` is true (turn complete), the entire buffer is stable.
pub fn find_stable_boundary(raw: &str, force_flush: bool) -> usize {
    if raw.is_empty() {
        return 0;
    }
    if force_flush {
        return raw.len();
    }

    let search_end = fence_safe_end(raw);
    let slice = &raw[..search_end];
    if let Some(pos) = slice.rfind("\n\n") {
        return pos + 2;
    }
    0
}

/// Cap parsing before an unclosed `` ``` `` fence.
fn fence_safe_end(raw: &str) -> usize {
    let mut count = 0usize;
    let mut last_open = 0usize;
    let mut pos = 0usize;
    while let Some(rel) = raw[pos..].find("```") {
        let abs = pos + rel;
        count += 1;
        last_open = abs;
        pos = abs + 3;
    }
    if count % 2 == 1 { last_open } else { raw.len() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_buffer_has_no_stable_prefix() {
        assert_eq!(find_stable_boundary("", false), 0);
    }

    #[test]
    fn paragraph_boundary_stabilizes_prefix() {
        let raw = "# Title\n\nBody one.\nPartial";
        assert_eq!(find_stable_boundary(raw, false), 9);
    }

    #[test]
    fn unclosed_fence_defers_stability() {
        let raw = "intro\n\n```rust\nlet x = 1;\nstill typing";
        assert_eq!(find_stable_boundary(raw, false), 7);
    }

    #[test]
    fn closed_fence_allows_stability_after_block() {
        let raw = "intro\n\n```rust\nlet x = 1;\n```\n\nDone.";
        let stable_through_fence = raw.find("Done.").expect("tail");
        assert_eq!(find_stable_boundary(raw, false), stable_through_fence);
        assert_eq!(find_stable_boundary(raw, true), raw.len());
    }

    #[test]
    fn force_flush_returns_full_length() {
        let raw = "```open\npartial";
        assert_eq!(find_stable_boundary(raw, true), raw.len());
    }
}
