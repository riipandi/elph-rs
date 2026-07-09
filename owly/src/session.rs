//! Session store: Turso checkpoint + thread identity for all agent runs.
//!
//! LangGraph parity without the graph runtime: pending writes are recorded on the
//! active checkpoint via [`TurnWriteContext`] (tool results during a turn, full
//! `messages` channel at turn end in [`save_messages`]).

use anyhow::{Context, Result};
use elph_agent::AgentMessage;
use elph_agent::uuidv7;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::checkpoint::{
    ASSISTANT_DRAFT, Checkpoint, CheckpointConfigurable, CheckpointListOptions, CheckpointMetadata, PendingWrite,
    RunnableConfig, TursoCheckpointSaver,
};

/// LangGraph messages channel name.
pub const MESSAGES_CHANNEL: &str = "messages";

/// Prefix for per-tool pending-write channels (`tool:bash`, `tool:read`, …).
pub const TOOL_CHANNEL_PREFIX: &str = "tool:";

/// One row shown by `/history`.
#[derive(Debug, Clone)]
pub struct CheckpointSummary {
    pub checkpoint_id: String,
    pub step: i64,
    pub source: String,
    pub message_count: usize,
}

/// Snapshot of the checkpoint config at turn start — target for in-flight `put_writes`.
#[derive(Clone)]
pub struct TurnWriteContext {
    saver: Arc<TursoCheckpointSaver>,
    config: RunnableConfig,
    assistant_draft: Arc<Mutex<String>>,
}

impl TurnWriteContext {
    pub async fn record_tool_result(
        &self,
        tool_call_id: &str,
        tool_name: &str,
        args_summary: &str,
        is_error: bool,
        output: &str,
    ) -> Result<()> {
        if self.config.configurable.checkpoint_id.is_none() {
            return Ok(());
        }
        let channel = tool_write_channel(tool_name);
        let value = json!({
            "id": tool_call_id,
            "name": tool_name,
            "args": args_summary,
            "is_error": is_error,
            "output": output,
        });
        self.saver
            .put_writes(&self.config, &[(channel, value)], tool_call_id)
            .await?;
        Ok(())
    }

    /// Persist accumulated assistant text for mid-turn crash recovery.
    pub async fn record_assistant_delta(&self, delta: &str) -> Result<()> {
        if delta.is_empty() || self.config.configurable.checkpoint_id.is_none() {
            return Ok(());
        }
        let mut draft = self.assistant_draft.lock().await;
        draft.push_str(delta);
        let value = json!({ "text": draft.as_str() });
        self.saver
            .put_writes(
                &self.config,
                &[(ASSISTANT_DRAFT.to_string(), value)],
                "assistant_stream",
            )
            .await?;
        Ok(())
    }
}

/// Turso-backed session with checkpoint persistence.
pub struct SessionStore {
    saver: Arc<TursoCheckpointSaver>,
    thread_id: String,
    checkpoint_config: RunnableConfig,
    step: i64,
}

impl SessionStore {
    pub async fn open(cwd: &Path) -> Result<Self> {
        Self::open_with_path(crate::checkpoint::default_db_path(), cwd).await
    }

    /// Open a session against an explicit database path.
    pub async fn open_with_path(db_path: impl AsRef<Path>, cwd: &Path) -> Result<Self> {
        let saver = Arc::new(TursoCheckpointSaver::open(Some(db_path.as_ref().to_path_buf())).await?);
        let thread_id = create_session_thread_id(cwd, None);
        let (checkpoint_config, _, step) = load_session_state(saver.as_ref(), &thread_id).await?;
        let mut store = Self {
            saver,
            thread_id,
            checkpoint_config,
            step,
        };
        store.ensure_bootstrap_checkpoint().await?;
        Ok(store)
    }

