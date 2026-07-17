//! Tool approval state and keyboard helpers.

use elph_tui::types::SelectOption;
use iocraft::prelude::{KeyCode, KeyModifiers};

use crate::agent::{ToolApprovalChoice, ToolApprovalRequest};
/// Number of selectable approval actions in the tool-permission dialog.
#[cfg_attr(not(test), allow(dead_code))]
pub const TOOL_APPROVAL_OPTION_COUNT: usize = 3;

/// Pending approval retained in shell state until the user responds.
pub struct PendingToolApproval {
    pub tool_name: String,
    pub args_summary: String,
    pub response_tx: tokio::sync::oneshot::Sender<ToolApprovalChoice>,
}

impl PendingToolApproval {
    pub fn from_request(req: ToolApprovalRequest) -> Self {
        Self {
            tool_name: req.tool_name,
            args_summary: req.args_summary,
            response_tx: req.response_tx,
        }
    }

    pub fn respond(self, choice: ToolApprovalChoice) {
        let _ = self.response_tx.send(choice);
    }
}

/// Footer hint for the tool-permission dialog (keyboard shortcuts live here, not on each row).
pub fn tool_approval_footer_hint() -> String {
    "↑↓ move · Enter confirm · y once · a session · n/Esc deny".to_string()
}

/// Select-list rows for the tool-permission dialog.
pub fn tool_approval_select_options() -> Vec<SelectOption> {
    [("Allow once", ""), ("Allow session", ""), ("Deny", "")]
        .into_iter()
        .map(|(name, detail)| SelectOption::new(name, detail))
        .collect()
}

/// Map y/a/n and digit keys to tool-approval list indices (0=allow once, 1=session, 2=deny).
pub fn pick_tool_approval_index_from_key(modifiers: KeyModifiers, code: KeyCode) -> Option<usize> {
    if !modifiers.is_empty() {
        return None;
    }
    match code {
        KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Char('1') => Some(0),
        KeyCode::Char('a') | KeyCode::Char('A') | KeyCode::Char('2') => Some(1),
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Char('3') => Some(2),
        _ => None,
    }
}

/// Map a zero-based list index to an approval choice.
pub fn choice_at_index(index: usize) -> Option<ToolApprovalChoice> {
    match index {
        0 => Some(ToolApprovalChoice::Approve),
        1 => Some(ToolApprovalChoice::AllowSession),
        2 => Some(ToolApprovalChoice::Reject),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choice_at_index_maps_three_actions() {
        assert_eq!(choice_at_index(0), Some(ToolApprovalChoice::Approve));
        assert_eq!(choice_at_index(1), Some(ToolApprovalChoice::AllowSession));
        assert_eq!(choice_at_index(2), Some(ToolApprovalChoice::Reject));
        assert_eq!(choice_at_index(3), None);
    }

    #[test]
    fn approval_keys_map_y_a_n_and_digits() {
        assert_eq!(
            pick_tool_approval_index_from_key(KeyModifiers::NONE, KeyCode::Char('y')),
            Some(0)
        );
        assert_eq!(
            pick_tool_approval_index_from_key(KeyModifiers::NONE, KeyCode::Char('a')),
            Some(1)
        );
        assert_eq!(
            pick_tool_approval_index_from_key(KeyModifiers::NONE, KeyCode::Char('n')),
            Some(2)
        );
        assert_eq!(
            pick_tool_approval_index_from_key(KeyModifiers::NONE, KeyCode::Char('2')),
            Some(1)
        );
    }

    #[test]
    fn select_options_are_label_only() {
        let options = tool_approval_select_options();
        assert_eq!(options.len(), TOOL_APPROVAL_OPTION_COUNT);
        assert_eq!(options[0].name, "Allow once");
        assert_eq!(options[1].name, "Allow session");
        assert_eq!(options[2].name, "Deny");
        assert!(options.iter().all(|opt| opt.description.is_empty()));
    }

    #[test]
    fn footer_hint_lists_shortcuts_once() {
        let hint = tool_approval_footer_hint();
        assert!(hint.contains("y once"));
        assert!(hint.contains("a session"));
        assert!(hint.contains("n/Esc deny"));
    }
}
