use std::sync::Arc;

use crate::api::faux::FauxCore;
use crate::auth::types::{AuthResolveInput, AuthResult, ModelAuth, ProviderAuth};
use crate::models::create_provider;
use crate::models::{CreateProviderOptions, Provider, ProviderApi};
use crate::providers::adapter::faux_api;

pub use crate::api::faux::RegisterFauxProviderOptions;
pub use crate::api::faux::{FauxApi, FauxModelDefinition, FauxResponseFactory, FauxResponseStep, FauxState};
pub use crate::api::faux::{faux_assistant_message, faux_text, faux_thinking, faux_tool_call};

pub struct FauxProviderHandle {
    pub provider: Provider,
    pub core: Arc<FauxCore>,
}

pub fn faux_provider(options: RegisterFauxProviderOptions) -> FauxProviderHandle {
    let core = Arc::new(FauxCore::new(options));
    let provider = create_provider(CreateProviderOptions {
        id: core.provider.clone(),
        name: Some("Faux".to_string()),
        base_url: None,
        headers: None,
        auth: ProviderAuth {
            api_key: Some(crate::auth::ApiKeyAuth {
                name: "Faux".to_string(),
                resolve: Arc::new(|_input: AuthResolveInput| {
                    Box::pin(async {
                        Some(AuthResult {
                            auth: ModelAuth {
                                api_key: None,
                                headers: None,
                                base_url: None,
                            },
                            env: None,
                            source: Some("faux".to_string()),
                        })
                    })
                }),
                login: None,
            }),
            oauth: None,
        },
        models: core.models.clone(),
        refresh_models: None,
        api: ProviderApi::Single(faux_api(core.api())),
    });
    FauxProviderHandle { provider, core }
}

impl FauxProviderHandle {
    pub fn set_responses(&self, responses: Vec<FauxResponseStep>) {
        self.core.set_responses(responses);
    }

    pub fn append_responses(&self, responses: Vec<FauxResponseStep>) {
        self.core.append_responses(responses);
    }

    pub fn pending_count(&self) -> usize {
        self.core.pending_count()
    }
}
