#![allow(dead_code)]

use super::exit_message;
use crate::app;
use elph_tui::disable_keyboard_enhancement;
use std::sync::atomic::AtomicBool;

#[cfg(unix)]
use libc::{SIGTERM, getppid, kill};

pub static WAS_INTERRUPTED: AtomicBool = AtomicBool::new(false);

#[cfg(unix)]
pub static SHOULD_KILL_PARENT: AtomicBool = AtomicBool::new(false);

#[cfg(unix)]
pub fn kill_parent() {
    let ppid = unsafe { getppid() };
    if ppid > 1 {
        unsafe {
            kill(ppid, SIGTERM);
        }
    }
}

pub type ExitCode = i32;

pub const EXIT_SUCCESS: ExitCode = 0;
pub const EXIT_ERROR: ExitCode = 1;
pub const EXIT_AUTH_ERROR: ExitCode = 3;
pub const EXIT_PERMISSION_DENIED: ExitCode = 4;
pub const EXIT_RATE_LIMITED: ExitCode = 5;
pub const EXIT_CONNECTION_ERROR: ExitCode = 6;
pub const EXIT_SERVER_ERROR: ExitCode = 7;
pub const EXIT_INTERRUPTED: ExitCode = 130;

struct KeyboardEnhancementGuard;

impl Drop for KeyboardEnhancementGuard {
    fn drop(&mut self) {
        if let Err(e) = disable_keyboard_enhancement() {
            tracing::error!(error = %e, "failed to restore keyboard enhancements");
        }
    }
}

pub fn run() {
    let _guard = KeyboardEnhancementGuard;
    let result = app::run_tui();
    exit_message::print_and_clear();
    if let Err(e) = result {
        tracing::error!(error = %e, "app error");
    }
}

#[cfg(test)]
mod tests {
    use elph_tui::sigint_channel;
    use std::time::Duration;

    #[cfg(unix)]
    fn raise_sigint() {
        unsafe {
            libc::raise(libc::SIGINT);
        }
    }

    #[cfg(unix)]
    #[test]
    fn sigint_channel_receives_signal() {
        elph_agent::block_on(async {
            let mut sigint = sigint_channel();
            std::thread::spawn(|| {
                std::thread::sleep(Duration::from_millis(100));
                raise_sigint();
            });
            let received = tokio::time::timeout(Duration::from_secs(2), sigint.recv())
                .await
                .expect("timed out waiting for SIGINT on tokio runtime");
            assert!(received);
        });
    }
}
