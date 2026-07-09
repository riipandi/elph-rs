---
title: "Architecture"
last_updated: 2026-07-08T20:00:00Z
category: architecture
tags:
    - architecture
    - design
    - modules
status: published
---

# Architecture

## Overview

Owly is a CLI agent that generates and maintains codebase documentation. It follows a pipeline: **CLI → Command → Agent → LLM → Filesystem**.

```
User Input
    │
    ├── No arguments ───────────────────────────────────────────────┐
    │                                                               │
    ▼                                                               ▼
┌──────────┐    ┌───────────┐    ┌────────────────┐    ┌──────────────────────┐
│  cli.rs  │───▶│commands.rs│───▶│   agent.rs     │───▶│    elph-ai (LLM)     │
│ (parsing)│    │(dispatch) │    │ (prompt + run) │    │                      │
└──────────┘    └───────────┘    └────────────────┘    └──────────────────────┘
                      │                  │                        │
                      │        ┌─────────┴─────────┐              │
                      │        ▼                   ▼              │
                      │  ┌───────────────┐  ┌────────────────┐    │
                      │  │ elph-agent    │  │  ask_user.rs   │    │
                      │  │ (tools + run) │  │ (interactive)  │    │
                      │  └───────────────┘  └────────────────┘    │
                      │         │                                 │
                      ▼         ▼                                 │
                  ┌────────────────────┐                          │
                  │    Filesystem      │◀─────────────────────────┘
                  │  (openwiki/ docs)  │
                  └────────────────────┘
```

---

## Module Architecture

### 1. Entrypoint — [`main.rs`](../owly/src/main.rs)

Initializes `tracing` logging, parses CLI arguments via `clap`, and calls `cli.execute()`.

### 2. CLI Layer — [`cli.rs`](../owly/src/cli.rs)

Defines the `Cli` struct with clap derive macros. Supported flags:

- `--init` / `--update` — select mode
- `--model` — override provider/model
- `--print` / `--stream` / `--verbose` — output control (`--stream` shows text deltas, `--verbose` adds thinking in dimmed gray)
- `--directory` — working directory
- Trailing argument — chat message

