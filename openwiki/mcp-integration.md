# MCP Integration

Elph embeds an MCP **client** (via [rmcp](https://crates.io/crates/rmcp)) so the agent can call tools exposed by external MCP servers. The MCP module is feature-gated behind `mcp` (enabled by default).

**Source**: `/crates/elph-agent/src/tools/mcp/`

## Architecture

```
Agent Tool Registry
       ↓
McpToolRegistry
       ↓
McpClient ──→ Transport (stdio / HTTP / SSE)
       ↓
McpSessionPool (connection reuse)
       ↓
McpServerSession (per-server state)
```

## Configuration

**File**: `/crates/elph-agent/src/tools/mcp/config.rs`

MCP servers are configured via JSON (the Elph product uses `~/.elph/mcp.json`):

```json
{
    "servers": {
        "filesystem": {
            "type": "stdio",
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
            "timeoutMs": 60000
        },
        "remote": {
            "type": "http",
            "url": "https://mcp.example.com/mcp",
            "authTokenEnv": "MCP_REMOTE_TOKEN",
            "headers": { "X-App": "elph" },
            "timeoutMs": 45000
        }
    }
}
```

| Config field                               | Transports | Description                         |
| ------------------------------------------ | ---------- | ----------------------------------- |
| `type`                                     | both       | `stdio` or `http`                   |
| `command`/`args`/`env`/`cwd`               | stdio      | Child process configuration         |
| `url`/`headers`/`authToken`/`authTokenEnv` | http       | Streamable HTTP endpoint            |
| `timeoutMs`                                | both       | Per list/call timeout (default 60s) |
| `disabled`                                 | both       | Skip during discovery and calls     |

Key config types:

- `McpConfig` — Top-level config (servers map)
- `McpServerConfig` — Single server configuration (enum over `Stdio` / `Http`)
- `McpStdioConfig` — Stdio transport configuration
- `McpHttpConfig` — HTTP transport configuration
- `McpLoadOptions` — Options for loading servers (include/exclude patterns)

## Transports

**File**: `/crates/elph-agent/src/tools/mcp/client.rs`

Three transport types are supported:

| Transport | File        | Description                    |
| --------- | ----------- | ------------------------------ |
| `stdio`   | `client.rs` | Child process with stdio       |
| `http`    | `client.rs` | Streamable HTTP                |
| `sse`     | `sse.rs`    | HTTP+SSE (2024-11-05 protocol) |

Connection functions:

- `connect()` — Auto-detect transport from config
- `connect_stdio()` — Connect to a stdio-based server
- `connect_http()` — Connect to an HTTP-based server
- `connect_with_context()` — Connect with OAuth context

## Tool Registry

**File**: `/crates/elph-agent/src/tools/mcp/registry.rs`

The `McpToolRegistry` manages:

- **Tool discovery** — List tools from connected servers
- **Tool naming** — `mcp_{server}__{tool}` for the model tool surface
- **Tool execution** — Route tool calls to the right server
- **Hot reload** — `tools/list_changed`, `resources/list_changed`, `prompts/list_changed`
- **Result formatting** — `mcp_result_to_agent()` / `mcp_result_to_agent_with_limit()`

Key types:

- `McpToolDescriptor` — Tool metadata from a server
- `McpResourceDescriptor` — Resource metadata
- `McpPromptDescriptor` — Prompt metadata
- `McpToolRegistry` — Aggregated registry across servers
- `McpLoadReport` / `McpServerLoadReport` — Load results

## Sessions & Connection Pool

**File**: `/crates/elph-agent/src/tools/mcp/session.rs`

| Type                                                                                           | Purpose |
| ---------------------------------------------------------------------------------------------- | ------- |
| `McpSessionPool` — Reuse MCP connections across tool calls                                     |         |
| `McpServerSession` — Per-server session state including capabilities and notification handlers |         |

## Deferred MCP Loading

**Source**: `/elph/src/agent/mcp_bootstrap.rs`, `/elph/src/tui/startup.rs`

MCP server discovery is now deferred: the agent session starts immediately, and `McpToolRegistry::load_with_options()` runs in the background. The TUI shows per-server progress/error rows in the transcript as each server is discovered.

- `discover_mcp_registry_with_progress()` — Loads MCP config best-effort, emits `McpServerLoadProgress` events
- `wire_mcp_into_session()` — Binds the loaded registry to a running `CodingAgentSession`
- Startup transcript rows use stable keys (`startup:mcp:{name}`) for upsert semantics
- Config warnings are surfaced inline alongside server status

## Config Compat Layer

**File**: `/crates/elph-agent/src/tools/mcp/compat.rs`

Normalizes editor-style MCP JSON configurations (Cursor, VS Code, Claude Code) into Elph's canonical shape before schema validation:

- Renames `mcpServers` → `servers` when the `servers` key is absent
- Infers `type: "http"` when a server has `url` but no `type`
- Infers `type: "stdio"` when a server has `command` but no `type`

This allows users to share MCP configs across tools without manual reformatting.

## Authentication & Encryption

### OAuth 2.1

**File**: `/crates/elph-agent/src/tools/mcp/auth.rs`

Supports OAuth 2.1 with PKCE for remote MCP servers:

| Function                       | Purpose                      |
| ------------------------------ | ---------------------------- |
| `run_oauth_flow()`             | Run OAuth 2.1 PKCE flow      |
| `run_oauth_flow_with_scopes()` | Run with custom scopes       |
| `resolve_oauth_access_token()` | Resolve/refresh stored token |
| `clear_credentials()`          | Clear stored credentials     |

### Auth Store

**File**: `/crates/elph-agent/src/tools/mcp/auth.rs` (within)

- `FileCredentialStore` — Encrypted file-based credential storage
- `FileCredentialStoreBuilder` — Builder with key path, auth file path
- `AuthStoreFile` — Represents stored auth data
- Default files: `auth.json` (encrypted credentials), `auth.key` (encryption key)

### Credential Encryption

**File**: `/crates/elph-agent/src/tools/mcp/crypto.rs`

- `Aes256Key` — AES-256-GCM encryption key
- `encrypt_async()` / `encrypt_string_async()` — Encrypt data
- `decrypt_async()` / `decrypt_string_async()` — Decrypt data
- `ENCRYPTED_PREFIX` — Values starting with `enc:` are encrypted

### Auth Resolution

**File**: `/crates/elph-agent/src/tools/mcp/auth_resolve.rs`

- `McpAuthSource` — Auth source enum (none, env, oauth, encrypted)
- `McpAuthSourceReport` — Report of all auth sources
- `resolve_remote_auth()` — Resolve auth for a remote server

### Auth Conflict Policy

**File**: `/crates/elph-agent/src/tools/mcp/config.rs`

`McpAuthConflictPolicy` — How to handle auth conflicts:

- `PreferNew` — Prefer new auth configuration
- `PreferStored` — Prefer stored credentials
- `Error` — Error on conflict

## Tool Policy

**File**: `/crates/elph-agent/src/tools/mcp/policy.rs`

| Type                                                                | Purpose |
| ------------------------------------------------------------------- | ------- |
| `McpPolicyConfig` — Policy configuration for all servers            |         |
| `McpPolicyAction` — Action to take (Allow / Deny / RequireApproval) |         |
| `mcp_tool_requires_approval()` — Check if a tool requires approval  |         |
| `pattern_matches()` — Pattern matching for tool names               |         |

## Validation

**File**: `/crates/elph-agent/src/tools/mcp/validate.rs`

- `McpConfigValidationError` — Validation error types
- `validate_mcp_config()` — Validate MCP config structure
- `validate_mcp_config_semantic()` — Semantic validation
- `validate_mcp_config_value()` — Value-level validation
- `validate_server_config()` — Validate single server config

## Events

**File**: `/crates/elph-agent/src/tools/mcp/events.rs`

- `McpEventBus` — Event bus for MCP server events
- `McpClientService` — Service wrapper around MCP client
- `McpServerEvent` — Event types (connected, disconnected, error, tool_list_changed, etc.)

## Store Lock

**File**: `/crates/elph-agent/src/tools/mcp/store_lock.rs`

Filesystem locking for the credential store to prevent concurrent access.

## Truncation

**File**: `/crates/elph-agent/src/tools/mcp/truncate.rs`

Result truncation to prevent oversized tool results from consuming too much context.

## CLI Commands

**File**: `/elph/src/cli/mcp.rs`

The `elph mcp` subcommand manages MCP server configurations:

- `elph mcp list` — List configured servers
- `elph mcp add` — Add a server configuration
- `elph mcp remove` — Remove a server
- `elph mcp test` — Test connection to a server
- `elph mcp auth` — Manage OAuth credentials

## Key source files

| Concern          | Path                                               |
| ---------------- | -------------------------------------------------- |
| Module root      | `/crates/elph-agent/src/tools/mcp/mod.rs`          |
| Auth & OAuth     | `/crates/elph-agent/src/tools/mcp/auth.rs`         |
| Auth resolution  | `/crates/elph-agent/src/tools/mcp/auth_resolve.rs` |
| Client & connect | `/crates/elph-agent/src/tools/mcp/client.rs`       |
| Config types     | `/crates/elph-agent/src/tools/mcp/config.rs`       |
| Crypto           | `/crates/elph-agent/src/tools/mcp/crypto.rs`       |
| Events           | `/crates/elph-agent/src/tools/mcp/events.rs`       |
| Policy           | `/crates/elph-agent/src/tools/mcp/policy.rs`       |
| Tool registry    | `/crates/elph-agent/src/tools/mcp/registry.rs`     |
| Sessions         | `/crates/elph-agent/src/tools/mcp/session.rs`      |
| SSE transport    | `/crates/elph-agent/src/tools/mcp/sse.rs`          |
| Store lock       | `/crates/elph-agent/src/tools/mcp/store_lock.rs`   |
| Truncation       | `/crates/elph-agent/src/tools/mcp/truncate.rs`     |
| Validation       | `/crates/elph-agent/src/tools/mcp/validate.rs`     |
| CLI commands     | `/elph/src/cli/mcp.rs`                             |
| Platform MCP     | `/elph/src/platform/mcp.rs`                        |

## Change guidance

- **New transport**: Add transport variant in `client.rs`, update `McpServerConfig` in `config.rs`
- **Auth changes**: Update `auth.rs` and `auth_resolve.rs` — verify encryption in `crypto.rs`
- **Tool registry**: Changes affect tool naming in `registry.rs` — `mcp_{server}__{tool}` convention
- **Tests**: `/crates/elph-agent/tests/mcp_deepwiki.rs`, `tests/encrypt_string.rs`
- **Example**: `/crates/elph-agent/examples/mcp_deepwiki.rs`
- **Schema**: `/schemas/mcp-schema.json`
- **Docs**: `/crates/elph-agent/docs/mcp.md`
