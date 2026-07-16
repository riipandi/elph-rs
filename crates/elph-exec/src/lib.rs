//! Configurable local shell and PTY execution.
//!
//! Use [`ShellConfig`] for defaults (shell binary, PTY size, base env) and
//! [`ShellExecOptions`] for per-run overrides (cwd, timeout, streaming callbacks, cancel).

mod error;
mod output;
#[cfg(unix)]
pub mod pty;
mod shell;
mod types;

pub use error::{ExecError, ExecErrorCode, Result};
pub use output::sanitize_binary_output;
pub use shell::{exec_shell_command, resolve_shell};
pub use types::{ShellConfig, ShellExecOptions, ShellExecResult, ShellOutputCallback};

#[cfg(unix)]
pub use pty::{Pts, PtyMaster, PtySize, open_pty};
