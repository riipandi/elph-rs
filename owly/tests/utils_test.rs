//! Tests for Owly utils module.
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki)
//! `test/utils.test.ts`. Original MIT License, Copyright (c) 2026 LangChain.

use owly::utils::strip_html_tags;

#[test]
fn test_strip_html_removes_complete_tag_pair() {
    assert_eq!(strip_html_tags("<div>hello</div>"), "hello");
}

#[test]
fn test_strip_html_removes_adjacent_and_nested_tags() {
    assert_eq!(strip_html_tags("<b><i>hi</i></b>"), "hi");
    assert_eq!(strip_html_tags("a<br/>b<hr>c"), "abc");
}

#[test]
fn test_strip_html_removes_html_comments() {
    assert_eq!(strip_html_tags("before<!-- secret -->after"), "beforeafter");
}

#[test]
fn test_strip_html_strips_unterminated_tag_fragment() {
    assert_eq!(strip_html_tags("text <script"), "text script");
    assert_eq!(strip_html_tags("<script"), "script");
}

#[test]
fn test_strip_html_never_leaves_angle_bracket() {
    let inputs = [
        "<div>hi</div>",
        "text <script",
        "<scr<script>ipt>",
        "<<script>>",
        "a < b > c",
    ];

    for input in inputs {
        let output = strip_html_tags(input);
        assert!(!output.contains('<'), "Found '<' in output for input: {}", input);
        assert!(!output.contains('>'), "Found '>' in output for input: {}", input);
    }
}

#[test]
fn test_strip_html_leaves_plain_text_untouched() {
    assert_eq!(strip_html_tags("just plain text"), "just plain text");
    assert_eq!(strip_html_tags(""), "");
}
