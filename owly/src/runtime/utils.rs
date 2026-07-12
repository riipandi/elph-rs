//! Utility functions for Owly.
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki)
//! `src/utils.ts`. Original MIT License, Copyright (c) 2026 LangChain.

/// Strip HTML tags from a string, returning only the text content.
///
/// This function removes:
/// - Complete tag pairs (e.g., `<div>hello</div>` -> `hello`)
/// - Self-closing tags (e.g., `<br/>`, `<hr>`)
/// - HTML comments (e.g., `<!-- comment -->`)
/// - Unterminated tag fragments (e.g., `<script` -> `script`)
///
/// The output will never contain `<` or `>` characters.
pub fn strip_html_tags(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '<' {
            // Check for HTML comment
            if chars.peek() == Some(&'!') {
                // Skip until -->
                while let Some(c) = chars.next() {
                    if c == '-' && chars.peek() == Some(&'-') {
                        chars.next(); // consume second -
                        if chars.peek() == Some(&'>') {
                            chars.next(); // consume >
                            break;
                        }
                    }
                }
            } else {
                // Skip until > (handles nested < inside tags)
                let mut depth = 1;
                let mut tag_content = String::new();
                for c in chars.by_ref() {
                    match c {
                        '<' => {
                            depth += 1;
                            tag_content.push(c);
                        }
                        '>' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            tag_content.push(c);
                        }
                        _ => tag_content.push(c),
                    }
                }
                // If unterminated tag, add the tag content back to result
                if depth > 0 {
                    result.push_str(&tag_content);
                }
            }
        } else {
            result.push(ch);
        }
    }

    result
}
#[cfg(test)]
mod tests {
    use super::*;

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
}
