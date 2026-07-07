use anyhow::{Result, anyhow};
use eventsource_stream::Eventsource;
use futures::StreamExt;
use serde_json::Value;

/// Collect all JSON events from an SSE response body.
pub async fn collect_sse_json_events(response: reqwest::Response) -> Result<Vec<Value>> {
    let mut events = Vec::new();
    let mut stream = Box::pin(response.bytes_stream().eventsource().filter_map(|event| async {
        match event {
            Ok(event) => {
                let data = event.data.trim();
                if data.is_empty() || data == "[DONE]" {
                    None
                } else {
                    match serde_json::from_str::<Value>(data) {
                        Ok(v) => Some(Ok(v)),
                        Err(e) => Some(Err(anyhow!("Invalid SSE JSON: {e}; data={data}"))),
                    }
                }
            }
            Err(e) => Some(Err(anyhow!(e.to_string()))),
        }
    }));
    while let Some(item) = stream.next().await {
        events.push(item?);
    }
    Ok(events)
}

/// Anthropic-style SSE decoder state.
#[derive(Default)]
pub struct SseDecoderState {
    event: Option<String>,
    data: Vec<String>,
    raw: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct ServerSentEvent {
    pub event: Option<String>,
    pub data: String,
    pub raw: Vec<String>,
}

fn flush_sse_event(state: &mut SseDecoderState) -> Option<ServerSentEvent> {
    if state.event.is_none() && state.data.is_empty() {
        return None;
    }
    let event = ServerSentEvent {
        event: state.event.take(),
        data: state.data.join("\n"),
        raw: std::mem::take(&mut state.raw),
    };
    state.data.clear();
    Some(event)
}

fn decode_sse_line(line: &str, state: &mut SseDecoderState) -> Option<ServerSentEvent> {
    if line.is_empty() {
        return flush_sse_event(state);
    }
    state.raw.push(line.to_string());
    if line.starts_with(':') {
        return None;
    }
    if let Some((field, value)) = line.split_once(':') {
        let value = value.strip_prefix(' ').unwrap_or(value);
        match field {
            "event" => state.event = Some(value.to_string()),
            "data" => state.data.push(value.to_string()),
            _ => {}
        }
    }
    None
}

/// Parse raw bytes into Anthropic SSE events.
pub fn decode_sse_buffer(buffer: &str, state: &mut SseDecoderState) -> Vec<ServerSentEvent> {
    let mut events = Vec::new();
    for line in buffer.split('\n') {
        let line = line.trim_end_matches('\r');
        if let Some(event) = decode_sse_line(line, state) {
            events.push(event);
        }
    }
    events
}

pub const ANTHROPIC_MESSAGE_EVENTS: &[&str] = &[
    "message_start",
    "message_delta",
    "message_stop",
    "content_block_start",
    "content_block_delta",
    "content_block_stop",
];
