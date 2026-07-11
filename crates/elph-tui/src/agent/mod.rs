//! Agent-facing state types for tuie shell hosts.

mod collapse;
mod states;

pub use collapse::CollapseState;
pub use states::{
    AuthStatus, ModelSelectorAction, ModelSelectorState, OAuthSelectorAction, OAuthSelectorState,
    PlanConfirmationAction, PlanConfirmationChoice, PlanConfirmationState, SessionSelectorAction, SessionSelectorState,
    ToolApprovalAction, ToolApprovalChoice as TuiToolApprovalChoice, ToolApprovalState, TreeNavigatorAction,
    TreeNavigatorState, mock_oauth_providers,
};
