//! Multiline prompt editor with agent-mode overlap label.

use elph_tui::Textarea;
use elph_tui::components::UiTheme;
use iocraft::prelude::*;

use crate::types::AgentMode;

use crate::tui::theme::rgb_color;
use crate::tui::theme::{EDITOR_CURSOR, EDITOR_TEXT_DIMMED, EDITOR_TEXT_FOCUSED};

fn editor_max_height(screen_height: u16) -> u16 {
    (screen_height / 4).clamp(4, 12)
}

#[derive(Default, Props)]
pub struct EditorProps {
    pub screen_width: u16,
    pub screen_height: u16,
    pub agent_mode: AgentMode,
    pub has_focus: bool,
    pub draft: Option<State<String>>,
    pub live_draft: Option<Ref<String>>,
    pub suppress_enter_newline: Option<Ref<bool>>,
    pub slash_palette_active: Option<Ref<bool>>,
    pub force_palette_sync: Option<Ref<bool>>,
    pub force_clear: Option<Ref<bool>>,
    pub on_submit: HandlerMut<'static, String>,
    pub on_escape: HandlerMut<'static, ()>,
}

#[component]
pub fn Editor(props: &mut EditorProps) -> impl Into<AnyElement<'static>> {
    let theme = UiTheme::default();
    let label_color = rgb_color(props.agent_mode.label_rgb());
    let has_focus = props.has_focus;
    let border_color = theme.shell_zone_border_color(has_focus);
    let inset = theme.shell_zone_padding();
    let inner_width = theme.shell_editor_inner_width(props.screen_width);
    let text_color = if has_focus {
        EDITOR_TEXT_FOCUSED
    } else {
        EDITOR_TEXT_DIMMED
    };

    element! {
        View(
            width: props.screen_width,
            flex_shrink: 0f32,
            border_style: BorderStyle::Round,
            border_color: border_color,
            position: Position::Relative,
            align_items: AlignItems::FlexStart,
            margin_bottom: 0,
            padding_top: 0,
            padding_bottom: 0,
            padding_left: inset,
            padding_right: inset,
        ) {
            Textarea(
                width: inner_width,
                min_height: 1u16,
                max_height: Some(editor_max_height(props.screen_height)),
                show_border: Some(false),
                has_focus: has_focus,
                value: props.draft,
                live_draft: props.live_draft,
                suppress_enter_newline: props.suppress_enter_newline,
                slash_palette_active: props.slash_palette_active,
                force_palette_sync: props.force_palette_sync,
                force_clear: props.force_clear,
                submit_on_enter: true,
                on_submit: props.on_submit.take(),
                on_escape: props.on_escape.take(),
                text_color: Some(text_color),
                cursor_color: Some(EDITOR_CURSOR),
            )
            View(
                position: Position::Absolute,
                right: 1,
                bottom: 0,
                margin_bottom: -1,
                background_color: Color::Reset,
            ) {
                Text(
                    color: label_color,
                    weight: Weight::Bold,
                    wrap: TextWrap::NoWrap,
                    content: format!(" {} ", props.agent_mode.footer_label()),
                )
            }
        }
    }
}
