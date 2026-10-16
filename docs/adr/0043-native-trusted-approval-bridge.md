# ADR-0043: Expose trusted user approval through separate native functions

Status: accepted for offline implementation; real consequential appliance mappings
and physical acceptance remain pending.

## Decision

Native API 1.13.0 adds three C functions outside the ordinary JSON dispatcher:
`homeverb_prepare_approval`, `homeverb_execute_approved` and
`homeverb_discard_approval`. Keep the 31 general JSON methods unchanged. Trusted
Flutter controller code owns these functions and must never forward their access
or approval identifiers to a model, plugin, imported command or remote endpoint.
The process is trusted; these exports are not an in-process sandbox.

Preparation takes closed metadata with schema version, operation ID and the exact
Command. It uses ADR-0042's typed foreground path and returns a query envelope
containing a process-local unsigned approval ID plus immutable command, descriptor,
action, connection and display expiry. Display that returned scope and obtain
explicit user consent before calling the confirmation function. Preparation does
not grant permission or send. Existing complete grants remain mandatory.

Session owns at most eight pending approvals and discards expired entries when
preparing another. IDs are unique across live sessions, cannot be transferred to
another handle or restart, and are neither credentials nor durable request IDs.
They must not be logged, persisted, put in ordinary command JSON or used as proof
that a user confirmed. A successful call of the separate trusted confirmation
function is the host's assertion of actual user interaction.

Confirmation takes only schema version, a new operation ID and the approval ID.
It accepts no replacement command, principal, grant or confirmation boolean.
A decoded confirmation attempt consumes a matching pending ID before operation
admission, including refusal. Malformed input that cannot identify a request does
not consume a preview. Pre-admission failures return the usual query error;
admitted execution returns the existing Execution receipt. Missing, dismissed,
consumed, cross-handle or pruned IDs return `APPROVAL_INVALID`; a retained expired
preview returns an unsent `APPROVAL_EXPIRED` execution. Lookup the original
command request ID after interruption; never automatically prepare or confirm a
replacement. Dismissal removes a pending preview and never affects a sent action.

Background/close clears all native previews, increments the lifecycle generation
and cancels active work. Preparation checks the generation before publishing its
preview, including a background/foreground race. No mutex is held over execution
or network work. Existing eight-active-operation admission, fixed 60-second
monotonic expiry and shorter command/host budgets still apply. All three C calls
use the existing string ownership and worker-thread conventions.

## Fictional acceptance path

Add an explicit Mock-only `approval-required` scenario exposing
`vendor.example.cycle.start {program: normal}` with elevated risk and unavailable
verification. In this scenario, `allow_mock_writes` adds a grant for that exact
fictional action alongside existing Mock power/climate grants. Ordinary execution still denies it, and the adapter independently
requires Core's approval marker even when descriptor risk is weakened. Default
Mock behavior and every Meiju permission/mapping remain unchanged. The scenario
lets Flutter exercise the full consent journey before real actions are admitted.

## Compatibility and verification

Publish the three symbols and pending/expiry limits beside the versioned API catalog. Require
API 1.13 before resolving symbols; older clients can continue using their seven
functions. There are now ten public C functions, plus the Android credential JNI
export. SQLite schema 4 and protected account payload 3 remain unchanged.

Verify actual C calls and closed schemas for prepare/confirm/dismiss, immutable
scope, ordinary-dispatch refusal, disabled grants, invalid/missing/foreign IDs,
capacity, consumption, descriptor changes, background/close, receipt lookup and
redaction. Typed Session tests additionally cover monotonic expiry and lifecycle
races without sleeping for a minute. Build all retained Android AAR variants and
check the new exports. No account or appliance access is required for this phase.
