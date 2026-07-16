# TUI & Shell

The TUI system currently lives directly in the `elph` binary crate while the iocraft-based shell is being rebuilt iteratively.

The `elph-tui` library crate provides iocraft component modules (17+ modules) and 21 examples, with integration tests. Once the public API stabilises, the reusable widget library will be extracted back into `elph-tui` and published to crates.io.

**TUI source**: `/elph/src/tui/` — Modular iocraft-based shell: `shell.rs`, `focus.rs`, `tool_approval.rs`, `user_question.rs`, `activity.rs`, `agent_bridge.rs`, `labels.rs`, `theme.rs`, and the subdirectories `chrome/`, `prompt/`, `transcript/`, `slash_palette/`.
**Shell**: `elph/src/tui/shell.rs` is the main interactive shell orchestrating all TUI components. Focus switching (`focus.rs`), tool approval modal (`tool_approval.rs`), user question prompts (`user_question.rs`), activity tracking (`activity.rs`), and the slash palette (`slash_palette/`) are all standalone modules.

## elph-tui (Widget Library — Component Stubs)

**Path**: `/crates/elph-tui/`

The crate provides `lib.rs` with 17+ component modules under `components/` (most are implemented, not stubs):

- `ascii_font`, `card`, `code`, `diff`, `frame_buffer`, `input`, `line_numbers`
- `markdown`, `progress_indicator`, `qr_code`, `scroll_bar`, `scroll_box`, `select`, `slider`, `tab_select`, `text`, `textarea`

`textarea` is a directory containing `component.rs`, `input/` (paste, submit, wire_edit sub-modules), `layout.rs`, and `state.rs`. `markdown` is now a full directory with 10 sub-modules: `blocks.rs`, `colors.rs`, `highlight.rs`, `layout.rs`, `linkify.rs`, `model.rs`, `parse.rs`, `parser_config.rs`, `render.rs`, `syntax.rs`, `theme.rs` — providing syntax-highlighted code blocks (via `syntect`) and auto-linked URLs. Additional modules live at the crate root: `text_editing/` (actions, input, line, submit, wire), `transcript_layout.rs`, `text_input_layout.rs`, `loader.rs`, `paste.rs`, and `utils.rs`.

In the `elph` binary TUI (`/elph/src/tui/transcript/message.rs`), the transcript now renders structured tool invocation cards via `ToolCardDetail` (name, args summary, output body), replacing inline text formatting. Theme constants `TOOL_ARGS_FG` and `TOOL_OUTPUT_FG` control card colors. The previous `format_tool_card_content` / `format_tool_card_result` public helpers have been removed in favor of the structured `tool_call()` constructor on `TranscriptMessage`.

### Examples (21 total in `crates/elph-tui/examples/`)

| Example           | Description                                            |
| ----------------- | ------------------------------------------------------ |
| `weather`         | Async data loading from remote APIs with iocraft       |
| `calculator`      | Calculator app with iocraft UI                         |
| `chat_layout`     | Chat layout with scrollable content, input, tool cards |
| `progress_bar`    | Animated progress bar demo                             |
| `basic_context`   | Context API usage example                              |
| `basic_counter`   | Simple counter with state management                   |
| `basic_form`      | Form with input validation                             |
| `basic_input`     | Text input handling demo                               |
| `basic_layout`    | Layout composition demo                                |
| `basic_output`    | Text output display                                    |
| `basic_overlap`   | Overlapping elements demo                              |
| `basic_scrolling` | Scrollable content                                     |
| `basic_table`     | Table layout demo                                      |
| `demo_code`       | Code block rendering                                   |
| `demo_diff`       | Diff output rendering                                  |
| `demo_input`      | Input widget demo                                      |
| `demo_markdown`   | Markdown rendering                                     |
| `demo_scroll`     | Scroll container demo                                  |
| `demo_select`     | Select widget demo                                     |
| `demo_special`    | Special elements demo                                  |
| `demo_text_card`  | Text card rendering                                    |

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

## Rich Markdown Rendering

**Source**: `/crates/elph-tui/src/components/markdown/`

The markdown component was upgraded from a simple stub to a full rendering pipeline with syntax highlighting and auto-linked URLs.

Key capabilities:

- **Parsing** — Uses `pulldown-cmark` to parse markdown into a `MarkdownDocument` model with structured blocks
- **Syntax highlighting** — Code fences with language tags are highlighted via `syntect` with the bundled Tokyo Night theme (`crates/elph-tui/assets/tokyo-night.tmTheme`)
- **Auto-links** — Plain URLs in text are detected and rendered in the theme's link color
- **Streaming** — `streaming_tail_document` supports incremental rendering as the model streams tokens
- **Theming** — `MarkdownTheme` controls colors for headers, code, links, inline code, and emphasis; `markdown_document_with_theme(source, theme)` allows per-document theme overrides

