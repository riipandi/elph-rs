//! Turso local databases for Elph (`metadata.db`, `memory.db`).

mod migrations;

use std::path::Path;

use thiserror::Error;
use turso::Builder;

use crate::appdir::Paths;

#[derive(Debug, Error)]
pub enum DatastoreError {
    #[error("turso database error: {0}")]
    Turso(#[from] turso::Error),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, DatastoreError>;

/// Initialize local Turso databases and apply pending migrations.
pub async fn ensure(paths: &Paths) -> Result<()> {
    ensure_parent_dir(&paths.metadata_db_path())?;
    ensure_parent_dir(&paths.memory_db_path())?;

    open_and_migrate(&paths.metadata_db_path(), migrations::metadata_migrations()).await?;
    open_and_migrate(&paths.memory_db_path(), migrations::memory_migrations()).await?;

    Ok(())
}

async fn open_and_migrate(path: &Path, migrations: &[migrations::Migration]) -> Result<()> {
    let db = Builder::new_local(&path.to_string_lossy()).build().await?;
    let conn = db.connect()?;

    migrations::run(&conn, migrations).await?;
    Ok(())
}

fn ensure_parent_dir(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::appdir::Paths;

    #[tokio::test]
    async fn creates_metadata_and_memory_databases() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let paths = Paths::from_dirs(tmp.path().join("config"), tmp.path().join("data"));

        ensure(&paths).await.expect("ensure databases");

        assert!(paths.metadata_db_path().exists());
        assert!(paths.memory_db_path().exists());
    }
}
