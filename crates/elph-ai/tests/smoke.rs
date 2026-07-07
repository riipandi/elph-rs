use elph_ai::api::codex_transport::{close_codex_websocket_sessions, reset_codex_websocket_debug_stats};
use elph_ai::api::openai_compat::{detect_compat, get_compat};
use elph_ai::api::openai_completions::convert_messages;
use elph_ai::auth::oauth::{
    OAuthProviderInterface, builtin_oauth_provider_ids, get_oauth_provider, get_oauth_providers,
    register_oauth_provider, reset_oauth_providers, unregister_oauth_provider,
};
use elph_ai::{
    AssistantContentBlock, Context, FauxResponseStep, Message, ProviderStreams, UserContent, builtin_models,
    faux_assistant_message, faux_provider, faux_text, get_builtin_model, get_builtin_providers,
};
use serde_json::json;

#[test]
fn builtin_catalog_has_all_providers() {
    let providers = get_builtin_providers();
    assert!(providers.len() >= 35);
    assert!(providers.contains(&"anthropic"));
    assert!(providers.contains(&"openai"));
    assert!(providers.contains(&"openrouter"));
}

#[test]
fn builtin_model_lookup() {
    let model = get_builtin_model("anthropic", "claude-3-5-sonnet-20241022");
    assert!(model.is_some());
    let model = model.unwrap();
    assert_eq!(model.api, "anthropic-messages");
    assert!(model.input.contains(&"text".to_string()));
}

#[test]
fn models_collection_registers_providers() {
    let models = builtin_models(None);
    let providers = models.get_providers();
    assert!(providers.len() >= 35);
    let all = models.get_models(None);
    assert!(all.len() > 500);
}

#[test]
fn detect_compat_matches_pi_ai_defaults() {
    let model = get_builtin_model("deepseek", "deepseek-v4-flash").expect("model exists");
    let compat = detect_compat(&model);
    assert_eq!(compat.thinking_format, "deepseek");
    assert!(!compat.supports_store);
    assert!(compat.requires_reasoning_content_on_assistant_messages);

    let openrouter = get_builtin_model("openrouter", "anthropic/claude-3-haiku").expect("model exists");
    let or_compat = get_compat(&openrouter);
    assert_eq!(or_compat.thinking_format, "openrouter");
    assert_eq!(or_compat.cache_control_format.as_deref(), Some("anthropic"));
}

#[tokio::test]
async fn faux_provider_streams_queued_responses() {
    let faux = faux_provider(Default::default());
    let model = faux.provider.get_models()[0].clone();
    faux.set_responses(vec![FauxResponseStep::Static(faux_assistant_message(
        vec![faux_text("hello faux")],
        None,
    ))]);

    let context = Context {
        system_prompt: None,
        messages: vec![Message::User {
            content: UserContent::Text("ping".to_string()),
            timestamp: 0,
        }],
        tools: None,
    };

    let mut stream = faux.core.api().stream_simple(&model, &context, None).into_stream();
    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event);
    }
    assert!(!events.is_empty());
    assert!(
        events
            .iter()
            .any(|e| matches!(e, elph_ai::AssistantMessageEvent::Done { .. }))
    );
    assert_eq!(faux.pending_count(), 0);
}

#[test]
fn oauth_registry_lists_builtin_providers() {
    reset_oauth_providers();
    let providers = get_oauth_providers();
    assert_eq!(providers.len(), 3);
    for id in builtin_oauth_provider_ids() {
        assert!(get_oauth_provider(id).is_some(), "missing provider {id}");
    }
}

#[test]
fn oauth_registry_register_and_unregister_custom_provider() {
    reset_oauth_providers();
    register_oauth_provider(OAuthProviderInterface {
        id: "custom-oauth".to_string(),
        name: "Custom".to_string(),
        auth: elph_ai::anthropic_oauth(),
        get_api_key: std::sync::Arc::new(|c| c.access.clone()),
        modify_models: None,
    });
    assert!(get_oauth_provider("custom-oauth").is_some());
    unregister_oauth_provider("custom-oauth");
    assert!(get_oauth_provider("custom-oauth").is_none());
    unregister_oauth_provider("anthropic");
    assert_eq!(get_oauth_provider("anthropic").unwrap().id, "anthropic");
}

