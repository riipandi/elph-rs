//! Top and mid chrome: header, status row, live stats.

mod header;
mod stats;
mod status_row;

pub use header::Header;
pub use stats::ChromeStats;
pub use stats::{header_stats_from_chrome, read_git_branch, refresh_chrome_stats};
pub use status_row::StatusRow;
pub use status_row::format_elapsed_secs;
