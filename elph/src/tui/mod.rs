//! TUI bridge between coding-agent events and the tuie transcript.

mod bridge;
mod dispatch;
mod transcript_loader;
mod widget;

pub use bridge::TranscriptApplier;
pub use dispatch::TurnDispatcher;
pub use transcript_loader::transcript_from_branch;
