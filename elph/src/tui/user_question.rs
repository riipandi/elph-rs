//! Ask-user question state and keyboard helpers.

use std::collections::BTreeMap;

use elph_tui::components::ConfirmButtonFocus;
use elph_tui::components::multi_choice_selected_indices;
use elph_tui::types::SelectOption;
use iocraft::prelude::*;
use serde_json::Value;

use crate::agent::{UserQuestionOption, UserQuestionRequest, UserQuestionStep};
use crate::tui::focus::ShellFocus;

/// Pending multi-step ask-user session retained until the user finishes or cancels.
pub struct PendingUserQuestion {
    steps: Vec<UserQuestionStep>,
    step_index: usize,
    collected: BTreeMap<String, String>,
    response_tx: tokio::sync::oneshot::Sender<String>,
}

/// Which part of an ask-user step receives keyboard focus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum QuestionInputFocus {
    /// Numbered choices or confirm buttons.
    #[default]
    Choices,
    /// Inline custom text field (`allow_custom`) or free-text step.
    Custom,
}

impl QuestionInputFocus {
    pub fn is_choices(self) -> bool {
        matches!(self, Self::Choices)
    }

    pub fn is_custom(self) -> bool {
        matches!(self, Self::Custom)
    }
}

/// Default focus for a step.
pub fn default_input_focus(pending: &PendingUserQuestion) -> QuestionInputFocus {
    if pending.needs_text_input() || (pending.needs_custom_input() && pending.options().is_some_and(|o| o.is_empty())) {
        QuestionInputFocus::Custom
    } else {
        QuestionInputFocus::Choices
    }
}

/// Whether a key press is Tab forward or Shift+Tab backward.
pub fn question_tab_reverse(modifiers: KeyModifiers, code: KeyCode) -> Option<bool> {
    if modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::ALT) {
        return None;
    }
    match code {
        KeyCode::Tab if modifiers.is_empty() => Some(false),
        KeyCode::BackTab => Some(true),
        KeyCode::Tab if modifiers.contains(KeyModifiers::SHIFT) => Some(true),
        _ => None,
    }
}

/// Toggle between choice list and inline custom input (`Tab` / `Shift+Tab`).
pub fn cycle_question_input_focus(
    pending: &PendingUserQuestion,
    current: QuestionInputFocus,
) -> Option<QuestionInputFocus> {
    if pending.is_confirm() || pending.needs_text_input() || !pending.allow_custom() {
        return None;
    }
    Some(if current.is_choices() {
        QuestionInputFocus::Custom
    } else {
        QuestionInputFocus::Choices
    })
}

/// Outcome of answering the active step.
pub enum StepSubmitOutcome {
    /// More steps remain; caller should reset per-step UI state.
    Advanced(PendingUserQuestion),
    /// Session finished and the tool response was sent.
    Completed,
}

impl PendingUserQuestion {
    pub fn from_request(req: UserQuestionRequest) -> Self {
        Self {
            steps: req.steps,
            step_index: 0,
            collected: BTreeMap::new(),
            response_tx: req.response_tx,
        }
    }

    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    pub fn step_index(&self) -> usize {
        self.step_index
    }

    pub fn current_step(&self) -> &UserQuestionStep {
        &self.steps[self.step_index]
    }

    pub fn question(&self) -> &str {
        &self.current_step().question
    }

    pub fn options(&self) -> Option<&[UserQuestionOption]> {
        self.current_step().options.as_deref()
    }

    pub fn default(&self) -> Option<&String> {
        self.current_step().default.as_ref()
    }

    pub fn allow_multiple(&self) -> bool {
        self.current_step().allow_multiple
    }

    pub fn allow_custom(&self) -> bool {
        self.current_step().allow_custom
    }

    pub fn custom_label(&self) -> &str {
        &self.current_step().custom_label
    }

    pub fn is_confirm(&self) -> bool {
        self.current_step().options.is_none()
            && self
                .current_step()
                .default
                .as_ref()
                .is_some_and(|value| value == "true" || value == "false")
    }

