//! YAML frontmatter parsing and generation for Owly documentation.
//!
//! Ported from [OpenWiki](https://github.com/langchain-ai/openwiki)
//! `src/constants.ts`. Original MIT License, Copyright (c) 2026 LangChain.
//!
//! Uses the `yaml-front-matter` crate for standard YAML frontmatter parsing.
//! Frontmatter is used to store metadata like title, category, tags,
//! and last update timestamp.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use yaml_front_matter::{Document, YamlFrontMatter};

/// Frontmatter metadata for a documentation file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Frontmatter {
    /// Document title
    pub title: String,

    /// When the document was last updated (ISO 8601)
    pub last_updated: String,

    /// Documentation category (e.g., "quickstart", "architecture", "workflow")
    pub category: String,

    /// Optional tags for the document
    #[serde(default)]
    pub tags: Vec<String>,

    /// Document status ("draft", "review", or "published")
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "published".to_string()
}

impl Frontmatter {
    /// Create new frontmatter with current timestamp
    pub fn new(title: &str, category: &str) -> Self {
        Self {
            title: title.to_string(),
            last_updated: Utc::now().to_rfc3339(),
            category: category.to_string(),
            tags: Vec::new(),
            status: "published".to_string(),
        }
    }

    /// Set tags for the frontmatter
    pub fn with_tags(mut self, tags: &[&str]) -> Self {
        self.tags = tags.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Set status for the frontmatter
    pub fn with_status(mut self, status: &str) -> Self {
        self.status = status.to_string();
        self
    }

    /// Parse frontmatter from a markdown document
    pub fn parse(content: &str) -> Option<(Self, String)> {
        let document: Document<Frontmatter> = YamlFrontMatter::parse(content).ok()?;
        Some((document.metadata, document.content))
    }

    /// Serialize frontmatter to YAML string
    pub fn to_yaml(&self) -> String {
        let mut lines = vec![
            "---".to_string(),
            format!("title: \"{}\"", self.title),
            format!("last_updated: {}", self.last_updated),
            format!("category: {}", self.category),
        ];

        if !self.tags.is_empty() {
            lines.push("tags:".to_string());
            for tag in &self.tags {
                lines.push(format!("  - {}", tag));
            }
        }

        lines.push(format!("status: {}", self.status));
        lines.push("---".to_string());

        lines.join("\n")
    }
}

/// Extract frontmatter from a markdown file
pub fn extract_frontmatter(content: &str) -> Option<Frontmatter> {
    Frontmatter::parse(content).map(|(fm, _)| fm)
}

/// Extract frontmatter and body from a markdown file
pub fn extract_frontmatter_and_body(content: &str) -> Option<(Frontmatter, String)> {
    Frontmatter::parse(content)
}

/// Check if content has frontmatter
pub fn has_frontmatter(content: &str) -> bool {
    let content = content.trim_start();
    content.starts_with("---")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_frontmatter() {
        let content = r#"---
title: "Quickstart Guide"
last_updated: 2024-01-15T10:30:00Z
category: quickstart
tags:
  - getting-started
  - overview
status: published
---

# Quickstart

This is a quickstart guide."#;

        let (frontmatter, body) = Frontmatter::parse(content).unwrap();
        assert_eq!(frontmatter.title, "Quickstart Guide");
        assert_eq!(frontmatter.category, "quickstart");
        assert_eq!(frontmatter.tags, vec!["getting-started", "overview"]);
        assert_eq!(frontmatter.status, "published");
        assert!(body.contains("# Quickstart"));
    }

    #[test]
    fn test_parse_frontmatter_no_tags() {
        let content = r#"---
title: "Architecture"
last_updated: 2024-01-15T10:30:00Z
category: architecture
status: published
---

# Architecture

Architecture details."#;

        let (frontmatter, body) = Frontmatter::parse(content).unwrap();
        assert_eq!(frontmatter.title, "Architecture");
        assert!(frontmatter.tags.is_empty());
        assert!(body.contains("# Architecture"));
    }

    #[test]
    fn test_no_frontmatter() {
        let content = "# No Frontmatter\n\nThis document has no frontmatter.";
        assert!(Frontmatter::parse(content).is_none());
    }

    #[test]
    fn test_generate_frontmatter() {
        let fm = Frontmatter::new("Test Document", "test")
            .with_tags(&["tag1", "tag2"])
            .with_status("draft");

        let yaml = fm.to_yaml();
        assert!(yaml.starts_with("---\n"));
        assert!(yaml.contains("title: \"Test Document\""));
        assert!(yaml.contains("category: test"));
        assert!(yaml.contains("tags:"));
        assert!(yaml.contains("  - tag1"));
        assert!(yaml.contains("  - tag2"));
        assert!(yaml.contains("status: draft"));
        assert!(yaml.ends_with("---"));
    }

    #[test]
    fn test_roundtrip() {
        let original = Frontmatter::new("Test", "test").with_tags(&["a", "b"]);

        let yaml = original.to_yaml();
        let content = format!("{}\n\n# Content", yaml);

        let (parsed, body) = Frontmatter::parse(&content).unwrap();
        assert_eq!(parsed.title, "Test");
        assert_eq!(parsed.category, "test");
        assert_eq!(parsed.tags, vec!["a", "b"]);
        assert!(body.contains("# Content"));
    }

    #[test]
    fn test_has_frontmatter() {
        assert!(has_frontmatter("---\ntitle: test\ncategory: test\n---"));
        assert!(!has_frontmatter("# No frontmatter"));
        assert!(has_frontmatter("\n---\ntitle: test\ncategory: test\n---"));
    }

    #[test]
    fn test_extract_frontmatter() {
        let content = r#"---
title: "Test"
last_updated: 2024-01-15T10:30:00Z
category: test
---

Body content."#;

        let fm = extract_frontmatter(content).unwrap();
        assert_eq!(fm.title, "Test");
    }
}
