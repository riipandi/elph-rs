//! Modal dialog for viewing the compiled system prompt.

use elph_tui::components::{
    DialogChrome, DialogHeader, DialogShellOverlay, ScrollBox, dialog_body_min_height, dialog_max_content_height,
};
use iocraft::prelude::*;

use crate::tui::focus::ShellFocus;

#[derive(Debug, Clone)]
pub struct PendingSystemPromptDialog {
    pub text: String,
    pub stashed_prompt_draft: Option<String>,
}

impl PendingSystemPromptDialog {
    pub fn open(text: String, stashed_prompt_draft: Option<String>) -> Self {
        Self {
            text,
            stashed_prompt_draft,
        }
    }
}

/// Arguments for [`open_system_prompt_dialog`].
pub struct OpenSystemPromptDialogArgs<'a> {
    pub pending: &'a mut Ref<Option<PendingSystemPromptDialog>>,
    pub draft: &'a mut State<String>,
    pub live_draft: &'a mut Ref<String>,
    pub shell_focus: &'a mut State<ShellFocus>,
    pub text: String,
}

pub fn open_system_prompt_dialog(args: OpenSystemPromptDialogArgs<'_>) {
    let stashed = {
        let current = args.live_draft.read().clone();
        if current.trim().is_empty() { None } else { Some(current) }
    };
    if stashed.is_some() {
        args.draft.set(String::new());
        args.live_draft.set(String::new());
    }
    args.pending
        .set(Some(PendingSystemPromptDialog::open(args.text, stashed)));
    args.shell_focus.set(ShellFocus::StatusDialog);
}

pub fn close_system_prompt_dialog(
    pending: &mut Ref<Option<PendingSystemPromptDialog>>,
    draft: &mut State<String>,
    live_draft: &mut Ref<String>,
    shell_focus: &mut State<ShellFocus>,
) {
    if let Some(mut dialog) = pending.write().take()
        && let Some(stashed) = dialog.stashed_prompt_draft.take()
    {
        draft.set(stashed.clone());
        live_draft.set(stashed);
    }
    shell_focus.set(ShellFocus::Prompt);
}

pub fn system_prompt_dialog_chrome(screen_width: u16, screen_height: u16) -> (DialogChrome, u16) {
    let outer = screen_width.clamp(56, 80);
    let chrome = DialogChrome {
        width: outer,
        ..DialogChrome::default()
    };
    let max_body = dialog_max_content_height(screen_height, &chrome, 8);
    let body_height = dialog_body_min_height(max_body);
    (
        DialogChrome {
            min_content_height: body_height,
            ..chrome
        },
        body_height,
    )
}

#[derive(Props)]
pub struct SystemPromptDialogOverlayProps {
    pub screen_width: u16,
    pub screen_height: u16,
    pub text: String,
    pub body_height: u16,
    pub chrome: DialogChrome,
    pub scroll_handle: Option<Ref<ScrollViewHandle>>,
    pub has_focus: bool,
}

impl Default for SystemPromptDialogOverlayProps {
    fn default() -> Self {
        Self {
            screen_width: 80,
            screen_height: 24,
            text: String::new(),
            body_height: 12,
            chrome: DialogChrome::default(),
            scroll_handle: None,
            has_focus: false,
        }
    }
}

#[component]
pub fn SystemPromptDialogOverlay(
    props: &mut SystemPromptDialogOverlayProps,
    _hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let body_width = props.chrome.inner_body_width();
    let header = DialogHeader::title("System prompt");

    element! {
        DialogShellOverlay(
            screen_width: props.screen_width,
            screen_height: props.screen_height,
            chrome: props.chrome.clone(),
            header: header,
        ) {
            ScrollBox(
                width: body_width,
                height: props.body_height,
                auto_scroll: false,
                keyboard_scroll: props.has_focus,
                scroll_step: 3u16,
                handle: props.scroll_handle,
            ) {
                Text(content: props.text.clone(), wrap: TextWrap::Wrap)
            }
        }
    }
}
