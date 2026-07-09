//! Compact live activity bar shown while the agent is running.

use elph_tui::{Theme, ToolExecutionState, ToolExecutionStatus};
use slt::{Color, Context, SpinnerState};

use super::tool_display::tool_chip_label;

pub struct ActivityBarState {
    pub spinner: SpinnerState,
}

impl Default for ActivityBarState {
    fn default() -> Self {
        Self {
            spinner: SpinnerState::dots(),
        }
    }
}

pub fn render_activity_bar(
    ui: &mut Context,
    state: &mut ActivityBarState,
    command: Option<&str>,
    live_tools: &[ToolExecutionState],
    theme: Theme,
) {
    let label = command
        .map(|name| format!("owly {name}"))
        .unwrap_or_else(|| "owly".to_string());

    let running_count = live_tools
        .iter()
        .filter(|tool| matches!(tool.status, ToolExecutionStatus::Running | ToolExecutionStatus::Pending))
        .count();

    let status_label = if running_count > 0 {
        format!("{label} · {running_count} tool(s)")
    } else {
        format!("{label} · working")
    };

    let _ = ui.row(|ui| {
        let _ = ui.spinner(&state.spinner);
        let _ = ui.text(status_label).fg(theme.muted);
        if !live_tools.is_empty() {
            let _ = ui.text("·").fg(theme.prompt_prefix);
            for tool in live_tools.iter().take(6) {
                let chip = tool_chip_label(tool, 24, 28);
                let _ = ui.text(chip).fg(tool_chip_color(tool.status, theme));
            }
        }
    });
}

fn tool_chip_color(status: ToolExecutionStatus, theme: Theme) -> Color {
    match status {
        ToolExecutionStatus::Running | ToolExecutionStatus::Pending => theme.muted,
        ToolExecutionStatus::Success => theme.prompt_prefix,
        ToolExecutionStatus::Error => Color::Red,
        ToolExecutionStatus::Cancelled => theme.muted,
    }
}
