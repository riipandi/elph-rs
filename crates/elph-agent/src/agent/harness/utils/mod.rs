//! Harness utility helpers — elph-agent module.

pub mod shell_output;
pub mod truncate;

pub use shell_output::{ShellCaptureOptions, ShellCaptureResult};
pub use shell_output::{execute_shell_with_capture, finalize_shell_capture, sanitize_binary_output};
pub use truncate::{DEFAULT_MAX_BYTES, DEFAULT_MAX_LINES, GREP_MAX_LINE_LENGTH};
pub use truncate::{TruncatedBy, TruncationOptions, TruncationResult};
pub use truncate::{format_size, select_line_range, truncate_head, truncate_line, truncate_tail};
