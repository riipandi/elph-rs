use elph_tui::{PlanConfirmationState, ToolApprovalState};

use crate::agent::AgentUiEvent;
use crate::shell::ElphApp;
use crate::tui::TranscriptApplier;

impl ElphApp {
    pub(super) fn poll_ui_events(&mut self) {
        while let Ok(event) = self.ui_rx.try_recv() {
            match event {
                AgentUiEvent::PlanConfirmationRequired(req) => {
                    self.plan_modal = PlanConfirmationState::open(req.plan_id, req.plan_text);
                }
                AgentUiEvent::ToolApprovalRequired(req) => {
                    self.tool_modal = ToolApprovalState::open(req.tool_call_id, req.tool_name, req.args_summary);
                    self.pending_tool_approval_tx = Some(req.response_tx);
                }
                AgentUiEvent::RunCompleted { elapsed_secs } => {
                    let mut applier =
                        TranscriptApplier::new(&mut self.chat.entries, &mut self.live_tools, self.show_thinking);
                    applier.apply(AgentUiEvent::RunCompleted { elapsed_secs });
                    self.agent_running = false;
                    self.last_turn_elapsed_secs = elapsed_secs;
                    self.total_api_secs += elapsed_secs;
                    self.activity.clear();
                    self.drain_prompt_queue();
                }
                other => {
                    let mut applier =
                        TranscriptApplier::new(&mut self.chat.entries, &mut self.live_tools, self.show_thinking);
                    applier.apply(other);
                }
            }
        }
    }
}
