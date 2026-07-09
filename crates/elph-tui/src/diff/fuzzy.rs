//! Fuzzy matching utilities.

use rayon::prelude::*;

/// Minimum item count before parallel scoring kicks in.
const PARALLEL_THRESHOLD: usize = 64;

/// Result of a fuzzy match attempt.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FuzzyMatch {
    pub matches: bool,
    pub score: f64,
}

/// Returns true when all query characters appear in order within `text`.
/// Lower score is a better match.
pub fn fuzzy_match(query: &str, text: &str) -> FuzzyMatch {
    let primary = match_query(query, text);
    if primary.matches {
        return primary;
    }

    if let Some(swapped_query) = swap_alpha_numeric(query) {
        let swapped_match = match_query(&swapped_query, text);
        if swapped_match.matches {
            return FuzzyMatch {
                matches: true,
                score: swapped_match.score + 5.0,
            };
        }
    }

    primary
}

const fn fold_ascii(c: char) -> char {
    c.to_ascii_lowercase()
}

fn chars_match(a: char, b: char) -> bool {
    a == b || fold_ascii(a) == fold_ascii(b)
}

fn match_query(query: &str, text: &str) -> FuzzyMatch {
    if query.is_empty() {
        return FuzzyMatch {
            matches: true,
            score: 0.0,
        };
    }

    let query_len = query.chars().count();
    let text_len = text.chars().count();
    if query_len > text_len {
        return FuzzyMatch {
            matches: false,
            score: 0.0,
        };
    }

    let mut query_chars = query.chars().map(fold_ascii);
    let Some(mut next_query) = query_chars.next() else {
        return FuzzyMatch {
            matches: true,
            score: 0.0,
        };
    };

    let mut score = 0.0f64;
    let mut last_match_index: Option<usize> = None;
    let mut consecutive_matches = 0usize;
    let mut matched = 0usize;
    let mut prev_ch: Option<char> = None;

    for (i, ch) in text.chars().enumerate() {
        if chars_match(ch, next_query) {
            let is_boundary = i == 0
                || prev_ch
                    .is_some_and(|prev| prev.is_ascii_whitespace() || matches!(prev, '-' | '_' | '.' | '/' | ':'));

            if i > 0 && last_match_index == Some(i - 1) {
                consecutive_matches += 1;
                score -= (consecutive_matches * 5) as f64;
            } else {
                consecutive_matches = 0;
                if let Some(last) = last_match_index
                    && i > last + 1
                {
                    score += ((i - last - 1) * 2) as f64;
                }
            }

            if is_boundary {
                score -= 10.0;
            }
            score = (i as f64).mul_add(0.1, score);
            last_match_index = Some(i);
            matched += 1;
            match query_chars.next() {
                Some(qc) => next_query = qc,
                None => break,
            }
        }
        prev_ch = Some(ch);
    }

    if matched < query_len {
        return FuzzyMatch {
            matches: false,
            score: 0.0,
        };
    }

    if query.eq_ignore_ascii_case(text) {
        score -= 100.0;
    }

    FuzzyMatch { matches: true, score }
}

fn swap_alpha_numeric(query: &str) -> Option<String> {
    if let Some(caps) = regex_like_alpha_num(query) {
        return Some(format!("{}{}", caps.1, caps.0));
    }
    None
}

fn regex_like_alpha_num(query: &str) -> Option<(&str, &str)> {
    let bytes = query.as_bytes();
    let mut split = None;
    for (i, &b) in bytes.iter().enumerate().skip(1) {
        let prev_alpha = bytes[i - 1].is_ascii_alphabetic();
        let prev_digit = bytes[i - 1].is_ascii_digit();
        let curr_alpha = b.is_ascii_alphabetic();
        let curr_digit = b.is_ascii_digit();
        if (prev_alpha && curr_digit) || (prev_digit && curr_alpha) {
            split = Some(i);
            break;
        }
    }
    let idx = split?;
    let (left, right) = query.split_at(idx);
    if left.is_empty() || right.is_empty() {
        return None;
    }
    Some((left, right))
}

fn score_item<T, F>(item: &T, tokens: &[&str], get_text: &F) -> Option<(T, f64)>
where
    T: Clone,
    F: Fn(&T) -> String,
{
    let text = get_text(item);
    let mut total_score = 0.0;
    for token in tokens {
        let m = fuzzy_match(token, &text);
        if m.matches {
            total_score += m.score;
        } else {
            return None;
        }
    }
    Some((item.clone(), total_score))
}

/// Filters and sorts items by fuzzy match quality (best matches first).
/// Whitespace- or slash-separated tokens must all match.
pub fn fuzzy_filter<T, F>(items: &[T], query: &str, get_text: F) -> Vec<T>
where
    T: Clone + Send + Sync,
    F: Fn(&T) -> String + Sync,
{
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return items.to_vec();
    }

    let tokens: Vec<&str> = trimmed
        .split(|c: char| c.is_whitespace() || c == '/')
        .filter(|t| !t.is_empty())
        .collect();
    if tokens.is_empty() {
        return items.to_vec();
    }

    let mut results: Vec<(T, f64)> = if items.len() >= PARALLEL_THRESHOLD {
        items
            .par_iter()
            .filter_map(|item| score_item(item, &tokens, &get_text))
            .collect()
    } else {
        items
            .iter()
            .filter_map(|item| score_item(item, &tokens, &get_text))
            .collect()
    };

    results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    results.into_iter().map(|(item, _)| item).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_subsequence() {
        let m = fuzzy_match("hl", "hello");
        assert!(m.matches);
    }

    #[test]
    fn filters_with_multiple_tokens() {
        let items = vec!["git status", "git commit", "cargo test"];
        let out = fuzzy_filter(&items, "git st", |s| (*s).to_string());
        assert_eq!(out.len(), 1);
        assert_eq!(out[0], "git status");
    }

    #[test]
    fn matches_case_insensitive() {
        let m = fuzzy_match("ABC", "abc");
        assert!(m.matches);
    }
}
