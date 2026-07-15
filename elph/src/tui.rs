//! iocraft-based TUI for Elph.
//!
//! Layout based on `crates/elph-tui/examples/chat_layout.rs`.
//! Zones (top → bottom): Header, Transcript, status row, Editor, Footer.
//! Color reference: https://www.ditig.com/256-colors-cheat-sheet

use anyhow::Result;
use chrono::Local;
use iocraft::prelude::*;
use std::time::Duration;

use crate::agent::agent_mode_from_setting;
use crate::platform::{Paths, Settings};
use crate::types::{AgentMode, ThinkingLevel};

const LOREM_IPSUM: &str = "Lorem ipsum odor amet, consectetuer adipiscing elit. \
Lobortis hendrerit nec ipsum dapibus quam. Donec malesuada tincidunt elementum \
mollis vehicula quisque purus. Est volutpat integer, donec sagittis placerat \
fermentum phasellus ipsum sollicitudin. Tempus laoreet ad tempus aptent proin \
per donec lectus. Quisque auctor urna; phasellus urna tortor ligula. Class \
pharetra bibendum tristique, quisque consectetur placerat potenti. Imperdiet ut \
torquent vestibulum eleifend bibendum et. Dictumst vulputate interdum iaculis \
at conubia venenatis.";

fn session_label(resume_id: Option<&str>) -> String {
    let id = resume_id.unwrap_or("019f631516e6g29o");
    format!("Session: {id} | turn: 0")
}

fn rgb_color((r, g, b): (u8, u8, u8)) -> Color {
    Color::Rgb { r, g, b }
}

fn project_footer_label(paths: &Paths) -> String {
    let name = paths.project_dir().file_name().and_then(|s| s.to_str()).unwrap_or("?");
    format!("~ {name} [branch-name]")
}

fn model_footer_label(provider_id: Option<&str>, model_id: Option<&str>) -> String {
    match (provider_id, model_id) {
        (Some(provider), Some(model)) => format!("{provider}/{model}"),
        (None, Some(model)) => model.to_string(),
        _ => "no model selected".to_string(),
    }
}

fn persist_session_prefs(paths: &Paths, mode: AgentMode, thinking: ThinkingLevel) {
    let Ok(mut settings) = Settings::load(paths) else {
        return;
    };
    settings.session.agent_mode = mode.footer_label().to_string();
    settings.session.thinking_level = thinking.label().to_string();
    if let Err(err) = Settings::save(paths, &settings) {
        log::warn!("failed to save session preferences: {err}");
    }
}

#[derive(Default, Props)]
struct MainShellProps {
    resume_id: Option<String>,
    initial_agent_mode: AgentMode,
    initial_thinking_level: ThinkingLevel,
    model_label: String,
    project_label: String,
}

#[derive(Default, Props)]
struct HeaderProps {
    screen_width: u16,
    session_label: String,
}

#[component]
fn Header(props: &HeaderProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            width: props.screen_width,
            flex_shrink: 0f32,
            background_color: Color::Reset,
            border_style: BorderStyle::Single,
            border_edges: Edges::Top,
            border_color: Color::Rgb { r: (88), g: (88), b: (88) },
            position: Position::Relative,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding_left: 1,
            padding_right: 1,
            margin_bottom: 0,
        ) {
            Text(color: Color::DarkGrey, wrap: TextWrap::NoWrap, content: props.session_label.clone())
            Text(color: Color::DarkGrey, wrap: TextWrap::NoWrap, content: "$0.00 | 0k | 0.0% (262k)")
        }
    }
}

#[derive(Default, Props)]
struct TranscriptPanelProps {
    screen_width: u16,
    screen_height: u16,
}