### Sub-modules

| Module             | Purpose                                       |
| ------------------ | --------------------------------------------- |
| `blocks.rs`        | Block-level rendering (paragraph, code, list) |
| `colors.rs`        | Color scheme definitions                      |
| `highlight.rs`     | syntect syntax highlighting integration       |
| `layout.rs`        | Layout calculations, row counts               |
| `linkify.rs`       | Plain-text URL detection                      |
| `model.rs`         | `MarkdownDocument`, block/section types       |
| `parse.rs`         | Markdown → `MarkdownDocument` conversion      |
| `parser_config.rs` | Parser configuration and options              |
| `render.rs`        | iocraft rendering of blocks and documents     |
| `syntax.rs`        | Language detection and syntax set loading     |
| `theme.rs`         | Default and custom themes                     |

### Transcript integration

In the binary TUI (`/elph/src/tui/transcript/markdown/`), a dedicated module handles rendering assistant responses with a `RenderWorker` that processes markdown documents incrementally. Components: `buffer.rs` (output accumulation), `layout.rs` (sizing), `render.rs` (iocraft rendering), `worker.rs` (incremental processing).

## Slash Palette

**Source**: `/elph/src/tui/slash_palette/`

An autocomplete overlay that appears above the prompt editor when the user types `/`. It provides fuzzy-filtered command completion for slash commands, prompt templates, and skills.

| Module          | Purpose                                                              |
| --------------- | -------------------------------------------------------------------- |
| `mod.rs`        | Re-exports: `SlashCommandPalette`, `build_snapshot`                  |
| `component.rs`  | Floating palette shell component                                     |
| `fuzzy.rs`      | Fuzzy matching logic for filtering commands                          |
| `keyboard.rs`   | Key-to-action mapping (`CompleteDraft`, `MoveSelection`, `Dismiss`)  |
| `layout.rs`     | Anchor calculations                                                  |
| `model.rs`      | `SlashPaletteSnapshot` building from draft text and command registry |
| `state.rs`      | Selection sync when filter query changes                             |
| `row_layout.rs` | Row rendering with icon and description                              |
| `card/`         | Bordered palette panel (chrome, header, body, frame)                 |

The slash palette integrates with `skills_load.rs` (`/elph/src/agent/skills_load.rs`) to discover and display `/skill:<name>` invocations. Skill names are registered as `skill:<name>` entries in the palette with truncated descriptions (72 char max).

## Focus Switching

**Source**: `/elph/src/tui/focus.rs`

The TUI supports keyboard-driven focus switching between the prompt editor and transcript.

- `ShellFocus` enum: `Prompt` or `Transcript` (default: `Prompt`)
- **Esc** — Moves focus from Transcript back to Prompt
- **Transcript navigation** — When transcript has focus, arrow keys, PageUp/Down, Home/End scroll the transcript content. Shift+arrows scroll by larger increments.
- **Auto-refocus** — Pressing a letter, space, or `/` while transcript has focus automatically refocuses the prompt and seeds the character (`prompt_focus_char`).

## Tool Approval

**Source**: `/elph/src/tui/tool_approval.rs`

A modal-like component that replaces the prompt editor when the agent requests approval for a tool call.

- `PendingToolApproval` stores tool name, args summary, and a `oneshot::Sender<ToolApprovalChoice>`
- **Key bindings**: `y`/`1`/`Enter` → Approve, `a`/`2` → AllowSession, `n`/`3`/`Esc` → Reject
- `ToolApprovalPrompt` renders a bordered panel showing tool name, truncated args preview (up to 4 lines with word-wrap), and key hints footer
- Activity labels update based on choice: "Running approved tool…", "Running tool (session allow)…", or "Tool denied"

## Stream Token Tracking

**Source**: `/elph/src/tui/activity.rs`, `/elph/src/tui/chrome/status_row.rs`

The status row displays real-time token consumption during model streaming.

- `TurnTokenTracker` (in `activity.rs`): Tracks `baseline_tokens` (at turn start) and `stream_tokens` (incremental delta, estimated at 1 token ≈ 4 characters)
- **Status row right segment**: Shows `+<delta> · <t/s>` throughput (e.g. `+240 · 12 t/s`)
- **Idle state**: Shows a rotating tip (10 tips) or an `idle_notice` like "Turn complete · 1.2s" for 5 seconds after a turn, then reverts to tips
- **Busy state**: Left side shows braille spinner + activity label + elapsed time (`⠋ Thinking · 1.2s`); right side shows token info + cancel hint (`+240 · 12 t/s | Ctrl+C cancel`)
- Turn count is displayed in chrome stats (`/elph/src/tui/chrome/stats.rs`), refreshed on each user input

