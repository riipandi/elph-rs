//! Tool-approval dialog below the status row (ask-user prompts use [`super::user_question_bar`] above).

use elph_tui::components::{SelectList, UiTheme, dialog_max_content_height, dialog_select_body_plan};
use iocraft::prelude::*;

use crate::tui::chrome::StatusRow;
use crate::tui::inline_dialog::{InlineDialogShell, inline_body_width};
use crate::tui::tool_approval::{PendingToolApproval, tool_approval_select_options};
use crate::tui::tool_params::{ToolParamsView, format_tool_params_display, parse_tool_params};

/// Tool-approval dialog shown below the status row.
#[derive(Debug, Clone)]
pub enum StatusDialogKind {
    ToolApproval { tool_name: String, args_summary: String },
}

/// Props for [`StatusZone`] — status row plus optional tool-approval dialog.
#[derive(Props)]
pub struct StatusZoneProps {
    pub screen_width: u16,
    pub screen_height: u16,
    pub busy: bool,
    pub activity_label: String,
    pub accent: Color,
    pub spinner_tick: u32,
    pub elapsed_secs: f64,
    pub idle_notice: Option<String>,
    pub busy_token_info: Option<String>,
    pub dialog: Option<StatusDialogKind>,
    pub approval_selected: Option<State<usize>>,
    pub approval_has_focus: bool,
}

impl Default for StatusZoneProps {
    fn default() -> Self {
        Self {
            screen_width: 80,
            screen_height: 24,
            busy: false,
            activity_label: String::new(),
            accent: Color::White,
            spinner_tick: 0,
            elapsed_secs: 0.0,
            idle_notice: None,
            busy_token_info: None,
            dialog: None,
            approval_selected: None,
            approval_has_focus: false,
        }
    }
}

fn tool_approval_list_height(screen_width: u16, screen_height: u16, args_text: &str, body_width: u16) -> u16 {
    let theme = UiTheme::default();
    let chrome = elph_tui::components::DialogChrome::from_theme(theme, screen_width);
    let max_body = dialog_max_content_height(screen_height, &chrome, 14);
    let options = tool_approval_select_options();
    let (_, list_h) = dialog_select_body_plan(&options, true, body_width, theme, args_text, 0, Some(max_body), true);
    list_h
}

fn render_tool_approval_dialog(
    props: &mut StatusZoneProps,
    tool_name: &str,
    args_summary: &str,
) -> AnyElement<'static> {
    let theme = UiTheme::default();
    let body_width = inline_body_width(props.screen_width);
    let list_height = tool_approval_list_height(
        props.screen_width,
        props.screen_height,
        &format_tool_params_display(args_summary),
        body_width,
    );
    let options = tool_approval_select_options();
    let has_args = !parse_tool_params(args_summary).is_empty();

    element! {
        InlineDialogShell(
            screen_width: props.screen_width,
            title: format!("Allow tool: {tool_name}"),
            has_focus: props.approval_has_focus,
        ) {
            View(
                width: body_width,
                flex_direction: FlexDirection::Column,
                gap: 0,
                flex_shrink: 0f32,
            ) {
                #(if has_args {
                    Some(element! {
                        View(width: body_width, padding_bottom: 1, flex_shrink: 0f32) {
                            ToolParamsView(
                                width: body_width,
                                raw: args_summary.to_string(),
                                key_color: theme.text_muted,
                                value_color: theme.text_secondary,
                            )
                        }
                    })
                } else {
                    None
                })
                SelectList(
                    width: body_width,
                    height: list_height,
                    options: options,
                    selected_index: props.approval_selected,
                    has_focus: props.approval_has_focus,
                    show_description: true,
                    compact: true,
                    theme: Some(theme),
                )
            }
        }
    }
    .into()
}

#[component]
pub fn StatusZone(props: &mut StatusZoneProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let _ = hooks;
    let tool_approval = props.dialog.as_ref().map(|kind| match kind {
        StatusDialogKind::ToolApproval {
            tool_name,
            args_summary,
        } => (tool_name.clone(), args_summary.clone()),
    });
    let dialog_element = tool_approval
        .as_ref()
        .map(|(tool_name, args_summary)| render_tool_approval_dialog(props, tool_name, args_summary));

    element! {
        View(
            width: props.screen_width,
            flex_shrink: 0f32,
            flex_direction: FlexDirection::Column,
        ) {
            StatusRow(
                screen_width: props.screen_width,
                busy: props.busy,
                activity_label: props.activity_label.clone(),
                accent: props.accent,
                spinner_tick: props.spinner_tick,
                elapsed_secs: props.elapsed_secs,
                idle_notice: props.idle_notice.clone(),
                busy_token_info: props.busy_token_info.clone(),
            )
            #(dialog_element)
        }
    }
}

/// Build the active tool-approval dialog, if any.
pub fn build_status_dialog_kind(tool: Option<&PendingToolApproval>) -> Option<StatusDialogKind> {
    let pending = tool?;
    Some(StatusDialogKind::ToolApproval {
        tool_name: pending.tool_name.clone(),
        args_summary: pending.args_summary.clone(),
    })
}
