//! Browse built-in models: list providers, filter by capability, show pricing.
//!
//! No API keys needed — reads from the embedded model catalog.
//!
//! ```bash
//! # All providers:
//! cargo run -p elph-ai --example browse_models
//!
//! # Filter models supporting reasoning:
//! cargo run -p elph-ai --example browse_models -- --reasoning
//!
//! # Show details for a specific provider:
//! cargo run -p elph-ai --example browse_models -- --provider openai
//! ```

use elph_ai::get_supported_thinking_levels;
use elph_ai::{Model, ThinkingLevel};
use elph_ai::{clamp_thinking_level, get_builtin_model, get_builtin_models, get_builtin_providers};

struct Args {
    reasoning_only: bool,
    provider_filter: Option<String>,
}

#[derive(Default)]
struct CatalogStats {
    total_models: usize,
    reasoning_models: usize,
    max_context: u32,
}

fn main() {
    let args = parse_args();
    let providers = get_builtin_providers();

    println!("Built-in model catalog");
    println!("Providers: {}\n", providers.len());

    let mut total = CatalogStats::default();

    for pid in &providers {
        // Skip non-matching providers when filtered
        if let Some(ref filter) = args.provider_filter
            && !pid.contains(filter)
        {
            continue;
        }

        let models = get_builtin_models(pid);
        if models.is_empty() {
            continue;
        }

        // Filter by reasoning capability
        let filtered: Vec<&Model> = models
            .iter()
            .filter(|m| if args.reasoning_only { m.reasoning } else { true })
            .collect();

        if filtered.is_empty() {
            continue;
        }

        total.total_models += models.len();
        total.reasoning_models += models.iter().filter(|m| m.reasoning).count();
        total.max_context = total
            .max_context
            .max(filtered.iter().map(|m| m.context_window).max().unwrap_or(0));

        println!("── {pid} ──");
        println!("  Models: {} ({} reasoning)", models.len(), {
            let n = models.iter().filter(|m| m.reasoning).count();
            if n > 0 { n.to_string() } else { "—".into() }
        });

        // Show up to 5 filtered models
        for m in filtered.iter().take(5) {
            let reasoning_badge = if m.reasoning { " 🧠" } else { "" };
            let context = format_k(m.context_window);
            let cost = format_cost(m);

            print!("    • {} ({})", m.name, m.id);
            println!("  ctx: {context}  cost: {cost}{reasoning_badge}");

            // Show thinking levels for reasoning models
            if m.reasoning && !args.reasoning_only {
                let levels = get_supported_thinking_levels(m);
                if !levels.is_empty() {
                    let clamped = clamp_thinking_level(m, ThinkingLevel::High);
                    println!(
                        "      thinking: {}  (clamped High → {:?})",
                        levels.iter().map(|l| format!("{l:?}")).collect::<Vec<_>>().join(", "),
                        clamped
                    );
                }
            }
        }

        if filtered.len() > 5 {
            println!("    … and {} more", filtered.len() - 5);
        }
        println!();
    }

    // Summary
    if let Some(ref filter) = args.provider_filter {
        let pid = providers.iter().find(|p| p.contains(filter.as_str()));
        if let Some(pid) = pid {
            let models = get_builtin_models(pid);
            let cheapest = models
                .iter()
                .min_by(|a, b| a.cost.input.partial_cmp(&b.cost.input).unwrap());
            if let Some(c) = cheapest {
                println!("Cheapest on {pid}: {} @ ${:.2}/M input", c.id, c.cost.input);
            }
            let search = get_builtin_model(pid, &models[0].id);
            if let Some(s) = search {
                println!("Lookup {}/{}: ✓", pid, s.id);
            }
        }
    }

    println!("────────────────────────────────");
    println!(
        "Total: {} models ({} reasoning), max context {}",
        total.total_models,
        total.reasoning_models,
        format_k(total.max_context)
    );
}

fn format_k(n: u32) -> String {
    if n >= 1_000_000 {
        format!("{}M", n / 1_000_000)
    } else if n >= 1_000 {
        format!("{}K", n / 1_000)
    } else {
        n.to_string()
    }
}

fn format_cost(m: &Model) -> String {
    if m.cost.input == 0.0 && m.cost.output == 0.0 {
        "free".into()
    } else {
        format!("${:.2}i/${:.2}o", m.cost.input, m.cost.output)
    }
}

fn parse_args() -> Args {
    let mut reasoning_only = false;
    let mut provider_filter = None;
    let raw: Vec<String> = std::env::args().skip(1).collect();
    let mut i = 0;
    while i < raw.len() {
        match raw[i].as_str() {
            "--reasoning" => reasoning_only = true,
            "--provider" => {
                i += 1;
                if let Some(val) = raw.get(i) {
                    provider_filter = Some(val.clone());
                }
            }
            _ => {}
        }
        i += 1;
    }
    Args {
        reasoning_only,
        provider_filter,
    }
}
