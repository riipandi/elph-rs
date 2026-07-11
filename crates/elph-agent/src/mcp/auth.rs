//! OAuth 2.1 credential storage and authorization helpers for remote MCP servers.
//!
//! Credentials for all MCP servers live in a single JSON file (default name
//! [`DEFAULT_AUTH_FILE_NAME`] = `auth.json`). Values are stored as AES-256-GCM
//! ciphertext with the [`crate::mcp::crypto::ENC_PREFIX`] (`enc:`) prefix.
//!
//! The path is **not** hardcoded to `~/.elph` — hosts (elph, eclaw, owly, …) pass
//! it via [`AuthStorePathBuilder`] / [`McpLoadOptions::auth_store_path`](super::config::McpLoadOptions).

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use rmcp::transport::auth::{
    AuthError, AuthorizationManager, AuthorizationSession, CredentialStore, StoredCredentials,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::RwLock;
use tracing::{info, warn};

use super::crypto::{
    Aes256Key, ENC_PREFIX, decrypt_json_async, default_auth_key_path, encrypt_json_async, is_encrypted_value,
};
use super::store_lock::{atomic_write_private, lock_auth_store};

/// Default OAuth scopes when the server does not advertise any.
pub const DEFAULT_OAUTH_SCOPES: &[&str] = &[];

/// Default credential store filename (joined under a host-provided config dir).
pub const DEFAULT_AUTH_FILE_NAME: &str = "auth.json";

// ---------------------------------------------------------------------------
// Path resolution (host-agnostic)
// ---------------------------------------------------------------------------

/// Builds a filesystem path for the shared auth store file.
///
/// # Examples
///
/// ```
/// use std::path::PathBuf;
/// use elph_agent::AuthStorePathBuilder;
///
/// let path = AuthStorePathBuilder::new()
///     .base_dir("/home/user/.elph")
///     .build();
/// assert_eq!(path, PathBuf::from("/home/user/.elph/auth.json"));
/// ```
#[derive(Debug, Clone)]
pub struct AuthStorePathBuilder {
    base_dir: Option<PathBuf>,
    file_name: String,
    path: Option<PathBuf>,
}

impl Default for AuthStorePathBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthStorePathBuilder {
    pub fn new() -> Self {
        Self {
            base_dir: None,
            file_name: DEFAULT_AUTH_FILE_NAME.to_string(),
            path: None,
        }
    }

    pub fn base_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.base_dir = Some(dir.into());
        self
    }

    pub fn file_name(mut self, name: impl Into<String>) -> Self {
        self.file_name = name.into();
        self
    }

    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn build(self) -> PathBuf {
        if let Some(path) = self.path {
            return path;
        }
        if let Some(base) = self.base_dir {
            return base.join(self.file_name);
        }
        PathBuf::from(self.file_name)
    }
}

/// Convenience: `config_dir/auth.json` using the default filename.
pub fn auth_store_path(config_dir: &Path) -> PathBuf {
    AuthStorePathBuilder::new().base_dir(config_dir).build()
}

/// Prefer [`auth_store_path`]. Thin alias for older call sites.
#[deprecated(note = "use auth_store_path(config_dir) — single auth.json, not a mcp-auth directory")]
pub fn mcp_auth_dir(config_dir: &Path) -> PathBuf {
    auth_store_path(config_dir)
}

// ---------------------------------------------------------------------------
// On-disk format (multi-server, encrypted entries)
// ---------------------------------------------------------------------------

/// Root document of `auth.json` as stored on disk.
///
/// Each MCP server entry is either:
/// - an encrypted string: `"enc:…"` (AES-256-GCM, preferred)
/// - a plain object (legacy / migration only — re-encrypted on next save)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStoreFile {
    /// Map of MCP server name → encrypted string or plain JSON object.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub mcp: BTreeMap<String, Value>,
}

