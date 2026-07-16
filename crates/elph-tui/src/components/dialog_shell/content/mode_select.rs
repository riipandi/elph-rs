//! Agent mode selector dialog body.

use super::layout::dialog_body_section_gap;
use crate::color::rgb;
use crate::components::SelectList;
use crate::components::select::SELECT_LIST_AUTO_HEIGHT;
use crate::components::theme::{UiTheme, resolve_ui_theme};
use crate::types::{DialogAgentMode, SelectOption};
use iocraft::prelude::*;

/// Build select options from [`DialogAgentMode`] variants.
pub fn dialog_mode_select_options() -> Vec<SelectOption> {
    DialogAgentMode::all()
        .into_iter()
        .map(|mode| SelectOption::new(mode.label(), mode.description()))
        .collect()
}

/// Map a select index to [`DialogAgentMode`].
pub fn dialog_mode_from_index(index: usize) -> DialogAgentMode {
    DialogAgentMode::all().get(index).copied().unwrap_or_default()
}

/// Accent color for one mode row label.
pub fn dialog_mode_accent(mode: DialogAgentMode) -> Color {
    let (r, g, b) = mode.accent_rgb();
    rgb(r, g, b)
}

/// Props for [`DialogModeSelectContent`].
#[derive(Clone, Default, Props)]
pub struct DialogModeSelectContentProps {
    pub width: u16,
    pub height: u16,
    pub selected_index: Option<State<usize>>,
    pub has_focus: bool,
    pub intro: String,
    pub theme: Option<UiTheme>,
}

/// Mode picker body backed by [`SelectList`].
#[component]
pub fn DialogModeSelectContent(props: &DialogModeSelectContentProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let theme = resolve_ui_theme(&hooks, props.theme);
    let intro = if props.intro.is_empty() {
        "Choose how much autonomy the agent has for this session.".to_string()
    } else {
        props.intro.clone()
    };

    element! {
        View(
            width: props.width,
            flex_direction: FlexDirection::Column,
            gap: dialog_body_section_gap(theme),
            flex_shrink: 0f32,
        ) {
            Text(content: intro, color: theme.text_secondary, wrap: TextWrap::Wrap)
            SelectList(
                width: props.width,
                height: if props.height == 0 { SELECT_LIST_AUTO_HEIGHT } else { props.height },
                options: dialog_mode_select_options(),
                selected_index: props.selected_index,
                has_focus: props.has_focus,
                show_description: true,
                theme: Some(theme),
            )
        }
    }
}