    pub async fn with_new_thread(cwd: &Path) -> Result<Self> {
        let saver = Arc::new(TursoCheckpointSaver::default().await?);
        let thread_id = create_session_thread_id(cwd, Some(uuidv7().as_str()));
        let checkpoint_config = interactive_config(&thread_id);
        let mut store = Self {
            saver,
            thread_id,
            checkpoint_config,
            step: 0,
        };
        store.ensure_bootstrap_checkpoint().await?;
        Ok(store)
    }

    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }

    pub fn db_path(&self) -> &Path {
        self.saver.db_path()
    }

    pub async fn load_messages(&self) -> Result<Vec<AgentMessage>> {
        if self.checkpoint_config.configurable.checkpoint_id.is_some()
            && let Some(tuple) = self.saver.get_tuple(&self.checkpoint_config).await?
        {
            return Ok(messages_from_checkpoint(&tuple.checkpoint));
        }
        load_messages(self.saver.as_ref(), &self.thread_id)
            .await
            .map(|(_, messages)| messages)
    }

    /// Frozen checkpoint handle for the current turn (before [`Self::save_messages`]).
    pub fn turn_write_context(&self) -> TurnWriteContext {
        TurnWriteContext {
            saver: Arc::clone(&self.saver),
            config: self.checkpoint_config.clone(),
            assistant_draft: Arc::new(Mutex::new(String::new())),
        }
    }

    /// Ensure a root checkpoint exists so turn-1 `put_writes` has a target.
    pub async fn ensure_bootstrap_checkpoint(&mut self) -> Result<()> {
        if self.checkpoint_config.configurable.checkpoint_id.is_some() {
            return Ok(());
        }
        let checkpoint = Checkpoint::default();
        let metadata = CheckpointMetadata {
            source: "bootstrap".to_string(),
            step: 0,
            parents: HashMap::new(),
        };
        self.checkpoint_config = self.saver.put(&self.checkpoint_config, &checkpoint, &metadata).await?;
        Ok(())
    }

    pub async fn save_messages(&mut self, messages: &[AgentMessage], source: &str) -> Result<()> {
        self.step += 1;
        self.checkpoint_config = save_messages(
            self.saver.as_ref(),
            &self.checkpoint_config,
            messages,
            self.step,
            source,
        )
        .await?;
        Ok(())
    }

    /// Start a fresh thread id and bootstrap a root checkpoint.
    pub async fn reset_thread(&mut self, cwd: &Path) -> Result<()> {
        self.thread_id = create_session_thread_id(cwd, Some(uuidv7().as_str()));
        self.checkpoint_config = interactive_config(&self.thread_id);
        self.step = 0;
        self.ensure_bootstrap_checkpoint().await
    }

    /// List recent checkpoints for the active thread (newest first).
    pub async fn list_checkpoint_history(&self, limit: usize) -> Result<Vec<CheckpointSummary>> {
        let limit = limit.clamp(1, 100) as u64;
        let tuples = self
            .saver
            .list(
                &interactive_config(&self.thread_id),
                &CheckpointListOptions {
                    limit: Some(limit),
                    before: None,
                    filter: None,
                },
            )
            .await?;
        Ok(tuples
            .into_iter()
            .map(|tuple| {
                let messages = messages_from_checkpoint(&tuple.checkpoint);
                CheckpointSummary {
                    checkpoint_id: tuple.config.configurable.checkpoint_id.unwrap_or_default(),
                    step: tuple.metadata.as_ref().map(|m| m.step).unwrap_or(0),
                    source: tuple.metadata.as_ref().map(|m| m.source.clone()).unwrap_or_default(),
                    message_count: messages.len(),
                }
            })
            .collect())
    }

    /// Resolve `/restore` argument: 1-based index from [`Self::list_checkpoint_history`] or id prefix.
    pub async fn resolve_checkpoint_id(&self, arg: &str) -> Result<String> {
        if let Ok(index) = arg.parse::<usize>() {
            if index == 0 {
                anyhow::bail!("checkpoint index must be >= 1");
            }
            let list = self.list_checkpoint_history(100).await?;
            let item = list
                .get(index - 1)
                .with_context(|| format!("checkpoint #{index} not found"))?;
            return Ok(item.checkpoint_id.clone());
        }
        let list = self.list_checkpoint_history(100).await?;
        let matches: Vec<_> = list
            .iter()
            .filter(|summary| summary.checkpoint_id.starts_with(arg))
            .collect();
        match matches.len() {
            0 => anyhow::bail!("no checkpoint matching '{arg}'"),
            1 => Ok(matches[0].checkpoint_id.clone()),
            n => anyhow::bail!("ambiguous checkpoint prefix '{arg}' ({n} matches)"),
        }
    }

    /// Load a checkpoint tuple for this thread (`checkpoint_id` = latest when `None`).
    pub async fn get_checkpoint_tuple(
        &self,
        checkpoint_id: Option<&str>,
    ) -> Result<Option<crate::checkpoint::CheckpointTuple>> {
        let mut config = interactive_config(&self.thread_id);
        if let Some(id) = checkpoint_id {
            config.configurable.checkpoint_id = Some(id.to_string());
        }
        self.saver.get_tuple(&config).await
    }

    /// Reposition the session to an earlier checkpoint (fork on next turn).
    pub async fn restore_checkpoint(&mut self, checkpoint_id: &str) -> Result<usize> {
        let config = RunnableConfig {
            configurable: CheckpointConfigurable {
                thread_id: self.thread_id.clone(),
                checkpoint_ns: String::new(),
                checkpoint_id: Some(checkpoint_id.to_string()),
            },
        };
        let tuple = self
            .saver
            .get_tuple(&config)
            .await?
            .with_context(|| format!("checkpoint {checkpoint_id} not found"))?;
        let messages = messages_from_checkpoint(&tuple.checkpoint);
        let count = messages.len();
        self.step = tuple.metadata.as_ref().map(|m| m.step).unwrap_or(0);
        self.checkpoint_config = tuple.config;
        Ok(count)
    }
}

