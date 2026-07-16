//! Scrollable transcript panel with sticky user prompts.

mod card;
mod layout;
pub(crate) mod markdown;
mod panel;
mod types;

pub use panel::TranscriptPanel;
pub use types::{TranscriptMessage, TranscriptStyle};
