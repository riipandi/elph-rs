pub mod context;
pub mod credential_store;
pub mod helpers;
pub mod oauth;
pub mod resolve;
pub mod types;

pub use context::{DefaultAuthContext, default_auth_context};
pub use credential_store::InMemoryCredentialStore;
pub use helpers::{OAuthLoader, env_api_key_auth, lazy_oauth};
pub use oauth::{
    OAuthApiKeyResult, OAuthProviderInterface, OPENAI_CODEX_BROWSER_LOGIN_METHOD,
    OPENAI_CODEX_DEVICE_CODE_LOGIN_METHOD, anthropic_oauth, builtin_oauth_provider_ids, get_oauth_api_key,
    get_oauth_provider, get_oauth_providers, github_copilot_oauth, oauth_provider_modify_models, openai_codex_oauth,
    refresh_oauth_token, register_oauth_provider, reset_oauth_providers, unregister_oauth_provider,
};
pub use resolve::{AuthResolutionOverrides, ModelsError, ModelsErrorCode, ProviderAuthHolder, resolve_provider_auth};
pub use types::*;
