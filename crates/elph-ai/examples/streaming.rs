//! Stream a chat completion token-by-token or buffered — no API key needed.
//!
//! Demonstrates: provider + model registration, context construction, event streaming
//! with text/thinking content, buffered completion via `Models::complete()`,
//! usage statistics, and token-cost estimation.
//!
//! ```bash
//! # Streaming (token-by-token):
//! cargo run -p elph-ai --example streaming -- --stream
//!
//! # Buffered (waits for full response):
//! cargo run -p elph-ai --example streaming
//! ```

use std::io::Write;

use elph_ai::{AssistantContentBlock, AssistantMessageEvent, Context, FauxModelDefinition, FauxResponseStep, Message};
use elph_ai::{Models, RegisterFauxProviderOptions, StopReason, UserContent};
use elph_ai::{calculate_cost, create_models, faux_assistant_message, faux_provider, faux_text, faux_thinking};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stream_mode = std::env::args().any(|a| a == "--stream");

    // ── 1. Models registry ──
    let models = setup_faux_provider();
    let model = models.get_model("demo", "demo-model").expect("demo-model registered");

    // ── 2. Context ──
    let context = Context {
        system_prompt: Some("You are a concise Rust expert.".into()),
        messages: vec![Message::User {
            content: UserContent::Text("What makes Rust unique?".into()),
            timestamp: timestamp(),
        }],
        tools: None,
    };

    // ── 3. Run ──
    if stream_mode {
        run_streaming(&models, &model, &context).await?;
    } else {
        run_buffered(&models, &model, &context).await?;
    }

    Ok(())
}

/// Register a faux provider with two queued responses.
fn setup_faux_provider() -> elph_ai::MutableModels {
    let mut models = create_models(None);

    let faux = faux_provider(RegisterFauxProviderOptions {
        provider: Some("demo".to_string()),
        models: Some(vec![FauxModelDefinition {
            id: "demo-model".to_string(),
            name: Some("Demo Model".to_string()),
            reasoning: Some(true),
            input: None,
            context_window: None,
            max_tokens: None,
        }]),
        ..Default::default()
    });

    // Queue 1: text + thinking
    faux.set_responses(vec![FauxResponseStep::Static(faux_assistant_message(
        vec![
            faux_thinking("The user is asking about Rust's core innovation."),
            faux_text(
                "Rust's ownership system enforces memory safety at compile time \
                 without a garbage collector. Every value has exactly one owner; \
                 references borrow temporarily. The compiler checks lifetimes, \
                 preventing use-after-free, double-free, and data races.",
            ),
        ],
        Some(StopReason::Stop),
    ))]);

    models.set_provider(faux.provider);
    models
}

/// Consume events as they arrive (token-by-token).
async fn run_streaming(models: &Models, model: &elph_ai::Model, context: &Context) -> anyhow::Result<()> {
    let mut events = models.stream(model, context, None).into_stream();

    print!("Assistant: ");
    let _ = std::io::stdout().flush();

    while let Some(event) = events.next().await {
        match event {
            AssistantMessageEvent::TextDelta { delta, .. } => {
                print!("{delta}");
                let _ = std::io::stdout().flush();
            }
            AssistantMessageEvent::ThinkingDelta { delta, .. } => {
                eprint!("{delta}");
                let _ = std::io::stderr().flush();
            }
            AssistantMessageEvent::Done { reason, message } => {
                println!();
                print_usage(&message, reason);
            }
            AssistantMessageEvent::Error { error, .. } => {
                anyhow::bail!("stream error: {}", error.error_message.unwrap_or_default());
            }
            _ => {}
        }
    }
    Ok(())
}

/// Wait for the complete response, then print everything at once.
async fn run_buffered(models: &Models, model: &elph_ai::Model, context: &Context) -> anyhow::Result<()> {
    let message = models.complete(model, context, None).await;

    if let Some(err) = &message.error_message {
        anyhow::bail!("error: {err}");
    }

    print!("Assistant: ");
    for block in &message.content {
        match block {
            AssistantContentBlock::Text(t) => print!("{}", t.text),
            AssistantContentBlock::Thinking(t) => eprint!("{}", t.thinking),
            AssistantContentBlock::ToolCall(tc) => {
                println!("\n[Tool call: {}]", tc.name);
            }
        }
    }
    println!();

    // Calculate and display monetary cost from usage
    let mut usage = message.usage.clone();
    calculate_cost(model, &mut usage);
    print_usage(&message, message.stop_reason);
    if usage.cost.total > 0.0 {
        println!("Cost:      ${:.6}", usage.cost.total);
    }

    Ok(())
}

fn print_usage(message: &elph_ai::AssistantMessage, reason: StopReason) {
    println!("Stop:      {reason:?}");
    println!(
        "Tokens:    {} in / {} out / {} cache read / {} cache write ({} total)",
        message.usage.input,
        message.usage.output,
        message.usage.cache_read,
        message.usage.cache_write,
        message.usage.total_tokens
    );
    if let Some(r) = message.usage.reasoning {
        println!("Reasoning: {r}");
    }
}

fn timestamp() -> i64 {
    chrono::Utc::now().timestamp_millis()
}
