//! Shared helpers for session repositories.

use crate::messages::now_iso_timestamp;
use crate::session::id::generate_session_id;
use crate::session::tree::Session;
use crate::session::types::{SessionError, SessionErrorCode, SessionStorage, SessionTreeEntry};

#[derive(Debug, Clone, Default)]
pub struct ForkEntriesOptions {
    pub entry_id: Option<String>,
    pub position: Option<ForkPosition>,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForkPosition {
    Before,
    At,
}

pub fn create_session_id() -> String {
    generate_session_id()
}

pub fn create_timestamp() -> String {
    now_iso_timestamp()
}

pub async fn get_entries_to_fork<S: SessionStorage>(
    storage: &S,
    options: &ForkEntriesOptions,
) -> Result<Vec<SessionTreeEntry>, SessionError> {
    if options.entry_id.is_none() {
        return Ok(storage.get_entries().await);
    }
    let entry_id = options.entry_id.as_ref().expect("entry id");
    let target = storage
        .get_entry(entry_id)
        .await
        .ok_or_else(|| SessionError::new(SessionErrorCode::NotFound, format!("Entry {entry_id} not found")))?;
    let effective_leaf_id = match options.position.unwrap_or(ForkPosition::Before) {
        ForkPosition::At => Some(entry_id.clone()),
        ForkPosition::Before => {
            let SessionTreeEntry::Message { message, .. } = &target else {
                return Err(SessionError::new(
                    SessionErrorCode::NotFound,
                    format!("Entry {entry_id} is not a user message"),
                ));
            };
            if message.role() != "user" {
                return Err(SessionError::new(
                    SessionErrorCode::NotFound,
                    format!("Entry {entry_id} is not a user message"),
                ));
            }
            target.parent_id().map(str::to_string)
        }
    };
    storage.get_path_to_root(effective_leaf_id.as_deref()).await
}

pub fn to_session<S>(storage: S) -> Session<S>
where
    S: SessionStorage,
{
    Session::new(storage)
}
