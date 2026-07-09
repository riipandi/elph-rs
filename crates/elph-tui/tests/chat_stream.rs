use elph_tui::{ChatStreamState, Theme, render_chat_stream};
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
