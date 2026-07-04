use elph_tui::{AgentMode, PromptInput};
use futures::{StreamExt, stream};
use iocraft::prelude::*;
use macro_rules_attribute::apply;
use smol_macros::test;

fn shift_enter(kind: KeyEventKind) -> KeyEvent {
    let mut event = KeyEvent::new(kind, KeyCode::Enter);
    event.modifiers = KeyModifiers::SHIFT;
    event
}

#[component]
fn Harness(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let prompt = hooks.use_state(String::new);
    let mut should_exit = hooks.use_state(|| false);

    if prompt.read().ends_with('!') {
        should_exit.set(true);
    }

    if should_exit.get() {
        system.exit();
    }

    element! {
        View(width: 40, height: 8, padding: 1) {
            PromptInput(
                value: Some(prompt),
                model_name: "test-model".to_string(),
                mode: AgentMode::Build,
                has_focus: true,
                on_submit: |_| {},
                on_mode_change: |_| {},
            )
        }
    }
}

#[component]
fn EnterHarness(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let prompt = hooks.use_state(String::new);
    let mut submit_count = hooks.use_state(|| 0u32);
    let mut should_exit = hooks.use_state(|| false);

    if submit_count.get() > 0 || prompt.read().ends_with('!') {
        should_exit.set(true);
    }

    if should_exit.get() {
        system.exit();
    }

    element! {
        View(width: 40, height: 8, padding: 1) {
            PromptInput(
                value: Some(prompt),
                model_name: "test-model".to_string(),
                mode: AgentMode::Build,
                has_focus: true,
                on_submit: move |_| submit_count.set(submit_count.get() + 1),
                on_mode_change: |_| {},
            )
        }
    }
}

#[apply(test)]
async fn typing_updates_prompt() {
    let events = stream::iter([
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Press, KeyCode::Char('h'))),
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Release, KeyCode::Char('h'))),
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Press, KeyCode::Char('i'))),
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Release, KeyCode::Char('i'))),
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Press, KeyCode::Char('!'))),
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Release, KeyCode::Char('!'))),
    ]);

    let output = element!(Harness)
        .mock_terminal_render_loop(MockTerminalConfig::with_events(events))
        .map(|frame| frame.to_string())
        .collect::<Vec<_>>()
        .await;

    assert!(
        output.iter().any(|frame| frame.contains("hi")),
        "expected typed text in a frame, got: {output:?}"
    );
}

#[component]
fn PrefilledEnterHarness(mut hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let mut system = hooks.use_context_mut::<SystemContext>();
    let prompt = hooks.use_state(|| "hi".to_string());
    let mut submit_count = hooks.use_state(|| 0u32);
    let mut should_exit = hooks.use_state(|| false);

    if submit_count.get() > 0 {
        should_exit.set(true);
    }

    if should_exit.get() {
        system.exit();
    }

    element! {
        View(width: 40, height: 8, padding: 1) {
            PromptInput(
                value: Some(prompt),
                model_name: "test-model".to_string(),
                mode: AgentMode::Build,
                has_focus: true,
                on_submit: move |_| submit_count.set(submit_count.get() + 1),
                on_mode_change: |_| {},
            )
        }
    }
}

#[apply(test)]
async fn plain_enter_submits_without_newline() {
    let events = stream::iter([
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Press, KeyCode::Enter)),
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Release, KeyCode::Enter)),
    ]);

    let output = element!(PrefilledEnterHarness)
        .mock_terminal_render_loop(MockTerminalConfig::with_events(events))
        .map(|frame| frame.to_string())
        .collect::<Vec<_>>()
        .await;

    assert!(
        output.iter().any(|frame| frame.contains("hi")),
        "prompt text should remain on one line before submit, got: {output:?}"
    );
    assert!(output.len() > 1, "plain Enter should submit and exit, got: {output:?}");
}

#[apply(test)]
async fn shift_enter_inserts_newline_without_submit() {
    let events = stream::iter([
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Press, KeyCode::Char('a'))),
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Release, KeyCode::Char('a'))),
        TerminalEvent::Key(shift_enter(KeyEventKind::Press)),
        TerminalEvent::Key(shift_enter(KeyEventKind::Release)),
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Press, KeyCode::Char('b'))),
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Release, KeyCode::Char('b'))),
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Press, KeyCode::Char('!'))),
        TerminalEvent::Key(KeyEvent::new(KeyEventKind::Release, KeyCode::Char('!'))),
    ]);

    let output = element!(EnterHarness)
        .mock_terminal_render_loop(MockTerminalConfig::with_events(events))
        .map(|frame| frame.to_string())
        .collect::<Vec<_>>()
        .await;

    let multiline = output.iter().any(|frame| {
        let a_line = frame.lines().any(|line| line.contains("a") && !line.contains('b'));
        let b_line = frame.lines().any(|line| line.contains('b') && !line.contains('a'));
        a_line && b_line
    });
    assert!(multiline, "expected multiline prompt text, got: {output:?}");
}