impl AuthStoreFile {
    pub async fn load_from_path(path: &Path) -> Result<Self, AuthError> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = tokio::fs::read(path)
            .await
            .map_err(|e| AuthError::InternalError(format!("read auth store: {e}")))?;
        if bytes.is_empty() {
            return Ok(Self::default());
        }
        serde_json::from_slice(&bytes).map_err(|e| AuthError::InternalError(format!("parse auth store: {e}")))
    }

    /// Save without taking the store lock (caller must hold [`lock_auth_store`]).
    pub async fn save_to_path_unlocked(&self, path: &Path) -> Result<(), AuthError> {
        let bytes = serde_json::to_vec_pretty(self)
            .map_err(|e| AuthError::InternalError(format!("serialize auth store: {e}")))?;
        atomic_write_private(path, &bytes)
            .await
            .map_err(|e| AuthError::InternalError(e.to_string()))?;
        Ok(())
    }

    /// Lock the store, then atomic-write.
    pub async fn save_to_path(&self, path: &Path) -> Result<(), AuthError> {
        let _guard = lock_auth_store(path)
            .await
            .map_err(|e| AuthError::InternalError(format!("lock auth store: {e}")))?;
        self.save_to_path_unlocked(path).await
    }

    pub fn contains_server(&self, server_name: &str) -> bool {
        self.mcp.contains_key(server_name)
    }
}

// ---------------------------------------------------------------------------
// Per-server CredentialStore backed by shared encrypted auth.json
// ---------------------------------------------------------------------------

#[derive(Clone)]
enum CryptoSource {
    /// Load or create key at this path on first use.
    AutoKeyFile(PathBuf),
    /// Explicit key provided by host.
    Key(Arc<Aes256Key>),
}

/// File-backed [`CredentialStore`] for **one** MCP server key inside a shared `auth.json`.
///
/// On-disk values are AES-256-GCM encrypted strings with the `enc:` prefix.
#[derive(Clone)]
pub struct FileCredentialStore {
    path: PathBuf,
    server_key: String,
    crypto: CryptoSource,
    key_cache: Arc<RwLock<Option<Arc<Aes256Key>>>>,
    cache: Arc<RwLock<Option<StoredCredentials>>>,
}

impl FileCredentialStore {
    /// Create a store for `server_key` inside the shared file at `path`.
    ///
    /// Encryption key is loaded/created at [`default_auth_key_path`](path) (`auth.key`).
    pub fn new(path: impl Into<PathBuf>, server_key: impl Into<String>) -> Self {
        let path = path.into();
        let key_path = default_auth_key_path(&path);
        Self {
            path,
            server_key: server_key.into(),
            crypto: CryptoSource::AutoKeyFile(key_path),
            key_cache: Arc::new(RwLock::new(None)),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    /// Use an explicit AES key (hosts / tests).
    pub fn with_key(path: impl Into<PathBuf>, server_key: impl Into<String>, key: Aes256Key) -> Self {
        Self {
            path: path.into(),
            server_key: server_key.into(),
            crypto: CryptoSource::Key(Arc::new(key)),
            key_cache: Arc::new(RwLock::new(None)),
            cache: Arc::new(RwLock::new(None)),
        }
    }

    pub fn builder() -> FileCredentialStoreBuilder {
        FileCredentialStoreBuilder::new()
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn server_key(&self) -> &str {
        &self.server_key
    }

    async fn resolve_key(&self) -> Result<Arc<Aes256Key>, AuthError> {
        {
            let cache = self.key_cache.read().await;
            if let Some(k) = cache.as_ref() {
                return Ok(Arc::clone(k));
            }
        }
        let key = match &self.crypto {
            CryptoSource::Key(k) => Arc::clone(k),
            CryptoSource::AutoKeyFile(key_path) => {
                let loaded = Aes256Key::load_or_create(key_path.clone())
                    .await
                    .map_err(|e| AuthError::InternalError(format!("auth key: {e}")))?;
                Arc::new(loaded)
            }
        };
        *self.key_cache.write().await = Some(Arc::clone(&key));
        Ok(key)
    }

    async fn load_entry(&self) -> Result<Option<StoredCredentials>, AuthError> {
        // Shared lock so we don't read a half-written file during refresh.
        let _guard = lock_auth_store(&self.path)
            .await
            .map_err(|e| AuthError::InternalError(format!("lock auth store: {e}")))?;
        let file = AuthStoreFile::load_from_path(&self.path).await?;
        let Some(value) = file.mcp.get(&self.server_key) else {
            return Ok(None);
        };
        let key = self.resolve_key().await?;
        decode_entry(key, value).await
    }

    /// Load-merge-save under exclusive lock (safe for concurrent token refresh).
    async fn write_entry(&self, credentials: Option<StoredCredentials>) -> Result<(), AuthError> {
        let _guard = lock_auth_store(&self.path)
            .await
            .map_err(|e| AuthError::InternalError(format!("lock auth store: {e}")))?;
        // Re-read under lock so concurrent writers (other servers / refresh) are not lost.
        let mut file = AuthStoreFile::load_from_path(&self.path).await?;
        match credentials {
            Some(creds) => {
                let key = self.resolve_key().await?;
                let enc = encrypt_json_async(key, creds)
                    .await
                    .map_err(|e| AuthError::InternalError(format!("encrypt credentials: {e}")))?;
                debug_assert!(is_encrypted_value(&enc));
                file.mcp.insert(self.server_key.clone(), Value::String(enc));
            }
            None => {
                file.mcp.remove(&self.server_key);
            }
        }
        file.save_to_path_unlocked(&self.path).await?;
        Ok(())
    }
}

async fn decode_entry(key: Arc<Aes256Key>, value: &Value) -> Result<Option<StoredCredentials>, AuthError> {
    match value {
        Value::String(s) if is_encrypted_value(s) => {
            let creds: StoredCredentials = decrypt_json_async(key, s.clone())
                .await
                .map_err(|e| AuthError::InternalError(format!("decrypt credentials: {e}")))?;
            Ok(Some(creds))
        }
        Value::String(s) => Err(AuthError::InternalError(format!(
            "credential string must start with {ENC_PREFIX}, got prefix {:?}",
            s.chars().take(8).collect::<String>()
        ))),
        Value::Object(_) => {
            // Legacy plaintext object — accept and let next save re-encrypt.
            let creds: StoredCredentials = serde_json::from_value(value.clone())
                .map_err(|e| AuthError::InternalError(format!("parse plain credentials: {e}")))?;
            Ok(Some(creds))
        }
        Value::Null => Ok(None),
        other => Err(AuthError::InternalError(format!(
            "unexpected credential entry type: {}",
            other
        ))),
    }
}

/// Builder for [`FileCredentialStore`].
#[derive(Debug, Clone, Default)]
pub struct FileCredentialStoreBuilder {
    path_builder: AuthStorePathBuilder,
    server_key: Option<String>,
    key: Option<Aes256Key>,
    key_path: Option<PathBuf>,
}

impl FileCredentialStoreBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn base_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.path_builder = self.path_builder.base_dir(dir);
        self
    }

    pub fn file_name(mut self, name: impl Into<String>) -> Self {
        self.path_builder = self.path_builder.file_name(name);
        self
    }

    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path_builder = self.path_builder.path(path);
        self
    }

