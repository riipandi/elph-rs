//! Session repositories for creating, opening, listing, forking, and deleting sessions.

use std::collections::HashMap;
use std::sync::Arc;

use crate::env::LocalExecutionEnv;
use crate::harness::types::{FileSystem, get_or_throw};
use crate::session::backends::{InMemorySessionStorage, JsonlSessionCreateOptions, JsonlSessionStorage};
use crate::session::repo_utils::{
    ForkEntriesOptions, create_session_id, create_timestamp, get_entries_to_fork, to_session,
};
use crate::session::tree::Session;
use crate::session::types::{JsonlSessionMetadata, SessionError, SessionErrorCode, SessionMetadata, SessionStorage};

#[derive(Debug, Clone, Default)]
pub struct InMemorySessionCreateOptions {
    pub id: Option<String>,
}

pub struct InMemorySessionRepo {
    sessions: HashMap<String, Session<InMemorySessionStorage>>,
}

impl Default for InMemorySessionRepo {
    fn default() -> Self {
        Self::new()
    }
}

impl InMemorySessionRepo {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub async fn create(
        &mut self,
        options: InMemorySessionCreateOptions,
    ) -> Result<Session<InMemorySessionStorage>, SessionError> {
        let metadata = SessionMetadata {
            id: options.id.unwrap_or_else(create_session_id),
            created_at: create_timestamp(),
        };
        let storage = InMemorySessionStorage::new(Some(crate::session::backends::InMemorySessionOptions {
            metadata: Some(metadata.clone()),
            ..Default::default()
        }))?;
        let session = to_session(storage);
        self.sessions.insert(metadata.id.clone(), session.clone());
        Ok(session)
    }

    pub async fn open(&self, metadata: &SessionMetadata) -> Result<Session<InMemorySessionStorage>, SessionError> {
        self.sessions.get(&metadata.id).cloned().ok_or_else(|| {
            SessionError::new(
                SessionErrorCode::NotFound,
                format!("Session not found: {}", metadata.id),
            )
        })
    }

    pub async fn list(&self) -> Vec<SessionMetadata> {
        let mut sessions = Vec::new();
        for session in self.sessions.values() {
            sessions.push(session.metadata().await);
        }
        sessions
    }

    pub async fn delete(&mut self, metadata: &SessionMetadata) {
        self.sessions.remove(&metadata.id);
    }

    pub async fn fork(
        &mut self,
        source_metadata: &SessionMetadata,
        options: ForkEntriesOptions,
    ) -> Result<Session<InMemorySessionStorage>, SessionError> {
        let source = self.open(source_metadata).await?;
        let forked_entries = get_entries_to_fork(source.storage(), &options).await?;
        let metadata = SessionMetadata {
            id: options.id.unwrap_or_else(create_session_id),
            created_at: create_timestamp(),
        };
        let storage = InMemorySessionStorage::new(Some(crate::session::backends::InMemorySessionOptions {
            metadata: Some(metadata.clone()),
            entries: Some(forked_entries),
            ..Default::default()
        }))?;
        let session = to_session(storage);
        self.sessions.insert(metadata.id.clone(), session.clone());
        Ok(session)
    }
}

#[derive(Debug, Clone)]
pub struct JsonlSessionRepoCreateOptions {
    pub cwd: String,
    pub id: Option<String>,
    pub parent_session_path: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct JsonlSessionListOptions {
    pub cwd: Option<String>,
}

pub struct JsonlSessionRepo {
    fs: Arc<LocalExecutionEnv>,
    sessions_root: String,
    resolved_root: tokio::sync::Mutex<Option<String>>,
}

impl JsonlSessionRepo {
    pub fn new(fs: Arc<LocalExecutionEnv>, sessions_root: impl Into<String>) -> Self {
        Self {
            fs,
            sessions_root: sessions_root.into(),
            resolved_root: tokio::sync::Mutex::new(None),
        }
    }

    async fn sessions_root(&self) -> Result<String, SessionError> {
        let mut guard = self.resolved_root.lock().await;
        if guard.is_none() {
            *guard = Some(get_or_throw(self.fs.absolute_path(&self.sessions_root, None).await));
        }
        Ok(guard.clone().expect("resolved root"))
    }

    async fn session_dir(&self, cwd: &str) -> Result<String, SessionError> {
        let root = self.sessions_root().await?;
        Ok(get_or_throw(
            self.fs
                .join_path(&[root.as_str(), encode_cwd(cwd).as_str()], None)
                .await,
        ))
    }

    async fn session_file_path(&self, cwd: &str, session_id: &str, timestamp: &str) -> Result<String, SessionError> {
        let dir = self.session_dir(cwd).await?;
        let file_name = format!("{}_{session_id}.jsonl", timestamp.replace([':', '.'], "-"));
        Ok(get_or_throw(
            self.fs.join_path(&[dir.as_str(), file_name.as_str()], None).await,
        ))
    }

