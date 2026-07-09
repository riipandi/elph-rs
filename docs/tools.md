# Built-in Tools

Design for the agent tool catalog — permissions, provider exposure, and execution behavior.

## Permission classes

| Permission          | Behavior                                                |
| ------------------- | ------------------------------------------------------- |
| `auto-allow`        | Runs without approval; user may require approval        |
| `requires-approval` | Approval dialog each run (except brave / session allow) |
| `always-approve`    | Always runs; cannot be restricted                       |

## File tools

| Tool          | Default approval  | Description                                                       |
| ------------- | ----------------- | ----------------------------------------------------------------- |
| Read          | Auto-allow        | Read text/image; `line_offset`, `n_lines`; negative offset = tail |
| Write         | Requires approval | Create/overwrite/append; fails on directories                     |
| Edit          | Requires approval | Exact string replace; `replace_all`; no-op guard                  |
| Grep          | Auto-allow        | Ripgrep search; content / files / count modes; context lines      |
| Glob / Find   | Auto-allow        | Glob file search; recursive directory listing                     |
| ReadMediaFile | Auto-allow        | Image/video; metadata + base64 for vision                         |

## Shell tools

| Tool | Default approval  | Description                                                             |
| ---- | ----------------- | ----------------------------------------------------------------------- |
| Bash | Requires approval | `bash -c` in workspace; default timeout 120s, max 300s; streamed output |

## Web tools

| Tool       | Default approval | Description                                   |
| ---------- | ---------------- | --------------------------------------------- |
| FetchURL   | Auto-allow       | HTTP fetch; HTML → text; SSRF protection      |
| WebSearch  | Auto-allow       | Multi-engine search with ranking and fallback |
| CodeSearch | Auto-allow       | GitHub/GitLab code search                     |

## Plan mode

| Tool          | Description                              |
| ------------- | ---------------------------------------- |
| EnterPlanMode | Enter planning mode                      |
| ExitPlanMode  | Exit and submit plan (user confirmation) |

## State management

| Tool     | Description                                                    |
| -------- | -------------------------------------------------------------- |
| TodoList | Session task list; Tasks panel in TUI; per-session persistence |

Item statuses: `pending` / `in_progress` / `done`. All `done` → hide panel + system notice.

## Goal tools

| Tool          | Description                               |
| ------------- | ----------------------------------------- |
| CreateGoal    | Objective + optional completion criterion |
| GetGoal       | Status, turns, tokens, budgets            |
| UpdateGoal    | Lifecycle transitions                     |
| SetGoalBudget | Token, turn, or time budget               |

## Collaboration

| Tool    | Description                                          |
| ------- | ---------------------------------------------------- |
| AskUser | Structured question to the user                      |
| Skill   | Invoke registered inline skill (max nesting depth 3) |

## Provider API exposure

Only a catalog subset is sent to the model. Exposure requires:

1. Known built-in tool
2. Approval class allowed for API (`auto-allow` or `requires-approval`)
3. Runtime can execute
4. Provider JSON schema exists

### Exposure matrix (design)

| Tool                                | Approval | API | Runtime          |
| ----------------------------------- | -------- | --- | ---------------- |
| Read, Grep, Glob, ReadMediaFile     | Auto     | Yes | Yes              |
| WebSearch, AskUser, TodoList, Skill | Auto     | Yes | Yes              |
| Write, Edit, Bash                   | Requires | Yes | Yes (+ approval) |
| Goal tools                          | Auto     | Yes | Yes              |
| FetchURL, CodeSearch, Plan mode     | Auto     | TBD | TBD              |

## User approval

**Write**, **Edit**, and **Bash** block the loop until the user chooses:

| Choice            | Shortcut | Effect                      |
| ----------------- | -------- | --------------------------- |
| Allow once        | `y`, `1` | This call only              |
| Allow for session | `a`, `2` | Skip approval until exit    |
| Deny              | `n`, `3` | Error returned to the model |

- **Enter** confirms default (allow once)
- **Esc** = deny
- Identical signature in the same turn → auto-deny without re-prompting
- **Brave** mode skips risky-tool approval

## WebSearch engines

Engines: DuckDuckGo (fallback), Jina, Brave, SerpAPI, Tavily, Firecrawl, Perplexity, Exa.

Omit `engine` → auto-select best configured backend; DuckDuckGo last.

Env keys: `JINA_API_KEY`, `BRAVE_SEARCH_API_KEY`, `SERPAPI_KEY`, `TAVILY_API_KEY`, `FIRECRAWL_API_KEY`, `PERPLEXITY_API_KEY`, `EXA_API_KEY`.

## Request flow

```mermaid
sequenceDiagram
    participant Session
    participant Loop as Agent loop
    participant Provider
    participant Runtime

    Session->>Loop: StartTurn
    Loop->>Provider: Complete + tool schemas
    Provider-->>Loop: tool_calls
    Loop->>Loop: Interact (AskUser / approval)
    Loop->>Runtime: Execute
    Runtime-->>Loop: output / error
    Loop->>Provider: tool_result follow-up
```

## Text markup fallback

If the model emits XML-style tags in streamed text instead of native calls:

- Parser strips markup from the visible bubble
- System prompt discourages invented tags
- Native tool calling remains the primary path

## Related

- [agent-runtime.md](./agent-runtime.md) — execution flow
- [slash-commands.md](./slash-commands.md) — diagnostics
- [tui.md](./tui.md) — tool display
