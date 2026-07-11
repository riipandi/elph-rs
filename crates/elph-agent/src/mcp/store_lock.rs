//! Cross-process + in-process locking for the shared auth store.
//!
//! Prevents lost updates when concurrent token refreshes (or multi-server
//! saves) race on `auth.json`.

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex as StdMutex, OnceLock};

use anyhow::{Context, Result};
use fs4::fs_std::FileExt;
use tokio::sync::Mutex as AsyncMutex;

/// Process-wide async mutexes keyed by canonical store path.
fn path_mutexes() -> &'static StdMutex<HashMap<PathBuf, Arc<AsyncMutex<()>>>> {
    static MAP: OnceLock<StdMutex<HashMap<PathBuf, Arc<AsyncMutex<()>>>>> = OnceLock::new();
    MAP.get_or_init(|| StdMutex::new(HashMap::new()))
}

fn path_key(path: &Path) -> PathBuf {
    // Prefer absolute; fall back to as-is.
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

fn async_mutex_for(path: &Path) -> Arc<AsyncMutex<()>> {
    let key = path_key(path);
    let mut map = path_mutexes().lock().unwrap_or_else(|e| e.into_inner());
    map.entry(key).or_insert_with(|| Arc::new(AsyncMutex::new(()))).clone()
}

/// Guard returned by [`lock_auth_store`].
///
/// Holds an in-process async mutex and a cross-process exclusive file lock.
pub struct AuthStoreGuard {
    _guard: tokio::sync::OwnedMutexGuard<()>,
    /// Keep the lock file open for the duration of the guard (`fs4` unlocks on drop).
    _file: File,
}

/// Acquire exclusive access to the auth store at `path` (creates parent dirs / lock file as needed).
pub async fn lock_auth_store(path: &Path) -> Result<AuthStoreGuard> {
    let mutex = async_mutex_for(path);
    let guard = mutex.clone().lock_owned().await;

    let lock_path = lock_file_path(path);
    if let Some(parent) = lock_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .with_context(|| format!("create lock dir {}", parent.display()))?;
    }

    let lock_path_clone = lock_path.clone();
    let file = tokio::task::spawn_blocking(move || {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path_clone)
            .with_context(|| format!("open lock {}", lock_path_clone.display()))?;
        file.lock_exclusive()
            .with_context(|| format!("lock exclusive {}", lock_path_clone.display()))?;
        Ok::<File, anyhow::Error>(file)
    })
    .await
    .context("join flock")??;

    Ok(AuthStoreGuard {
        _guard: guard,
        _file: file,
    })
}

fn lock_file_path(store_path: &Path) -> PathBuf {
    // auth.json → auth.json.lock
    let mut s = store_path.as_os_str().to_os_string();
    s.push(".lock");
    PathBuf::from(s)
}

/// Atomically write `bytes` to `path` (temp file + rename) under an existing lock.
pub async fn atomic_write_private(path: &Path, bytes: &[u8]) -> Result<()> {
    let path = path.to_path_buf();
    let bytes = bytes.to_vec();
    tokio::task::spawn_blocking(move || atomic_write_private_sync(&path, &bytes))
        .await
        .context("join atomic write")?
}

fn atomic_write_private_sync(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let tmp = {
        let mut p = path.as_os_str().to_os_string();
        p.push(".tmp");
        PathBuf::from(p)
    };
    std::fs::write(&tmp, bytes).with_context(|| format!("write temp {}", tmp.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(&tmp, perms).with_context(|| format!("chmod {}", tmp.display()))?;
    }
    std::fs::rename(&tmp, path).with_context(|| format!("rename {} → {}", tmp.display(), path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn lock_serializes_access() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("auth.json");
        let p1 = path.clone();
        let p2 = path.clone();

        let h1 = tokio::spawn(async move {
            let _g = lock_auth_store(&p1).await.unwrap();
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            1
        });
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        let h2 = tokio::spawn(async move {
            let _g = lock_auth_store(&p2).await.unwrap();
            2
        });
        let a = h1.await.unwrap();
        let b = h2.await.unwrap();
        assert_eq!((a, b), (1, 2));
    }

    #[tokio::test]
    async fn atomic_write_roundtrip() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("auth.json");
        atomic_write_private(&path, b"{\"ok\":true}").await.unwrap();
        let s = tokio::fs::read_to_string(&path).await.unwrap();
        assert_eq!(s, "{\"ok\":true}");
    }
}
