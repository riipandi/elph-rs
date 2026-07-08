//! Tests for Owly metadata module.
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki)
//! `test/update-noop.test.ts`. Original MIT License, Copyright (c) 2026 LangChain.

use std::fs;
use tempfile::TempDir;

/// Create a temporary git repo with openwiki documentation
fn create_repo_with_openwiki() -> TempDir {
    let temp_dir = TempDir::new().unwrap();
    let cwd = temp_dir.path();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(cwd)
        .output()
        .expect("Failed to init git repo");

    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(cwd)
        .output()
        .expect("Failed to set git email");

    std::process::Command::new("git")
        .args(["config", "user.name", "Owly Test"])
        .current_dir(cwd)
        .output()
        .expect("Failed to set git name");

    // Create initial files
    fs::write(cwd.join("README.md"), "# Test Repo\n").unwrap();
    fs::create_dir_all(cwd.join("openwiki")).unwrap();
    fs::write(cwd.join("openwiki/quickstart.md"), "# Quickstart\n").unwrap();

    // Commit
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(cwd)
        .output()
        .expect("Failed to git add");

    std::process::Command::new("git")
        .args(["commit", "-m", "initial"])
        .current_dir(cwd)
        .output()
        .expect("Failed to git commit");

    temp_dir
}

/// Write last update metadata
fn write_last_update(cwd: &std::path::Path, git_head: &str) {
    let metadata = serde_json::json!({
        "updated_at": chrono::Utc::now().to_rfc3339(),
        "command": "update",
        "git_head": git_head,
        "model": "test-model"
    });

    let metadata_path = cwd.join("openwiki/.last-update.json");
    fs::write(metadata_path, serde_json::to_string_pretty(&metadata).unwrap()).unwrap();
}

/// Get current git HEAD
fn get_git_head(cwd: &std::path::Path) -> String {
    let output = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(cwd)
        .output()
        .expect("Failed to get git HEAD");

    String::from_utf8(output.stdout).unwrap().trim().to_string()
}

#[test]
fn test_detects_clean_update_as_noop() {
    let temp_dir = create_repo_with_openwiki();
    let cwd = temp_dir.path();

    let head = get_git_head(cwd);
    write_last_update(cwd, &head);

    let status = owly::metadata::get_update_noop_status(cwd);

    match status {
        owly::metadata::UpdateNoopStatus::Skip { .. } => {} // Expected
        owly::metadata::UpdateNoopStatus::Proceed { reason } => {
            panic!("Expected Skip, got Proceed: {}", reason);
        }
    }
}

#[test]
fn test_does_not_skip_with_uncommitted_changes() {
    let temp_dir = create_repo_with_openwiki();
    let cwd = temp_dir.path();

    let head = get_git_head(cwd);
    write_last_update(cwd, &head);

    // Make uncommitted changes
    fs::write(cwd.join("README.md"), "# Test Repo\nChanged\n").unwrap();

    let status = owly::metadata::get_update_noop_status(cwd);

    match status {
        owly::metadata::UpdateNoopStatus::Skip { .. } => {
            panic!("Expected Proceed, got Skip");
        }
        owly::metadata::UpdateNoopStatus::Proceed { .. } => {} // Expected
    }
}

#[test]
fn test_skips_when_only_openwiki_files_changed() {
    let temp_dir = create_repo_with_openwiki();
    let cwd = temp_dir.path();

    let head = get_git_head(cwd);
    write_last_update(cwd, &head);

    // Update only openwiki files
    fs::write(cwd.join("openwiki/quickstart.md"), "# Quickstart\nUpdated\n").unwrap();

    std::process::Command::new("git")
        .args(["add", "openwiki/quickstart.md"])
        .current_dir(cwd)
        .output()
        .expect("Failed to git add");

    std::process::Command::new("git")
        .args(["commit", "-m", "update openwiki docs"])
        .current_dir(cwd)
        .output()
        .expect("Failed to git commit");

    let status = owly::metadata::get_update_noop_status(cwd);

    match status {
        owly::metadata::UpdateNoopStatus::Skip { .. } => {} // Expected
        owly::metadata::UpdateNoopStatus::Proceed { reason } => {
            panic!("Expected Skip, got Proceed: {}", reason);
        }
    }
}

#[test]
fn test_does_not_skip_when_source_files_changed() {
    let temp_dir = create_repo_with_openwiki();
    let cwd = temp_dir.path();

    let head = get_git_head(cwd);
    write_last_update(cwd, &head);

    // Update source files
    fs::write(cwd.join("README.md"), "# Test Repo\nChanged\n").unwrap();

    std::process::Command::new("git")
        .args(["add", "README.md"])
        .current_dir(cwd)
        .output()
        .expect("Failed to git add");

    std::process::Command::new("git")
        .args(["commit", "-m", "update readme"])
        .current_dir(cwd)
        .output()
        .expect("Failed to git commit");

    let status = owly::metadata::get_update_noop_status(cwd);

    match status {
        owly::metadata::UpdateNoopStatus::Skip { .. } => {
            panic!("Expected Proceed, got Skip");
        }
        owly::metadata::UpdateNoopStatus::Proceed { .. } => {} // Expected
    }
}

#[test]
fn test_is_update_noop_returns_true_when_skipping() {
    let temp_dir = create_repo_with_openwiki();
    let cwd = temp_dir.path();

    let head = get_git_head(cwd);
    write_last_update(cwd, &head);

    assert!(owly::metadata::is_update_noop(cwd));
}

#[test]
fn test_is_update_noop_returns_false_when_not_skipping() {
    let temp_dir = create_repo_with_openwiki();
    let cwd = temp_dir.path();

    let head = get_git_head(cwd);
    write_last_update(cwd, &head);

    // Make uncommitted changes
    fs::write(cwd.join("README.md"), "# Test Repo\nChanged\n").unwrap();

    assert!(!owly::metadata::is_update_noop(cwd));
}

#[test]
fn test_create_git_summary() {
    let temp_dir = create_repo_with_openwiki();
    let cwd = temp_dir.path();

    let summary = owly::metadata::create_git_summary(cwd, None);

    assert!(summary.contains("git status --short"));
    assert!(summary.contains("git rev-parse HEAD"));
    assert!(summary.contains("git log"));
}

#[test]
fn test_load_metadata_nonexistent() {
    let temp_dir = TempDir::new().unwrap();
    let cwd = temp_dir.path();

    let metadata = owly::metadata::load_metadata(cwd);
    assert!(metadata.is_none());
}
