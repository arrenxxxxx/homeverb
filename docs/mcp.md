# Local MCP interface

Status: local stdio supports Mock reads and explicit Meiju startup. Meiju defaults
to read-only; reviewed startup grants can admit the existing experimental AC power
and vendor fan controls, explicit power-on with mode selection, and profile-scoped
temperature targets. Protocol and fictional execution tests cover this path;
validation inside a particular AI host and physical controls remain pending.

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

For Meiju, connect through the [trusted CLI flow](cli.md#meiju-account-and-read-commands)
first, then configure this argument list with the same private data directory/profile:

```json
{
  "mcpServers": {
    "homeverb": {
      "command": "/absolute/path/to/homeverb",
      "args": ["--adapter", "midea.meiju", "--data-dir", "/private/homeverb", "--profile", "meiju", "mcp", "--stdio"]
    }
  }
}
```

The example paths are placeholders. `--data-dir` is a parent; the profile is
appended. The host must run as the same OS user with access to the intended secret
store for live reads. Cached reads, descriptions and receipts need no secret read.
Startup/initialization never logs in, discovers devices or refreshes an account.
Account connection, recovery/removal and discovery stay outside MCP. Selecting
Meiju never starts a Mock appliance.

To admit exact reviewed controls, add `--meiju-grants` and an absolute path to an
[owner-controlled file](cli.md#scoped-meiju-controls) in the trusted host argument
list. Explicit CLI account refresh/discovery with that file publishes action
descriptors before MCP starts. Nonempty valid grants expose two additional tools;
missing/empty grants keep four read tools. Invalid files refuse startup before
opening the profile. Mock MCP remains read-only.

The policy is read once at startup. Editing/removing the file does **not** revoke
an active MCP session. Stop the host and restart with revised/empty grants to
replace its authority; no tool can reload permissions. An already sent operation
cannot be undone by revocation. The owner must separately authorize any real send
performed during development. See [ADR-0038](adr/0038-scoped-meiju-mcp.md).

The host owns the subprocess and its lifetime. No HTTP endpoint, relay, or independently deployed service exists. The implementation negotiates protocol `2025-06-18`, uses newline-delimited JSON-RPC 2.0, and accepts up to eight concurrent tool calls with a 1 MiB frame limit. Reads use a ten-second budget; each command supplies its own 1-180000 ms budget. Startup rejects global request IDs, dry runs, explicit timeouts and nondefault route/wait options instead of silently ignoring them.

## Tools and policy

| Tool | Arguments | Behavior |
|---|---|---|
| device_list | Empty object | Local records only |
| device_describe | device | Cached capability description |
| device_get_state | device, optional component/cached | Explicit live or cached read |
| execution_get | request_id | One retained local receipt, without refresh or replay |
| command_plan | command | Granted sessions only: local Core validation, without discovery, refresh, journal or send |
| command_execute | command | Granted sessions only: Core execution with existing per-device permissions, receipts and conservative outcomes |

The four default tools and `command_plan` have read-only annotations.
`command_execute` is explicitly effectful. Annotations and tool visibility never
grant authority: Core independently checks the device, component, action, arguments,
risk and route. The closed `command` field embeds the existing
[Command schema](../schemas/command.schema.json). Missing fields, extra principal,
approval, profile or grant fields are rejected. No batch or authentication tool is
exposed. A valid plan does not establish saved inventory membership, cloud
reachability or physical correctness; execution repeats those checks.

A `meiju_mode_grants` entry permits only the combined
`vendor.midea.ac.activate_mode` action for that device and mode. Describe it to
the user as turning on the AC and selecting a mode. Independent power/fan grants
do not imply this permission. See the [activation contract](native-ui-contract.md#scoped-experimental-ac-mode-activation-startup).

A `meiju_temperature_grants` entry permits only `main / climate.set` with that
exact `target_temperature` and device over cloud. The described action's bounds
and step must also permit it; a grant cannot establish a missing product mapping.
There is no power/mode/fan side effect in the request. Use the same generic
`command_plan` and `command_execute` tools and keep accepted/unknown outcomes
unverified. See the [temperature contract](native-ui-contract.md#scoped-experimental-ac-temperature-startup).

For example, this fictional tool request plans a single allowed power-off command:

```json
{
  "jsonrpc": "2.0",
  "id": "rpc-preview-1",
  "method": "tools/call",
  "params": {
    "name": "command_plan",
    "arguments": {
      "command": {
        "schema_version": "1.0",
        "request_id": "owner-request-1",
        "device_id": "30000000-0000-4000-8000-000000000001",
        "component_id": "main",
        "capability": "power",
        "action": "set",
        "arguments": {"state": false},
        "connection": "cloud",
        "timeout_ms": 10000,
        "wait_for": "verified"
      }
    }
  }
}
```

For an authorized send, use `command_execute` with the reviewed command. The RPC
`id` correlates a protocol response; `command.request_id` identifies the durable
appliance operation. Preserve the latter across host sessions and retries. Never
invent a new command ID to repeat an accepted/unknown operation. Identical commands
return retained results; changed commands conflict and retired IDs remain refused.

## Results and shutdown

Results include `structuredContent`, the same envelope serialized in a JSON text
content block, and `isError`. Queries and plans use the query envelope. Admitted
`command_execute` calls return the [Execution envelope](../schemas/command-result.schema.json)
directly, including failed preflight results. Malformed input returns a query error
envelope. No generic error replaces an existing execution receipt.

| Execution result | MCP `isError` |
|---|---|
| `verified` | `false` |
| `accepted`, requested `accepted` | `false`; no physical verification claim |
| `accepted`, requested `verified` | `true`; preserve `VERIFICATION_PENDING` and receipt |
| `failed`, `unknown`, `partial` | `true`; preserve error, attempts and per-item outcomes |

For `execution_get`, a found receipt is a successful tool query even when its stored status is `unknown`, `failed`, or `partial`. Missing/retired receipts produce `REQUEST_NOT_FOUND` / `REQUEST_RETIRED` with `isError: true`; no outcome is inferred and nothing is resent. Receipt fields are local history, not a redacted diagnostic export.

Cancellation notifications use the RPC request ID to cancel matching active work.
EOF/SIGINT cancel all remaining operations and await cleanup. Input/output failures
also cancel and drain active work; output backpressure has a two-second bound.
Shutdown can prevent response delivery, so inspect the durable command ID through
`execution_get` or the CLI after restarting. Cancellation is not physical rollback.
Idle sessions create no appliance requests or Internet listeners. Error messages
are English and contain no raw vendor responses.
