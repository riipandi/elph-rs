mod progress;

use std::fs;
use std::io;
use std::path::Path;

use serde::Serialize;
use thiserror::Error;

use crate::appdir::Paths;
use crate::bundled::BundledManifest;
use crate::datastore;
use crate::settings::Settings;
use crate::trust::TrustStore;
use crate::version::VersionFile;

use progress::InitProgress;

pub type Result<T> = std::result::Result<T, InitError>;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("datastore error: {0}")]
    Datastore(#[from] datastore::DatastoreError),
}

/// Create required directories and default files for a fresh Elph install.
pub async fn ensure(app_version: &str) -> Result<Paths> {
    let progress = InitProgress::new();
    progress.advance("Resolving home directories");
    let paths = Paths::resolve()?;
    run_init_steps(&paths, app_version, &progress).await?;
    progress.finish();
    Ok(paths)
}

/// Initialize a specific config/data layout (useful in tests and custom installs).
pub async fn ensure_with_paths(paths: &Paths, app_version: &str) -> Result<()> {
    let progress = InitProgress::new();
    run_init_steps(paths, app_version, &progress).await?;
    progress.finish();
    Ok(())
}

/// Blocking wrapper for CLI startup code paths that are not async yet.
pub fn ensure_blocking(app_version: &str) -> Result<Paths> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|err| InitError::Io(io::Error::other(err)))?
        .block_on(ensure(app_version))
}

async fn run_init_steps(paths: &Paths, app_version: &str, progress: &InitProgress) -> Result<()> {
    progress.advance("Creating directories");
    ensure_dirs(paths)?;

    progress.advance("Writing configuration");
    ensure_files(paths, app_version)?;

    progress.advance("Initializing databases");
    datastore::ensure(paths).await?;
    Ok(())
}

fn ensure_dirs(paths: &Paths) -> Result<()> {
    fs::create_dir_all(&paths.config_dir)?;
    fs::create_dir_all(&paths.data_dir)?;

    for dir in paths.required_dirs() {
        fs::create_dir_all(dir)?;
    }

    Ok(())
}

fn ensure_files(paths: &Paths, app_version: &str) -> Result<()> {
    Settings::ensure(paths)?;
    TrustStore::ensure(paths)?;
    VersionFile::ensure(paths, app_version)?;
    BundledManifest::ensure(paths, app_version)?;
    Ok(())
}

pub(crate) fn write_json_file<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut payload = serde_json::to_string_pretty(value)?;
    payload.push('\n');

    write_private_file(path, payload.as_bytes())
}

fn write_private_file(path: &Path, contents: &[u8]) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;

        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new().write(true).create_new(true).mode(0o600).open(path)?;

        file.write_all(contents)?;
        return Ok(());
    }

    #[cfg(not(unix))]
    {
        fs::write(path, contents)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn ensure_creates_full_layout() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = tmp.path().join("config");
        let data = tmp.path().join("data");
        let paths = Paths::from_dirs(config, data);

        ensure_with_paths(&paths, "0.0.10-test").await.expect("ensure layout");

        assert!(paths.settings_path().exists());
        assert!(paths.trust_path().exists());
        assert!(paths.version_path().exists());
        assert!(paths.bundled_manifest_path().exists());
        assert!(paths.metadata_db_path().exists());
        assert!(paths.memory_db_path().exists());
        assert!(paths.bundled_dir().join("agents").is_dir());
        assert!(paths.bundled_dir().join("personas").is_dir());
        assert!(paths.bundled_dir().join("skills").is_dir());
        assert!(paths.bundled_dir().join("user-guide").is_dir());
        assert!(paths.prompts_dir().is_dir());
        assert!(paths.providers_dir().is_dir());
        assert!(paths.sessions_dir().is_dir());
        assert!(paths.skills_dir().is_dir());
        assert!(paths.worktrees_dir().is_dir());
        assert!(paths.attachments_dir().is_dir());
        assert!(paths.downloads_dir().is_dir());
        assert!(paths.logs_dir().is_dir());
        assert!(paths.vendor_dir().is_dir());
    }
}