#[component]
fn TranscriptPanel(props: &TranscriptPanelProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            width: props.screen_width,
            flex_grow: 1f32,
            flex_shrink: 1f32,
            min_height: 0,
            overflow: Overflow::Hidden,
            border_style: BorderStyle::Single,
            border_edges: Edges::Top,
            border_color: Color::Rgb { r: (88), g: (88), b: (88) },
            margin_bottom: 1,
        ) {
            View(
                width: 100pct,
                height: 100pct,
                overflow: Overflow::Hidden,
            ) {
                ScrollView(
                    scroll_step: 2,
                    scrollbar: true,
                    scrollbar_thumb_color: Color::Rgb { r: (88), g: (88), b: (88) },
                    scrollbar_track_color: Color::Rgb { r: (48), g: (48), b: (48) },
                    keyboard_scroll: true,
                    auto_scroll: true,
                ) {
                    View(
                        width: props.screen_width,
                        height: props.screen_height - 3,
                        background_color: Color::Reset,
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::End,
                        align_items: AlignItems::Baseline,
                        padding_top: 0,
                        padding_bottom: 0,
                        padding_left: 1,
                        padding_right: 1,
                        gap: 1,
                    ) {
                        View(
                            width: props.screen_width - 3,
                            background_color: Color::Rgb { r: (48), g: (48), b: (48) },
                            margin_bottom: 0,
                            padding: 1,
                        ) {
                            Text(color: Color::DarkGrey, content: LOREM_IPSUM)
                        }
                        View(
                            width: props.screen_width - 3,
                            background_color: Color::Rgb { r: (48), g: (48), b: (48) },
                            margin_bottom: 0,
                            padding: 1,
                        ) {
                            Text(color: Color::White, content: LOREM_IPSUM)
                        }
                        View(
                            width: props.screen_width - 3,
                            background_color: Color::Rgb { r: (48), g: (48), b: (48) },
                            margin_bottom: 0,
                            padding: 1,
                        ) {
                            Text(color: Color::DarkGreen, content: LOREM_IPSUM)
                        }
                        View(
                            width: props.screen_width - 3,
                            background_color: Color::Rgb { r: (48), g: (48), b: (48) },
                            margin_bottom: 0,
                            padding: 1,
                        ) {
                            Text(color: Color::DarkRed, content: LOREM_IPSUM)
                        }
                        View(
                            width: props.screen_width - 3,
                            background_color: Color::Reset,
                            margin_bottom: 0,
                            padding: 0,
                        ) {
                            Text(color: Color::DarkGrey, content: LOREM_IPSUM)
                        }
                        View(
                            width: props.screen_width - 3,
                            background_color: Color::Reset,
                            margin_bottom: 0,
                            padding: 0,
                        ) {
                            Text(color: Color::White, content: LOREM_IPSUM)
                        }
                        View(
                            width: props.screen_width - 3,
                            background_color: Color::Rgb { r: (0), g: (95), b: (175) },
                            margin_bottom: 0,
                            padding: 1,
                        ) {
                            Text(color: Color::White, content: "read_file : /U/a/b/c/d/project-dir/examples/chat_layout.rs")
                        }
                    }
                }
            }
        }
    }
}

#[derive(Default, Props)]
struct StatusRowProps {
    screen_width: u16,
    time_label: String,
}

#[component]
fn StatusRow(props: &StatusRowProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            width: props.screen_width,
            flex_shrink: 0f32,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding_left: 1,
            padding_right: 1,
        ) {
            View(
                width: props.screen_width / 2,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Start,
                padding: 0,
            ) {
                Text(color: Color::DarkGrey, wrap: TextWrap::NoWrap, content: props.time_label.clone())
            }
            View(
                width: props.screen_width / 2,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::End,
                padding: 0,
            ) {
                Text(color: Color::DarkGrey, wrap: TextWrap::NoWrap, content: "Press Ctrl+D to quit.")
            }
        }
    }
}

#[derive(Clone, Copy, Default, Props)]
struct EditorProps {
    screen_width: u16,
    agent_mode: AgentMode,
}

#[component]
fn Editor(props: &EditorProps) -> impl Into<AnyElement<'static>> {
    let label_color = rgb_color(props.agent_mode.label_rgb());

    element! {
        View(
            width: props.screen_width,
            min_height: 3,
            border_style: BorderStyle::Round,
            border_color: Color::Rgb { r: (108), g: (108), b: (108) },
            position: Position::Relative,
            align_items: AlignItems::Baseline,
            margin_bottom: 0,
            padding_top: 0,
            padding_bottom: 0,
            padding_left: 1,
            padding_right: 1,
        ) {
            TextInput(
                has_focus: true,
                multiline: true,
                color: Color::Grey,
                cursor_color: Color::DarkGrey,
                value: "Ask anything... \"Fix broken tests\"",
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

#[derive(Clone, Default, Props)]
struct FooterLeftProps {
    width: u16,
    project_label: String,
}

#[component]
fn FooterLeft(props: &FooterLeftProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            width: props.width,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Start,
            padding: 0,
        ) {
            Text(color: Color::DarkGrey, wrap: TextWrap::NoWrap, content: props.project_label.clone())
        }
    }
}

#[derive(Clone, Default, Props)]
struct FooterRightProps {
    width: u16,
    model_label: String,
    thinking_level: ThinkingLevel,
}

#[component]
fn FooterRight(props: &FooterRightProps) -> impl Into<AnyElement<'static>> {
    let footer_right = format!("IMG | {} | {}", props.model_label, props.thinking_level.label());

    element! {
        View(
            width: props.width,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::End,
            padding: 0,
        ) {
            Text(color: Color::DarkGrey, wrap: TextWrap::NoWrap, content: footer_right)
        }
    }
}

#[derive(Clone, Default, Props)]
struct FooterProps {
    screen_width: u16,
    project_label: String,
    model_label: String,
    thinking_level: ThinkingLevel,
}

