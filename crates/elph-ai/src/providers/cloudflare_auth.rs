use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::auth::types::{ApiKeyAuth, AuthModel, AuthResolveInput, AuthResult, ModelAuth};

const CLOUDFLARE_API_KEY: &str = "CLOUDFLARE_API_KEY";
const CLOUDFLARE_ACCOUNT_ID: &str = "CLOUDFLARE_ACCOUNT_ID";
const CLOUDFLARE_GATEWAY_ID: &str = "CLOUDFLARE_GATEWAY_ID";

#[derive(Clone, Copy)]
enum CloudflareAuthKind {
    WorkersAi,
    AiGateway,
}

async fn resolve_value(name: &str, input: &AuthResolveInput) -> Option<String> {
    if let Some(cred) = &input.credential {
        if name == CLOUDFLARE_API_KEY {
            return cred.key.clone();
        }
        return cred.env.as_ref().and_then(|e| e.get(name).cloned());
    }
    input.ctx.env(name).await
}

fn resolve_cloudflare_base_url(model_base_url: &str, account_id: &str, gateway_id: Option<&str>) -> String {
    model_base_url
        .replace(&format!("{{{CLOUDFLARE_ACCOUNT_ID}}}"), account_id)
        .replace(&format!("{{{CLOUDFLARE_GATEWAY_ID}}}"), gateway_id.unwrap_or(""))
}

fn model_base_url(model: &AuthModel) -> &str {
    match model {
        AuthModel::Chat(m) => &m.base_url,
        AuthModel::Images(m) => &m.base_url,
    }
}

async fn resolve_cloudflare_env(kind: CloudflareAuthKind, input: AuthResolveInput) -> Option<AuthResult> {
    let api_key = resolve_value(CLOUDFLARE_API_KEY, &input).await?;
    let account_id = resolve_value(CLOUDFLARE_ACCOUNT_ID, &input).await?;
    let gateway_id = match kind {
        CloudflareAuthKind::AiGateway => Some(resolve_value(CLOUDFLARE_GATEWAY_ID, &input).await?),
        CloudflareAuthKind::WorkersAi => None,
    };

    let base_url = resolve_cloudflare_base_url(model_base_url(&input.model), &account_id, gateway_id.as_deref());

    let mut env = std::collections::HashMap::from([(CLOUDFLARE_ACCOUNT_ID.to_string(), account_id)]);
    if let Some(gw) = gateway_id {
        env.insert(CLOUDFLARE_GATEWAY_ID.to_string(), gw);
    }

    let source = if input.credential.is_some() {
        "stored credential".to_string()
    } else {
        CLOUDFLARE_API_KEY.to_string()
    };

    let auth = match kind {
        CloudflareAuthKind::WorkersAi => ModelAuth {
            api_key: Some(api_key),
            headers: None,
            base_url: Some(base_url),
        },
        CloudflareAuthKind::AiGateway => {
            let mut headers = std::collections::HashMap::new();
            headers.insert("cf-aig-authorization".to_string(), Some(format!("Bearer {api_key}")));
            headers.insert("Authorization".to_string(), None);
            headers.insert("x-api-key".to_string(), None);
            ModelAuth {
                api_key: None,
                headers: Some(headers),
                base_url: Some(base_url),
            }
        }
    };

    Some(AuthResult {
        auth,
        env: Some(env),
        source: Some(source),
    })
}

fn cloudflare_auth(kind: CloudflareAuthKind, name: &'static str) -> ApiKeyAuth {
    ApiKeyAuth {
        name: name.to_string(),
        resolve: Arc::new(move |input| {
            Box::pin(resolve_cloudflare_env(kind, input)) as Pin<Box<dyn Future<Output = Option<AuthResult>> + Send>>
        }),
        login: None,
    }
}

pub fn cloudflare_workers_ai_auth() -> ApiKeyAuth {
    cloudflare_auth(CloudflareAuthKind::WorkersAi, "Cloudflare API key")
}

pub fn cloudflare_ai_gateway_auth() -> ApiKeyAuth {
    cloudflare_auth(CloudflareAuthKind::AiGateway, "Cloudflare API key")
}