pub fn tool_write_channel(tool_name: &str) -> String {
    format!("{TOOL_CHANNEL_PREFIX}{tool_name}")
}

/// Stable session thread id for a repository root (OpenWiki-style).
pub fn create_session_thread_id(cwd: &Path, run_id: Option<&str>) -> String {
    let resolved = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
    let digest = Sha256::digest(resolved.to_string_lossy().as_bytes());
    let hex: String = digest.iter().map(|b| format!("{b:02x}")).collect();
    match run_id {
        Some(run) => format!("owly-{}-{run}", &hex[..32]),
        None => format!("owly-{}-interactive", &hex[..32]),
    }
}

/// Backward-compatible alias.
pub fn create_interactive_thread_id(cwd: &Path) -> String {
    create_session_thread_id(cwd, None)
}

pub fn interactive_config(thread_id: impl Into<String>) -> RunnableConfig {
    RunnableConfig {
        configurable: CheckpointConfigurable {
            thread_id: thread_id.into(),
            checkpoint_ns: String::new(),
            checkpoint_id: None,
        },
    }
}

async fn load_session_state(
    saver: &TursoCheckpointSaver,
    thread_id: &str,
) -> Result<(RunnableConfig, Vec<AgentMessage>, i64)> {
    let (config, messages) = load_messages(saver, thread_id).await?;
    let step = saver
        .get_tuple(&config)
        .await?
        .and_then(|t| t.metadata.map(|m| m.step))
        .unwrap_or(0);
    Ok((config, messages, step))
}

/// Load the latest conversation messages for a thread, if any.
pub async fn load_messages(
    saver: &TursoCheckpointSaver,
    thread_id: &str,
) -> Result<(RunnableConfig, Vec<AgentMessage>)> {
    let config = interactive_config(thread_id);
    let Some(tuple) = saver.get_tuple(&config).await? else {
        return Ok((config, Vec::new()));
    };

    let messages = messages_from_checkpoint(&tuple.checkpoint);
    Ok((tuple.config, messages))
}

/// Persist the full conversation after a completed turn.
pub async fn save_messages(
    saver: &TursoCheckpointSaver,
    config: &RunnableConfig,
    messages: &[AgentMessage],
    step: i64,
    source: &str,
) -> Result<RunnableConfig> {
    let mut checkpoint = copy_checkpoint_for_save(config, saver).await?;
    checkpoint
        .channel_values
        .insert(MESSAGES_CHANNEL.to_string(), serde_json::to_value(messages)?);
    bump_channel_version(&mut checkpoint, MESSAGES_CHANNEL);

    let metadata = CheckpointMetadata {
        source: source.to_string(),
        step,
        parents: HashMap::new(),
    };

    // LangGraph contract: pending writes attach to the checkpoint being executed
    // (current `config.checkpoint_id`), then `put` creates the next checkpoint.
    if config.configurable.checkpoint_id.is_some() {
        persist_channel_writes(saver, config, messages, source).await?;
    }

    saver.put(config, &checkpoint, &metadata).await
}

