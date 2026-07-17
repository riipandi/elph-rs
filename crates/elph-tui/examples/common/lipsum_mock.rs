//! Lorem ipsum helpers for example mock content.

use elph_tui::prelude::SelectOption;
use lipsum::{lipsum, lipsum_title, lipsum_words};

/// Short pseudo-Latin phrase (~`words` words).
pub fn mock_phrase(words: usize) -> String {
    lipsum_words(words.max(1))
}

/// One mock sentence (~12 words).
pub fn mock_sentence() -> String {
    lipsum(12)
}

/// One mock paragraph (~32 words).
pub fn mock_paragraph() -> String {
    lipsum(32)
}

/// Title-case label for dialog headers or options.
pub fn mock_title() -> String {
    lipsum_title()
}

/// Build numbered select options with lipsum labels and descriptions.
pub fn mock_select_options(count: usize) -> Vec<SelectOption> {
    (0..count)
        .map(|i| SelectOption::new(format!("{}. {}", i + 1, mock_title()), mock_phrase(8)))
        .collect()
}
