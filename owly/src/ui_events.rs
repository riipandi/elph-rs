//! Agent → TUI event bridge (streaming text, tools, status).

/// Live UI events emitted while an agent run is in progress.
#[derive(Debug, Clone)]
pub enum AgentUiEvent {
    /// Init/update/chat command header.
    CommandStart {
        command: String,
        provider: String,
        model: String,
    },
    /// Command finished with a user-facing summary.
    CommandComplete { message: String, success: bool },
    /// Short status line (auth, noop skip, progress).
    Status(String),
    /// Incremental assistant text.
    TextDelta(String),
    /// Incremental thinking text (verbose mode).
    ThinkingDelta(String),
    /// Tool invocation started.
    ToolStart {
        id: String,
        name: String,
        args_summary: String,
    },
    /// Streaming/partial tool output.
    ToolUpdate { id: String, output: String },
    /// Tool invocation finished.
    ToolEnd { id: String, is_error: bool, output: String },
    /// Agent run finished.
    RunCompleted { elapsed_secs: f64 },
}