    pub async fn create(
        &self,
        options: JsonlSessionRepoCreateOptions,
    ) -> Result<Session<JsonlSessionStorage>, SessionError> {
        let id = options.id.unwrap_or_else(create_session_id);
        let created_at = create_timestamp();
        let session_dir = self.session_dir(&options.cwd).await?;
        get_or_throw(
            FileSystem::create_dir(
                self.fs.as_ref(),
                &session_dir,
                Some(crate::harness::types::CreateDirOptions {
                    recursive: true,
                    abort_token: None,
                }),
            )
            .await,
        );
        let file_path = self.session_file_path(&options.cwd, &id, &created_at).await?;
        let storage = JsonlSessionStorage::create(
            &file_path,
            JsonlSessionCreateOptions {
                cwd: options.cwd,
                session_id: id,
                parent_session_path: options.parent_session_path,
            },
        )
        .await?;
        Ok(to_session(storage))
    }

    pub async fn open(&self, metadata: &JsonlSessionMetadata) -> Result<Session<JsonlSessionStorage>, SessionError> {
        if !get_or_throw(self.fs.exists(&metadata.path, None).await) {
            return Err(SessionError::new(
                SessionErrorCode::NotFound,
                format!("Session not found: {}", metadata.path),
            ));
        }
        Ok(to_session(JsonlSessionStorage::open(&metadata.path).await?))
    }

    pub async fn list(&self, options: JsonlSessionListOptions) -> Result<Vec<JsonlSessionMetadata>, SessionError> {
        let dirs = if let Some(cwd) = options.cwd {
            vec![self.session_dir(&cwd).await?]
        } else {
            self.list_session_dirs().await?
        };
        let mut sessions = Vec::new();
        for dir in dirs {
            if !get_or_throw(self.fs.exists(&dir, None).await) {
                continue;
            }
            let entries = get_or_throw(self.fs.list_dir(&dir, None).await);
            for entry in entries {
                if entry.kind != crate::harness::types::FileKind::File || !entry.name.ends_with(".jsonl") {
                    continue;
                }
                match crate::session::backends::load_jsonl_session_metadata(&entry.path).await {
                    Ok(metadata) => sessions.push(metadata),
                    Err(error) if error.code == SessionErrorCode::InvalidSession => {}
                    Err(error) => return Err(error),
                }
            }
        }
        sessions.sort_by(|left, right| right.created_at.cmp(&left.created_at));
        Ok(sessions)
    }

    pub async fn delete(&self, metadata: &JsonlSessionMetadata) -> Result<(), SessionError> {
        get_or_throw(
            self.fs
                .remove(
                    &metadata.path,
                    Some(crate::harness::types::RemoveOptions {
                        recursive: false,
                        force: true,
                        abort_token: None,
                    }),
                )
                .await,
        );
        Ok(())
    }

    pub async fn fork(
        &self,
        source_metadata: &JsonlSessionMetadata,
        options: JsonlSessionRepoCreateOptions,
        fork_options: ForkEntriesOptions,
    ) -> Result<Session<JsonlSessionStorage>, SessionError> {
        let source = self.open(source_metadata).await?;
        let forked_entries = get_entries_to_fork(source.storage(), &fork_options).await?;
        let id = options.id.unwrap_or_else(create_session_id);
        let created_at = create_timestamp();
        let session_dir = self.session_dir(&options.cwd).await?;
        get_or_throw(
            FileSystem::create_dir(
                self.fs.as_ref(),
                &session_dir,
                Some(crate::harness::types::CreateDirOptions {
                    recursive: true,
                    abort_token: None,
                }),
            )
            .await,
        );
        let file_path = self.session_file_path(&options.cwd, &id, &created_at).await?;
        let mut storage = JsonlSessionStorage::create(
            &file_path,
            JsonlSessionCreateOptions {
                cwd: options.cwd,
                session_id: id,
                parent_session_path: options
                    .parent_session_path
                    .or_else(|| Some(source_metadata.path.clone())),
            },
        )
        .await?;
        for entry in forked_entries {
            SessionStorage::append_entry(&mut storage, entry).await?;
        }
        Ok(to_session(storage))
    }

    async fn list_session_dirs(&self) -> Result<Vec<String>, SessionError> {
        let root = self.sessions_root().await?;
        if !get_or_throw(self.fs.exists(&root, None).await) {
            return Ok(Vec::new());
        }
        let entries = get_or_throw(self.fs.list_dir(&root, None).await);
        Ok(entries
            .into_iter()
            .filter(|entry| entry.kind == crate::harness::types::FileKind::Directory)
            .map(|entry| entry.path)
            .collect())
    }
}

fn encode_cwd(cwd: &str) -> String {
    format!(
        "--{}--",
        cwd.trim_start_matches(['/', '\\']).replace(['/', '\\', ':'], "-")
    )
}