    pub fn needs_text_input(&self) -> bool {
        !self.is_confirm() && self.current_step().options.is_none()
    }

    pub fn needs_custom_input(&self) -> bool {
        self.allow_custom() && self.current_step().options.is_some()
    }

    pub fn is_multi_select(&self) -> bool {
        self.allow_multiple() && self.current_step().options.is_some()
    }

    pub fn is_single_select(&self) -> bool {
        self.current_step().options.is_some() && !self.allow_multiple()
    }

    pub fn respond_confirm(self, yes: bool) -> StepSubmitOutcome {
        self.submit_step(yes.to_string())
    }

    pub fn respond_option(self, value: String) -> StepSubmitOutcome {
        self.submit_step(value)
    }

    pub fn respond(self, answer: String) -> StepSubmitOutcome {
        self.submit_step(answer)
    }

    /// Abort the whole session (e.g. Ctrl+C) with an empty tool result.
    pub fn cancel(self) {
        let _ = self.response_tx.send(String::new());
    }

    pub fn submit_step(mut self, answer: String) -> StepSubmitOutcome {
        let id = self.current_step().id.clone();
        self.collected.insert(id, answer);
        if self.step_index + 1 < self.steps.len() {
            self.step_index += 1;
            StepSubmitOutcome::Advanced(self)
        } else {
            let response = self.finalize_response();
            let _ = self.response_tx.send(response);
            StepSubmitOutcome::Completed
        }
    }

    pub fn default_select_index(&self) -> usize {
        default_select_index(self.options(), self.default())
    }

    pub fn default_multi_checked(&self) -> Vec<bool> {
        default_multi_checked(self.options(), self.default())
    }

    fn finalize_response(&self) -> String {
        if self.steps.len() == 1 {
            let step = &self.steps[0];
            let answer = self.collected.get(&step.id).cloned().unwrap_or_default();
            if !step.allow_multiple {
                return answer;
            }
        }
        serde_json::to_string(&self.collected).unwrap_or_default()
    }
}

/// Apply a step outcome to shell state (advance wizard or return focus to prompt).
#[allow(clippy::too_many_arguments)]
pub fn apply_step_submit_outcome(
    outcome: StepSubmitOutcome,
    pending_user_question: &mut Ref<Option<PendingUserQuestion>>,
    selected: &mut State<usize>,
    confirm_focus: &mut State<ConfirmButtonFocus>,
    answer: &mut State<String>,
    multi_checked: &mut State<Vec<bool>>,
    input_focus: &mut State<QuestionInputFocus>,
    shell_focus: &mut State<ShellFocus>,
    activity_label: &mut State<String>,
) {
    match outcome {
        StepSubmitOutcome::Completed => {
            shell_focus.set(ShellFocus::Prompt);
            activity_label.set("Thinking".to_string());
            answer.set(String::new());
            input_focus.set(QuestionInputFocus::Choices);
        }
        StepSubmitOutcome::Advanced(pending) => {
            reset_ui_for_step(&pending, selected, confirm_focus, answer, multi_checked, input_focus);
            activity_label.set(step_activity_label(&pending));
            pending_user_question.set(Some(pending));
        }
    }
}

/// Reset shell UI state when advancing to the next step.
pub fn reset_ui_for_step(
    pending: &PendingUserQuestion,
    selected: &mut State<usize>,
    confirm_focus: &mut State<ConfirmButtonFocus>,
    answer: &mut State<String>,
    multi_checked: &mut State<Vec<bool>>,
    input_focus: &mut State<QuestionInputFocus>,
) {
    selected.set(pending.default_select_index());
    confirm_focus.set(confirm_focus_from_default(pending.default()));
    answer.set(pending.default().cloned().unwrap_or_default());
    multi_checked.set(pending.default_multi_checked());
    input_focus.set(default_input_focus(pending));
}

