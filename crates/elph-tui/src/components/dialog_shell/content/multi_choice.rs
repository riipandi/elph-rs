//! Multi-select question dialog body.

use super::layout::dialog_body_section_gap;
use crate::components::select::{
    SELECT_LIST_AUTO_HEIGHT, select_hidden_rows_above, select_inner_width, select_measured_row_counts,
    select_resolve_viewport_rows, select_window_start_for_rows,
};
use crate::components::theme::{
    UiTheme, dialog_option_desc_style, dialog_option_name_style, dialog_row_surface, list_marker, resolve_ui_theme,
};
use crate::types::SelectOption;
use iocraft::prelude::*;

fn multi_choice_row_surface(theme: UiTheme, focused: bool) -> Color {
    dialog_row_surface(theme, focused)
}

/// Toggle the checked state at `index` in `checked`.
pub fn multi_choice_toggle(checked: &mut [bool], index: usize) {
    if let Some(slot) = checked.get_mut(index) {
        *slot = !*slot;
    }
}

/// Selected option indices from parallel `checked` flags.
pub fn multi_choice_selected_indices(checked: &[bool]) -> Vec<usize> {
    checked
        .iter()
        .enumerate()
        .filter_map(|(i, on)| on.then_some(i))
        .collect()
}

/// Map a key press to multi-choice navigation or toggle (`None` = no change).
pub fn multi_choice_key_action(
    cursor: usize,
    len: usize,
    code: KeyCode,
    modifiers: KeyModifiers,
    step: usize,
) -> Option<MultiChoiceAction> {
    if len == 0 {
        return None;
    }
    if modifiers.contains(KeyModifiers::CONTROL) || modifiers.contains(KeyModifiers::ALT) {
        return None;
    }
    let fast = modifiers.contains(KeyModifiers::SHIFT);
    match code {
        KeyCode::Up | KeyCode::Char('k') => {
            let delta = if fast { step } else { 1 };
            Some(MultiChoiceAction::MoveCursor(cursor.saturating_sub(delta)))
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let delta = if fast { step } else { 1 };
            Some(MultiChoiceAction::MoveCursor((cursor + delta).min(len - 1)))
        }
        KeyCode::Char(' ') if modifiers.is_empty() => Some(MultiChoiceAction::ToggleCurrent),
        KeyCode::Enter if modifiers.is_empty() => Some(MultiChoiceAction::Submit),
        _ => None,
    }
}

/// Keyboard outcome for [`multi_choice_key_action`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MultiChoiceAction {
    MoveCursor(usize),
    ToggleCurrent,
    Submit,
}

/// Props for [`DialogMultiChoiceContent`].
#[derive(Props)]
pub struct DialogMultiChoiceContentProps {
    pub width: u16,
    pub height: u16,
    pub question: String,
    pub options: Vec<SelectOption>,
    pub cursor_index: Option<State<usize>>,
    pub checked: Option<State<Vec<bool>>>,
    pub has_focus: bool,
    pub show_description: bool,
    pub fast_scroll_step: usize,
    pub theme: Option<UiTheme>,
    pub on_submit: HandlerMut<'static, Vec<usize>>,
}

impl Default for DialogMultiChoiceContentProps {
    fn default() -> Self {
        Self {
            width: 40,
            height: SELECT_LIST_AUTO_HEIGHT,
            question: String::new(),
            options: Vec::new(),
            cursor_index: None,
            checked: None,
            has_focus: true,
            show_description: true,
            fast_scroll_step: 3,
            theme: None,
            on_submit: HandlerMut::default(),
        }
    }
}

