# Consideration

Dependency candidates for Elph, mapped to crates and planned workspace deps (`fff-search`, `rmcp`, `jsonschema`, `agent-client-protocol`).

**Verdicts:** **Adopt** · **Keep** · **Done** · **Defer** · **Ref** (study only) · **Skip**

**Near-term stack**

| Layer                | Crates                                                                  |
| -------------------- | ----------------------------------------------------------------------- |
| `elph-ai`            | `genai`, `schemars` (keep)                                              |
| `elph-agent`         | `fff-search` (done), `rmcp`, `jsonschema`                               |
| `elph-tui`           | `superlighttui`, `syntect`, `anstyle-syntect` (+ keep `pulldown-cmark`) |
| `elph-core` / `elph` | `figment`, `jsonc-parser`                                               |
| Shared               | `tracing`, `tokio`, `chrono`, `memchr`                                  |

---

## LLM & provider (`elph-ai`)

| Verdict   | Item                                                                                                                                                                         | Rationale                                                                         |
| --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| **Adopt** | [genai](https://crates.io/crates/genai)                                                                                                                                      | 25+ providers, streaming, tools, auth. Pin `0.6.5`; try `0.7.0-beta.9` on branch. |
| **Keep**  | [schemars](https://crates.io/crates/schemars)                                                                                                                                | Already in `elph-ai`; tool JSON Schema from Rust types. Expand as tools grow.     |
| **Defer** | [adk-anthropic](https://crates.io/crates/adk-anthropic)                                                                                                                      | Anthropic-only fallback if genai lacks thinking/cache/batch.                      |
| **Defer** | [anthropic-auth](https://crates.io/crates/anthropic-auth)                                                                                                                    | Claude OAuth/PKCE for `oauth_selector`.                                           |
| **Defer** | [anthropic-async](https://crates.io/crates/anthropic-async)                                                                                                                  | Anthropic + prompt cache; niche vs genai.                                         |
| **Defer** | [async-openai](https://github.com/64bit/async-openai)                                                                                                                        | Deep OpenAI (⭐1956); only if genai OpenAI path falls short.                      |
| **Defer** | [llm-bridge-core](https://crates.io/crates/llm-bridge-core)                                                                                                                  | Protocol transform; gateway/proxy only.                                           |
| **Defer** | [iac-rs](https://crates.io/crates/iac-rs)                                                                                                                                    | Inter-agent protocol; `elph-swarm`, not MVP.                                      |
| **Defer** | [adk-rust](https://github.com/zavora-ai/adk-rust)                                                                                                                            | Full framework (39 crates); alternative to custom `elph-agent`.                   |
| **Skip**  | [anthropic-ai-sdk](https://crates.io/crates/anthropic-ai-sdk), [anthropic-rs](https://github.com/AbdelStark/anthropic-rs), [claude-sdk](https://crates.io/crates/claude-sdk) | Anthropic-only; superseded.                                                       |
| **Skip**  | [genai-rs](https://crates.io/crates/genai-rs)                                                                                                                                | Gemini-only; name collision with genai.                                           |
| **Skip**  | [openai-api-rs](https://crates.io/crates/openai-api-rs), [openai_dive](https://github.com/tjardoo/openai-client/tree/master/openai_dive)                                     | OpenAI-only.                                                                      |
| **Skip**  | [agent-sdk-rs](https://crates.io/crates/agent-sdk-rs), [ai.rs](https://github.com/prabirshrestha/ai.rs)                                                                      | Immature.                                                                         |

---

## Agent runtime (`elph-agent`)

| Verdict  | Item                                                                                                                                                          | Rationale                                                                 |
| -------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| **Done** | [fff-search](https://crates.io/crates/fff-search)                                                                                                             | Wired in `elph-agent` (`grep`/`find` tools). Pin `0.9.x` in workspace.    |
| **Ref**  | [yoagent](https://github.com/yologdev/yoagent)                                                                                                                | Pi-style loop blueprint (⭐171). Don't adopt as dep — overlaps `elph-ai`. |
| **Ref**  | [open-agent-sdk-rust](https://github.com/codeany-ai/open-agent-sdk-rust)                                                                                      | Tools/hooks/session patterns (⭐23).                                      |
| **Ref**  | [jcode](https://github.com/1jehuang/jcode), [rusty-gitclaw](https://github.com/open-gitagent/rusty-gitclaw), [codeany](https://github.com/codeany-ai/codeany) | Competitors / UX reference, not libs.                                     |
| **Skip** | [zag](https://github.com/niclaslindstedt/zag)                                                                                                                 | Immature multi-provider CLI (⭐6).                                        |

---

## TUI, markdown & prompts

### Agent shell (`elph-tui` + SuperLightTUI)

Two rendering layers, composed at the app level (`elph`, `owly`):

| Layer           | Crate / module                                                     | Role                                                                                             |
| --------------- | ------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| Agent chrome    | [superlighttui](https://github.com/subinium/SuperLightTUI) (`slt`) | Immediate-mode shell: prompt (`TextareaState`), chat stream, banners, setup wizard, activity bar |
| Rich components | `elph-tui::diff` + `bridge`                                        | Differential ANSI engine: `Editor`, `SelectList`, overlays; `bridge` embeds diff output into SLT |

| Verdict   | Item                                                                                               | Rationale                                                                    |
| --------- | -------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------- |
| **Keep**  | [superlighttui](https://github.com/subinium/SuperLightTUI)                                         | Agent shell for `elph` and `owly`; `async` feature for Owly dispatch bridge. |
| **Keep**  | [pulldown-cmark](https://github.com/pulldown-cmark/pulldown-cmark)                                 | `render_markdown_lines` in production.                                       |
| **Adopt** | [syntect](https://crates.io/crates/syntect)                                                        | Code-block highlighting (Pi parity).                                         |
| **Adopt** | [anstyle-syntect](https://crates.io/crates/anstyle-syntect)                                        | Adapter `syntect` → `anstyle` tokens; adopt with `syntect`.                  |
| **Defer** | [anstyle-git](https://crates.io/crates/anstyle-git)                                                | Git-diff colors for `diff/content.rs`; custom `similar` layout today.        |
| **Defer** | [termimad](https://github.com/Canop/termimad)                                                      | Full markdown TUI (⭐1198); redundant with pulldown + syntect.               |
| **Skip**  | [comrak](https://github.com/kivikakk/comrak), [markdown-rs](https://github.com/wooorm/markdown-rs) | Parser-only; still need custom layout.                                       |
| **Skip**  | [iocraft](https://github.com/cfeeley/iocraft)                                                      | Replaced by SuperLightTUI; do not reintroduce.                               |

### Terminal styling (anstyle ecosystem)

`elph-tui` agent chrome renders via **SuperLightTUI**; `diff/` uses hand-built ANSI in `diff/ansi.rs`. `clap` (feature `color`) already pulls `anstream`, `anstyle`, and `anstyle-parse` transitively — do not add them as direct deps unless writing colored output outside clap.

| Verdict   | Item                                                                                                                                                | Rationale                                                                                      |
| --------- | --------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| **Keep**  | [anstream](https://crates.io/crates/anstream), [anstyle](https://crates.io/crates/anstyle), [anstyle-parse](https://crates.io/crates/anstyle-parse) | Transitive via `clap`; covers CLI help/errors.                                                 |
| **Defer** | [anstyle-crossterm](https://crates.io/crates/anstyle-crossterm)                                                                                     | Bridge to `crossterm::style`; only if migrating `ansi.rs` — TUI styles strings, not crossterm. |
| **Defer** | Full `ansi.rs` → `anstyle` migration                                                                                                                | Large refactor; ROI only when adding `syntect` / git-diff colors.                              |
| **Skip**  | [human-panic](https://crates.io/crates/human-panic)                                                                                                 | Incompatible with release `panic = "abort"`; use `anyhow` + `tracing` instead.                 |
| **Skip**  | [proc-exit](https://crates.io/crates/proc-exit)                                                                                                     | `cmd::run() → exit(code)` in `main` is sufficient.                                             |
| **Defer** | [termtree](https://crates.io/crates/termtree)                                                                                                       | ASCII tree printer; niche unless adding `elph session tree` CLI viz.                           |

### CLI prompts (`elph` / `eclaw` / `owly` non-interactive paths)

Blocking prompts for wizards and pickers — **not** inside the active SLT loop (raw-mode conflict).

| Verdict   | Item                                              | Rationale                                                                                                     |
| --------- | ------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| **Keep**  | [dialoguer](https://crates.io/crates/dialoguer)   | `owly` onboarding and `ask_user` tool paths today.                                                            |
| **Defer** | [inquire](https://github.com/mikaelmello/inquire) | Alternative for future CLI wizards (`session` picker, `doctor`, bootstrap). Pin `0.9.x`, `crossterm` backend. |
| **Skip**  | inquire in `elph-tui`                             | SLT `TextareaState` + `diff::SelectList` overlays already cover interactive agent UX.                         |

**Overlap:** `dialoguer`/`inquire` for one-shot CLI flows; SLT + `diff` overlays inside `run_with` / `run` — never nest blocking CLI prompts in an active TUI session.

---

## Configuration (`elph-core` / `elph`)

Schema: `schemas/elph-config-schema.json`. Types: `elph/src/runtime/settings.rs`.

**Merge order (low → high):** defaults → `~/.elph/settings.*` → `<workDir>/.elph/settings.*` → `ELPH_*` env → CLI (later).

| Format            | figment native | Notes                               |
| ----------------- | -------------- | ----------------------------------- |
| JSON              | ✅             | Canonical on save (`settings.json`) |
| JSONC             | ❌             | Via `jsonc-parser` pre-pass         |
| TOML / YAML / ENV | ✅             | Optional user formats               |

| Verdict   | Item                                                                                                                 | Rationale                                                               |
| --------- | -------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------- |
| **Adopt** | [figment](https://crates.io/crates/figment)                                                                          | Layered merge + serde extract. Features: `json`, `toml`, `yaml`, `env`. |
| **Adopt** | [jsonc-parser](https://crates.io/crates/jsonc-parser)                                                                | Comments + trailing commas → `serde_json`.                              |
| **Defer** | [dotenvy](https://crates.io/crates/dotenvy)                                                                          | `.env` for local dev secrets.                                           |
| **Defer** | [confique](https://crates.io/crates/confique)                                                                        | Derive DX if `Settings` boilerplate grows.                              |
| **Defer** | [json5](https://crates.io/crates/json5), [json_comments](https://crates.io/crates/json_comments)                     | Alternative JSONC paths — pick one dialect.                             |
| **Defer** | [shellexpand](https://crates.io/crates/shellexpand)                                                                  | `${VAR}` / `~` in config strings.                                       |
| **Defer** | [config-rs](https://github.com/rust-cli/config-rs)                                                                   | Overlaps figment; weaker error provenance.                              |
| **Skip**  | [twelf](https://crates.io/crates/twelf), [envy](https://crates.io/crates/envy), [ron](https://github.com/ron-rs/ron) | Too narrow or extra format.                                             |

**Wire:** `Figment` merge chain → extract `Settings` → validate with `jsonschema` → write strict JSON to home on save.

---

## Infra

| Verdict   | Item                                                                                             | Rationale                                                                                 |
| --------- | ------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------- |
| **Keep**  | [tracing](https://github.com/tokio-rs/tracing)                                                   | Active in `elph-core`.                                                                    |
| **Keep**  | [chrono](https://crates.io/crates/chrono)                                                        | RFC 3339 timestamps in `elph-core`, `owly`, `elph-ai`; replaced custom `utils/time.rs`.   |
| **Keep**  | [memchr](https://crates.io/crates/memchr)                                                        | Zero-copy line splitting (`elph-core/utils/lines`, `elph-agent` truncate, `elph-ai` SSE). |
| **Keep**  | [rayon](https://crates.io/crates/rayon)                                                          | Parallel fuzzy filter in `elph-tui` (lists ≥ 64 items).                                   |
| **Defer** | [rapidhash](https://crates.io/crates/rapidhash)                                                  | Cache keys / content fingerprints (`make build` only today).                              |
| **Skip**  | [rustix](https://crates.io/crates/rustix), [async-stream](https://crates.io/crates/async-stream) | Already transitive; add only if needed in-tree.                                           |

---

## Memory & integrations

| Verdict   | Item                                                            | Rationale                                                                             |
| --------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| **Done**  | [memelord](https://github.com/glommer/memelord) → `floppy`      | Ported in-tree as `floppy`; Turso agent memory with vector search and weight scoring. |
| **Defer** | [cortex-mem](https://github.com/sopaco/cortex-mem)              | Long-term memory layer (⭐288).                                                       |
| **Defer** | [liteparse](https://github.com/run-llama/liteparse)             | RAG ingestion (⭐11k).                                                                |
| **Defer** | [obscura](https://docs.obscura.sh/guides/use-as-a-rust-library) | Embedded browser; high build cost.                                                    |
| **Defer** | [pest](https://github.com/pest-parser/pest)                     | Custom DSL only.                                                                      |
| **Skip**  | [teloxide](https://github.com/teloxide/teloxide)                | Telegram; out of scope.                                                               |

---

## Dev & CI (not runtime deps)

| Verdict | Item                                             | Rationale                              |
| ------- | ------------------------------------------------ | -------------------------------------- |
| **Ref** | [crit](https://github.com/tomasz-tomczyk/crit)   | Human→agent review (⭐681, Go binary). |
| **Ref** | [ast-grep](https://github.com/ast-grep/ast-grep) | Structural lint/search CLI (⭐15k).    |
| **Ref** | [ffizer](https://crates.io/crates/ffizer)        | Scaffolding templates.                 |

---

## Decisions to avoid

| Rule                | Detail                                                                                         |
| ------------------- | ---------------------------------------------------------------------------------------------- |
| One provider layer  | Not `genai` + `adk-anthropic` + `async-openai`.                                                |
| One agent framework | Custom loop (yoagent ref) _or_ `adk-rust` — not both + SDK refs.                               |
| One markdown stack  | `pulldown-cmark` + `syntect` + `anstyle-syntect`; no second parser.                            |
| One styling path    | Manual `ansi.rs` _or_ `anstyle` migration — not both without a planned cutover.                |
| One config stack    | `figment` + one JSONC helper; not `config-rs` + `confique`.                                    |
| One comment dialect | JSONC (VS Code) _or_ JSON5.                                                                    |
| Two prompt layers   | SLT + `diff` for agent shell; `dialoguer`/`inquire` for CLI only — never nested in active TUI. |
| No panic UX layer   | Release uses `panic = "abort"`; skip `human-panic` and unwind-based hooks.                     |
| One time library    | `chrono` for RFC 3339; no in-tree date math.                                                   |
