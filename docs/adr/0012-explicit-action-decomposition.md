# ADR-0012: Require explicit permission to split action arguments

Status: accepted for the unreleased Core MVP.

## Context

Core previously split every non-atomic action with multiple arguments when each single-field command passed validation. An action's lack of atomicity does not establish that its arguments are independent. A start command with optional program and duration arguments could become two separate starts. Even an idempotent setter may require its parameters to travel together.

## Decision

Add optional `independent_fields: boolean` to the action descriptor, defaulting to false. Core decomposes a multi-field command only when this declaration is true. The declaration requires `atomic: false` and `idempotency: set_value`; contradictory descriptors fail operational admission with `CAPABILITY_UNSUPPORTED` before any adapter call or journal reservation.

The declaration promises that setting each supplied field separately has the intended semantics, that field order does not change the result, and that fields do not implicitly reset one another. Protocol evidence belongs to the adapter. Single-field argument validation and whole-command/per-step grants still apply before any send; a declaration cannot bypass either check.

Without the declaration, Core passes the complete argument object to one adapter invocation per permitted attempt. It does not claim that the physical operation is atomic. Proven-unsent fallback remains allowed; uncertain writes are never replayed. An acknowledgement without qualifying evidence remains `accepted`, and a lost acknowledgement remains `unknown`.

## Compatibility and verification

The JSON field is optional and older stored descriptors deserialize with false. The unreleased Rust `Action` struct gains a field, so direct struct constructors must provide it. Strict consumers of the previous capability schema must update their validator. The command, result, native request, and C ABI layouts are unchanged.

The fictional climate capability advances to revision 2 and explicitly opts in, preserving its existing per-field receipts. Rediscover Mock devices to update persisted capability metadata. Older persisted descriptors remain usable but no longer imply decomposition; they send the complete argument object. Existing request-ID receipts still replay without a new send.

Regression tests cover full-argument delivery across idempotency categories, acknowledgement loss without field loss or route replay, explicit false/true declarations, invalid combinations, and the existing composite permission, partial-result, and replay cases. No real-platform support is established.
