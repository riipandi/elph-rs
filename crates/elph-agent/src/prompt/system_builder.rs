//! System prompt assembly for generic agent hosts.

use thiserror::Error;

use super::context::SystemPromptTemplateContext;
use super::template::{PromptRenderError, PromptTemplateEngine, default_prompt_engine};

/// How a domain-specific body is combined with the generic base template.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PromptAssemblyMode {
    /// Render `base` then append the domain template or body.
    #[default]
    Extend,
    /// Render only the domain template (or raw body).
    Full,
}

#[derive(Debug, Error)]
pub enum SystemPromptBuildError {
    #[error("prompt template error: {0}")]
    Template(#[from] PromptRenderError),
    #[error("no domain template or body configured for full assembly mode")]
    MissingDomain,
}

/// Pi-style project context wrapper for AGENTS.md / context files.
pub fn format_project_context(path: &str, content: &str) -> String {
    format!(
        "<project_context>\n\nProject-specific instructions and guidelines:\n\n\
         <project_instructions path=\"{path}\">\n{content}\n</project_instructions>\n\n\
         </project_context>"
    )
}

fn append_project_context(out: &mut String, agents_md: &str) {
    if agents_md.trim().is_empty() {
        return;
    }
    out.push_str("\n\n");
    out.push_str(&format_project_context("AGENTS.md", agents_md.trim()));
}

/// Builder for host- and domain-specific system prompts.
#[derive(Debug, Clone)]
pub struct SystemPromptBuilder {
    engine: PromptTemplateEngine,
    mode: PromptAssemblyMode,
    context: SystemPromptTemplateContext,
    domain_template: Option<&'static str>,
    domain_body: Option<String>,
}

impl Default for SystemPromptBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl SystemPromptBuilder {
    pub fn new() -> Self {
        Self {
            engine: default_prompt_engine(),
            mode: PromptAssemblyMode::Extend,
            context: SystemPromptTemplateContext::default(),
            domain_template: None,
            domain_body: None,
        }
    }

    pub fn with_engine(engine: PromptTemplateEngine) -> Self {
        Self {
            engine,
            mode: PromptAssemblyMode::Extend,
            context: SystemPromptTemplateContext::default(),
            domain_template: None,
            domain_body: None,
        }
    }

    pub fn mode(mut self, mode: PromptAssemblyMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn context(mut self, context: SystemPromptTemplateContext) -> Self {
        self.context = context;
        self
    }

    pub fn persona(mut self, persona: impl Into<String>) -> Self {
        self.context.persona = persona.into();
        self
    }

    pub fn domain_template(mut self, name: &'static str) -> Self {
        self.domain_template = Some(name);
        self
    }

    pub fn register_domain_template(
        mut self,
        name: &'static str,
        source: &'static str,
    ) -> Result<Self, SystemPromptBuildError> {
        self.engine.register_embedded(name, source)?;
        Ok(self)
    }

    pub fn domain_body(mut self, body: impl Into<String>) -> Self {
        self.domain_body = Some(body.into());
        self
    }

    pub fn render(&self) -> Result<String, SystemPromptBuildError> {
        match self.mode {
            PromptAssemblyMode::Extend => {
                let mut out = self.engine.render("base", &self.context)?;
                if let Some(body) = &self.domain_body {
                    if !body.trim().is_empty() {
                        out.push_str("\n\n");
                        out.push_str(body);
                    }
                } else if let Some(name) = self.domain_template {
                    out.push_str("\n\n");
                    out.push_str(&self.engine.render(name, &self.context)?);
                }
                if !self.context.mode_section.trim().is_empty() {
                    out.push_str("\n\n");
                    out.push_str(&self.context.mode_section);
                }
                append_project_context(&mut out, &self.context.agents_md);
                Ok(out)
            }
            PromptAssemblyMode::Full => {
                let mut out = if let Some(name) = self.domain_template {
                    self.engine.render(name, &self.context)?
                } else if let Some(body) = &self.domain_body {
                    body.clone()
                } else {
                    return Err(SystemPromptBuildError::MissingDomain);
                };
                if !self.context.mode_section.trim().is_empty() {
                    out.push_str("\n\n");
                    out.push_str(&self.context.mode_section);
                }
                if !self.context.skills_section.trim().is_empty() && !out.contains(&self.context.skills_section) {
                    out.push_str("\n\n");
                    out.push_str(&self.context.skills_section);
                }
                if !self.context.agents_md.trim().is_empty() && !out.contains(&self.context.agents_md) {
                    append_project_context(&mut out, &self.context.agents_md);
                }
                Ok(out)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_context_uses_pi_xml_wrapper() {
        let block = format_project_context("AGENTS.md", "Be concise.");
        assert!(block.contains("<project_context>"));
        assert!(block.contains("<project_instructions path=\"AGENTS.md\">"));
        assert!(block.contains("Be concise."));
    }
}
