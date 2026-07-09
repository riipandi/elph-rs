use std::time::Duration;

use slt::RunConfig;

/// Default runtime settings for Elph/Owly agent shells.
///
/// Enables mouse scrolling, leaves Ctrl+C to app handlers, and uses a moderate
/// tick rate so activity spinners animate smoothly without busy-polling.
pub fn default_run_config() -> RunConfig {
    RunConfig::default()
        .mouse(true)
        .tick_rate(Duration::from_millis(50))
        .kitty_keyboard(true)
        .handle_ctrl_c(false)
}

/// Spinner preset for the activity line (`⡿ Label · N.Ns`).
pub fn default_activity_spinner() -> slt::widgets::SpinnerState {
    slt::widgets::SpinnerState::moon()
}
