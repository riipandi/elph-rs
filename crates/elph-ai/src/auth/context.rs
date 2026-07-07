use std::env;
use std::path::Path;

use super::types::AuthContext;

/// Default auth context: environment variables and filesystem checks.
pub struct DefaultAuthContext;

impl DefaultAuthContext {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultAuthContext {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl AuthContext for DefaultAuthContext {
    async fn env(&self, name: &str) -> Option<String> {
        env::var(name).ok().filter(|v| !v.trim().is_empty())
    }

    async fn file_exists(&self, path: &str) -> bool {
        let home = env::var_os("HOME").map(std::path::PathBuf::from);
        let resolved = if let Some(rest) = path.strip_prefix("~/") {
            home.map(|h| h.join(rest))
                .unwrap_or_else(|| Path::new(path).to_path_buf())
        } else if path == "~" {
            home.unwrap_or_else(|| Path::new(path).to_path_buf())
        } else {
            Path::new(path).to_path_buf()
        };
        resolved.exists()
    }
}

pub fn default_auth_context() -> DefaultAuthContext {
    DefaultAuthContext::new()
}
