# Elph — OpenWiki Quickstart

**Elph** is a Rust workspace for AI agent applications: a coding agent CLI, shared agent runtime libraries, and terminal UI components. It is a port of the [pi](https://pi.dev) TypeScript ecosystem to Rust, with additional MCP (Model Context Protocol) support, WASM extensions, and an iocraft-based interactive TUI.

## Repository overview

| Layer       | Crate               | Purpose                                                                                                                    |
| ----------- | ------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| **Binary**  | `elph/`             | Coding agent CLI + TUI — product shell                                                                                     |
| **Runtime** | `crates/elph-agent` | App-agnostic agent runtime: turn loop, session persistence, compaction, goals, subagents, skills, MCP client, WASM plugins |
| **AI**      | `crates/elph-ai`    | Unified LLM provider layer: model catalog, provider abstraction, OAuth, image generation, web tools                        |
| **Core**    | `crates/elph-core`  | Shared primitives: `floppy` memory store (Turso vector DB), logger, path resolution, filesystem helpers                    |
| **TUI**     | `crates/elph-tui`   | iocraft component stubs + examples; primary TUI in `elph` binary (`tui.rs`)                                                |
| **Swarm**   | `crates/elph-swarm` | Multi-agent coordination (early stage)                                                                                     |

## Key concepts

- **Agent** — `elph::agent` wraps `elph-agent`'s `AgentHarness` with session orchestration for the coding use case.
- **AgentHarness** — Stateful, session-backed agent runner with hooks, compaction, branching, and plan mode.
- **Agent Loop** — Low-level turn runner: stream completion → tool call → result → repeat until model stops.
- **Session** — Tree-structured persistence (filesystem or Turso). Sessions can fork, branch, and resume.
- **Compaction** — Automatic context window management via summarization and branch clipping.
- **Goals** — Persisted session objectives with auto-steering.
- **Subagents** — Codex-style multi-agent orchestration (spawn, control, merge).
- **MCP** — Model Context Protocol client supporting stdio, streamable HTTP, and SSE transports with OAuth 2.1 and AES-256-GCM credential encryption.
- **Skills** — Reusable `SKILL.md` files following the [agentskills.io](https://agentskills.io) spec.
- **TOON Encoding** — Optional structured-data encoding for tool results (reduces token usage on tabular payloads).
- **Extensions** — WASM-based dynamic plugins compiled with `wasmtime`.

## Project state (HEAD `fbdfa1e`)

This repository is under **active development**. Recent milestones:

- **TUI rich markdown rendering** — Full markdown-to-terminal pipeline in `crates/elph-tui/src/components/markdown/`: syntax-highlighted code blocks (syntect with Tokyo Night theme), auto-linked URLs, streaming tail rendering (`aa0aa69`, `a80a7f6`).
- **Slash palette** — Autocomplete overlay with fuzzy search, keyboard navigation, and skill command registration (`75f5b21`, `3995657`).
- **Focus switching** — `ShellFocus::Prompt` / `Transcript` with Esc-to-prompt, scroll keys, and auto-refocus on character input (`96fa4f7`).
- **Tool approval modal** — Key-driven approval (`y`/`a`/`n`) in a bordered panel replacing the editor during tool confirmation (`55485b2`).
- **Stream token tracking** — Real-time `+<delta> · <t/s>` display in the status row, turn count in chrome stats (`c90341f`, `384f9f8`).
- **Quit confirmation during active turn** — Two-step quit flow: transcript notice + `y quit / n stay` key binding, force-quit with `/exit!` (`fbdfa1e`).
- **Skills argument hints** — `SKILL.md` frontmatter now supports `argument-hint` for validating user-provided arguments and registering `/skill:<name>` slash invocations (`3995657`).
- **Subagent abort** — Improved control channel for aborting subagents mid-execution (`a80a7f6`).
- **elph-tui progress indicators** — Replaced `indicatif` with native `cli_progress.rs` component (`55485b2`).
- **TUI overhaul** — Replaced single-file `tui.rs` with a modular `tui/` directory (shell, focus, tool_approval, user_question, activity, agent_bridge, chrome, prompt, transcript, slash_palette). elph-tui crate now has 17+ implemented component modules, markdown sub-modules, textarea/ directory with sub-modules, and 14 integration tests.
- **Session IDs migrated from TSID to Kalid** — Time-sortable 16-char IDs with no prefix, replacing 13-char TSID. Floppy memory store also migrated (`066dd00`).

## Documentation map

| Page                                                 | What it covers                                                           |
| ---------------------------------------------------- | ------------------------------------------------------------------------ |
| [quickstart.md](quickstart.md)                       | This page — overview and navigation                                      |
| [architecture/overview.md](architecture/overview.md) | Crate architecture, module map, design principles                        |
| [agent-runtime.md](agent-runtime.md)                 | Agent harness, sessions, turn loop, compaction, goals, subagents, skills |
| [ai-providers.md](ai-providers.md)                   | Model catalog, provider APIs, auth/OAuth, image generation               |
| [mcp-integration.md](mcp-integration.md)             | MCP client, transports, OAuth, encryption, validation, policy            |
| [tui-shell.md](tui-shell.md)                         | iocraft TUI, elph-tui components, slash commands, prompt encoding        |
| [operations.md](operations.md)                       | CLI commands, settings, paths, CI/CD, publishing                         |
| [testing.md](testing.md)                             | Test structure, key patterns, running tests                              |

## Source reading order

For new contributors or future agents:

1. **Cargo.toml** (`/Cargo.toml`) — workspace manifest, dependency versions
2. **elph/src/main.rs** — binary entrypoint
3. **elph/src/cli/mod.rs** — CLI subcommand definitions
4. **crates/elph-agent/src/lib.rs** — agent runtime public API surface
5. **crates/elph-ai/src/lib.rs** — AI provider layer public API
6. **crates/elph-tui/src/lib.rs** — TUI component stubs (16 component modules); examples in `crates/elph-tui/examples/`
7. **crates/elph-core/src/lib.rs** — core library surface

## Quick build & test

```sh
# Prerequisites: Rust >= 1.97
make prepare          # install toolchain, setup hooks
make check            # cargo check --workspace
make test             # cargo nextest run
make build            # release build elph binary
make run              # cargo run --bin elph
make lint             # cargo clippy --workspace -D warnings
```

See [operations.md](operations.md) for CI/CD and publishing, [testing.md](testing.md) for test patterns.

## Design docs

Product design specs live in [`docs/`](/docs/README.md). The `docs/` folder holds _what_ Elph should do; this wiki holds _how_ it works today.