/// Activity label while a step is active.
pub fn step_activity_label(pending: &PendingUserQuestion) -> String {
    if pending.step_count() > 1 {
        format!("Question {}/{}", pending.step_index() + 1, pending.step_count())
    } else {
        "Awaiting your answer".to_string()
    }
}

/// Format a multi-select answer as a JSON array string.
pub fn format_multi_select_answer(options: &[UserQuestionOption], indices: &[usize]) -> String {
    let values: Vec<&str> = indices
        .iter()
        .filter_map(|index| options.get(*index))
        .map(|option| option.value.as_str())
        .collect();
    serde_json::to_string(&values).unwrap_or_else(|_| "[]".to_string())
}

/// Resolve the answer when the user submits a text / custom input field.
pub fn resolve_user_text_submit_answer(
    pending: &PendingUserQuestion,
    text: &str,
    selected_index: usize,
    multi_checked: &[bool],
) -> Option<String> {
    if pending.needs_text_input() {
        return Some(if text.trim().is_empty() {
            pending.default().cloned().unwrap_or_default()
        } else {
            text.to_string()
        });
    }
    if pending.needs_custom_input() && pending.is_single_select() {
        return Some(resolve_select_or_custom(text, pending.options().unwrap_or(&[]), selected_index));
    }
    if pending.is_multi_select() && pending.allow_custom() {
        if !text.trim().is_empty() {
            return Some(text.trim().to_string());
        }
        let options = pending.options().unwrap_or(&[]);
        let indices = multi_choice_selected_indices(multi_checked);
        return Some(format_multi_select_answer(options, &indices));
    }
    None
}

/// Resolve custom-or-selected answer for single-select + inline input.
pub fn resolve_select_or_custom(custom_text: &str, options: &[UserQuestionOption], selected_index: usize) -> String {
    if !custom_text.trim().is_empty() {
        return custom_text.trim().to_string();
    }
    options
        .get(selected_index)
        .map(|option| option.value.clone())
        .unwrap_or_default()
}

/// Keyboard shortcut hint shown under each selectable row (numbered keys only).
pub fn option_shortcut_description(index: usize, value: &str, label: &str) -> String {
    let hint = format!("Press {}", index + 1);
    if !value.is_empty() && value != label {
        format!("{hint} · returns `{value}`")
    } else {
        hint
    }
}

/// Map agent options to [`SelectOption`] rows for dialog lists.
pub fn user_question_select_options(options: &[UserQuestionOption]) -> Vec<SelectOption> {
    options
        .iter()
        .enumerate()
        .map(|(index, option)| {
            SelectOption::new(
                option.label.clone(),
                option_shortcut_description(index, &option.value, &option.label),
            )
        })
        .collect()
}

/// Initial Yes/No focus for confirm dialogs from an optional default.
pub fn confirm_focus_from_default(default: Option<&String>) -> ConfirmButtonFocus {
    match default.map(String::as_str) {
        Some("false") => ConfirmButtonFocus::No,
        _ => ConfirmButtonFocus::Yes,
    }
}

/// Resolve the initial selection from an optional default value.
pub fn default_select_index(options: Option<&[UserQuestionOption]>, default: Option<&String>) -> usize {
    let Some(options) = options else {
        return 0;
    };
    let Some(default) = default else {
        return 0;
    };
    options
        .iter()
        .position(|option| option.value == *default || option.label == *default)
        .unwrap_or(0)
}

fn default_multi_checked(options: Option<&[UserQuestionOption]>, default: Option<&String>) -> Vec<bool> {
    let Some(options) = options else {
        return Vec::new();
    };
    let mut checked = vec![false; options.len()];
    let Some(default) = default else {
        return checked;
    };
    if let Ok(Value::Array(items)) = serde_json::from_str::<Value>(default) {
        for item in items {
            let token = match item {
                Value::String(text) => text,
                Value::Number(num) => num.to_string(),
                _ => continue,
            };
            if let Some(index) = options
                .iter()
                .position(|option| option.value == token || option.label == token)
            {
                checked[index] = true;
            }
        }
        return checked;
    }
    if let Some(index) = options
        .iter()
        .position(|option| option.value == *default || option.label == *default)
    {
        checked[index] = true;
    }
    checked
}

