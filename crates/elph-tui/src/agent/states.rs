use crate::diff::SelectItem;

/// OAuth/login flow status for display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthStatus {
    #[default]
    Idle,
    Waiting,
    Success,
    Error,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ModelSelectorState {
    pub selected: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelSelectorAction {
    None,
    Selected(SelectItem),
    Cancelled,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OAuthSelectorState {
    pub selected: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OAuthSelectorAction {
    None,
    Selected(SelectItem),
    Cancelled,
}

pub fn mock_oauth_providers() -> Vec<SelectItem> {
    vec![
        SelectItem::new("anthropic", "Anthropic").with_description("Claude models"),
        SelectItem::new("openai", "OpenAI").with_description("GPT models"),
        SelectItem::new("google", "Google").with_description("Gemini models"),
    ]
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanConfirmationChoice {
    StayInPlan,
    Implement,
    ImplementFresh,
}

impl PlanConfirmationChoice {
    pub fn label(self) -> &'static str {
        match self {
            Self::StayInPlan => "Stay in plan",
            Self::Implement => "Implement",
            Self::ImplementFresh => "Implement (fresh context)",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlanConfirmationState {
    pub plan_id: String,
    pub plan_text: String,
    pub selected: usize,
    pub visible: bool,
}

impl PlanConfirmationState {
    pub fn open(plan_id: String, plan_text: String) -> Self {
        Self {
            plan_id,
            plan_text,
            selected: 1,
            visible: true,
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }
}

pub enum PlanConfirmationAction {
    None,
    Resolved(PlanConfirmationChoice),
    Cancelled,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SessionSelectorState {
    pub selected: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionSelectorAction {
    None,
    Selected(SelectItem),
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolApprovalChoice {
    Approve,
    Reject,
    AllowSession,
}

impl ToolApprovalChoice {
    pub fn label(self) -> &'static str {
        match self {
            Self::Approve => "Approve once",
            Self::Reject => "Reject",
            Self::AllowSession => "Allow for session",
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ToolApprovalState {
    pub tool_call_id: String,
    pub tool_name: String,
    pub args_summary: String,
    pub selected: usize,
    pub visible: bool,
}

impl ToolApprovalState {
    pub fn open(tool_call_id: String, tool_name: String, args_summary: String) -> Self {
        Self {
            tool_call_id,
            tool_name,
            args_summary,
            selected: 0,
            visible: true,
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
    }
}

pub enum ToolApprovalAction {
    None,
    Resolved(ToolApprovalChoice),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TreeNavigatorState {
    pub selected: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TreeNavigatorAction {
    None,
    Selected(SelectItem),
    Cancelled,
}
