use serde_json::Value;

const VALID_JSON_ESCAPES: &[char] = &['"', '\\', '/', 'b', 'f', 'n', 'r', 't', 'u'];

fn is_control_character(ch: char) -> bool {
    let code = ch as u32;
    code <= 0x1f
}

fn escape_control_character(ch: char) -> String {
    match ch {
        '\x08' => "\\b".to_string(),
        '\x0C' => "\\f".to_string(),
        '\n' => "\\n".to_string(),
        '\r' => "\\r".to_string(),
        '\t' => "\\t".to_string(),
        _ => format!("\\u{:04x}", ch as u32),
    }
}

/// Repairs malformed JSON string literals by escaping control characters.
pub fn repair_json(json: &str) -> String {
    let mut repaired = String::new();
    let mut in_string = false;
    let mut chars = json.chars().peekable();

    while let Some(ch) = chars.next() {
        if !in_string {
            repaired.push(ch);
            if ch == '"' {
                in_string = true;
            }
            continue;
        }

        if ch == '"' {
            repaired.push(ch);
            in_string = false;
            continue;
        }

        if ch == '\\' {
            if let Some(&next) = chars.peek() {
                if next == 'u' {
                    let unicode: String = chars.by_ref().take(5).collect();
                    if unicode.len() == 5 && unicode[1..].chars().all(|c| c.is_ascii_hexdigit()) {
                        repaired.push_str(&unicode);
                        continue;
                    }
                }
                if VALID_JSON_ESCAPES.contains(&next) {
                    repaired.push(ch);
                    repaired.push(chars.next().unwrap());
                    continue;
                }
            }
            repaired.push_str("\\\\");
            continue;
        }

        if is_control_character(ch) {
            repaired.push_str(&escape_control_character(ch));
        } else {
            repaired.push(ch);
        }
    }

    repaired
}

pub fn parse_json_with_repair(json: &str) -> Result<Value, serde_json::Error> {
    match serde_json::from_str::<Value>(json) {
        Ok(v) => Ok(v),
        Err(e) => {
            let repaired = repair_json(json);
            if repaired != json {
                serde_json::from_str(&repaired)
            } else {
                Err(e)
            }
        }
    }
}

/// Parse potentially incomplete JSON during streaming.
pub fn parse_streaming_json(partial_json: Option<&str>) -> Value {
    let partial_json = partial_json.unwrap_or("").trim();
    if partial_json.is_empty() {
        return Value::Object(serde_json::Map::new());
    }

    if let Ok(v) = parse_json_with_repair(partial_json) {
        return v;
    }

    // Best-effort partial parse: close open braces/brackets
    for suffix in ["", "}", "]}", "\"}", "\":\"\"}", "]}"] {
        let attempt = format!("{partial_json}{suffix}");
        if let Ok(v) = parse_json_with_repair(&attempt) {
            return v;
        }
    }

    Value::Object(serde_json::Map::new())
}
