//! Agent → TUI event bridge.

/// Live UI events emitted while an agent run is in progress.
#[derive(Debug)]
pub enum AgentUiEvent {
    Status(String),
    TextDelta(String),
    ThinkingDelta(String),
    ToolStart {
        id: String,
        name: String,
        args_summary: String,
    },
    ToolUpdate {
        id: String,
        output: String,
    },
    ToolEnd {
        id: String,
        is_error: bool,
        output: String,
    },
    RunCompleted {
        elapsed_secs: f64,
    },
    PlanConfirmationRequired(PlanConfirmationRequest),
    ToolApprovalRequired(ToolApprovalRequest),
    SubagentStatus {
        agent_id: String,
        agent_path: String,
        message: String,
    },
    GoalUpdated {
        objective: Option<String>,
        status: Option<String>,
    },
    UserQuestionRequired(UserQuestionRequest),
}

#[derive(Debug, Clone)]
pub struct PlanConfirmationRequest {
    pub plan_id: String,
    pub plan_text: String,
}

#[derive(Debug)]
pub struct ToolApprovalRequest {
    pub tool_call_id: String,
    pub tool_name: String,
    pub args_summary: String,
    pub response_tx: tokio::sync::oneshot::Sender<ToolApprovalChoice>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolApprovalChoice {
    Approve,
    Reject,
    AllowSession,
}

/// One step in a single- or multi-step ask-user flow.
#[derive(Debug, Clone)]
pub struct UserQuestionStep {
    pub id: String,
    pub question: String,
    pub options: Option<Vec<UserQuestionOption>>,
    pub allow_multiple: bool,
    pub allow_custom: bool,
    pub custom_label: String,
    pub default: Option<String>,
    /// When false, the user may skip this step with Esc (empty answer).
    pub required: bool,
    /// Minimum length for free-text answers (ignored for select / confirm steps).
    pub min_length: Option<usize>,
    /// Optional regex pattern for free-text answers.
    pub pattern: Option<String>,
    /// Short label shown in the multi-step header tab row.
    pub tab_label: Option<String>,
}

/// Ask-user session presented by the `ask_user_question` tool.
#[derive(Debug)]
pub struct UserQuestionRequest {
    pub steps: Vec<UserQuestionStep>,
    pub response_tx: tokio::sync::oneshot::Sender<String>,
}

#[derive(Debug, Clone, Default)]
pub struct UserQuestionOption {
    pub value: String,
    pub label: String,
    /// Optional dimmed detail shown below the label in the question dialog.
    pub hint: Option<String>,
}
