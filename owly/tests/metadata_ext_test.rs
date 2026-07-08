//! Extended tests for Owly metadata module.

use owly::metadata::*;
use tempfile::TempDir;

#[test]
fn test_update_metadata_serialization() {
    use chrono::Utc;

    let metadata = UpdateMetadata {
        updated_at: Utc::now(),
        command: "init".to_string(),
        git_head: Some("abc123".to_string()),
        model: "opencode/big-pickle".to_string(),
    };

    let json = serde_json::to_string_pretty(&metadata).unwrap();
    assert!(json.contains("init"));
    assert!(json.contains("abc123"));
    assert!(json.contains("opencode/big-pickle"));

    // Deserialize back
    let deserialized: UpdateMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.command, "init");
    assert_eq!(deserialized.git_head, Some("abc123".to_string()));
    assert_eq!(deserialized.model, "opencode/big-pickle");
}

#[test]
fn test_update_metadata_no_git_head() {
    use chrono::Utc;

    let metadata = UpdateMetadata {
        updated_at: Utc::now(),
        command: "update".to_string(),
        git_head: None,
        model: "opencode/big-pickle".to_string(),
    };

    let json = serde_json::to_string_pretty(&metadata).unwrap();
    assert!(json.contains("null"));

    let deserialized: UpdateMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.git_head, None);
}

#[test]
fn test_load_metadata_invalid_json() {
    let temp_dir = TempDir::new().unwrap();
    let cwd = temp_dir.path();

    // Create invalid JSON file
    let metadata_path = cwd.join("openwiki/.last-update.json");
    std::fs::create_dir_all(metadata_path.parent().unwrap()).unwrap();
    std::fs::write(&metadata_path, "invalid json").unwrap();

    let result = load_metadata(cwd);
    assert!(result.is_none());
}

#[test]
fn test_get_git_head_non_repo() {
    let temp_dir = TempDir::new().unwrap();
    let result = get_git_head(temp_dir.path());
    assert!(result.is_none());
}

#[test]
fn test_is_update_noop_no_metadata() {
    let temp_dir = TempDir::new().unwrap();
    let result = is_update_noop(temp_dir.path());
    assert!(!result);
}

#[test]
fn test_get_git_summary_non_repo() {
    let temp_dir = TempDir::new().unwrap();
    let summary = create_git_summary(temp_dir.path(), None);
    // Should return empty or partial summary
    assert!(summary.is_empty() || summary.contains("git"));
}
