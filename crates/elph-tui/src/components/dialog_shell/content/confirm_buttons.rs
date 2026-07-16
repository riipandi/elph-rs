//! Confirmation dialog with Yes / No action buttons.

use super::layout::dialog_body_section_gap;
use crate::components::theme::{UiTheme, resolve_ui_theme};
use iocraft::prelude::*;

/// Which confirm button is focused.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ConfirmButtonFocus {
    #[default]
    Yes,
    No,
}

impl ConfirmButtonFocus {
    pub fn toggle(self) -> Self {
        match self {
            Self::Yes => Self::No,
            Self::No => Self::Yes,
        }
    }
}

/// Map a key press to confirm focus movement or answer (`None` = no change).
pub fn confirm_button_key_action(
    focus: ConfirmButtonFocus,
    code: KeyCode,
    modifiers: KeyModifiers,
) -> Option<ConfirmButtonAction> {
    if modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::ALT) {
        return None;
    }
    match code {
        KeyCode::Left | KeyCode::Right | KeyCode::Tab | KeyCode::BackTab => Some(ConfirmButtonAction::ToggleFocus),
        KeyCode::Char('y') | KeyCode::Char('Y') if modifiers.is_empty() => Some(ConfirmButtonAction::Answer(true)),
        KeyCode::Char('n') | KeyCode::Char('N') if modifiers.is_empty() => Some(ConfirmButtonAction::Answer(false)),
        KeyCode::Enter if modifiers.is_empty() => Some(ConfirmButtonAction::Answer(focus == ConfirmButtonFocus::Yes)),
        KeyCode::Esc if modifiers.is_empty() => Some(ConfirmButtonAction::Answer(false)),
        _ => None,
    }
}

/// Keyboard outcome for [`confirm_button_key_action`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConfirmButtonAction {
    ToggleFocus,
    Answer(bool),
}

/// Props for [`DialogConfirmButtonsContent`].
#[derive(Props)]
pub struct DialogConfirmButtonsContentProps {
    pub width: u16,
    pub message: String,
    pub yes_label: String,
    pub no_label: String,
    pub focused_button: Option<State<ConfirmButtonFocus>>,
    pub has_focus: bool,
    pub theme: Option<UiTheme>,
    pub on_yes: HandlerMut<'static, ()>,
    pub on_no: HandlerMut<'static, ()>,
}

impl Default for DialogConfirmButtonsContentProps {
    fn default() -> Self {
        Self {
            width: 40,
            message: String::new(),
            yes_label: "Yes".to_string(),
            no_label: "No".to_string(),
            focused_button: None,
            has_focus: true,
            theme: None,
            on_yes: HandlerMut::default(),
            on_no: HandlerMut::default(),
        }
    }
}

#[component]
pub fn DialogConfirmButtonsContent(
    props: &mut DialogConfirmButtonsContentProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = resolve_ui_theme(&hooks, props.theme);
    let internal_focus = hooks.use_state(ConfirmButtonFocus::default);
    let focus = props.focused_button.unwrap_or(internal_focus);
    let has_focus = props.has_focus;
    let yes_label = props.yes_label.clone();
    let no_label = props.no_label.clone();
    let width = props.width;

    hooks.use_terminal_events({
        let mut focus = focus;
        let mut on_yes = props.on_yes.take();
        let mut on_no = props.on_no.take();
        move |event| {
            if !has_focus {
                return;
            }
            let TerminalEvent::Key(KeyEvent {
                code, kind, modifiers, ..
            }) = event
            else {
                return;
            };
            if kind == KeyEventKind::Release {
                return;
            }
            match confirm_button_key_action(focus.get(), code, modifiers) {
                Some(ConfirmButtonAction::ToggleFocus) => focus.set(focus.get().toggle()),
                Some(ConfirmButtonAction::Answer(true)) => on_yes(()),
                Some(ConfirmButtonAction::Answer(false)) => on_no(()),
                None => {}
            }
        }
    });

    let yes_focused = focus.get() == ConfirmButtonFocus::Yes;

    let section_gap = dialog_body_section_gap(theme);

    element! {
        View(
            width: width,
            flex_direction: FlexDirection::Column,
            gap: section_gap,
            flex_shrink: 0f32,
        ) {
            Text(
                content: props.message.clone(),
                color: theme.text_secondary,
                wrap: TextWrap::Wrap,
            )
            View(
                width: width,
                flex_direction: FlexDirection::Row,
                gap: section_gap,
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
            ) {
                ConfirmActionButton(
                    label: yes_label,
                    active: yes_focused,
                    theme: theme,
                )
                ConfirmActionButton(
                    label: no_label,
                    active: !yes_focused,
                    theme: theme,
                )
            }
            Text(
                content: "←/→ focus · Enter/y yes · n/Esc no".to_string(),
                color: theme.text_muted,
                wrap: TextWrap::NoWrap,
            )
        }
    }
}

/// Compact action chip width from label length.
pub fn confirm_action_button_width(label: &str, theme: UiTheme) -> u16 {
    let chars = label.chars().count() as u16;
    chars
        .saturating_add(theme.padding_sm.saturating_mul(2))
        .saturating_add(2)
        .clamp(4, 16)
}

#[derive(Clone, Default, Props)]
struct ConfirmActionButtonProps {
    label: String,
    active: bool,
    theme: UiTheme,
}

#[component]
fn ConfirmActionButton(props: &ConfirmActionButtonProps) -> impl Into<AnyElement<'static>> {
    let border = if props.active {
        BorderStyle::Round
    } else {
        BorderStyle::Single
    };
    let border_color = if props.active {
        props.theme.border_focus
    } else {
        props.theme.border
    };
    let text_color = if props.active {
        props.theme.text_primary
    } else {
        props.theme.text_secondary
    };

    let width = confirm_action_button_width(&props.label, props.theme);

    element! {
        View(
            width: width,
            border_style: border,
            border_color: border_color,
            padding_left: props.theme.padding_sm,
            padding_right: props.theme.padding_sm,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_shrink: 0f32,
        ) {
            Text(content: props.label.clone(), color: text_color, wrap: TextWrap::NoWrap)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_button_toggle_focus_on_arrows() {
        assert_eq!(
            confirm_button_key_action(ConfirmButtonFocus::Yes, KeyCode::Right, KeyModifiers::empty()),
            Some(ConfirmButtonAction::ToggleFocus)
        );
    }

    #[test]
    fn confirm_button_enter_answers_focused_yes() {
        assert_eq!(
            confirm_button_key_action(ConfirmButtonFocus::Yes, KeyCode::Enter, KeyModifiers::empty()),
            Some(ConfirmButtonAction::Answer(true))
        );
        assert_eq!(
            confirm_button_key_action(ConfirmButtonFocus::No, KeyCode::Enter, KeyModifiers::empty()),
            Some(ConfirmButtonAction::Answer(false))
        );
    }

    #[test]
    fn confirm_button_esc_answers_no() {
        assert_eq!(
            confirm_button_key_action(ConfirmButtonFocus::Yes, KeyCode::Esc, KeyModifiers::empty()),
            Some(ConfirmButtonAction::Answer(false))
        );
    }
}
