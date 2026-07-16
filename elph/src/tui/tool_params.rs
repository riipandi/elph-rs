//! Structured parsing and rendering for tool call parameters.

use elph_tui::components::UiTheme;
use iocraft::prelude::*;
use serde_json::Value;

use crate::tui::theme::TOOL_ARGS_FG;

/// One logical parameter row (object field or a single scalar fallback).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolParam {
    pub key: Option<String>,
    pub value: String,
}

/// Parse raw tool args (JSON object/array/scalar or plain text) into display rows.
pub fn parse_tool_params(raw: &str) -> Vec<ToolParam> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }

    let Ok(value) = serde_json::from_str::<Value>(trimmed) else {
        return vec![ToolParam {
            key: None,
            value: trimmed.to_string(),
        }];
    };

    params_from_json(&value)
}

fn params_from_json(value: &Value) -> Vec<ToolParam> {
    match value {
        Value::Object(map) if map.is_empty() => Vec::new(),
        Value::Object(map) => map
            .iter()
            .map(|(key, val)| ToolParam {
                key: Some(key.clone()),
                value: format_json_value(val),
            })
            .collect(),
        Value::Array(items) => items
            .iter()
            .enumerate()
            .map(|(index, val)| ToolParam {
                key: Some((index + 1).to_string()),
                value: format_json_value(val),
            })
            .collect(),
        other => vec![ToolParam {
            key: None,
            value: format_json_value(other),
        }],
    }
}

fn format_json_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Number(num) => num.to_string(),
        Value::Bool(flag) => flag.to_string(),
        Value::Null => "null".to_string(),
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(format_json_value).collect();
            parts.join(", ")
        }
        Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn display_value(key: Option<&str>, value: &str) -> String {
    if key == Some("command") && !value.starts_with('$') {
        format!("$ {value}")
    } else {
        value.to_string()
    }
}

fn show_key_column(params: &[ToolParam]) -> bool {
    params.len() > 1
        || params
            .first()
            .and_then(|param| param.key.as_deref())
            .is_some_and(|key| key == "command")
}

fn key_column_width(params: &[ToolParam]) -> u16 {
    if !show_key_column(params) {
        return 0;
    }
    let max = params
        .iter()
        .filter_map(|param| param.key.as_ref())
        .map(|key| key.chars().count() + 1)
        .max()
        .unwrap_or(0);
    (max as u16).clamp(5, 14)
}

fn pad_key_label(key: &str, width: u16) -> String {
    let label = format!("{key}:");
    let chars = label.chars().count();
    if chars >= width as usize {
        return label;
    }
    format!("{label}{}", " ".repeat(width as usize - chars))
}

/// Compact single-line summary (transcript layout / logs).
pub fn format_tool_params_display(raw: &str) -> String {
    let params = parse_tool_params(raw);
    if params.is_empty() {
        return String::new();
    }
    if params.len() == 1 && params[0].key.is_none() {
        return params[0].value.clone();
    }
    if params.len() == 1 {
        let param = &params[0];
        return match param.key.as_deref() {
            Some("command") => display_value(Some("command"), &param.value),
            Some(_) => param.value.clone(),
            None => param.value.clone(),
        };
    }
    params
        .iter()
        .map(|param| match &param.key {
            Some(key) => format!("{key}: {}", display_value(Some(key.as_str()), &param.value)),
            None => param.value.clone(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Props for [`ToolParamsView`].
#[derive(Clone, Props)]
pub struct ToolParamsViewProps {
    pub width: u16,
    pub raw: String,
    pub key_color: Color,
    pub value_color: Color,
}

impl Default for ToolParamsViewProps {
    fn default() -> Self {
        let theme = UiTheme::default();
        Self {
            width: 40,
            raw: String::new(),
            key_color: TOOL_ARGS_FG,
            value_color: theme.text_secondary,
        }
    }
}

/// Aligned key/value rows for tool parameters.
#[component]
pub fn ToolParamsView(props: &ToolParamsViewProps, hooks: Hooks) -> impl Into<AnyElement<'static>> {
    let _ = hooks;
    let params = parse_tool_params(&props.raw);
    if params.is_empty() {
        return element! { View(width: props.width) };
    }

    let show_keys = show_key_column(&params);
    let key_width = key_column_width(&params);
    let value_width = if show_keys {
        props.width.saturating_sub(key_width).saturating_sub(1).max(8)
    } else {
        props.width
    };
    let mut rows: Vec<AnyElement<'static>> = Vec::new();

    for param in &params {
        let value = display_value(param.key.as_deref(), &param.value);
        let row = if show_keys {
            let key = param.key.as_deref().unwrap_or("");
            element! {
                View(
                    width: props.width,
                    flex_direction: FlexDirection::Row,
                    gap: 1,
                    flex_shrink: 0f32,
                ) {
                    Text(
                        content: pad_key_label(key, key_width),
                        color: props.key_color,
                        wrap: TextWrap::NoWrap,
                    )
                    View(width: value_width, flex_shrink: 0f32) {
                        Text(
                            content: value,
                            color: props.value_color,
                            wrap: TextWrap::Wrap,
                        )
                    }
                }
            }
            .into()
        } else {
            element! {
                View(width: props.width, flex_shrink: 0f32) {
                    Text(
                        content: value,
                        color: props.value_color,
                        wrap: TextWrap::Wrap,
                    )
                }
            }
            .into()
        };
        rows.push(row);
    }

    element! {
        View(
            width: props.width,
            flex_direction: FlexDirection::Column,
            gap: 0,
            flex_shrink: 0f32,
        ) {
            #(rows)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_object_into_keyed_rows() {
        let params = parse_tool_params(r#"{"command":"date","path":"main.rs"}"#);
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].key.as_deref(), Some("command"));
        assert_eq!(params[1].value, "main.rs");
    }

    #[test]
    fn command_values_get_shell_prefix() {
        let text = format_tool_params_display(r#"{"command":"cargo test"}"#);
        assert_eq!(text, "$ cargo test");
    }

    #[test]
    fn single_path_key_shows_value_only_in_compact_line() {
        assert_eq!(format_tool_params_display(r#"{"path":"src/lib.rs"}"#), "src/lib.rs");
    }

    #[test]
    fn plain_text_becomes_scalar_row() {
        let params = parse_tool_params("npm test");
        assert_eq!(params.len(), 1);
        assert!(params[0].key.is_none());
        assert_eq!(params[0].value, "npm test");
    }

    #[test]
    fn multi_key_uses_one_line_per_field() {
        let text = format_tool_params_display(r#"{"a":"1","b":"2"}"#);
        assert_eq!(text, "a: 1\nb: 2");
    }
}
