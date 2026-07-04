#[cfg(unix)]
use super::app::SHOULD_KILL_PARENT;
use super::app::WAS_INTERRUPTED;
use iocraft::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// Suppresses the duplicate SIGINT delivery that follows a clear-from-non-empty prompt.
static LAST_INTERRUPT_CLEAR_MS: AtomicU64 = AtomicU64::new(0);

const INTERRUPT_COALESCE_MS: u64 = 250;

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// First Ctrl+C / SIGINT clears the prompt; second exits (when prompt is empty).
pub fn handle_prompt_interrupt(
    prompt: &mut State<String>,
    should_exit: &mut State<bool>,
    prompt_reset: &mut State<u32>,
) {
    if !prompt.read().is_empty() {
        prompt.set(String::new());
        prompt_reset.set(prompt_reset.get().wrapping_add(1));
        LAST_INTERRUPT_CLEAR_MS.store(now_ms(), Ordering::Relaxed);
        return;
    }

    let cleared_at = LAST_INTERRUPT_CLEAR_MS.load(Ordering::Relaxed);
    if cleared_at != 0 && now_ms().saturating_sub(cleared_at) < INTERRUPT_COALESCE_MS {
        return;
    }

    should_exit.set(true);
    WAS_INTERRUPTED.store(true, Ordering::Relaxed);
    #[cfg(unix)]
    SHOULD_KILL_PARENT.store(true, Ordering::Relaxed);
}

#[cfg(test)]
fn interrupt_coalesce_should_suppress_exit(cleared_at_ms: u64, now_ms: u64) -> bool {
    cleared_at_ms != 0 && now_ms.saturating_sub(cleared_at_ms) < INTERRUPT_COALESCE_MS
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coalesce_suppresses_exit_immediately_after_clear() {
        assert!(interrupt_coalesce_should_suppress_exit(1_000, 1_100));
        assert!(!interrupt_coalesce_should_suppress_exit(1_000, 1_300));
    }
}
