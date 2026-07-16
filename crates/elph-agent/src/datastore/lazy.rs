use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::Result;
use tokio::sync::Mutex;

use super::DatabaseSpec;
use super::ensure_databases;

static READY: AtomicBool = AtomicBool::new(false);
static LOCK: Mutex<()> = Mutex::const_new(());

/// Initialize databases once per process; subsequent calls are no-ops.
pub async fn ensure_databases_once(specs: &[DatabaseSpec<'_>]) -> Result<()> {
    if READY.load(Ordering::Acquire) {
        return Ok(());
    }

    let _guard = LOCK.lock().await;
    if READY.load(Ordering::Acquire) {
        return Ok(());
    }

    ensure_databases(specs).await?;
    READY.store(true, Ordering::Release);
    Ok(())
}