/// Selected option value for a zero-based index.
pub fn select_value_at(options: &[UserQuestionOption], index: usize) -> Option<String> {
    options.get(index).map(|option| option.value.clone())
}

/// Map digit keys (`1`–`9`) to a zero-based option index.
pub fn pick_numbered_option_index_from_key(
    modifiers: KeyModifiers,
    code: KeyCode,
    option_count: usize,
) -> Option<usize> {
    if option_count == 0 || !modifiers.is_empty() {
        return None;
    }
    match code {
        KeyCode::Char(c @ '1'..='9') => {
            let index = (c as u8 - b'0') as usize - 1;
            (index < option_count).then_some(index)
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::UserQuestionRequest;

    fn sample_request(steps: Vec<UserQuestionStep>) -> UserQuestionRequest {
        let (tx, _rx) = tokio::sync::oneshot::channel();
        UserQuestionRequest { steps, response_tx: tx }
    }

    #[test]
    fn multi_step_advances_then_completes() {
        let steps = vec![
            UserQuestionStep {
                id: "a".into(),
                question: "First?".into(),
                options: None,
                allow_multiple: false,
                allow_custom: false,
                custom_label: "Other…".into(),
                default: None,
            },
            UserQuestionStep {
                id: "b".into(),
                question: "Second?".into(),
                options: None,
                allow_multiple: false,
                allow_custom: false,
                custom_label: "Other…".into(),
                default: None,
            },
        ];
        let pending = PendingUserQuestion::from_request(sample_request(steps));
        let StepSubmitOutcome::Advanced(pending) = pending.submit_step("one".into()) else {
            panic!("expected advance");
        };
        assert_eq!(pending.step_index(), 1);
        assert!(matches!(pending.submit_step("two".into()), StepSubmitOutcome::Completed));
    }

    #[test]
    fn text_submit_uses_default_when_empty() {
        let pending = PendingUserQuestion::from_request(sample_request(vec![UserQuestionStep {
            id: "note".into(),
            question: "Note?".into(),
            options: None,
            allow_multiple: false,
            allow_custom: false,
            custom_label: "Other…".into(),
            default: Some("default".into()),
        }]));
        assert_eq!(resolve_user_text_submit_answer(&pending, "  ", 0, &[]), Some("default".into()));
        assert_eq!(resolve_user_text_submit_answer(&pending, "hello", 0, &[]), Some("hello".into()));
    }

    #[test]
    fn custom_input_wins_over_selection() {
        let options = vec![UserQuestionOption {
            value: "preset".into(),
            label: "Preset".into(),
        }];
        assert_eq!(resolve_select_or_custom("custom", &options, 0), "custom");
        assert_eq!(resolve_select_or_custom("  ", &options, 0), "preset");
    }

    #[test]
    fn single_step_returns_plain_string() {
        let steps = vec![UserQuestionStep {
            id: "answer".into(),
            question: "Name?".into(),
            options: None,
            allow_multiple: false,
            allow_custom: false,
            custom_label: "Other…".into(),
            default: None,
        }];
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        let pending = PendingUserQuestion::from_request(UserQuestionRequest { steps, response_tx: tx });
        assert!(matches!(pending.submit_step("Alice".into()), StepSubmitOutcome::Completed));
        assert_eq!(rx.try_recv().unwrap(), "Alice");
    }

    #[test]
    fn multi_step_returns_json_object() {
        let steps = vec![
            UserQuestionStep {
                id: "color".into(),
                question: "Color?".into(),
                options: Some(vec![UserQuestionOption {
                    value: "red".into(),
                    label: "Red".into(),
                }]),
                allow_multiple: false,
                allow_custom: true,
                custom_label: "Other…".into(),
                default: None,
            },
            UserQuestionStep {
                id: "tags".into(),
                question: "Tags?".into(),
                options: Some(vec![
                    UserQuestionOption {
                        value: "a".into(),
                        label: "A".into(),
                    },
                    UserQuestionOption {
                        value: "b".into(),
                        label: "B".into(),
                    },
                ]),
                allow_multiple: true,
                allow_custom: false,
                custom_label: "Other…".into(),
                default: None,
            },
        ];
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        let pending = PendingUserQuestion::from_request(UserQuestionRequest { steps, response_tx: tx });
        let StepSubmitOutcome::Advanced(pending) = pending.submit_step("crimson".into()) else {
            panic!("expected advance");
        };
        assert!(matches!(pending.submit_step(r#"["a"]"#.into()), StepSubmitOutcome::Completed));
        let json: serde_json::Value = serde_json::from_str(&rx.try_recv().unwrap()).unwrap();
        assert_eq!(json["color"], "crimson");
        assert_eq!(json["tags"], r#"["a"]"#);
    }

    #[test]
    fn format_multi_select_serializes_values() {
        let options = vec![
            UserQuestionOption {
                value: "x".into(),
                label: "X".into(),
            },
            UserQuestionOption {
                value: "y".into(),
                label: "Y".into(),
            },
        ];
        assert_eq!(format_multi_select_answer(&options, &[0, 1]), r#"["x","y"]"#);
    }

    #[test]
    fn tab_cycles_custom_focus_on_select_steps() {
        let pending = PendingUserQuestion::from_request(sample_request(vec![UserQuestionStep {
            id: "color".into(),
            question: "Color?".into(),
            options: Some(vec![UserQuestionOption {
                value: "red".into(),
                label: "Red".into(),
            }]),
            allow_multiple: false,
            allow_custom: true,
            custom_label: "Other…".into(),
            default: None,
        }]));
        assert_eq!(
            cycle_question_input_focus(&pending, QuestionInputFocus::Choices),
            Some(QuestionInputFocus::Custom)
        );
        assert_eq!(
            cycle_question_input_focus(&pending, QuestionInputFocus::Custom),
            Some(QuestionInputFocus::Choices)
        );
    }

    #[test]
    fn tab_ignored_on_text_and_confirm_steps() {
        let text = PendingUserQuestion::from_request(sample_request(vec![UserQuestionStep {
            id: "note".into(),
            question: "Note?".into(),
            options: None,
            allow_multiple: false,
            allow_custom: false,
            custom_label: "Other…".into(),
            default: None,
        }]));
        assert_eq!(cycle_question_input_focus(&text, QuestionInputFocus::Choices), None);

        let confirm = PendingUserQuestion::from_request(sample_request(vec![UserQuestionStep {
            id: "ok".into(),
            question: "Proceed?".into(),
            options: None,
            allow_multiple: false,
            allow_custom: false,
            custom_label: "Other…".into(),
            default: Some("true".into()),
        }]));
        assert_eq!(cycle_question_input_focus(&confirm, QuestionInputFocus::Choices), None);
    }

    #[test]
    fn question_tab_reverse_detects_tab_keys() {
        assert_eq!(question_tab_reverse(KeyModifiers::NONE, KeyCode::Tab), Some(false));
        assert_eq!(question_tab_reverse(KeyModifiers::SHIFT, KeyCode::Tab), Some(true));
        assert_eq!(question_tab_reverse(KeyModifiers::NONE, KeyCode::BackTab), Some(true));
        assert!(question_tab_reverse(KeyModifiers::CONTROL, KeyCode::Tab).is_none());
    }

    #[test]
    fn numbered_keys_map_to_option_index() {
        assert_eq!(
            pick_numbered_option_index_from_key(KeyModifiers::NONE, KeyCode::Char('1'), 3),
            Some(0)
        );
        assert_eq!(
            pick_numbered_option_index_from_key(KeyModifiers::NONE, KeyCode::Char('y'), 3),
            None
        );
    }
}
