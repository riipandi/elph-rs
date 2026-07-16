//! Incremental markdown cache for one streaming assistant message.

use super::layout::markdown_part_row_count;
use super::partition::find_stable_boundary;

/// One stable markdown segment (elements rendered on demand at display time).
#[derive(Clone)]
pub struct RenderedPart {
    pub source_end: usize,
    pub row_count: u16,
}

/// Streaming markdown state for [`crate::tui::transcript::TranscriptMessage`].
#[derive(Clone, Default)]
pub struct AssistantMarkdownBuffer {
    pub stable_end: usize,
    pub parts: Vec<RenderedPart>,
    pub wrap_width: u16,
    pub stream_complete: bool,
}

impl AssistantMarkdownBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tail<'a>(&self, raw: &'a str) -> &'a str {
        raw.get(self.stable_end..).unwrap_or("")
    }

    pub fn has_rendered_body(&self) -> bool {
        !self.parts.is_empty() || self.stable_end > 0
    }

    /// Advance stable boundary and refresh cached parts when the prefix grew.
    ///
    /// Returns `true` when `parts` or `stable_end` changed.
    pub fn refresh_stable(&mut self, raw: &str, wrap_width: u16) -> bool {
        if wrap_width == 0 {
            return false;
        }
        if self.wrap_width != wrap_width && self.has_rendered_body() {
            self.stable_end = 0;
            self.parts.clear();
        }
        self.wrap_width = wrap_width;

        let force = self.stream_complete;
        let new_end = find_stable_boundary(raw, force);
        if new_end <= self.stable_end {
            return false;
        }

        let stable = &raw[..new_end];
        self.parts = vec![RenderedPart {
            source_end: new_end,
            row_count: markdown_part_row_count(stable, wrap_width),
        }];
        self.stable_end = new_end;
        true
    }

    pub fn mark_stream_complete(&mut self) {
        self.stream_complete = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refresh_grows_stable_prefix() {
        let mut buf = AssistantMarkdownBuffer::new();
        let raw = "# Hi\n\nParagraph.";
        assert!(buf.refresh_stable(raw, 40));
        assert_eq!(buf.stable_end, 6);
        buf.mark_stream_complete();
        assert!(buf.refresh_stable(raw, 40));
        assert_eq!(buf.stable_end, raw.len());
        assert_eq!(buf.parts.len(), 1);
        assert!(buf.parts[0].row_count > 0);
    }

    #[test]
    fn refresh_skips_when_boundary_unchanged() {
        let mut buf = AssistantMarkdownBuffer::new();
        let raw = "no paragraph break yet";
        assert!(!buf.refresh_stable(raw, 40));
        assert_eq!(buf.stable_end, 0);
        assert!(buf.parts.is_empty());
    }

    #[test]
    fn width_change_invalidates_cache() {
        let mut buf = AssistantMarkdownBuffer::new();
        let raw = "A\n\nB";
        assert!(buf.refresh_stable(raw, 40));
        assert!(buf.refresh_stable(raw, 30));
        assert_eq!(buf.wrap_width, 30);
    }
}