/// Mirror LangGraph pending writes into the `writes` table for the active checkpoint.
pub async fn persist_channel_writes(
    saver: &TursoCheckpointSaver,
    config: &RunnableConfig,
    messages: &[AgentMessage],
    task_id: &str,
) -> Result<()> {
    let message_value = serde_json::to_value(messages)?;
    let writes: Vec<PendingWrite> = vec![(MESSAGES_CHANNEL.to_string(), message_value)];
    saver.put_writes(config, &writes, task_id).await?;
    Ok(())
}

async fn copy_checkpoint_for_save(config: &RunnableConfig, saver: &TursoCheckpointSaver) -> Result<Checkpoint> {
    if let Some(tuple) = saver.get_tuple(config).await? {
        let mut checkpoint = crate::checkpoint::copy_checkpoint(&tuple.checkpoint);
        checkpoint.id = uuidv7();
        checkpoint.ts = chrono::Utc::now().to_rfc3339();
        return Ok(checkpoint);
    }

    Ok(Checkpoint::default())
}

fn bump_channel_version(checkpoint: &mut Checkpoint, channel: &str) {
    let next = checkpoint
        .channel_versions
        .get(channel)
        .and_then(|v| v.parse::<i64>().ok())
        .map(|v| (v + 1).to_string())
        .unwrap_or_else(|| "1".to_string());
    checkpoint.channel_versions.insert(channel.to_string(), next);
}

pub fn messages_from_checkpoint(checkpoint: &Checkpoint) -> Vec<AgentMessage> {
    checkpoint
        .channel_values
        .get(MESSAGES_CHANNEL)
        .and_then(|value| serde_json::from_value::<Vec<AgentMessage>>(value.clone()).ok())
        .unwrap_or_default()
}

