# ADR-0038: Reuse Meiju Core access in a local MCP session

Status: accepted for implementation; target-host and physical acceptance remain pending.

## Decision

Allow explicit `--adapter midea.meiju mcp --stdio` startup with the same private
profile already connected through the trusted CLI or native host. Default MCP
startup has no write grants. It exposes the existing local list/description,
on-demand state and receipt tools without login, discovery or account management.
No tool accepts credentials or selects a profile, principal or adapter.

An owner-controlled `--meiju-grants` file, validated under ADR-0037, may admit the
existing exact-device power/fan values. Only nonempty startup grants expose the
additional `command_plan` and `command_execute` tools. Each accepts one complete
Command under a closed `command` field, including an explicit stable request ID.
Core still checks the full command, risk and route; tool visibility is not authority.
Mock MCP remains read-only. This adds no batch tool or new appliance action.

The host reads grants once at startup. Editing or removing the file does not revoke
an existing session; stop/restart the host to replace its policy. This differs from
one-shot CLI invocation. Ordinary tool input cannot reload or broaden permissions.
Startup rejects nondefault global route/wait options and global request/deadline/dry-run options
instead of silently ignoring them; reads use the existing ten-second budget and
commands use their own bounded timeout.

Plans return the existing query envelope and do not discover, refresh or journal.
Execution returns the existing Execution envelope inside MCP structured content
and matching text content. `isError` is false only for verified results or accepted
results that requested acceptance. Failed, unknown, partial and unverified accepted
results remain explicit; the receipt is never replaced by a generic error. Parsing
errors return the query error envelope. Receipt lookup remains a successful query
when it finds a failed/unknown receipt. MCP request IDs correlate RPC responses;
Command request IDs preserve execution/replay identity across sessions. They are
not interchangeable. Never invent a new Command ID to retry uncertainty.

Cancellation notifications cancel the matching operation. EOF/SIGINT cancel all
active work and await its existing bounded cleanup. Input/output failure also
cancels and drains work; output backpressure is bounded to two seconds. A sent command may still act;
later receipt lookup preserves its outcome without replay. Idle startup/session
work must not open appliance connections or refresh accounts. No public listener,
relay, background task or independent service is introduced.

## Verification and compatibility

Test actual stdio startup, the unchanged default tool set, scoped tools/schema
parity, invalid/forged inputs, local plans, pending-account pre-send refusal,
independent grants and receipt replay with fictional profiles. Use a fictional
adapter to test admitted results, cancellation, EOF and reopen without real sends.
Keep native API 1.10.0, seven C functions, SQLite schema 4 and protected payload
version 2 unchanged. Target AI host integration and physical/off-LAN control remain
separate acceptance gates.
