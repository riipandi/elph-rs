//! Terminal UI components for Elph agent.
//!
//! OpenTUI-inspired component APIs implemented with [iocraft](https://crates.io/crates/iocraft).
//!
//! @ref: https://opentui.com/docs/getting-started

pub mod cli_progress;
pub mod color;
pub mod components;
pub mod loader;
pub mod paste;
pub mod text_editing;
pub mod text_input_layout;
pub mod transcript_layout;
pub mod types;
pub mod utils;

pub use cli_progress::{CliProgress, CliSpinner};
pub use cli_progress::{progress_enabled, progress_spinner};
pub use color::{from_hex, rgb};
pub use components::*;
pub use loader::{KittScanner, KittScannerConfig, LoaderCell, SpinnerLoader};
pub use types::{SelectOption, TabItem};

/// Convenience re-exports for application authors.
pub mod prelude {
    pub use crate::color::{from_hex, rgb};
    pub use crate::components::*;
    pub use crate::types::{SelectOption, TabItem};
    pub use iocraft::prelude::*;
}
