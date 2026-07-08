//! Tests for Owly filesystem error handling.
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki)
//! `test/fs-errors.test.ts`. Original MIT License, Copyright (c) 2026 LangChain.
//!
//! Note: Rust's error handling is different from TypeScript's, so these tests
//! verify that our error handling functions work correctly with anyhow::Error.

use anyhow::anyhow;

/// Check if an error is a "file not found" type error
fn is_file_not_found_error(error: &anyhow::Error) -> bool {
    let error_str = format!("{error}");
    error_str.contains("not found") || error_str.contains("No such file")
}

/// Check if an error is an expected snapshot race error
fn is_expected_snapshot_race_error(error: &anyhow::Error) -> bool {
    let error_str = format!("{error}").to_lowercase();
    error_str.contains("eisdir")
        || error_str.contains("enoent")
        || error_str.contains("enotdir")
        || error_str.contains("not found")
        || error_str.contains("is a directory")
        || error_str.contains("no such file")
        || error_str.contains("not a directory")
}

#[test]
fn test_is_file_not_found_error_for_not_found() {
    let error = anyhow!("No such file or directory");
    assert!(is_file_not_found_error(&error));
}

#[test]
fn test_is_file_not_found_error_for_other_error() {
    let error = anyhow!("Permission denied");
    assert!(!is_file_not_found_error(&error));
}

#[test]
fn test_is_file_not_found_error_for_generic_error() {
    let error = anyhow!("something went wrong");
    assert!(!is_file_not_found_error(&error));
}

#[test]
fn test_is_expected_snapshot_race_error_for_enoent() {
    let error = anyhow!("No such file or directory");
    assert!(is_expected_snapshot_race_error(&error));
}

#[test]
fn test_is_expected_snapshot_race_error_for_eisdir() {
    let error = anyhow!("Is a directory");
    assert!(is_expected_snapshot_race_error(&error));
}

#[test]
fn test_is_expected_snapshot_race_error_for_enotdir() {
    let error = anyhow!("Not a directory");
    assert!(is_expected_snapshot_race_error(&error));
}

#[test]
fn test_is_expected_snapshot_race_error_for_other_error() {
    let error = anyhow!("Permission denied");
    assert!(!is_expected_snapshot_race_error(&error));
}

#[test]
fn test_is_expected_snapshot_race_error_for_generic_error() {
    let error = anyhow!("something went wrong");
    assert!(!is_expected_snapshot_race_error(&error));
}
