//! Documentation generation and management.
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki)
//! `src/agent/utils.ts`. Original MIT License, Copyright (c) 2026 LangChain.
//!
//! Handles documentation snapshots, frontmatter generation, and update metadata.

use anyhow::Result;
use chrono::Utc;
use std::path::{Path, PathBuf};

use crate::constants::OWLY_DIR;
use crate::metadata::{UpdateMetadata, get_git_head, save_metadata};

/// Create a snapshot of the current documentation state
pub fn create_snapshot(cwd: &Path) -> Result<DocumentationSnapshot> {
    let owly_dir = cwd.join(OWLY_DIR);

    if !owly_dir.exists() {
        return Ok(DocumentationSnapshot {
            files: Vec::new(),
            exists: false,
        });
    }

    let mut files = Vec::new();
    collect_files(&owly_dir, &owly_dir, &mut files)?;

    Ok(DocumentationSnapshot { files, exists: true })
}

/// Compare two snapshots to check if documentation has changed
pub fn has_changed(before: &DocumentationSnapshot, after: &DocumentationSnapshot) -> bool {
    if before.exists != after.exists {
        return true;
    }

    if before.files.len() != after.files.len() {
        return true;
    }

    // Compare file contents
    for (before_file, after_file) in before.files.iter().zip(after.files.iter()) {
        if before_file.relative_path != after_file.relative_path {
            return true;
        }
        if before_file.content_hash != after_file.content_hash {
            return true;
        }
    }

    false
}

/// Write documentation content to a file with frontmatter
pub fn write_doc_file(
    cwd: &Path,
    relative_path: &str,
    title: &str,
    category: &str,
    content: &str,
    tags: Option<&[&str]>,
) -> Result<PathBuf> {
    let path = cwd.join(OWLY_DIR).join(relative_path);

    // Ensure directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut frontmatter = crate::frontmatter::Frontmatter::new(title, category);
    if let Some(tags) = tags {
        frontmatter = frontmatter.with_tags(tags);
    }

    let frontmatter_yaml = frontmatter.to_yaml();
    let full_content = format!("{frontmatter_yaml}\n{content}\n");

    std::fs::write(&path, full_content)?;

    Ok(path)
}

/// Read documentation file and extract frontmatter and body
pub fn read_doc_file(path: &Path) -> Result<Option<(crate::frontmatter::Frontmatter, String)>> {
    let content = std::fs::read_to_string(path)?;

    match crate::frontmatter::Frontmatter::parse(&content) {
        Some((frontmatter, body)) => Ok(Some((frontmatter, body.to_string()))),
        None => Ok(None),
    }
}

/// Save update metadata after successful run
pub fn save_update_metadata(cwd: &Path, command: &str, model: &str) -> Result<()> {
    let git_head = get_git_head(cwd);

    let metadata = UpdateMetadata {
        updated_at: Utc::now(),
        command: command.to_string(),
        git_head,
        model: model.to_string(),
    };

    save_metadata(cwd, &metadata)
}

/// Save metadata only when documentation content actually changed.
pub fn save_update_metadata_if_changed(
    cwd: &Path,
    command: &str,
    model: &str,
    before: &DocumentationSnapshot,
) -> Result<bool> {
    let after = create_snapshot(cwd)?;
    if !has_changed(before, &after) {
        return Ok(false);
    }
    save_update_metadata(cwd, command, model)?;
    Ok(true)
}

/// Get git summary for changes since last update
pub fn get_git_summary(cwd: &Path) -> String {
    let last_update = crate::metadata::load_metadata(cwd);
    crate::metadata::create_git_summary(cwd, last_update.as_ref())
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub relative_path: String,
    pub content_hash: u64,
}

#[derive(Debug, Clone)]
pub struct DocumentationSnapshot {
    pub files: Vec<FileEntry>,
    pub exists: bool,
}

fn collect_files(base: &Path, current: &Path, files: &mut Vec<FileEntry>) -> Result<()> {
    for entry in std::fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            collect_files(base, &path, files)?;
        } else {
            let relative = path.strip_prefix(base).unwrap_or(&path).to_string_lossy().to_string();

            let content = std::fs::read_to_string(&path).unwrap_or_default();
            let mut hasher = std::collections::hash_map::DefaultHasher::new();
            content.hash(&mut hasher);
            let content_hash = hasher.finish();

            files.push(FileEntry {
                relative_path: relative,
                content_hash,
            });
        }
    }

    Ok(())
}

use std::hash::{Hash, Hasher};