The `execute()` method resolves the command enum and calls `run_command()`, forwarding the `stream` flag. When no arguments are provided, the CLI now dispatches to interactive chat (`Command::Chat { message: None }`) instead of exiting with an error — see [Interactive Mode](#interactive-mode) below.

**Banner output** uses ANSI color codes (cyan for logo, green for values, dimmed for labels).

**Source:** [`owly/src/cli.rs`](../owly/src/cli.rs) — ported from OpenWiki `src/cli.tsx`.

### 3. Command Dispatch — [`commands.rs`](../owly/src/commands.rs)

Three command variants:

| Command            | Behavior                                                                                                         |
| ------------------ | ---------------------------------------------------------------------------------------------------------------- |
| `Init`             | Checks if `openwiki/` exists. If yes, delegates to update path. If no, runs agent with init prompt.              |
| `Update`           | Checks if `openwiki/` exists. If no, delegates to init path. Checks no-op status. Runs agent with update prompt. |
| `Chat { message }` | With a message: single-turn chat (read-only tools). With `None`: multi-turn interactive chat (see below).        |

Each command:

1. Loads credentials from `~/.owly/.env`
2. Resolves configuration (provider, model)
3. Sets up environment
4. Prepares system + user prompts
5. Runs the agent
6. Saves update metadata on success (init/update only)

#### Interactive Mode

When `owly` is run with no arguments, it enters interactive multi-turn chat:

1. The agent is created once with read-only tools + `ask_text`, `ask_select`, `ask_confirm` tools
2. A prompt loop reads user input from stdin, sends it to the same agent instance (preserving conversation history)
3. The agent can ask the user questions via the `ask_*` tools (text input, selection, confirmation)
4. `/exit`, `/quit`, `exit`, or `quit` ends the session
5. Conversation checkpointing via `SqliteSaver` is initialized (future use)

**Source:** `run_interactive()` in [`owly/src/agent.rs`](../owly/src/agent.rs), interactive tools in [`owly/src/ask_user.rs`](../owly/src/ask_user.rs), checkpoint store in [`owly/src/checkpoint.rs`](../owly/src/checkpoint.rs).

**Source:** [`owly/src/commands.rs`](../owly/src/commands.rs) — ported from OpenWiki `src/commands.ts`.

### 4. Agent Layer — [`agent.rs`](../owly/src/agent.rs)

The core integration with `elph-agent` and `elph-ai`. Key functions:

- **`resolve_model_and_auth()`** — Extracted helper that resolves the model from config, obtains authentication, and returns the model handle, models arc, and stream function.
- **`create_event_subscriber()`** — Extracted factory that returns an `AgentListener` closure for streaming display. Controls spinner, text deltas, thinking deltas, and tool call logging based on `stream` and `verbose` flags.
- **`run_agent()`** — Accepts a `RunAgentOptions` struct (command name, system/user prompts, config, cwd, print/stream/verbose flags). Sets up the agent with tools, subscribes to streaming events via `create_event_subscriber()`, sends prompts, waits for completion.
- **`run_interactive()`** — Manages a multi-turn interactive chat session. Creates a persistent agent with read-only + `ask_user` tools, enters a stdin read loop, and preserves conversation history across turns.
- **`prepare_init_command()`** — Creates system prompt + init user prompt.
- **`prepare_update_command()`** — Creates system prompt + update user prompt (includes git summary).
- **`prepare_chat_command()`** — Creates system prompt + chat user prompt.

**`RunAgentOptions` struct** replaces the earlier positional-parameter approach. Fields: `command`, `system_prompt`, `user_prompt`, `config`, `cwd`, `print_mode`, `stream`, `verbose`.

**Tool selection:**

- Init/update mode: all tools (`read`, `bash`, `edit`, `write`, `grep`, `find`, `ls`)
- Chat mode (single-turn): read-only tools (`read`, `grep`, `find`, `ls`)
- Chat mode (interactive): read-only tools + `ask_text`, `ask_select`, `ask_confirm` (from [`ask_user.rs`](../owly/src/ask_user.rs))

The tool names are appended to the system prompt after tool selection, forming a line like `Available tools for this session: read, bash, edit, write, grep, find, ls`.

**Streaming:** The agent subscribes to `AgentEvent` variants to display progress:

- `TextDelta` — live text output (shown with `--stream` or `--verbose`)
- `ThinkingDelta` — model reasoning (shown only with `--verbose`, in dimmed gray)
- `ToolExecutionStart` / `ToolExecutionEnd` — tool call logging (in verbose mode)
- `AgentEnd` — final stats (tool call count)

**Source:** [`owly/src/agent.rs`](../owly/src/agent.rs) — ported from OpenWiki `src/agent/index.ts`.

### 5. Prompt Generation — [`prompts.rs`](../owly/src/prompts.rs)

Contains the full system prompt that defines Owly's behavior. The prompt variants include:

- **`create_system_prompt()`** — Base prompt used across all modes.
- **`create_interactive_system_prompt()`** — Extends the base prompt with instructions for interactive chat sessions: mentions the `ask_text`/`ask_select`/`ask_confirm` tools, tells the agent not to create or update docs unless asked, and defines exit commands (`/exit`, `/quit`).
- **Init/update/chat prompts** — Mode-specific user-facing text appended to the system prompt.

The base prompt includes:

- **Role definition**: Expert technical writer, software architect, product analyst
- **Run discipline**: Filesystem tool usage rules
- **Git discipline**: How to use git evidence
- **Existing documentation discipline**: How to handle existing docs
- **Security rules**: Secret redaction requirements
- **Documentation goals**: Quality standards
- **Section quality rules**: Page structure guidelines
- **Frontmatter requirements**: YAML frontmatter format

This instruction set guides the LLM's documentation behavior.

**Source:** [`owly/src/prompts.rs`](../owly/src/prompts.rs) — ported from OpenWiki `src/agent/prompt.ts`.

### 6. Configuration — [`config.rs`](../owly/src/config.rs)

The `Config` struct holds resolved provider, model ID, and working directory. `Config::resolve()`:

1. Checks `--model` flag (supports `provider/model` format)
2. Falls back to `OWLY_PROVIDER` / `OWLY_MODEL_ID` env vars
3. Falls back to auto-detection based on available API keys
4. Validates provider exists in known provider list
5. Warns if API key is missing but doesn't fail (agent will error with a clear message)

Also supports `~/.owly/config.json` for persistent settings.

**Source:** [`owly/src/config.rs`](../owly/src/config.rs) — ported from OpenWiki `src/constants.ts` and `src/env.ts`.

### 7. Provider Registry — [`constants.rs`](../owly/src/constants.rs)

Defines all supported LLM providers with their display labels and API key environment variables. See [configuration page](configuration.md) for the full list.

**Provider auto-detection:** Checks environment variables in priority order: `OPENCODE_API_KEY` → `ANTHROPIC_API_KEY` → `OPENAI_API_KEY` → etc.

**Source:** [`owly/src/constants.rs`](../owly/src/constants.rs).

### 8. Documentation Management — [`docs.rs`](../owly/src/docs.rs)

Handles reading/writing documentation files with frontmatter, creating snapshots for change detection, and saving update metadata.

**Snapshot system:** Before an update, a hash-based snapshot is taken of all `openwiki/` files. After the run, the new snapshot is compared to detect changes.

**Source:** [`owly/src/docs.rs`](../owly/src/docs.rs) — ported from OpenWiki `src/agent/utils.ts`.

### 9. Metadata Tracking — [`metadata.rs`](../owly/src/metadata.rs)

Tracks the last successful update in `openwiki/.last-update.json`. The no-op check:

1. Loads last update metadata
2. Compares current git HEAD to last known HEAD
3. Checks `git status --short` for uncommitted changes
4. Skips update if only `openwiki/` files changed since last HEAD

**Source:** [`owly/src/metadata.rs`](../owly/src/metadata.rs).

### 10. Supporting Modules

| Module           | Responsibility                                                                      | Source                                                  |
| ---------------- | ----------------------------------------------------------------------------------- | ------------------------------------------------------- |
| `ask_user.rs`    | Interactive tools: `ask_text`, `ask_select`, `ask_confirm` for multi-turn chat      | [`owly/src/ask_user.rs`](../owly/src/ask_user.rs)       |
| `checkpoint.rs`  | Conversation checkpointing with `SqliteSaver` (port of langgraph-checkpoint-sqlite) | [`owly/src/checkpoint.rs`](../owly/src/checkpoint.rs)   |
| `credentials.rs` | Loads `~/.owly/.env`, applies to process environment                                | [`owly/src/credentials.rs`](../owly/src/credentials.rs) |
| `env.rs`         | Validates environment, provides debug info                                          | [`owly/src/env.rs`](../owly/src/env.rs)                 |
| `frontmatter.rs` | Parses/generates YAML frontmatter                                                   | [`owly/src/frontmatter.rs`](../owly/src/frontmatter.rs) |
| `diagnostics.rs` | Redacts secrets from error output, detects provider 500s                            | [`owly/src/diagnostics.rs`](../owly/src/diagnostics.rs) |
| `utils.rs`       | HTML tag stripping utility                                                          | [`owly/src/utils.rs`](../owly/src/utils.rs)             |

---

## Agent Execution Flow (Init/Update)

```
1. CLI parses args → Command::Init or Command::Update
2. Credentials loaded from ~/.owly/.env
3. Config resolved (provider, model, cwd)
4. Environment validated (API key check)
5. System prompt built from prompts.rs + mode-specific instructions
6. User prompt built:
   - Init: repository context instructions
   - Update: last update metadata + git change summary
7. Agent created with:
   - System prompt (with available tool list appended)
   - Model (resolved via elph-ai)
   - Tools (all tools for init/update)
8. Event subscriptions attached (streaming display, controlled by `stream` and `verbose` flags)
9. User prompt sent to agent
10. Agent executes: thinks, calls tools (read files, write docs)
11. On completion: update metadata saved to .last-update.json
```

---

## Change Guidance

### Adding a new provider

1. Add entry to `provider_config()` in [`constants.rs`](../owly/src/constants.rs)
2. Add to `all_providers()` list
3. Add API key env var to `MANAGED_ENV_KEYS` in [`credentials.rs`](../owly/src/credentials.rs)
4. Add to auto-detect chain in `resolve_configured_provider()` in [`constants.rs`](../owly/src/constants.rs)
5. Add to `API_KEY_ENV_VARS` in [`diagnostics.rs`](../owly/src/diagnostics.rs) for redaction

### Modifying agent behavior

- **Prompts** are in [`prompts.rs`](../owly/src/prompts.rs) — base system prompt, interactive prompt, init/update/chat templates
- **Tool selection** by mode happens in [`agent.rs`](../owly/src/agent.rs) (`create_all_tools` vs `create_read_only_tools`); chat mode adds `ask_user` tools via `create_ask_text_tool()`, `create_ask_select_tool()`, `create_ask_confirm_tool()`; tool names are appended to the system prompt after selection
- **Streaming vs verbose**: `--stream` shows `TextDelta` only; `--verbose` shows everything including `ThinkingDelta` and tool call logs; controlled by the `stream` and `verbose` fields in `RunAgentOptions`
- **Event handling** for streaming display is in the `create_event_subscriber()` factory function, extracted from the inline closure in `run_agent()`
- **Interactive mode** is managed by `run_interactive()` in `agent.rs`, which creates a persistent agent and runs a stdin read loop across turns

### Adding a new command

1. Add variant to `Command` enum in [`commands.rs`](../owly/src/commands.rs)
2. Add match arm in `run_command()`
3. Add CLI flag in [`cli.rs`](../owly/src/cli.rs)
4. Add prompt preparation function in [`agent.rs`](../owly/src/agent.rs)

### Adding a new interactive tool

1. Add a creation function in [`ask_user.rs`](../owly/src/ask_user.rs) using `simple_tool()`
2. Import and push it in the tool setup sections of both `run_agent()` (chat path) and `run_interactive()` in [`agent.rs`](../owly/src/agent.rs)

### Relevant tests

When modifying any of these areas, run the corresponding tests:

| Area                | Test File(s)                                                       |
| ------------------- | ------------------------------------------------------------------ |
| Agent commands      | [`agent_test.rs`](../owly/tests/agent_test.rs)                     |
| Config resolution   | [`config_test.rs`](../owly/tests/config_test.rs)                   |
| Frontmatter         | [`frontmatter_ext_test.rs`](../owly/tests/frontmatter_ext_test.rs) |
| Metadata/no-op      | [`metadata_ext_test.rs`](../owly/tests/metadata_ext_test.rs)       |
| Prompts             | [`prompts_test.rs`](../owly/tests/prompts_test.rs)                 |
| Secret redaction    | [`redaction_ext_test.rs`](../owly/tests/redaction_ext_test.rs)     |
| Environment         | [`env_ext_test.rs`](../owly/tests/env_ext_test.rs)                 |
| Documentation files | [`docs_test.rs`](../owly/tests/docs_test.rs)                       |