    pub fn server_key(mut self, key: impl Into<String>) -> Self {
        self.server_key = Some(key.into());
        self
    }

    /// Explicit AES-256 key material.
    pub fn encryption_key(mut self, key: Aes256Key) -> Self {
        self.key = Some(key);
        self
    }

    /// Override path of the key file (default: sibling `auth.key`).
    pub fn key_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.key_path = Some(path.into());
        self
    }

    pub fn build(self) -> Result<FileCredentialStore> {
        let server_key = self
            .server_key
            .filter(|s| !s.trim().is_empty())
            .context("FileCredentialStore requires a non-empty server_key")?;
        let path = self.path_builder.build();
        if let Some(key) = self.key {
            return Ok(FileCredentialStore::with_key(path, server_key, key));
        }
        let mut store = FileCredentialStore::new(path, server_key);
        if let Some(key_path) = self.key_path {
            store.crypto = CryptoSource::AutoKeyFile(key_path);
        }
        Ok(store)
    }
}

#[async_trait]
impl CredentialStore for FileCredentialStore {
    async fn load(&self) -> Result<Option<StoredCredentials>, AuthError> {
        {
            let cache = self.cache.read().await;
            if cache.is_some() {
                return Ok(cache.clone());
            }
        }
        let loaded = self.load_entry().await?;
        *self.cache.write().await = loaded.clone();
        Ok(loaded)
    }

    async fn save(&self, credentials: StoredCredentials) -> Result<(), AuthError> {
        self.write_entry(Some(credentials.clone())).await?;
        *self.cache.write().await = Some(credentials);
        Ok(())
    }

