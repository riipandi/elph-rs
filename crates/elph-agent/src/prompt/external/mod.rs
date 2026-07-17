//! Load slash-command prompt templates from the filesystem.

mod load;
mod parse;

pub use load::{load_prompt_templates, load_sourced_prompt_templates};