#[test]
fn convert_messages_developer_role_and_reasoning_content() {
    let model = get_builtin_model("deepseek", "deepseek-v4-flash").expect("model exists");
    let compat = get_compat(&model);
    let context = Context {
        system_prompt: Some("system".to_string()),
        messages: vec![Message::Assistant(elph_ai::AssistantMessage {
            role: "assistant".to_string(),
            content: vec![AssistantContentBlock::Text(elph_ai::TextContent::new("hi"))],
            api: model.api.clone(),
            provider: model.provider.clone(),
            model: model.id.clone(),
            usage: Default::default(),
            stop_reason: elph_ai::StopReason::Stop,
            timestamp: 0,
            response_id: None,
            response_model: None,
            error_message: None,
        })],
        tools: None,
    };
    let messages = convert_messages(&model, &context, &compat);
    let mut openai = get_builtin_model("openai", "o3-mini").expect("model exists");
    openai.reasoning = true;
    let openai_compat = get_compat(&openai);
    let openai_messages = convert_messages(
        &openai,
        &Context {
            system_prompt: Some("system".to_string()),
            messages: vec![],
            tools: None,
        },
        &openai_compat,
    );
    assert_eq!(
        openai_messages[0].get("role").and_then(|v| v.as_str()),
        Some("developer")
    );
    assert_eq!(
        messages
            .iter()
            .find(|m| m.get("role") == Some(&json!("assistant")))
            .and_then(|m| m.get("reasoning_content"))
            .and_then(|v| v.as_str()),
        Some("")
    );
}

#[test]
fn convert_messages_groups_tool_results_with_images() {
    let model = get_builtin_model("openai", "gpt-4o").expect("model exists");
    let compat = get_compat(&model);
    let context = Context {
        system_prompt: None,
        messages: vec![
            Message::ToolResult {
                tool_call_id: "call_1".to_string(),
                tool_name: "screenshot".to_string(),
                content: vec![elph_ai::ContentBlock::Image {
                    data: "aGVsbG8=".to_string(),
                    mime_type: "image/png".to_string(),
                }],
                details: None,
                is_error: false,
                timestamp: 0,
            },
            Message::User {
                content: UserContent::Text("next".to_string()),
                timestamp: 1,
            },
        ],
        tools: None,
    };
    let messages = convert_messages(&model, &context, &compat);
    let tool_msg = messages
        .iter()
        .find(|m| m.get("role") == Some(&json!("tool")))
        .expect("tool result");
    assert_eq!(
        tool_msg.get("content").and_then(|v| v.as_str()),
        Some("(see attached image)")
    );
    let image_user = messages
        .iter()
        .find(|m| {
            m.get("role") == Some(&json!("user"))
                && m.pointer("/content/0/text")
                    .and_then(|v| v.as_str())
                    .map(|s| s.contains("Attached image"))
                    .unwrap_or(false)
        })
        .expect("image follow-up user message");
    assert!(image_user.pointer("/content/1/type").and_then(|v| v.as_str()) == Some("image_url"));
}

#[test]
fn codex_websocket_debug_stats_reset() {
    close_codex_websocket_sessions(Some("session-test"));
    reset_codex_websocket_debug_stats(Some("session-test"));
    assert!(elph_ai::get_codex_websocket_debug_stats("session-test").is_none());
}

#[tokio::test]
async fn stream_without_api_key_returns_error_message() {
    let models = builtin_models(None);
    let model = get_builtin_model("openai", "gpt-4o-mini").expect("model exists");
    let context = Context {
        system_prompt: Some("You are helpful.".to_string()),
        messages: vec![Message::User {
            content: UserContent::Text("Hello".to_string()),
            timestamp: 0,
        }],
        tools: None,
    };
    let response = models.complete(&model, &context, None).await;
    assert!(
        response.error_message.is_some() || response.stop_reason == elph_ai::StopReason::Error,
        "expected auth/network error without API key"
    );
}
