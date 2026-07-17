//! Fuzzy scoring for slash palette command filtering.

use super::SlashCommand;

const PREFIX_MATCH_BASE: i32 = 10_000;
const NAME_WEIGHT: i32 = 4;
const SKILL_SUFFIX_WEIGHT: i32 = 5;
const DESCRIPTION_WEIGHT: i32 = 1;

/// Best fuzzy score for a command against a lowercase query, if any field matches.
pub fn command_match_score(cmd: &SlashCommand, query: &str) -> Option<i32> {
    let query = query.trim();
    if query.is_empty() {
        return Some(0);
    }

    let name = cmd.name.to_ascii_lowercase();
    let mut best = field_score(query, &name, NAME_WEIGHT, true);

    if let Some(skill) = name.strip_prefix("skill:") {
        best = max_score(best, field_score(query, skill, SKILL_SUFFIX_WEIGHT, true));
    }

    let description = cmd.description.to_ascii_lowercase();
    best = max_score(best, field_score(query, &description, DESCRIPTION_WEIGHT, false));

    best
}

fn max_score(left: Option<i32>, right: Option<i32>) -> Option<i32> {
    match (left, right) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn field_score(query: &str, text: &str, weight: i32, allow_prefix: bool) -> Option<i32> {
    let raw = if allow_prefix {
        prefix_score(query, text).or_else(|| subsequence_score(query, text))
    } else {
        subsequence_score(query, text)
    };
    raw.map(|score| score.saturating_mul(weight))
}

fn prefix_score(query: &str, text: &str) -> Option<i32> {
    if text.starts_with(query) {
        let length_bonus = 200i32.saturating_sub(text.len() as i32);
        Some(PREFIX_MATCH_BASE + length_bonus)
    } else {
        None
    }
}

fn subsequence_score(query: &str, text: &str) -> Option<i32> {
    let query_chars: Vec<char> = query.chars().collect();
    if query_chars.is_empty() {
        return Some(0);
    }

    let mut score = 0i32;
    let mut query_idx = 0usize;
    let mut prev_match: Option<usize> = None;
    let text_chars: Vec<char> = text.chars().collect();

    for (text_idx, &ch) in text_chars.iter().enumerate() {
        if query_idx >= query_chars.len() {
            break;
        }
        if ch != query_chars[query_idx] {
            continue;
        }

        score += 1;
        if query_idx == 0 {
            score += 15;
            if text_idx == 0 {
                score += 20;
            }
        }

        if let Some(prev) = prev_match {
            if text_idx == prev + 1 {
                score += 10;
            } else {
                score -= (text_idx - prev - 1) as i32;
            }
        }

        if text_idx > 0 && is_word_boundary(text_chars[text_idx - 1]) {
            score += 12;
        }

        prev_match = Some(text_idx);
        query_idx += 1;
    }

    if query_idx == query_chars.len() {
        score += 100i32.saturating_sub(text.len() as i32 / 2);
        Some(score)
    } else {
        None
    }
}

fn is_word_boundary(ch: char) -> bool {
    matches!(ch, ':' | '-' | '_' | ' ' | '/')
}

/// Filter and rank commands by fuzzy relevance to `query`.
pub fn filter_commands(commands: &[SlashCommand], query: &str) -> Vec<SlashCommand> {
    let query = query.trim().to_ascii_lowercase();
    if query.is_empty() {
        return commands.to_vec();
    }

    let mut scored: Vec<(SlashCommand, i32)> = commands
        .iter()
        .filter_map(|cmd| command_match_score(cmd, &query).map(|score| (cmd.clone(), score)))
        .collect();

    scored.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.name.cmp(&right.0.name)));
    scored.into_iter().map(|(cmd, _)| cmd).collect()
}