## Quit Confirmation During Active Turn

**Source**: `/elph/src/tui/shell.rs`, `/elph/src/tui/activity.rs`

When the user issues a quit command (`/exit`, `:q`, `Ctrl+D`) while a turn is in progress, the shell does not exit immediately — it confirms first.

- **First quit attempt while busy**: Posts "Agent is still responding. Press y to quit (cancels the turn), n to keep waiting, or repeat /exit, :q, or Ctrl+D to confirm." to the transcript. Arms pending quit state.
- **Confirmation**: `y`/`Y` → confirm quit (cancels turn, rejects pending approvals/questions, aborts session); `n`/`N`/`Esc` → dismiss
- **Force quit**: `/exit!` or repeat quit command while pending confirm → immediate exit
- **Ctrl+C while busy**: Cancels the current turn (aborts, clears queue, rejects approvals)
- **Ctrl+C while idle**: Clears the prompt draft
- Status row right segment appends `" | y quit · n stay"` when quit confirmation is pending

## Key source files

| Concern                            | Path                                                                                                                                 |
| ---------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| TUI (current)                      | `/elph/src/tui/` (shell.rs, focus.rs, tool_approval.rs, activity.rs, agent_bridge.rs, chrome/, prompt/, transcript/, slash_palette/) |
| Shell implementation               | `/elph/src/tui/shell.rs`                                                                                                             |
| Focus switching                    | `/elph/src/tui/focus.rs`                                                                                                             |
| Tool approval modal                | `/elph/src/tui/tool_approval.rs`                                                                                                     |
| User question prompts              | `/elph/src/tui/user_question.rs`                                                                                                     |
| Activity + token tracking          | `/elph/src/tui/activity.rs`                                                                                                          |
| Slash palette                      | `/elph/src/tui/slash_palette/`                                                                                                       |
| Isomorphic text editor             | `/elph/src/tui/prompt/editor.rs`                                                                                                     |
| Transcript cards + markdown        | `/elph/src/tui/transcript/`                                                                                                          |
| Chrome (header, stats, status_row) | `/elph/src/tui/chrome/`                                                                                                              |
| Agent interaction                  | `/elph/src/agent/`                                                                                                                   |
| Diagnostics tool                   | `/elph/src/agent/diagnostics.rs`                                                                                                     |
| Skills load + slash parse          | `/elph/src/agent/skills_load.rs`                                                                                                     |
| Tools catalog reconciliation       | `/elph/src/agent/tools_catalog.rs`                                                                                                   |
| ui-components                      | `/crates/elph-tui/src/components/`                                                                                                   |
| TUI examples                       | `/crates/elph-tui/examples/`                                                                                                         |
| Prompt encoding                    | `/crates/elph-agent/src/prompt/encoding/`                                                                                            |
| Slash commands                     | `/elph/src/agent/slash_commands.rs`                                                                                                  |
| Agent runtime                      | `/elph/src/agent/runtime.rs`                                                                                                         |

## Change guidance

- **New slash command**: Implement in `/elph/src/agent/slash_commands.rs`
- **Slash palette update**: Modify `/elph/src/tui/slash_palette/model.rs` (snapshot building) and `/elph/src/tui/slash_handler.rs`
- **Skills slash parsing**: `/elph/src/agent/skills_load.rs` (`parse_skill_slash`, `skill_slash_name`)
- **TUI development**: Modify files under `/elph/src/tui/` (shell, focus, tool_approval, user_question, activity, agent_bridge, chrome, prompt, transcript, slash_palette)
- **Focus behavior**: `/elph/src/tui/focus.rs` — `ShellFocus` enum, `prompt_focus_char`, `transcript_nav_key`
- **Tool approval**: `/elph/src/tui/tool_approval.rs` — key bindings, `PendingToolApproval`
- **Token tracking**: `/elph/src/tui/activity.rs` — `TurnTokenTracker`, `format_busy_token_info`
- **Chrome / status row**: `/elph/src/tui/chrome/status_row.rs` — `busy_token_info`, `idle_notice` props
- **New TUI example**: Add to `/crates/elph-tui/examples/` and register in the crate's `Cargo.toml`
- **New component stub**: Add module to `/crates/elph-tui/src/components/` and register in `mod.rs`
- **Prompt encoding**: Test in `/crates/elph-agent/tests/prompt_encoding.rs`
- **TUI tests**: `/crates/elph-tui/tests/` — 14 test files including `transcript_layout`, `textarea`, `text_editing`, `text_input_layout`, `scroll`, `color`, `components_helpers`, `components_mock`, `coverage_gaps`, `coverage_helpers`
