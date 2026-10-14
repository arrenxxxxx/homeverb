# Local MCP interface

## Execution model

Only stdio is supported. A local AI host launches and owns the subprocess. There is no HTTP endpoint, reverse proxy, or public cloud-callable service. Hosts supporting remote URLs only are outside scope. See [R1](references.md).

```json
{
  "mcpServers": {
    "homeverb": {
      "command": "homeverb",
      "args": ["mcp", "--stdio", "--profile", "home"]
    }
  }
}
```

This is an illustrative common host configuration, not a universal format. No `homeverb` binary exists yet. Verify configuration and negotiated protocol versions against each target host and SDK before release.

## Proposed tools

| Tool | Purpose | Default authorization |
|---|---|---|
| device_list | Enumerate local device records with optional filters | Read |
| device_describe | Capabilities, constraints, and available routes | Read |
| device_get_state | Explicit live or cached read | Read |
| power_set | Explicit on/off | Device-scoped write grant required |
| climate_set | Set target temperature or mode | Parameter-bounded write grant required |
| device_execute | Future generic action entry point | P1 allowlist; never expose every action automatically |

Project tool schemas from capabilities into the subset supported by the selected SDK and host. Do not expose arbitrary private vendor APIs. Contract tests must ensure that CLI and MCP schema projections preserve the same semantics.

## Trusted principal

The principal is fixed by trusted startup configuration. A model cannot pass `principal=admin`. Default to read-only; a trusted user interface or local policy grants specific device/capability writes. Core rechecks risk and bounds. Tool annotations are not authorization.

Do not provide password, token, or OTP collection parameters. Users authenticate through CLI or App first. Tool calls cannot arbitrarily switch profiles or read another OS user's secret store.

## Results and cancellation

Return structured execution outcomes while preserving `accepted` versus `unknown`. Failure to meet the requested wait condition is a tool-level failure, not evidence that no physical action occurred. Do not insert raw vendor responses, secret-bearing authorization URLs, or stderr logs into model context.

Cancellation stops waiting and future stages, not an already-executed action. EOF or host termination closes resources. Idle sessions do not scan, poll, or retain device subscriptions. Short-lived observation requires an explicit tool call and budget.

## Tests

Protocol-only stdout; actual host/SDK initialization and version negotiation; default denial of writes; rejection of forged principal/confirmation fields; cancellation without leaked tasks; exit with the host; and separate transport versus business-error mapping.
