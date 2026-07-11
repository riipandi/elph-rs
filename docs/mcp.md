# MCP integration

Elph connects to [Model Context Protocol](https://modelcontextprotocol.io/) servers and exposes their tools (plus resources/prompts bridges) to the agent loop.

## Config

File: `~/.elph/mcp.json` (schema: [`schemas/mcp-schema.json`](../schemas/mcp-schema.json)).

```json
{
  "policy": {
    "default": "requireApproval",
    "allow": ["mcp_fs__list*", "mcp_fs__read*"],
    "deny": ["mcp_dangerous__*"]
  },
  "servers": {
    "fs": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]
    },
    "remote": {
      "type": "http",
      "url": "https://example.com/mcp",
      "oauth": true
    },
    "legacy": {
      "type": "sse",
      "url": "http://localhost:3000/sse"
    }
  }
}
```

### Transports

| `type` | Meaning |
|--------|---------|
| `stdio` | Local child process |
| `http` / `streamableHttp` / `streamable-http` | Streamable HTTP (current remote standard) |
| `sse` | Legacy HTTP+SSE |

### Auth

- **Bearer**: `authToken` or `authTokenEnv`
- **OAuth 2.1 + PKCE**: set `"oauth": true`, then:

```bash
elph mcp auth remote
elph mcp logout remote
```

Credentials: shared file `~/.elph/auth.json` (keyed by server name under `mcp`, mode `0600` on Unix).

The library does not hardcode this path — hosts pass it via `AuthStorePathBuilder` /
`McpLoadOptions.auth_store_path` (default filename `auth.json`).

## CLI

| Command | Behavior |
|---------|----------|
| `elph mcp list` | Servers + oauth status (no secrets) |
| `elph mcp add <name> <json\|file>` | Upsert server |
| `elph mcp remove <name>` | Remove server + OAuth creds |
| `elph mcp doctor` | Probe connectivity |
| `elph mcp auth <name>` | OAuth browser flow |
| `elph mcp logout <name>` | Clear OAuth tokens |

## Agent surface

Tools are named `mcp_{server}__{tool}` (sanitized).

Bridge tools (when the server supports the capability):

- `mcp_{server}__list_resources`
- `mcp_{server}__read_resource`
- `mcp_{server}__list_prompts`
- `mcp_{server}__get_prompt`

### Policy

- **deny** — not exposed
- **allow** — exposed, no approval
- **requireApproval** (default) — exposed; TUI approval dialog (unless Brave mode)

Patterns: exact, `prefix*`, `*suffix`, `*`.

### Hot reload

When a server sends `notifications/tools/list_changed` (or resource/prompt variants), the registry refreshes that server and updates harness tools.

## Library

```rust
use elph_agent::{McpConfig, McpLoadOptions, McpToolRegistry};

let mut options = McpLoadOptions::default();
options.auth_store_path = Some(paths.auth_store_path());
let registry = McpToolRegistry::load_with_options(config, options).await?;
let tools = registry.create_agent_tools();
```
