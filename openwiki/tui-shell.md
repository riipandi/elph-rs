# TUI & Shell

The TUI system currently lives directly in the `elph` binary crate while the iocraft-based shell is being rebuilt iteratively.

The `elph-tui` library crate provides iocraft component modules (17+ modules) and 21 examples, with integration tests. Once the public API stabilises, the reusable widget library will be extracted back into `elph-tui` and published to crates.io.

**TUI source**: `/elph/src/tui/` — Modular iocraft-based shell: `shell.rs`, `editor.rs`, `transcript/`, `agent_bridge.rs`, `chrome.rs`, `activity.rs`, and more.
**Shell**: No separate `shell/` module — the interactive TUI replaced the old shell module. `TuiOptions` is in `elph/src/tui/shell.rs`.

## elph-tui (Widget Library — Component Stubs)

**Path**: `/crates/elph-tui/`

The crate provides `lib.rs` with 17+ component modules under `components/` (most are implemented, not stubs):

- `ascii_font`, `card`, `code`, `diff`, `frame_buffer`, `input`, `line_numbers`
- `markdown`, `progress_indicator`, `qr_code`, `scroll_bar`, `scroll_box`, `select`, `slider`, `tab_select`, `text`, `textarea`

`textarea` is a directory containing `component.rs`, `input/` (paste, submit, wire_edit sub-modules), `layout.rs`, and `state.rs`. Additional modules live at the crate root: `text_editing/` (actions, input, line, submit, wire), `transcript_layout.rs`, `text_input_layout.rs`, `loader.rs`, `paste.rs`, and `utils.rs`.

In the `elph` binary TUI (`/elph/src/tui/transcript/message.rs`), the transcript now renders structured tool invocation cards via `ToolCardDetail` (name, args summary, output body), replacing inline text formatting. Theme constants `TOOL_ARGS_FG` and `TOOL_OUTPUT_FG` control card colors. The previous `format_tool_card_content` / `format_tool_card_result` public helpers have been removed in favor of the structured `tool_call()` constructor on `TranscriptMessage`.

### Examples (21 total in `crates/elph-tui/examples/`)

| Example             | Description                                           |
| ------------------- | ----------------------------------------------------- |
| `weather`           | Async data loading from remote APIs with iocraft      |
| `calculator`        | Calculator app with iocraft UI                        |
| `chat_layout`       | Chat layout with scrollable content, input, tool cards|
| `progress_bar`      | Animated progress bar demo                            |
| `basic_context`     | Context API usage example                             |
| `basic_counter`     | Simple counter with state management                  |
| `basic_form`        | Form with input validation                            |
| `basic_input`       | Text input handling demo                              |
| `basic_layout`      | Layout composition demo                               |
| `basic_output`      | Text output display                                   |
| `basic_overlap`     | Overlapping elements demo                             |
| `basic_scrolling`   | Scrollable content                                    |
| `basic_table`       | Table layout demo                                     |
| `demo_code`         | Code block rendering                                  |
| `demo_diff`         | Diff output rendering                                 |
| `demo_input`        | Input widget demo                                     |
| `demo_markdown`     | Markdown rendering                                    |
| `demo_scroll`        | Scroll container demo                                 |
| `demo_select`       | Select widget demo                                    |
| `demo_special`      | Special elements demo                                 |
| `demo_text_card`    | Text card rendering                                   |

Run examples with: `cargo run -p elph-tui --example <name>`

## Slash Commands

**File**: `/elph/src/agent/slash_commands.rs` (implementation)
**Design doc**: `/docs/slash-commands.md`

Dispatch order:

1. **Built-in commands** (defined in `elph-builtin_commands`)
2. **Extension commands** (WASM plugins)
3. **Prompt templates** (`~/.elph/prompts/*.md` and `<project>/.elph/prompts/*.md`)

### Built-in commands

| Command                     | Aliases       | Description                                 |
| --------------------------- | ------------- | ------------------------------------------- |
| `/help`                     | —             | List all commands                           |
| `/model`                    | —             | Open model selector                         |
| `/goal`                     | `/goals`      | Manage session goals                        |
| `/exit`                     | `/quit`, `/q` | Quit                                        |
| `/commit`                   | —             | Generate commit message from staged changes |
| `/compact`                  | `/c`          | Compact history                             |
| `/reload`                   | —             | Reload extensions + resources               |
| `/diagnostic:list-tools`    | —             | List available tools                        |
| `/diagnostic:system-prompt` | —             | Show assembled system prompt                |
| `/diagnostic:open-log`      | —             | Open session log                            |

## TOON Prompt Encoding

**File**: `/crates/elph-agent/src/prompt/encoding/`

Optional structured-data encoding for tool results using the [TOON format](https://github.com/toon-format/toon). Enabled via `ELPH_PROMPT_ENCODING` env var or harness config.

| Mode   | Behavior                                                             |
| ------ | -------------------------------------------------------------------- |
| `off`  | Default — tool results pass through unchanged                        |
| `toon` | Encode eligible JSON at or above size threshold (default 1024 chars) |
| `auto` | Encode only uniform tabular JSON arrays                              |

### Implementation

| File           | Purpose                                                 |
| -------------- | ------------------------------------------------------- |
| `config.rs`    | `PromptEncodingConfig`, modes, targets, size thresholds |
| `encode.rs`    | `encode_value` — TOON encoding                          |
| `decode.rs`    | `decode_toon_fence` — Decode TOON fenced blocks         |
| `apply.rs`     | `apply_to_tool_result` — Apply encoding to tool results |
| `extract.rs`   | `extract_json_value` — Extract JSON from delimiters     |
| `fence.rs`     | `parse_toon_fence` — Parse TOON fence markers           |
| `heuristic.rs` | Heuristic detection of uniform tabular arrays           |

### Savings ratio

The encoding typically achieves 30–70% token savings on tabular data. The heuristic check avoids encoding non-tabular or small payloads.

## Key source files

| Concern           | Path                                                                                    |
| ----------------- | --------------------------------------------------------------------------------------- |
| TUI (current)     | `/elph/src/tui/` (directory: shell, editor, transcript, chrome, agent_bridge, activity) |
| Agent interaction | `/elph/src/agent/`                                                                      |
| Diagnostics tool  | `/elph/src/agent/diagnostics.rs`                                                        |
| ui-components     | `/crates/elph-tui/src/components/`                                                      |
| TUI examples      | `/crates/elph-tui/examples/`                                                            |
| Prompt encoding   | `/crates/elph-agent/src/prompt/encoding/`                                               |
| Slash commands    | `/elph/src/agent/slash_commands.rs`                                                     |
| Agent runtime     | `/elph/src/agent/runtime.rs`                                                            |

## Change guidance

- **New slash command**: Implement in `/elph/src/agent/slash_commands.rs`
- **TUI development**: Modify files under `/elph/src/tui/` (iocraft-based shell, editor, transcript, chrome)
- **New TUI example**: Add to `/crates/elph-tui/examples/` and register in the crate's `Cargo.toml`
- **New component stub**: Add module to `/crates/elph-tui/src/components/` and register in `mod.rs`
- **Prompt encoding**: Test in `/crates/elph-agent/tests/prompt_encoding.rs`
- **TUI tests**: `/crates/elph-tui/tests/` — 14 test files including `transcript_layout`, `textarea`, `text_editing`, `text_input_layout`, `scroll`, `color`, `components_helpers`, `components_mock`, `coverage_gaps`, `coverage_helpers`
