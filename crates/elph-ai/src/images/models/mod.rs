use once_cell::sync::Lazy;

use crate::types::{ImagesModel, ModelCost};

#[derive(serde::Deserialize)]
struct RawImageModel {
    id: String,
    name: String,
    api: String,
    provider: String,
    #[serde(rename = "baseUrl")]
    base_url: String,
    input: Vec<String>,
    output: Vec<String>,
    cost: RawCost,
}

#[derive(serde::Deserialize)]
struct RawCost {
    input: f64,
    output: f64,
    #[serde(rename = "cacheRead")]
    cache_read: f64,
    #[serde(rename = "cacheWrite")]
    cache_write: f64,
}

fn parse_image_models(json: &str) -> Vec<ImagesModel> {
    let raw: std::collections::HashMap<String, RawImageModel> =
        serde_json::from_str(json).expect("invalid image model catalog");
    raw.into_values()
        .map(|m| ImagesModel {
            id: m.id,
            name: m.name,
            api: m.api,
            provider: m.provider,
            base_url: m.base_url,
            input: m.input,
            output: m.output,
            cost: ModelCost {
                input: m.cost.input,
                output: m.cost.output,
                cache_read: m.cost.cache_read,
                cache_write: m.cost.cache_write,
            },
            headers: None,
        })
        .collect()
}

pub static OPENROUTER_IMAGE_MODELS: Lazy<Vec<ImagesModel>> =
    Lazy::new(|| parse_image_models(include_str!("openrouter.json")));

pub fn get_builtin_image_models(provider: &str) -> Vec<ImagesModel> {
    match provider {
        "openrouter" => OPENROUTER_IMAGE_MODELS.to_vec(),
        _ => vec![],
    }
}
