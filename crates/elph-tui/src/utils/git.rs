use std::path::Path;

/// Reads the current branch name from `.git/HEAD` (no full repo scan).
pub fn read_git_branch(cwd: &Path) -> Option<String> {
    let head = std::fs::read_to_string(cwd.join(".git/HEAD")).ok()?;
    let trimmed = head.trim();
    if let Some(branch) = trimmed.strip_prefix("ref: refs/heads/") {
        Some(branch.to_string())
    } else if !trimmed.is_empty() {
        Some(trimmed.chars().take(8).collect())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_branch_ref() {
        let dir = std::env::temp_dir().join(format!("elph_git_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(dir.join(".git"));
        std::fs::write(dir.join(".git/HEAD"), "ref: refs/heads/main\n").unwrap();
        assert_eq!(read_git_branch(&dir), Some("main".to_string()));
        let _ = std::fs::remove_dir_all(dir);
    }
}
