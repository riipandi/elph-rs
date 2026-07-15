//! Shared types for interactive components.

use serde::{Deserialize, Serialize};

/// One selectable row (OpenTUI Select option).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectOption {
    pub name: String,
    pub description: String,
}

impl SelectOption {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

/// One tab in a tab selector.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TabItem {
    pub label: String,
    pub content: String,
}

impl TabItem {
    pub fn new(label: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            content: content.into(),
        }
    }
}
