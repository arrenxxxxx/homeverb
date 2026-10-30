# ADR-0118: CLI and local MCP event occurrence history

Status: Implemented; synthetic and local subprocess verification passed.
Date: October 30, 2026.

## Decision

Add `event-history options` and `event-history read --file` to the CLI, and
`event_history_options` / `event_history_read` to host-managed local stdio MCP.
Both reuse ADR-0116 Core methods and closed EventHistoryRequest types. CLI files
are regular files bounded to 16 KiB; aliases, raw provider keys/payloads/cursors,
principals, grants and unrelated command flags are refused. MCP embeds the same
request schema, keeps its ten-second read budget and eight active-call bound,
and retains notification/EOF/SIGINT/host cancellation without retry.

A new explicit read-only Mijia Core composition opts into reviewed event history
and modern transports. The existing legacy-only constructor remains compatible.
CLI/MCP use the new composition so saved SID=mijia accounts select the modern
protected protocol; legacy accounts keep their original route. No read logs in,
renews authorization or falls back to another protocol. CLI account operations
use native startup event opt-in so an explicit connect/refresh can publish the
reviewed descriptors. Existing scalar-only saved descriptors require such a
refresh; starting MCP or inspecting options never refreshes them automatically.

MCP adds two read-only tools, for eight default tools or ten with separately
admitted Meiju controls. Mijia retains no write grants. Tool annotations do not
grant authority. Core and the adapter independently check exact event/connection,
protected account membership/model/specification and operation lifetime. Results
preserve same-second occurrence timestamps, rejected counts, limits and unproven
completeness, without payloads, state-cache/receipt writes or inferred causality.
There is no new background work, subscription, HTTP listener or public API.

## Compatibility and verification

This is additive command/tool coverage; native API 1.44, C/JNI signatures,
SQLite 5 and protected payload 3 do not change. The Android package 93cabfd
already contains native/App event history and remains unchanged. Test actual
CLI/MCP subprocesses with fictional profiles, closed input schemas and exit/error
semantics; use injected adapters for successful event reads and pending-request
cleanup. Check the read-only composition is empty/local at startup, both exact
protected protocols in the existing adapter tests, and broader CLI/Core/native
regressions. No real credentials, provider traffic or physical effect is evidence
for this phase. Payload interpretation, paging and real acceptance remain open.

## Executed evidence

Rust 1.96 offline/locked workspace tests passed 740 tests with one ignored,
excluding `homeverb-midea` because its known macOS LAN-alias failures remain open.
The focused CLI checks passed 19 tests and Mijia Host startup passed two tests.
Strict workspace Clippy, formatting and the full workspace build passed. Actual
`scripts/verify_mvp.py` subprocess acceptance passed 31 of 38 tests; seven were
skipped on macOS (six Linux-only grant-file checks and one pipe-capacity check).
The final run checks the complete eight-tool Mijia MCP list.

Checks cover regular-file/FIFO admission, closed inputs and exact selectors,
duplicate occurrences, cancellation/deadline cleanup and MCP EOF behavior.
Existing protected adapter tests retain both legacy and modern protocol coverage.
Contract validation passed 26 schemas, 88 positive and 1,966 negative fixtures;
three reference projection tests passed. Documentation and repository checks
passed. These results establish fictional/local behavior, not provider or phone
acceptance. No real account/device operation, installation, push or publication
was performed.
