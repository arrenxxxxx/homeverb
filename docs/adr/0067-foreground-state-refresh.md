# ADR-0067: Refresh visible App device state every three seconds

Status: accepted by the owner on October 17, 2026.

## Decision

The owner requested automatic device-state refresh while the App is open, at a
three-second interval, with no notification for refresh failures. This narrowly
supersedes the previous App prohibition on periodic reads. CLI, MCP, adapters,
account inventory and background work retain their existing lifecycle rules.

The production native App schedules read-only `read_state` operations on device
views while resumed: supported cards in the current home/search/room filter, or
the selected device's components in its controls/compartment view. Changing the
filter changes the next cycle's targets. Startup still loads local metadata and
cache first; the first automatic cycle follows after three seconds. Non-device
views suspend the schedule. No discovery, account inventory refresh, grant setup,
write, verification replay or credential-recovery flow is scheduled.

Cycles never overlap. If a cycle exceeds the interval, intervening ticks are
skipped rather than queued. Every read uses the existing bounded Core request and
chosen route. A user operation cancels and drains an automatic read before using
the native session; automatic work never cancels a user operation. Navigation,
backgrounding and disposal cancel the schedule and in-flight reads and fence late
results. Resuming starts a fresh interval without catch-up requests.

Successful snapshots update the same card/component caches as manual reads.
Provider observations retain their timestamp, source and stale markers; a periodic
cloud response is not proof of physical freshness. Failures preserve the prior
snapshot and produce no loading indicator, error banner, snackbar or modal. They
do not overwrite an existing manual-operation error or execution receipt. The next
regular cycle may read again, without extra within-cycle retry or backoff burst.

## Boundaries and compatibility

The timer is owned by the foreground App, not Core, an adapter, a daemon or an OS
service. There is no background keep-alive, persistent subscription, account-wide
inventory refresh, HTTP control bridge or change to CLI/MCP idle behavior.
Forced termination promises no callback. Existing write outcomes and uncertain
receipt recovery remain unchanged. No C/JSON method, native API 1.22.0, SQLite
schema 5, protected payload 3 or adapter-support claim changes.

## Verification

Use virtual time and fictional Core fixtures to check the three-second cadence,
silent partial failures, component and filter scope, no overlapping reads or
queued ticks, foreground cancellation/resume, late-result fencing and user-write
priority. Confirm that automatic reads do not prepare grants, refresh accounts,
replay commands, change receipts or introduce global activity UI. Build/archive
checks and in-place emulator installation remain separate from physical-device
and provider-rate acceptance.