pub fn messages_to_channel_value(messages: &[AgentMessage]) -> Result<Value> {
    Ok(serde_json::to_value(messages)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checkpoint::{Checkpoint, CheckpointMetadata};
    use std::path::PathBuf;
    use tempfile::TempDir;

    async fn test_session(dir: &TempDir, cwd: &Path) -> SessionStore {
        let path: PathBuf = dir.path().join("test.sqlite");
        SessionStore::open_with_path(path, cwd).await.expect("open session")
    }

    #[test]
    fn tool_write_channel_formats_name() {
        assert_eq!(tool_write_channel("bash"), "tool:bash");
    }

    #[tokio::test]
    async fn turn_write_context_records_tool_pending_write() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("test.sqlite");
        let saver = Arc::new(TursoCheckpointSaver::open(Some(path)).await.expect("open"));
        let thread_id = "thread-tool-writes";
        let mut config = interactive_config(thread_id);
        let checkpoint = Checkpoint::default();
        config = saver
            .put(
                &config,
                &checkpoint,
                &CheckpointMetadata {
                    source: "bootstrap".to_string(),
                    step: 0,
                    parents: HashMap::new(),
                },
            )
            .await
            .expect("bootstrap put");

        let ctx = TurnWriteContext {
            saver: Arc::clone(&saver),
            config: config.clone(),
            assistant_draft: Arc::new(Mutex::new(String::new())),
        };
        ctx.record_tool_result("call-1", "write", r#"{"path":"a.md"}"#, false, "Wrote 10 bytes")
            .await
            .expect("tool write");

        let tuple = saver.get_tuple(&config).await.expect("get").expect("tuple");
        assert_eq!(tuple.pending_writes.len(), 1);
        assert_eq!(tuple.pending_writes[0].0, "call-1");
        assert_eq!(tuple.pending_writes[0].1, tool_write_channel("write"));
        assert_eq!(
            tuple.pending_writes[0].2.get("output").and_then(|v| v.as_str()),
            Some("Wrote 10 bytes")
        );
        assert!(tool_write_channel("bash").starts_with(TOOL_CHANNEL_PREFIX));
    }

    #[tokio::test]
    async fn turn_write_context_records_assistant_draft() {
        let dir = TempDir::new().expect("tempdir");
        let path = dir.path().join("draft.sqlite");
        let saver = Arc::new(TursoCheckpointSaver::open(Some(path)).await.expect("open"));
        let thread_id = "thread-draft";
        let mut config = interactive_config(thread_id);
        config = saver
            .put(
                &config,
                &Checkpoint::default(),
                &CheckpointMetadata {
                    source: "bootstrap".to_string(),
                    step: 0,
                    parents: HashMap::new(),
                },
            )
            .await
            .expect("bootstrap");

        let ctx = TurnWriteContext {
            saver: Arc::clone(&saver),
            config: config.clone(),
            assistant_draft: Arc::new(Mutex::new(String::new())),
        };
        ctx.record_assistant_delta("Hello").await.expect("first delta");
        ctx.record_assistant_delta(" world").await.expect("second delta");

        let tuple = saver.get_tuple(&config).await.expect("get").expect("tuple");
        assert_eq!(tuple.pending_writes.len(), 1);
        assert_eq!(tuple.pending_writes[0].1, ASSISTANT_DRAFT);
        assert_eq!(
            tuple.pending_writes[0].2.get("text").and_then(|v| v.as_str()),
            Some("Hello world")
        );
    }

    #[tokio::test]
    async fn full_turn_persists_tool_and_message_writes() {
        let dir = TempDir::new().expect("tempdir");
        let cwd = dir.path().join("repo");
        std::fs::create_dir_all(&cwd).expect("cwd");
        let mut store = test_session(&dir, &cwd).await;
        let turn_config = store.turn_write_context().config.clone();

        store
            .turn_write_context()
            .record_tool_result("call-1", "read", "{}", false, "file contents")
            .await
            .expect("tool write");

        let user = elph_agent::llm_message_to_agent(elph_ai::Message::User {
            content: elph_ai::UserContent::Text("hi".into()),
            timestamp: 0,
        });
        store
            .save_messages(std::slice::from_ref(&user), "chat")
            .await
            .expect("save");

        let turn_id = turn_config.configurable.checkpoint_id.expect("turn checkpoint id");
        let parent = store
            .get_checkpoint_tuple(Some(&turn_id))
            .await
            .expect("get")
            .expect("parent tuple");
        assert_eq!(parent.pending_writes.len(), 2);
        let channels: Vec<_> = parent.pending_writes.iter().map(|(_, ch, _)| ch.as_str()).collect();
        assert!(channels.contains(&tool_write_channel("read").as_str()));
        assert!(channels.contains(&MESSAGES_CHANNEL));
    }

    #[tokio::test]
    async fn reset_thread_bootstraps_new_checkpoint() {
        let dir = TempDir::new().expect("tempdir");
        let cwd = dir.path().join("repo");
        std::fs::create_dir_all(&cwd).expect("cwd");
        let mut store = test_session(&dir, &cwd).await;
        let first_id = store
            .checkpoint_config
            .configurable
            .checkpoint_id
            .clone()
            .expect("bootstrap id");

        store.reset_thread(&cwd).await.expect("reset");
        let second_id = store
            .checkpoint_config
            .configurable
            .checkpoint_id
            .clone()
            .expect("new bootstrap id");
        assert_ne!(first_id, second_id);
        assert_eq!(store.step, 0);
    }

    #[tokio::test]
    async fn restore_checkpoint_repositions_session() {
        let dir = TempDir::new().expect("tempdir");
        let cwd = dir.path().join("repo");
        std::fs::create_dir_all(&cwd).expect("cwd");
        let mut store = test_session(&dir, &cwd).await;

        let user = elph_agent::llm_message_to_agent(elph_ai::Message::User {
            content: elph_ai::UserContent::Text("one".into()),
            timestamp: 0,
        });
        store
            .save_messages(std::slice::from_ref(&user), "chat")
            .await
            .expect("save1");
        let first_id = store
            .checkpoint_config
            .configurable
            .checkpoint_id
            .clone()
            .expect("first");

        let user2 = elph_agent::llm_message_to_agent(elph_ai::Message::User {
            content: elph_ai::UserContent::Text("two".into()),
            timestamp: 0,
        });
        store.save_messages(&[user, user2], "chat").await.expect("save2");
        assert_eq!(store.load_messages().await.expect("load").len(), 2);

        let restored = store.restore_checkpoint(&first_id).await.expect("restore");
        assert_eq!(restored, 1);
        assert_eq!(store.load_messages().await.expect("reload").len(), 1);
        assert_eq!(
            store.checkpoint_config.configurable.checkpoint_id.as_deref(),
            Some(first_id.as_str())
        );
    }
}
