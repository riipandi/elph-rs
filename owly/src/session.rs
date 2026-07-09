//! Session store: Turso checkpoint + thread identity for all agent runs.

use anyhow::Result;
use elph_agent::AgentMessage;
use elph_agent::uuidv7;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::Path;

use crate::checkpoint::{Checkpoint, CheckpointConfigurable, CheckpointMetadata, RunnableConfig, TursoCheckpointSaver};

/// LangGraph messages channel name.
pub const MESSAGES_CHANNEL: &str = "messages";

/// Turso-backed session with checkpoint persistence.
pub struct SessionStore {
    saver: TursoCheckpointSaver,
    thread_id: String,
    checkpoint_config: RunnableConfig,
    step: i64,
}

impl SessionStore {
    pub async fn open(cwd: &Path) -> Result<Self> {
        let saver = TursoCheckpointSaver::default().await?;
        let thread_id = create_session_thread_id(cwd, None);
        let (checkpoint_config, _, step) = load_session_state(&saver, &thread_id).await?;
        Ok(Self {
            saver,
            thread_id,
            checkpoint_config,
            step,
        })
    }

    pub async fn with_new_thread(cwd: &Path) -> Result<Self> {
        let saver = TursoCheckpointSaver::default().await?;
        let thread_id = create_session_thread_id(cwd, Some(uuidv7().as_str()));
        let checkpoint_config = interactive_config(&thread_id);
        Ok(Self {
            saver,
            thread_id,
            checkpoint_config,
            step: 0,
        })
    }

    pub fn thread_id(&self) -> &str {
        &self.thread_id
    }

    pub fn db_path(&self) -> &Path {
        self.saver.db_path()
    }

    pub async fn load_messages(&self) -> Result<Vec<AgentMessage>> {
        load_messages(&self.saver, &self.thread_id)
            .await
            .map(|(_, messages)| messages)
    }

    pub async fn save_messages(&mut self, messages: &[AgentMessage], source: &str) -> Result<()> {
        self.step += 1;
        self.checkpoint_config =
            save_messages(&self.saver, &self.checkpoint_config, messages, self.step, source).await?;
        Ok(())
    }

    pub fn reset_thread(&mut self, cwd: &Path) {
        self.thread_id = create_session_thread_id(cwd, Some(uuidv7().as_str()));
        self.checkpoint_config = interactive_config(&self.thread_id);
        self.step = 0;
    }
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

    saver.put(config, &checkpoint, &metadata).await
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
