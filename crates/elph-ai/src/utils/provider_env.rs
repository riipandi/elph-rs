use crate::types::ProviderEnv;

pub fn get_provider_env_value(key: &str, env: Option<&ProviderEnv>) -> Option<String> {
    if let Some(env) = env {
        if let Some(v) = env.get(key) {
            return Some(v.clone());
        }
    }
    std::env::var(key).ok()
}