#[component]
pub fn DialogMultiChoiceContent(
    props: &mut DialogMultiChoiceContentProps,
    mut hooks: Hooks,
) -> impl Into<AnyElement<'static>> {
    let theme = resolve_ui_theme(&hooks, props.theme);
    let option_count = props.options.len();
    let internal_cursor = hooks.use_state(|| 0usize);
    let internal_checked = hooks.use_state(|| vec![false; option_count]);
    let cursor = props.cursor_index.unwrap_or(internal_cursor);
    let mut checked = props.checked.unwrap_or(internal_checked);
    let has_focus = props.has_focus;
    let step = props.fast_scroll_step.max(1);
    let show_description = props.show_description;

    let checked_len = checked.read().len();
    if checked_len != option_count && option_count > 0 {
        checked.set(vec![false; option_count]);
    }

    hooks.use_terminal_events({
        let mut cursor = cursor;
        let mut checked = checked;
        let mut on_submit = props.on_submit.take();
        move |event| {
            if !has_focus || option_count == 0 {
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
            match multi_choice_key_action(cursor.get(), option_count, code, modifiers, step) {
                Some(MultiChoiceAction::MoveCursor(next)) => cursor.set(next),
                Some(MultiChoiceAction::ToggleCurrent) => {
                    let idx = cursor.get().min(option_count.saturating_sub(1));
                    let mut flags = checked.read().clone();
                    multi_choice_toggle(&mut flags, idx);
                    checked.set(flags);
                }
                Some(MultiChoiceAction::Submit) => {
                    on_submit(multi_choice_selected_indices(&checked.read()));
                }
                None => {}
            }
        }
    });

    let index = cursor.get().min(option_count.saturating_sub(1));
    let flags = checked.read();
    let inner_width = select_inner_width(theme, props.width);
    let container_surface = Color::Reset;
    let (viewport_rows, total_rows) =
        select_resolve_viewport_rows(&props.options, show_description, props.width, theme, props.height, false);
    let scrollable = total_rows > viewport_rows;
    let row_counts = select_measured_row_counts(&props.options, show_description, props.width, theme, false);
    let window_start = select_window_start_for_rows(index, viewport_rows, &row_counts);

    let mut option_rows: Vec<AnyElement<'static>> = Vec::new();
    let mut used_rows = 0usize;

    if option_count == 0 {
        option_rows.push(
            element! {
                Text(content: "(no options)".to_string(), color: theme.text_muted, wrap: TextWrap::NoWrap)
            }
            .into(),
        );
    } else {
        let hidden_above = select_hidden_rows_above(window_start, &row_counts);
        if hidden_above > 0 {
            option_rows.push(
                element! {
                    Text(
                        content: format!("  ↑ {hidden_above} more"),
                        color: theme.text_hint,
                        wrap: TextWrap::NoWrap,
                    )
                }
                .into(),
            );
            used_rows += 1;
        }

        for (i, opt) in props.options.iter().enumerate().skip(window_start) {
            let item_rows = row_counts[i];
            if used_rows > 0 && used_rows + item_rows > viewport_rows {
                break;
            }

            let row_focused = i == index;
            let is_on = flags.get(i).copied().unwrap_or(false);
            let box_glyph = if is_on { "[x]" } else { "[ ]" };
            let marker = list_marker(row_focused);
            let (name_color, name_weight) = dialog_option_name_style(theme, row_focused);
            let desc_color = dialog_option_desc_style(theme);
            let show_desc = show_description && !opt.description.is_empty();

            option_rows.push(
                element! {
                    View(
                        width: inner_width,
                        flex_direction: FlexDirection::Column,
                        gap: 0,
                        background_color: multi_choice_row_surface(theme, row_focused),
                        padding_left: theme.container_inset(),
                        padding_right: theme.container_inset(),
                        padding_top: if i > window_start { theme.gap_sm } else { 0 },
                    ) {
                        View(flex_direction: FlexDirection::Row, gap: theme.gap_md, align_items: AlignItems::Center) {
                            Text(content: format!("{marker}{box_glyph}"), color: name_color, wrap: TextWrap::NoWrap)
                            Text(content: opt.name.clone(), color: name_color, weight: name_weight, wrap: TextWrap::NoWrap)
                        }
                        #(if show_desc {
                            Some(element! {
                                View(padding_left: theme.list_desc_padding_left()) {
                                    Text(
                                        content: opt.description.clone(),
                                        color: desc_color,
                                        wrap: TextWrap::Wrap,
                                    )
                                }
                            })
                        } else {
                            None
                        })
                    }
                }
                .into(),
            );
            used_rows += item_rows;
        }

        let rows_before = select_hidden_rows_above(window_start, &row_counts);
        let rows_shown = used_rows.saturating_sub(usize::from(hidden_above > 0));
        let hidden_below = row_counts
            .iter()
            .sum::<usize>()
            .saturating_sub(rows_before + rows_shown);
        if hidden_below > 0 {
            option_rows.push(
                element! {
                    Text(
                        content: format!("  ↓ {hidden_below} more"),
                        color: theme.text_hint,
                        wrap: TextWrap::NoWrap,
                    )
                }
                .into(),
            );
        }
    }

    let section_gap = dialog_body_section_gap(theme);

    let list_viewport: AnyElement<'static> = if scrollable {
        element! {
            View(
                width: props.width,
                height: viewport_rows as u16,
                min_height: viewport_rows as u16,
                flex_direction: FlexDirection::Column,
                gap: theme.gap_sm,
                background_color: container_surface,
                overflow: Overflow::Hidden,
                flex_shrink: 0f32,
            ) {
                #(option_rows)
            }
        }
        .into()
    } else {
        element! {
            View(
                width: props.width,
                flex_direction: FlexDirection::Column,
                gap: theme.gap_sm,
                background_color: container_surface,
                flex_shrink: 0f32,
            ) {
                #(option_rows)
            }
        }
        .into()
    };

    element! {
        View(
            width: props.width,
            flex_direction: FlexDirection::Column,
            gap: section_gap,
            flex_shrink: 0f32,
        ) {
            Text(
                content: props.question.clone(),
                color: theme.text_secondary,
                wrap: TextWrap::Wrap,
            )
            #(list_viewport)
            Text(
                content: "Space toggle · Enter confirm · ↑↓ move".to_string(),
                color: theme.text_muted,
                wrap: TextWrap::NoWrap,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_choice_toggle_flips_slot() {
        let mut flags = vec![false, true];
        multi_choice_toggle(&mut flags, 0);
        assert!(flags[0]);
        multi_choice_toggle(&mut flags, 1);
        assert!(!flags[1]);
    }

    #[test]
    fn multi_choice_selected_indices_collects_checked() {
        let flags = vec![true, false, true];
        assert_eq!(multi_choice_selected_indices(&flags), vec![0, 2]);
    }

    #[test]
    fn multi_choice_space_toggles() {
        assert_eq!(
            multi_choice_key_action(1, 3, KeyCode::Char(' '), KeyModifiers::empty(), 3),
            Some(MultiChoiceAction::ToggleCurrent)
        );
    }
}
