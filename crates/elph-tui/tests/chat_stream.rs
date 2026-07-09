use elph_tui::{
    ChatStreamState, Theme, TranscriptEntry, is_pinned_to_bottom, render_chat_stream, render_chat_stream_with_agent,
};
use slt::TestBackend;

#[test]
fn chat_stream_renders_messages() {
    let mut backend = TestBackend::new(60, 12);
    let mut state = ChatStreamState::with_messages(vec!["hello".to_string(), "world".to_string()]);
    let theme = Theme::dark();

    backend.render(|ui| {
        render_chat_stream(ui, &mut state, theme);
    });

    backend.assert_contains("hello");
    backend.assert_contains("world");
}

#[test]
fn streaming_pins_to_tail() {
    let mut backend = TestBackend::new(40, 8);
    let mut state = ChatStreamState::new();
    state.pin_to_tail();
    state.entries = (0..40)
        .map(|i| TranscriptEntry::user(format!("line {i}")))
        .chain([TranscriptEntry::assistant_streaming("streaming tail")])
        .collect();

    let theme = Theme::dark();
    for _ in 0..3 {
        backend.render(|ui| {
            render_chat_stream_with_agent(ui, &mut state, theme, true);
        });
    }

    assert!(state.auto_scroll);
    assert!(is_pinned_to_bottom(&state.scroll));
}
