use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use tokio::sync::Mutex;

use super::types::{Credential, CredentialStore};

/// In-memory credential store with per-provider serialized writes.
pub struct InMemoryCredentialStore {
    credentials: Mutex<HashMap<String, Credential>>,
    chains: Mutex<HashMap<String, Arc<Mutex<()>>>>,
}

impl Default for InMemoryCredentialStore {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemoryCredentialStore {
    pub fn new() -> Self {
        Self {
            credentials: Mutex::new(HashMap::new()),
            chains: Mutex::new(HashMap::new()),
        }
    }

    async fn lock_chain(&self, provider_id: &str) -> Arc<Mutex<()>> {
        let mut chains = self.chains.lock().await;
        chains
            .entry(provider_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone()
    }
}

#[async_trait::async_trait]
impl CredentialStore for InMemoryCredentialStore {
    async fn read(&self, provider_id: &str) -> Option<Credential> {
        self.credentials.lock().await.get(provider_id).cloned()
    }

    async fn modify(
        &self,
        provider_id: &str,
        f: Box<dyn FnOnce(Option<Credential>) -> Pin<Box<dyn Future<Output = Option<Credential>> + Send>> + Send>,
    ) -> Option<Credential> {
        let chain = self.lock_chain(provider_id).await;
        let _guard = chain.lock().await;
        let current = self.credentials.lock().await.get(provider_id).cloned();
        let next = f(current).await;
        if let Some(ref cred) = next {
            self.credentials
                .lock()
                .await
                .insert(provider_id.to_string(), cred.clone());
        }
        if next.is_some() {
            next
        } else {
            self.credentials.lock().await.get(provider_id).cloned()
        }
    }

    async fn delete(&self, provider_id: &str) {
        let chain = self.lock_chain(provider_id).await;
        let _guard = chain.lock().await;
        self.credentials.lock().await.remove(provider_id);
    }
}
