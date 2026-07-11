# MCP integration

`elph-agent` embeds an MCP **client** (via [rmcp](https://crates.io/crates/rmcp)) so the agent can call tools exposed by external MCP servers.

Feature flag: **`mcp`** (enabled by default).

## Configuration

JSON file (Elph product: `~/.elph/mcp.json`):

```json
{
    "servers": {
        "filesystem": {
            "type": "stdio",
            "command": "npx",
            "args": ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"],
            "env": {},
            "timeoutMs": 60000
        },
        "remote": {
            "type": "http",
            "url": "https://mcp.example.com/mcp",
            "authTokenEnv": "MCP_REMOTE_TOKEN",
            "headers": {
                "X-App": "elph"
            },
            "timeoutMs": 45000
        },
        "off": {
            "type": "stdio",
            "command": "unused",
            "disabled": true
        }
    }
}
```

| Field                                            | Transports | Description                          |
| ------------------------------------------------ | ---------- | ------------------------------------ |
| `type`                                           | both       | `stdio`, `http`, or `streamableHttp` |
| `command` / `args` / `env` / `cwd`               | stdio      | Child process                        |
| `url` / `headers` / `authToken` / `authTokenEnv` | http       | Streamable HTTP endpoint             |
| `timeoutMs`                                      | both       | Per list/call timeout (default 60s)  |
| `disabled`                                       | both       | Skip during discovery and calls      |

## API surface

| Type / fn                             | Role                                  |
| ------------------------------------- | ------------------------------------- |
| `McpConfig`                           | Root config                           |
| `McpServerConfig`                     | `Stdio` \| `Http`                     |
| `McpLoadOptions`                      | Fail-open load, concurrency           |
| `McpToolRegistry::load`               | Discover tools (default fail-open)    |
| `McpToolRegistry::create_agent_tools` | `mcp_{server}__{tool}` agent tools    |
| `McpSessionPool`                      | Long-lived connections with reconnect |
| `probe_server`                        | Connectivity check for doctor/CLI     |

### Tool naming

Exposed names: `mcp_{sanitized_server}__{sanitized_tool}`
Helpers: `expose_tool_name`, `parse_exposed_tool_name`.

### Production behavior

1. **Load** runs discovery concurrently (default max 4). Failed servers are logged and skipped (`continue_on_error: true`).
2. **Calls** use a **session pool**: one stdio process / HTTP session per server, mutexed, with **one automatic reconnect** on failure.
3. **Timeouts** apply per operation (list tools, call tool).
4. **Shutdown**: `McpToolRegistry::shutdown` or drop of session pool closes clients.

## Usage

```rust
use elph_agent::{McpConfig, McpLoadOptions, McpServerConfig, McpToolRegistry};
use std::sync::Arc;

let mut config = McpConfig::default();
config.servers.insert(
    "fs".into(),
    McpServerConfig::stdio("npx", vec![
        "-y".into(),
        "@modelcontextprotocol/server-filesystem".into(),
        "/tmp".into(),
    ]),
);

let registry = Arc::new(
    McpToolRegistry::load_with_options(config, McpLoadOptions::default()).await?,
);
let mut tools = elph_agent::create_coding_tools(env);
tools.extend(registry.create_agent_tools());
// pass tools into AgentHarness / AgentLoop
```

Elph app wiring: `elph/src/agent/runtime.rs` loads `mcp.json` and extends the tool list.

## Limitations

- MCP **server** role (hosting tools for other clients) is out of scope.
- OAuth browser login for remote MCP is not fully productized (token via `authToken` / `authTokenEnv`).
- Resource/prompt MCP surfaces are not yet mapped to agent tools (tools only).
