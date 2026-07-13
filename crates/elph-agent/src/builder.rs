use std::path::PathBuf;
use std::sync::Arc;

use elph_core::logger::LoggingOptions;

use crate::env::LocalExecutionEnv;
use crate::types::AgentTool;

/// Output of [`AgentBuilder::build`].
#[derive(Debug, Clone)]
pub struct AgentInit {
    pub app_version: &'static str,
    pub quiet_env: Option<&'static str>,
    pub logging: LoggingOptions,
}

/// Builder for application initialization settings shared across Elph apps.
///
/// Built-in tools are selected at compile time via Cargo features (`builtin-tools`, `tools-core`, …).
/// Use [`BuiltinToolsBuilder`] to assemble the enabled tools at runtime.
#[derive(Debug, Clone)]
pub struct AgentBuilder {
    app_version: &'static str,
    env_prefix: &'static str,
    app_name: &'static str,
    quiet_env: Option<&'static str>,
    logs_dir: Option<PathBuf>,
    console_enabled: bool,
}

impl AgentBuilder {
    pub fn new(app_version: &'static str) -> Self {
        Self {
            app_version,
            env_prefix: "",
            app_name: "",
            quiet_env: None,
            logs_dir: None,
            console_enabled: true,
        }
    }

    pub fn env_prefix(mut self, prefix: &'static str) -> Self {
        self.env_prefix = prefix;
        self
    }

    pub fn app_name(mut self, name: &'static str) -> Self {
        self.app_name = name;
        self
    }

    pub fn quiet_env(mut self, env: &'static str) -> Self {
        self.quiet_env = Some(env);
        self
    }

    pub fn logs_dir(mut self, dir: PathBuf) -> Self {
        self.logs_dir = Some(dir);
        self
    }

    pub fn console_enabled(mut self, enabled: bool) -> Self {
        self.console_enabled = enabled;
        self
    }

    pub fn build(self) -> AgentInit {
        AgentInit {
            app_version: self.app_version,
            quiet_env: self.quiet_env,
            logging: LoggingOptions::resolve(self.env_prefix, self.app_name, self.logs_dir, self.console_enabled),
        }
    }
}

/// Assembles compile-time enabled built-in tools for an agent harness.
#[derive(Clone)]
pub struct BuiltinToolsBuilder {
    env: Arc<LocalExecutionEnv>,
    include_web: bool,
}

impl BuiltinToolsBuilder {
    pub fn new(env: Arc<LocalExecutionEnv>) -> Self {
        Self {
            env,
            include_web: false,
        }
    }

    /// Start a builder that includes every built-in tool group enabled by Cargo features.
    pub fn all(env: Arc<LocalExecutionEnv>) -> Self {
        Self { env, include_web: true }
    }

    pub fn without_web(mut self) -> Self {
        self.include_web = false;
        self
    }

    pub fn with_web(mut self) -> Self {
        self.include_web = true;
        self
    }

    pub fn build(self) -> Vec<AgentTool> {
        let mut tools = Vec::new();
        #[cfg(feature = "tools-read")]
        tools.push(crate::tools::create_read_tool(self.env.clone()));
        #[cfg(feature = "tools-bash")]
        tools.push(crate::tools::create_bash_tool(self.env.clone()));
        #[cfg(feature = "tools-edit")]
        tools.push(crate::tools::create_edit_tool(self.env.clone()));
        #[cfg(feature = "tools-write")]
        tools.push(crate::tools::create_write_tool(self.env.clone()));
        #[cfg(feature = "tools-grep")]
        tools.push(crate::tools::create_grep_tool(self.env.clone()));
        #[cfg(feature = "tools-find")]
        tools.push(crate::tools::create_find_tool(self.env.clone()));
        #[cfg(feature = "tools-ls")]
        tools.push(crate::tools::create_ls_tool(self.env.clone()));
        if self.include_web {
            #[cfg(feature = "tools-web")]
            tools.extend(crate::tools::create_web_tools());
        }
        tools
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn builder_resolves_logging_without_logs_dir() {
        let init = AgentBuilder::new("0.0.12-test")
            .env_prefix("ELPH")
            .app_name("elph")
            .console_enabled(false)
            .build();

        assert_eq!(init.app_version, "0.0.12-test");
        assert!(!init.logging.file_enabled);
        assert!(!init.logging.console_enabled);
        assert_eq!(init.logging.level, "info");
    }

    #[cfg(feature = "builtin-tools")]
    #[test]
    fn builtin_tools_builder_includes_all_enabled_groups() {
        let env = Arc::new(LocalExecutionEnv::new(PathBuf::from(".").as_path()));
        let tools = BuiltinToolsBuilder::all(env).build();
        let names: Vec<_> = tools.iter().map(|tool| tool.name().to_string()).collect();
        assert!(names.contains(&"read".to_string()));
        assert!(names.contains(&"bash".to_string()));
        assert!(names.contains(&"grep".to_string()));
        assert!(names.contains(&"websearch".to_string()));
    }
}
