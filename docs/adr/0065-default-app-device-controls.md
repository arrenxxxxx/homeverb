# ADR-0065: Make supported App device controls available by default

Status: accepted by the owner on October 16, 2026.

The later [ADR-0067](0067-foreground-state-refresh.md) supersedes App state-read
timing only. The on-demand grant-preparation decision below remains unchanged.

## Decision

Remove the Flutter session-control selection, review, explicit capability-refresh
and revocation interface. The owner explicitly requested default access to all
supported controls without additional checkboxes. This replaces the previous
App permission workflow, not the Core/native/CLI/MCP defaults.

On a foreground user action that opens a device, the trusted App inspects its
exact saved cloud binding and reviewed control options. For the existing AC
host mappings, it prepares every model-admitted low-risk, set-value power,
temperature, fan and mode value using the existing trusted startup grant fields.
It closes/reopens the same private native profile, then refreshes only the selected
device's account to publish the current descriptors. There is no separate consent
step for these controls. No appliance write is part of preparation.

Configuration is scoped to the opened device and held in memory. It is derived
again on demand after process restart or device switching, never imported from
ordinary preferences or public requests. Native argument bounds, identity/model
evidence, permissions and independent adapter checks still apply. Unsupported
models and actions stay unavailable; this policy adds no elevated/high-risk or
non-idempotent mapping and does not replace trusted single-command confirmation.

## Lifecycle and errors

App startup still loads local metadata only. Preparation is on demand, bounded
by existing Core operations and foreground cancellation; it neither polls nor
refreshes all saved accounts. Navigation/backgrounding invalidates pending setup.
A replacement completed after cancellation is closed. Failed descriptor refresh
cannot be treated as prepared controls, and a later explicit device open retries
setup. It never replays a command or reinterprets an unknown receipt.

## Compatibility and verification

No C/JSON method, native API 1.22, SQLite schema 5 or protected payload 3 changes.
Core and non-App callers retain their existing explicit trusted startup policy.
The old Flutter permission widget is removed, including its technical inspection
and revocation actions from Device Information. Shared stable locale identifiers
remain compatible even when their former UI is no longer rendered.

Verify exact option scope, no startup networking or implicit commands, same-device
reuse, foreign-response refusal, failed-refresh retry, lifecycle recovery and
absence of the old UI. Real appliance acceptance must still check actual command
receipts, subsequent timestamped reads and the limits of physical evidence.
