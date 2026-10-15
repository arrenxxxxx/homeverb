# ADR-0042: Bind trusted user approval to one prepared command

Status: accepted for offline Core implementation; appliance admission and the
Flutter/C bridge remain separate work.

## Need

Dishwasher program starts and refrigerator cooling changes need reviewed risk
and preservation semantics. Core currently denies every elevated/high-risk
operation. Do not relabel consequential actions as low risk or accept a command
field such as `confirmed: true` to make them executable.

## Decision

Add a typed, in-process trusted-host boundary for elevated actions. A normal
complete Grant must still permit the exact device, component, action, arguments
and route. `prepare_user_approval` validates those constraints without adapter,
credential or network calls and returns an opaque, nonserializable,
non-cloneable `PendingApproval`. Its immutable view includes the exact Command,
Descriptor, selected Action/Connection and display expiry. Preparation alone
is not user consent. Only trusted UI code may consume it through
`execute_user_approved` after displaying and confirming that view.

Bind each preparation to its Core instance, current approval generation, exact
command and descriptor, and a fixed 60-second monotonic lifetime. Require one
explicit transport and exactly one admitted connection; no automatic routing,
batch, independent-field decomposition or high-risk action is admitted. Metadata
changes, another Core instance, expiry or explicit invalidation refuse before
send. A used request ID cannot be prepared again; receipt lookup is the recovery
path. Two preparations racing for the same request ID still use existing journal
and device-lock rules and cannot cause a second send.

Consume the preparation on every execution attempt, including refusal. Revalidate
all constraints before sending and cap the operation deadline by the approval
expiry. Invalidate pending preparations on native backgrounding/close; cancel
in-flight operations using existing host tokens. Direct Rust hosts must perform
the same invalidation and cancellation on their own lifecycle events. Revocation
and expiry cannot roll back a request already sent.

Adapters receive an unforgeable-in-safe-Rust approval marker only through Core's
validated command. Consequential adapters must require this marker in addition
to their fixed model/protocol/risk checks, so weakening public descriptor risk
cannot grant authority. The marker has no caller-writable JSON representation.
Retained receipts include a fixed warning that identifies the trusted approval
entry point; the journal already binds the complete command and request ID.
Acknowledgements, uncertainty, verification, no-replay and storage failures keep
their existing meanings. A prepared scope is not a live state/precondition check.

## Boundaries and compatibility

This phase adds Rust Core/Session methods only. It adds no ordinary JSON method,
C function, CLI flag, MCP tool or startup permission list. Default denial remains
in effect on all existing production entry points. Native API 1.12.1, the 31 JSON
methods, seven C functions, public command/result schemas, SQLite schema 4 and
protected payload 3 are unchanged. A later reviewed native bridge must keep
preparation/confirmation out of model-controlled JSON and implement the trusted
foreground UI flow before any real action can use it. High-risk support is still
excluded. No real appliance action is admitted by this foundation.

## Verification

Use a fictional elevated action to check matching grants, exact scope, explicit
single connection, expiry, metadata changes, wrong Core, one-use ownership,
invalidation, host background/close/cancellation, journal recovery/replay,
accepted/unknown/rejected results and adapter approval-marker enforcement.
Negative cases must remain unsent. Compile-fail examples prevent serialization
and cloning. Ordinary execute/plan and forged JSON continue to deny elevated
operations. No owner account, appliance, phone or remote CI result is implied.
