//! Preset dialog body components.

mod confirm;
mod confirm_buttons;
mod layout;
mod mode_select;
mod multi_choice;
mod question;
mod todo_list;
mod todo_progress;
mod user_input;

pub use confirm::{DialogConfirmContent, DialogConfirmContentProps};
pub use confirm_buttons::{
    ConfirmButtonAction, ConfirmButtonFocus, DialogConfirmButtonsContent, DialogConfirmButtonsContentProps,
    confirm_button_key_action,
};
pub use mode_select::{
    DialogModeSelectContent, DialogModeSelectContentProps, dialog_mode_accent, dialog_mode_from_index,
    dialog_mode_select_options,
};
pub use multi_choice::{
    DialogMultiChoiceContent, DialogMultiChoiceContentProps, MultiChoiceAction, multi_choice_key_action,
    multi_choice_selected_indices, multi_choice_toggle,
};
pub use question::{DialogQuestionContent, DialogQuestionContentProps};
pub use todo_list::{DialogTodoListContent, DialogTodoListContentProps, todo_row_line, todo_row_prefix};
pub use todo_progress::{DialogTodoProgressContent, DialogTodoProgressContentProps, progress_row_glyph};
pub use user_input::{DialogUserInputContent, DialogUserInputContentProps};
