use std::borrow::Cow;
use std::io::{IsTerminal, stderr};
use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

const INIT_STEPS: u64 = 4;

pub struct InitProgress {
    bar: ProgressBar,
}

impl InitProgress {
    pub fn new() -> Self {
        let bar = if Self::enabled() {
            let bar = ProgressBar::new(INIT_STEPS);
            bar.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} {wide_msg:.cyan} [{bar:24.cyan/blue}] {pos}/{len}")
                    .expect("valid init progress template")
                    .progress_chars("━━╸─"),
            );
            bar.enable_steady_tick(Duration::from_millis(80));
            bar
        } else {
            ProgressBar::hidden()
        };

        Self { bar }
    }

    pub fn advance(&self, message: impl Into<Cow<'static, str>>) {
        self.bar.inc(1);
        self.bar.set_message(message);
    }

    pub fn finish(&self) {
        self.bar.finish_and_clear();
    }

    fn enabled() -> bool {
        if cfg!(test) {
            return false;
        }
        if std::env::var_os("ELPH_QUIET").is_some() {
            return false;
        }
        if std::env::var("NO_COLOR").as_deref() == Ok("true") {
            return false;
        }
        stderr().is_terminal()
    }
}
