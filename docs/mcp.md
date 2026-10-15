# Local MCP interface

Status: read-only stdio is implemented. The protocol harness verifies initialization, discovery of tools, reads, write denial, protocol-only stdout, and EOF exit. Validation inside a particular AI host remains pending.

## Host configuration

```json
{
  "mcpServers": {
    "homeverb": {
      "command": "homeverb",
      "args": ["mcp", "--stdio", "--mock", "--profile", "mock"]
    }
  }
}
```

Use the built executable's absolute path when it is not on PATH. Run explicit CLI discovery for the same profile first. CLI and MCP share local SQLite only when their directory/profile match. No background discovery occurs.

The host owns the subprocess and its lifetime. No HTTP endpoint, relay, or independently deployed service exists. The implementation negotiates protocol `2025-06-18`, uses newline-delimited JSON-RPC 2.0, and accepts up to eight concurrent tool calls with a 1 MiB frame limit.

## Tools and policy

| Tool | Arguments | Behavior |
|---|---|---|
| device_list | Empty object | Local records only |
| device_describe | device | Cached capability description |
| device_get_state | device, optional component/cached | Explicit live or cached read |
| execution_get | request_id | One retained local receipt, without refresh or replay |

Every tool has a read-only annotation, but authorization is enforced independently: this host's Core has no write grants. Unlisted write tools and forged principal fields fail. Model parameters cannot change profile or collect credentials. Write tools and real authentication remain pending reviewed work.

Results include `structuredContent`, a JSON text content block, and `isError`. For `execution_get`, a found receipt is a successful tool query even when its stored status is `unknown`, `failed`, or `partial`. Missing/retired receipts produce `REQUEST_NOT_FOUND` / `REQUEST_RETIRED` with `isError: true`; no outcome is inferred and nothing is resent. Extra fields such as a principal or approval flag are rejected. Receipt fields are local history, not a redacted diagnostic export. Cancellation notifications cancel the matching active operation. EOF cancels remaining work and exits. Idle sessions create no appliance requests or Internet listeners. Error messages are English and contain no raw vendor responses.
