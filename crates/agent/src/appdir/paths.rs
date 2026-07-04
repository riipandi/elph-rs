use std::path::{Path, PathBuf};

const CONFIG_DIR_NAME: &str = ".elph";
const DATA_DIR_NAME: &str = "elph";

/// Resolved Elph config and data directories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

impl Paths {
    /// Resolve paths from `ELPH_HOME` and `XDG_DATA_HOME` (or defaults).
    pub fn resolve() -> std::io::Result<Self> {
        Ok(Self::from_dirs(config_dir()?, data_dir()?))
    }

    pub fn from_dirs(config_dir: PathBuf, data_dir: PathBuf) -> Self {
        Self { config_dir, data_dir }
    }

    pub fn settings_path(&self) -> PathBuf {
        self.config_dir.join("settings.json")
    }

    pub fn trust_path(&self) -> PathBuf {
        self.config_dir.join("trust.json")
    }

    pub fn bundled_dir(&self) -> PathBuf {
        self.config_dir.join("bundled")
    }

    pub fn bundled_manifest_path(&self) -> PathBuf {
        self.bundled_dir().join("manifest.json")
    }

    pub fn prompts_dir(&self) -> PathBuf {
        self.config_dir.join("prompts")
    }

    pub fn providers_dir(&self) -> PathBuf {
        self.config_dir.join("providers")
    }

    pub fn sessions_dir(&self) -> PathBuf {
        self.config_dir.join("sessions")
    }

    pub fn skills_dir(&self) -> PathBuf {
        self.config_dir.join("skills")
    }

    pub fn worktrees_dir(&self) -> PathBuf {
        self.config_dir.join("worktrees")
    }

    pub fn attachments_dir(&self) -> PathBuf {
        self.data_dir.join("attachments")
    }

    pub fn downloads_dir(&self) -> PathBuf {
        self.data_dir.join("downloads")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.data_dir.join("logs")
    }

    pub fn vendor_dir(&self) -> PathBuf {
        self.data_dir.join("vendor")
    }

    pub fn metadata_db_path(&self) -> PathBuf {
        self.data_dir.join("metadata.db")
    }

    pub fn memory_db_path(&self) -> PathBuf {
        self.data_dir.join("memory.db")
    }

    pub fn version_path(&self) -> PathBuf {
        self.data_dir.join("version.json")
    }

    /// All directories that must exist after initialization.
    pub fn required_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.bundled_dir().join("agents"),
            self.bundled_dir().join("personas"),
            self.bundled_dir().join("skills"),
            self.bundled_dir().join("user-guide"),
            self.prompts_dir(),
            self.providers_dir(),
            self.sessions_dir(),
            self.skills_dir(),
            self.worktrees_dir(),
            self.attachments_dir(),
            self.downloads_dir(),
            self.logs_dir(),
            self.vendor_dir(),
        ]
    }
}

fn config_dir() -> std::io::Result<PathBuf> {
    if let Ok(home) = std::env::var("ELPH_HOME") {
        let trimmed = home.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }

    let home = user_home()?;
    Ok(home.join(CONFIG_DIR_NAME))
}

fn data_dir() -> std::io::Result<PathBuf> {
    if let Ok(dir) = std::env::var("ELPH_DATA_DIR") {
        let trimmed = dir.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }

    if let Ok(xdg) = std::env::var("XDG_DATA_HOME") {
        let trimmed = xdg.trim();
        if !trimmed.is_empty() {
            return Ok(Path::new(trimmed).join(DATA_DIR_NAME));
        }
    }

    let home = user_home()?;
    Ok(home.join(".local").join("share").join(DATA_DIR_NAME))
}

fn user_home() -> std::io::Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HOME is not set"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_expected_file_paths() {
        let tmp = tempfile::tempdir().expect("tempdir");
        let config = tmp.path().join("config");
        let data = tmp.path().join("data");
        let paths = Paths::from_dirs(config.clone(), data.clone());

        assert_eq!(paths.metadata_db_path(), data.join("metadata.db"));
        assert_eq!(paths.memory_db_path(), data.join("memory.db"));
        assert_eq!(paths.bundled_manifest_path(), config.join("bundled/manifest.json"));
        assert_eq!(paths.required_dirs().len(), 13);
    }
}