    async fn clear(&self) -> Result<(), AuthError> {
        *self.cache.write().await = None;
        self.write_entry(None).await?;
        let _guard = lock_auth_store(&self.path)
            .await
            .map_err(|e| AuthError::InternalError(format!("lock auth store: {e}")))?;
        if let Ok(file) = AuthStoreFile::load_from_path(&self.path).await
            && file.mcp.is_empty()
            && self.path.exists()
        {
            let _ = tokio::fs::remove_file(&self.path).await;
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Public helpers
// ---------------------------------------------------------------------------

/// True when `auth.json` contains an entry for `server_name` (encrypted or plain).
pub fn has_stored_credentials(auth_store_path: &Path, server_name: &str) -> bool {
    // Sync probe for CLI; best-effort read without decrypt.
    if !auth_store_path.exists() {
        return false;
    }
    let Ok(bytes) = std::fs::read(auth_store_path) else {
        return false;
    };
    let Ok(file) = serde_json::from_slice::<AuthStoreFile>(&bytes) else {
        return false;
    };
    file.contains_server(server_name)
}

/// Remove stored OAuth credentials for a server from the shared store.
pub async fn clear_credentials(auth_store_path: &Path, server_name: &str) -> Result<bool> {
    if !has_stored_credentials(auth_store_path, server_name) {
        return Ok(false);
    }
    let store = FileCredentialStore::new(auth_store_path, server_name);
    store
        .clear()
        .await
        .map_err(|e| anyhow::anyhow!("clear credentials: {e}"))?;
    Ok(true)
}

/// Result of an interactive OAuth authorization flow.
#[derive(Debug)]
pub struct McpOAuthFlowResult {
    pub server_name: String,
    pub credentials_path: PathBuf,
    pub client_id: String,
}

/// Run the OAuth 2.1 authorization-code + PKCE flow for an MCP HTTP server URL.
pub async fn run_oauth_flow(
    server_name: &str,
    server_url: &str,
    auth_store_path: &Path,
    scopes: &[&str],
) -> Result<McpOAuthFlowResult> {
    let store = FileCredentialStore::new(auth_store_path, server_name);

    let mut manager = AuthorizationManager::new(server_url)
        .await
        .map_err(|e| anyhow::anyhow!("init OAuth manager: {e}"))?;
    manager.set_credential_store(store);

    let metadata = manager
        .discover_metadata()
        .await
        .map_err(|e| anyhow::anyhow!("discover OAuth metadata: {e}"))?;
    manager.set_metadata(metadata);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .context("bind OAuth callback listener")?;
    let port = listener.local_addr()?.port();
    let redirect_uri = format!("http://127.0.0.1:{port}/callback");

    let session = AuthorizationSession::new(manager, scopes, &redirect_uri, Some("Elph MCP Client"), None)
        .await
        .map_err(|e| anyhow::anyhow!("start OAuth session: {e}"))?;

    let auth_url = session.get_authorization_url().to_string();
    info!(%server_name, %auth_url, "opening browser for MCP OAuth");
    println!("Open this URL to authorize MCP server '{server_name}':\n  {auth_url}\n");
    if let Err(error) = open_browser(&auth_url) {
        warn!(%error, "failed to open browser; paste the URL manually");
    }

    let callback_url = wait_for_oauth_callback(listener)
        .await
        .context("wait for OAuth callback")?;
    let _token = session
        .handle_callback_url(&callback_url)
        .await
        .map_err(|e| anyhow::anyhow!("OAuth token exchange failed: {e}"))?;

    let (client_id, _) = session
        .get_credentials()
        .await
        .map_err(|e| anyhow::anyhow!("read OAuth credentials: {e}"))?;

    println!(
        "Authorized MCP server '{server_name}'. Credentials saved (encrypted) to {}.",
        auth_store_path.display()
    );

    Ok(McpOAuthFlowResult {
        server_name: server_name.to_string(),
        credentials_path: auth_store_path.to_path_buf(),
        client_id,
    })
}

/// Build an [`AuthorizationManager`] with file-backed credentials for an existing session.
pub async fn authorization_manager_from_store(
    server_url: &str,
    auth_store_path: &Path,
    server_name: &str,
) -> Result<Option<AuthorizationManager>> {
    if !has_stored_credentials(auth_store_path, server_name) {
        return Ok(None);
    }
    let store = FileCredentialStore::new(auth_store_path, server_name);
    let mut manager = AuthorizationManager::new(server_url)
        .await
        .map_err(|e| anyhow::anyhow!("init OAuth manager: {e}"))?;
    manager.set_credential_store(store);
    let ready = manager
        .initialize_from_store()
        .await
        .map_err(|e| anyhow::anyhow!("load OAuth credentials: {e}"))?;
    if ready { Ok(Some(manager)) } else { Ok(None) }
}

fn open_browser(url: &str) -> Result<()> {
    let status = {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("open").arg(url).status()
        }
        #[cfg(target_os = "linux")]
        {
            std::process::Command::new("xdg-open").arg(url).status()
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("cmd")
                .args(["/C", "start", "", url])
                .status()
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            let _ = url;
            return Err(anyhow::anyhow!("opening a browser is not supported on this platform"));
        }
    };
    status.context("launch browser")?;
    Ok(())
}

async fn wait_for_oauth_callback(listener: TcpListener) -> Result<String> {
    let (mut socket, _) = listener.accept().await.context("accept OAuth callback connection")?;
    let mut buf = vec![0u8; 8192];
    let n = socket.read(&mut buf).await.context("read OAuth callback request")?;
    let request = String::from_utf8_lossy(&buf[..n]);
    let path_line = request.lines().next().context("empty OAuth callback request")?;
    let path = path_line
        .split_whitespace()
        .nth(1)
        .context("malformed OAuth callback request line")?;
    let host = listener.local_addr()?;
    let callback_url = format!("http://{}{path}", host);

    let body = b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nConnection: close\r\n\r\n\
<!DOCTYPE html><html><body><h1>Authorization complete</h1>\
<p>You can close this window and return to Elph.</p></body></html>";
    let _ = socket.write_all(body).await;
    let _ = socket.shutdown().await;

    if !path.contains("code=") {
        anyhow::bail!("OAuth callback missing authorization code: {path}");
    }
    Ok(callback_url)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn path_builder_defaults_to_auth_json() {
        let path = AuthStorePathBuilder::new().base_dir("/home/u/.elph").build();
        assert_eq!(path, PathBuf::from("/home/u/.elph/auth.json"));
    }

    #[test]
    fn path_builder_custom_file_and_explicit_path() {
        let path = AuthStorePathBuilder::new()
            .base_dir("/home/u/.owly")
            .file_name("creds.json")
            .build();
        assert_eq!(path, PathBuf::from("/home/u/.owly/creds.json"));

        let path = AuthStorePathBuilder::new()
            .base_dir("/ignored")
            .path("/var/lib/eclaw/auth.json")
            .build();
        assert_eq!(path, PathBuf::from("/var/lib/eclaw/auth.json"));
    }

    #[test]
    fn auth_store_path_helper() {
        assert_eq!(
            auth_store_path(Path::new("/tmp/cfg")),
            PathBuf::from("/tmp/cfg/auth.json")
        );
    }

    #[tokio::test]
    async fn multi_server_store_encrypted_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("auth.json");

        let a = FileCredentialStore::new(&path, "server-a");
        let b = FileCredentialStore::new(&path, "server-b");

        let creds_a = StoredCredentials::new("client-a".into(), None, vec!["read".into()], Some(1));
        a.save(creds_a.clone()).await.unwrap();

        let creds_b = StoredCredentials::new("client-b".into(), None, vec![], Some(2));
        b.save(creds_b.clone()).await.unwrap();

        // On disk: entries must be enc: strings
        let raw = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(raw.contains(ENC_PREFIX), "expected encrypted payload: {raw}");
        assert!(!raw.contains("client-a"), "client id must not appear in plaintext");

        assert!(has_stored_credentials(&path, "server-a"));
        assert!(has_stored_credentials(&path, "server-b"));

        let loaded_a = a.load().await.unwrap().unwrap();
        assert_eq!(loaded_a.client_id, "client-a");
        let loaded_b = b.load().await.unwrap().unwrap();
        assert_eq!(loaded_b.client_id, "client-b");

        a.clear().await.unwrap();
        assert!(!has_stored_credentials(&path, "server-a"));
        assert!(has_stored_credentials(&path, "server-b"));
        assert!(path.exists());

        b.clear().await.unwrap();
        assert!(!path.exists());
    }

    #[tokio::test]
    async fn migrates_plain_entry_on_resave() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("auth.json");
        // Seed legacy plaintext object
        let plain = serde_json::json!({
            "mcp": {
                "legacy": {
                    "client_id": "old-client",
                    "granted_scopes": [],
                    "token_received_at": 1
                }
            }
        });
        tokio::fs::write(&path, serde_json::to_vec_pretty(&plain).unwrap())
            .await
            .unwrap();

        let store = FileCredentialStore::new(&path, "legacy");
        let loaded = store.load().await.unwrap().unwrap();
        assert_eq!(loaded.client_id, "old-client");
        store.save(loaded).await.unwrap();

        let raw = tokio::fs::read_to_string(&path).await.unwrap();
        assert!(raw.contains(ENC_PREFIX));
        assert!(!raw.contains("old-client"));
    }

    #[test]
    fn store_builder_requires_server_key() {
        let result = FileCredentialStore::builder().base_dir("/tmp").build();
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().contains("server_key"));
    }
}
