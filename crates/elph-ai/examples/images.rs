//! Generate an image using the built-in OpenRouter image provider.
//!
//! ```bash
//! export OPENROUTER_API_KEY="your-key"
//! cargo run -p elph-ai --example images
//! ```

use elph_ai::builtin_images_models;
use elph_ai::{ContentBlock, ImagesContext};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let models = builtin_images_models(None);
    let providers = models.get_providers();

    println!("Image providers: {}", providers.len());
    for p in &providers {
        println!("  {} ({} models)", p.name, p.get_models().len());
    }
    println!();

    // Pick first model
    let model = match models.get_models(None).into_iter().next() {
        Some(m) => m,
        None => anyhow::bail!("no image models available"),
    };

    println!("Model: {} ({})", model.name, model.id);
    println!("API:   {}", model.api);

    // Resolve auth
    let auth = models.get_auth(&model).await?;
    let Some(ref auth) = auth else {
        anyhow::bail!("OpenRouter not configured. Set OPENROUTER_API_KEY");
    };
    println!("Auth:  {}", auth.source.as_deref().unwrap_or("unknown"));
    println!();

    // Generate
    let context = ImagesContext {
        input: vec![ContentBlock::Text {
            text: "A cute cat wearing a wizard hat, digital art".into(),
        }],
    };

    println!("Generating...");
    let result = models.generate_images(&model, &context, None).await;

    if let Some(err) = &result.error_message {
        anyhow::bail!("generation failed: {err}");
    }

    println!("Stop:  {:?}", result.stop_reason);
    println!("Model: {}", result.model);
    println!("Output blocks: {}", result.output.len());
    for (i, block) in result.output.iter().enumerate() {
        match block {
            ContentBlock::Text { text } => {
                println!("  [{i}] text ({})", text.len());
            }
            ContentBlock::Image { data, mime_type } => {
                println!("  [{i}] image ({mime_type}, {} bytes)", data.len());
            }
        }
    }

    Ok(())
}
