//! MiniJinja template engine for system prompts with custom delimiters.

use minijinja::{Environment, syntax::SyntaxConfig};
use thiserror::Error;

/// Custom delimiters (`${{` / `${%`) to avoid collisions with `{{` in markdown and code examples.
pub fn custom_prompt_syntax() -> SyntaxConfig {
    SyntaxConfig::builder()
        .variable_delimiters("${{", "}}")
        .block_delimiters("${%", "%}")
        .build()
        .expect("valid syntax config")
}

#[derive(Debug, Error)]
pub enum PromptRenderError {
    #[error("template render failed: {0}")]
    Render(#[from] minijinja::Error),
}

/// Registry for embedded system prompt templates.
#[derive(Clone)]
pub struct PromptTemplateEngine {
    env: Environment<'static>,
}

impl Default for PromptTemplateEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl PromptTemplateEngine {
    pub fn new() -> Self {
        let mut env = Environment::new();
        env.set_syntax(custom_prompt_syntax());
        Self { env }
    }

    pub fn register_embedded(&mut self, name: &'static str, source: &'static str) -> Result<(), PromptRenderError> {
        self.env.add_template(name, source)?;
        Ok(())
    }

    pub fn render<T: serde::Serialize>(&self, name: &str, ctx: &T) -> Result<String, PromptRenderError> {
        let template = self.env.get_template(name)?;
        Ok(template.render(ctx)?)
    }
}

impl std::fmt::Debug for PromptTemplateEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PromptTemplateEngine").finish_non_exhaustive()
    }
}

/// Shared engine with the generic base template pre-registered.
pub fn default_prompt_engine() -> PromptTemplateEngine {
    let mut engine = PromptTemplateEngine::new();
    engine
        .register_embedded("base", include_str!("../../templates/base.md"))
        .expect("base template is valid");
    engine
}
