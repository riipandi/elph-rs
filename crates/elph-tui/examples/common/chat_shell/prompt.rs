//! Editor + footer column (bottom chrome).

use super::types::ThinkingLevel;
use elph_tui::prelude::*;
use elph_tui::slash_palette::{PaletteSnapshot, palette_anchor_bottom};

use crate::common::palette_ui::SlashCommandPalette;

fn editor_max_height(screen_height: u16) -> u16 {
    (screen_height / 4).clamp(4, 12)
}

fn mode_label_color(mode: DialogAgentMode) -> Color {
    let (r, g, b) = mode.accent_rgb();
    rgb(r, g, b)
}

#[derive(Default, Props)]
struct EditorProps {
    screen_width: u16,
    screen_height: u16,
    agent_mode: DialogAgentMode,
    has_focus: bool,
    draft: Option<State<String>>,
    live_draft: Option<Ref<String>>,
    suppress_enter_newline: Option<Ref<bool>>,
    slash_palette_active: Option<Ref<bool>>,
    force_palette_sync: Option<Ref<bool>>,
    palette_visible: bool,
    on_submit: HandlerMut<'static, String>,
    on_escape: HandlerMut<'static, ()>,
}

#[component]
fn Editor(props: &mut EditorProps) -> impl Into<AnyElement<'static>> {
    let theme = UiTheme::default();
    let label_color = mode_label_color(props.agent_mode);
    let has_focus = props.has_focus;
    let border_color = theme.input_border_color(has_focus);
    let inset = theme.shell_zone_padding();
    let inner_width = theme.shell_editor_inner_width(props.screen_width);

    element! {
        View(
            width: props.screen_width,
            flex_shrink: 0f32,
            border_style: BorderStyle::Round,
            border_color: border_color,
            position: Position::Relative,
            align_items: AlignItems::FlexStart,
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
                submit_on_enter: true,
                on_submit: props.on_submit.take(),
                on_escape: if props.palette_visible {
                    HandlerMut::default()
                } else {
                    props.on_escape.take()
                },
                theme: Some(theme),
            )
            View(
                position: Position::Absolute,
                right: inset,
                bottom: 0,
                margin_bottom: -1,
                background_color: Color::Reset,
            ) {
                Text(
                    color: label_color,
                    weight: Weight::Bold,
                    wrap: TextWrap::NoWrap,
                    content: format!(" {} ", props.agent_mode.label()),
                )
            }
        }
    }
}

#[derive(Clone, Default, Props)]
struct FooterLeftProps {
    width: u16,
    project_label: String,
}

#[component]
fn FooterLeft(props: &FooterLeftProps) -> impl Into<AnyElement<'static>> {
    let theme = UiTheme::default();
    element! {
        View(width: props.width, align_items: AlignItems::Center, justify_content: JustifyContent::Start) {
            Text(color: theme.text_hint, wrap: TextWrap::NoWrap, content: props.project_label.clone())
        }
    }
}

#[derive(Clone, Default, Props)]
struct FooterRightProps {
    width: u16,
    model_label: String,
    thinking_level: ThinkingLevel,
    supports_images: bool,
}

#[component]
fn FooterRight(props: &FooterRightProps) -> impl Into<AnyElement<'static>> {
    let theme = UiTheme::default();
    let footer_right = if props.supports_images {
        format!("IMG | {} | {}", props.model_label, props.thinking_level.label())
    } else {
        format!("{} | {}", props.model_label, props.thinking_level.label())
    };

    element! {
        View(width: props.width, align_items: AlignItems::Center, justify_content: JustifyContent::End) {
            Text(color: theme.text_hint, wrap: TextWrap::NoWrap, content: footer_right)
        }
    }
}

#[derive(Clone, Default, Props)]
struct FooterProps {
    screen_width: u16,
    project_label: String,
    model_label: String,
    thinking_level: ThinkingLevel,
    supports_images: bool,
}

#[component]
fn Footer(props: &FooterProps) -> impl Into<AnyElement<'static>> {
    let theme = UiTheme::default();
    let half = props.screen_width / 2;

    element! {
        View(
            width: props.screen_width,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding_left: theme.shell_zone_padding(),
            padding_right: theme.shell_zone_padding(),
        ) {
            FooterLeft(width: half, project_label: props.project_label.clone())
            FooterRight(
                width: half,
                model_label: props.model_label.clone(),
                thinking_level: props.thinking_level,
                supports_images: props.supports_images,
            )
        }
    }
}

#[derive(Default, Props)]
pub struct PromptChromeProps {
    pub screen_width: u16,
    pub screen_height: u16,
    pub agent_mode: DialogAgentMode,
    pub thinking_level: ThinkingLevel,
    pub has_focus: bool,
    pub project_label: String,
    pub model_label: String,
    pub supports_images: bool,
    pub draft: Option<State<String>>,
    pub live_draft: Option<Ref<String>>,
    pub suppress_enter_newline: Option<Ref<bool>>,
    pub slash_palette_active: Option<Ref<bool>>,
    pub force_palette_sync: Option<Ref<bool>>,
    pub slash_palette_snapshot: PaletteSnapshot,
    pub slash_palette_selected: Option<State<usize>>,
    pub on_submit: HandlerMut<'static, String>,
    pub on_escape: HandlerMut<'static, ()>,
}

#[component]
pub fn PromptChrome(props: &mut PromptChromeProps) -> impl Into<AnyElement<'static>> {
    let draft_text = props
        .live_draft
        .as_ref()
        .map(|live| live.read().clone())
        .or_else(|| props.draft.as_ref().map(|draft| draft.read().clone()))
        .unwrap_or_default();
    let palette_anchor = palette_anchor_bottom(&draft_text, props.screen_width, props.screen_height);
    let palette_visible = props.slash_palette_snapshot.visible;

    element! {
        View(
            width: props.screen_width,
            flex_shrink: 0f32,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::FlexStart,
        ) {
            View(
                width: props.screen_width,
                flex_shrink: 0f32,
                position: Position::Relative,
                align_items: AlignItems::FlexStart,
            ) {
                Editor(
                    screen_width: props.screen_width,
                    screen_height: props.screen_height,
                    agent_mode: props.agent_mode,
                    has_focus: props.has_focus,
                    draft: props.draft,
                    live_draft: props.live_draft,
                    suppress_enter_newline: props.suppress_enter_newline,
                    slash_palette_active: props.slash_palette_active,
                    force_palette_sync: props.force_palette_sync,
                    palette_visible: palette_visible,
                    on_submit: props.on_submit.take(),
                    on_escape: props.on_escape.take(),
                )
                SlashCommandPalette(
                    screen_width: props.screen_width,
                    screen_height: props.screen_height,
                    snapshot: props.slash_palette_snapshot.clone(),
                    anchor_bottom: palette_anchor,
                    selected_index: props.slash_palette_selected,
                )
            }
            Footer(
                screen_width: props.screen_width,
                project_label: props.project_label.clone(),
                model_label: props.model_label.clone(),
                thinking_level: props.thinking_level,
                supports_images: props.supports_images,
            )
        }
    }
}
