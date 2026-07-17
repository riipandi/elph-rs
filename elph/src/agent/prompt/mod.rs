//! Coding-agent system prompt templates and builders.

mod agents_md;
mod builder;
mod modes;

pub use agents_md::agents_md_for_cwd;
pub use builder::build_coding_system_prompt;