#[component]
fn Footer(props: &FooterProps) -> impl Into<AnyElement<'static>> {
    let half = props.screen_width / 2;

    element! {
        View(
            width: props.screen_width,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding_left: 1,
            padding_right: 1,
        ) {
            FooterLeft(width: half, project_label: props.project_label.clone())
            FooterRight(
                width: half,
                model_label: props.model_label.clone(),
                thinking_level: props.thinking_level,
            )
        }
    }
}

#[derive(Clone, Default, Props)]
struct PromptChromeProps {
    screen_width: u16,
    agent_mode: AgentMode,
    thinking_level: ThinkingLevel,
    project_label: String,
    model_label: String,
}

#[component]
fn PromptChrome(props: &PromptChromeProps) -> impl Into<AnyElement<'static>> {
    element! {
        View(
            width: props.screen_width,
            flex_shrink: 0f32,
            border_style: BorderStyle::None,
            align_items: AlignItems::Baseline,
            flex_direction: FlexDirection::Column,
            margin_bottom: 0,
            padding_top: 0,
            padding_bottom: 0,
            padding_left: 0,
            padding_right: 0,
        ) {
            Editor(
                screen_width: props.screen_width,
                agent_mode: props.agent_mode,
            )
            Footer(
                screen_width: props.screen_width,
                project_label: props.project_label.clone(),
                model_label: props.model_label.clone(),
                thinking_level: props.thinking_level,
            )
        }
    }
}

#[component]
fn MainShell(props: &MainShellProps, mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let (screen_width, screen_height) = hooks.use_terminal_size();
    let mut system = hooks.use_context_mut::<SystemContext>();
    let mut time = hooks.use_state(Local::now);
    let mut should_exit = hooks.use_state(|| false);
    let mut agent_mode = hooks.use_state(|| props.initial_agent_mode);
    let mut thinking_level = hooks.use_state(|| props.initial_thinking_level);
    let session_label = session_label(props.resume_id.as_deref());
    let paths = hooks.use_state(|| Paths::resolve().expect("resolve elph paths"));

    hooks.use_future(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;
            time.set(Local::now());
        }
    });

    hooks.use_terminal_events({
        let paths = paths.read().clone();
        move |event| {
            let TerminalEvent::Key(KeyEvent {
                code, kind, modifiers, ..
            }) = event
            else {
                return;
            };
            if kind == KeyEventKind::Release {
                return;
            }

            match (modifiers, code) {
                (m, KeyCode::Char('a')) if m.contains(KeyModifiers::CONTROL) => {
                    let next = agent_mode.get().next();
                    agent_mode.set(next);
                    persist_session_prefs(&paths, next, thinking_level.get());
                }
                (m, KeyCode::BackTab) if m.contains(KeyModifiers::SHIFT) => {
                    let next = thinking_level.get().next();
                    thinking_level.set(next);
                    persist_session_prefs(&paths, agent_mode.get(), next);
                }
                (m, KeyCode::Char('d')) if m.contains(KeyModifiers::CONTROL) => should_exit.set(true),
                _ => {}
            }
        }
    });

    if should_exit.get() {
        system.exit();
    }

    let time_label = format!("Current Time: {}", time.get().format("%r"));

    element! {
        View(
            width: screen_width,
            height: screen_height,
            background_color: Color::Reset,
            border_style: BorderStyle::None,
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            margin: 0,
            padding: 0,
        ) {
            Header(
                screen_width: screen_width,
                session_label: session_label,
            )
            TranscriptPanel(
                screen_width: screen_width,
                screen_height: screen_height,
            )
            StatusRow(
                screen_width: screen_width,
                time_label: time_label,
            )
            PromptChrome(
                screen_width: screen_width,
                agent_mode: agent_mode.get(),
                thinking_level: thinking_level.get(),
                project_label: props.project_label.clone(),
                model_label: props.model_label.clone(),
            )
        }
    }
}

/// Launch the Elph TUI.
pub async fn run_tui(resume_id: Option<String>) -> Result<()> {
    let paths = Paths::resolve()?;
    Settings::ensure(&paths)?;
    let settings = Settings::load(&paths)?;

    element!(MainShell(
        resume_id: resume_id,
        initial_agent_mode: agent_mode_from_setting(&settings.session.agent_mode),
        initial_thinking_level: ThinkingLevel::from_setting(&settings.session.thinking_level),
        model_label: model_footer_label(
            settings.session.provider_id.as_deref(),
            settings.session.model_id.as_deref(),
        ),
        project_label: project_footer_label(&paths),
    ))
    .render_loop()
    .fullscreen()
    .enable_mouse_capture()
    .ignore_ctrl_c()
    .await?;
    Ok(())
}
