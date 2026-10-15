# ADR-0014: Inspect a recorded command without replaying it

Status: accepted for the unreleased Core MVP.

## Decision

Expose `Core::execution(request_id, operation)` as a read-only lookup of one retained receipt. CLI uses `execution <request-id>`, native requests use `get_execution`, and the read-only MCP tool is `execution_get`. Every entry point calls Core directly and preserves its operation budget.

Return the saved receipt in the query envelope without changing its status, timestamps, evidence, or item results. Do not require current write grants, device metadata validation, or a device lock. Do not call adapters, read appliance state, update the journal, or resend any command. This also permits inspection while another local operation holds the device lock.

Lookup uses the exact nonblank request ID, without trimming or alias resolution. Internal composite IDs may exceed the 256-byte caller-command limit and remain queryable. Existing host frame limits still apply. A retired ID returns `REQUEST_RETIRED`; an absent ID returns `REQUEST_NOT_FOUND`. Neither error establishes that a physical action failed or authorizes replay.

## Result semantics

A successful query returns exit code 0 even if its stored receipt is `failed`, `accepted`, `unknown`, or `partial`. The query succeeded; the nested receipt preserves the command outcome. A later fresh state read is separate evidence and does not rewrite the old receipt or prove causality. Active records are conservative snapshots and may change while their owning operation continues.

Retention still applies. Exact lookup does not resurrect a retired receipt, and inspecting a record does not extend its retention. Ordinary host startup may perform the existing journal recovery before handling a query; the lookup itself has no recovery side effects.

## Compatibility and verification

The Core method, CLI command, native method, and MCP tool are additive. Existing command/result envelopes, C ABI functions, and SQLite schema remain unchanged. Older hosts reject the unknown request/tool; clients must handle that response. Lookup exposes the same local receipt fields as history, so it is not a redacted diagnostic export.

Tests cover lookup after lost acknowledgement, changed metadata and removed write grants, active records beyond the recent-history limit, missing/retired/blank IDs, long child IDs, cancellation/deadline refusal, no journal or adapter mutations, and CLI/native/MCP queries.
