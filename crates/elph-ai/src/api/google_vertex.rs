//! Google Vertex AI — shares streaming logic with Google Generative AI via REST.

use crate::api::simple_options::build_base_options;
use crate::models::clamp_thinking_level;
use crate::types::{Context, Model, ProviderStreams, SimpleStreamOptions, StreamOptions};
use crate::utils::event_stream::AssistantMessageEventStream;
use crate::utils::provider_env::get_provider_env_value;

use super::google_generative_ai::{GoogleGenerativeAIApi, GoogleOptions, GoogleThinkingConfig, get_google_budget};

const GCP_VERTEX_CREDENTIALS_MARKER: &str = "gcp-vertex-credentials";

pub struct GoogleVertexApi;

impl ProviderStreams for GoogleVertexApi {
    fn stream(&self, model: &Model, context: &Context, options: Option<StreamOptions>) -> AssistantMessageEventStream {
        GoogleGenerativeAIApi.stream(model, context, options)
    }

    fn stream_simple(
        &self,
        model: &Model,
        context: &Context,
        options: Option<SimpleStreamOptions>,
    ) -> AssistantMessageEventStream {
        let opts = options.as_ref();
        let base = build_base_options(model, context, opts, opts.and_then(|o| o.base.api_key.clone()));
        if opts.and_then(|o| o.reasoning).is_none() {
            return GoogleGenerativeAIApi.stream_with_options(
                model,
                context,
                GoogleOptions {
                    base,
                    thinking: Some(GoogleThinkingConfig {
                        enabled: false,
                        budget_tokens: None,
                        level: None,
                    }),
                    ..Default::default()
                },
            );
        }
        let reasoning = clamp_thinking_level(model, opts.unwrap().reasoning.unwrap());
        GoogleGenerativeAIApi.stream_with_options(
            model,
            context,
            GoogleOptions {
                base,
                thinking: Some(GoogleThinkingConfig {
                    enabled: true,
                    budget_tokens: Some(get_google_budget(model, reasoning)),
                    level: None,
                }),
                ..Default::default()
            },
        )
    }
}

pub fn resolve_project(options: &GoogleOptions) -> Result<String, String> {
    options
        .base
        .env
        .as_ref()
        .and_then(|e| {
            e.get("GOOGLE_CLOUD_PROJECT")
                .or_else(|| e.get("GCLOUD_PROJECT"))
                .cloned()
        })
        .or_else(|| get_provider_env_value("GOOGLE_CLOUD_PROJECT", options.base.env.as_ref()))
        .or_else(|| get_provider_env_value("GCLOUD_PROJECT", options.base.env.as_ref()))
        .ok_or_else(|| "Vertex AI requires a project ID".to_string())
}

pub fn resolve_location(options: &GoogleOptions) -> Result<String, String> {
    options
        .base
        .env
        .as_ref()
        .and_then(|e| e.get("GOOGLE_CLOUD_LOCATION").cloned())
        .or_else(|| get_provider_env_value("GOOGLE_CLOUD_LOCATION", options.base.env.as_ref()))
        .ok_or_else(|| "Vertex AI requires a location".to_string())
}

pub fn resolve_api_key(options: &GoogleOptions) -> Option<String> {
    let api_key = options.base.api_key.as_deref()?.trim();
    if api_key.is_empty() || api_key == GCP_VERTEX_CREDENTIALS_MARKER || api_key.starts_with('<') {
        None
    } else {
        Some(api_key.to_string())
    }
}

pub fn vertex_stream_url(model: &Model, project: &str, location: &str) -> String {
    format!(
        "https://{location}-aiplatform.googleapis.com/v1/projects/{project}/locations/{location}/publishers/google/models/{}:streamGenerateContent?alt=sse",
        model.id
    )
}
