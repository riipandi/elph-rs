//! Compaction result types.

/// File-operation details stored on generated compaction entries.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionDetails {
    pub read_files: Vec<String>,
    pub modified_files: Vec<String>,
}

/// Generated compaction data ready to be persisted as a compaction entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompactionResult {
    pub summary: String,
    pub first_kept_entry_id: String,
    pub tokens_before: u64,
    pub details: Option<CompactionDetails>,
}
