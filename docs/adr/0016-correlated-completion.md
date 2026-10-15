# ADR-0016: Verify bounded correlated completion evidence

Status: accepted for the unreleased Core MVP.

## Decision

An adapter may return `AdapterOutcome.completion: Option<CorrelatedCompletion>` from the existing bounded `execute` call. This is evidence of successful completion of the entire supplied command, never an enqueue acknowledgement. The adapter must validate the protocol's request correlation and success semantics before constructing it. A local request ID alone does not establish device-side correlation. Raw provider tokens and responses remain inside the adapter and are not persisted or exported.

The evidence binds the exact local request ID, device ID, component ID, and selected connection ID. It carries local receipt time and, only when trustworthy, device completion time. Core requires receipt time at or after the attempt began and no later than its current clock; an available completion time must be between attempt start and receipt. It uses the selected connection's transport in the public verification record.

Only an accepted outcome for an action declaring `correlated_completion`, with `wait_for: verified`, can use this evidence. Missing, mismatched, or invalid evidence leaves `accepted` with `VERIFICATION_PENDING`. Rejected, unsent, and uncertain outcomes cannot be promoted. Core does not substitute state matching for a declared completion requirement, replay the write, or switch routes to obtain evidence. An accepted-only caller does not request completion work.

The adapter may await a protocol response within this one operation. It marks acceptance as soon as known, so cancellation or deadline expiry during the wait preserves acceptance. There is no additional observer API, timer, background subscription, or completion query after host exit. Correlated evidence already obtained is preserved if subsequent journal persistence fails. It never fabricates a state snapshot or establishes that the device's current state still equals an earlier target.

## Compatibility and limits

Adding the optional field breaks Rust struct literals for the unreleased adapter API: existing adapters set `completion: None`. The public JSON contract already supports `verification.kind: correlated_completion`; no command, native request, C ABI, SQLite migration, or locale key changes are needed. State-match actions and normal Mock behavior remain unchanged. Composite children carry separate evidence; their parent summary makes no atomic or exactly-once claim.

The `correlated-completion` Mock scenario supplies fictional completion evidence and declares the matching verification method on discovery. Rediscover an isolated profile under that scenario before using it. Rediscover with the normal scenario to restore state matching. This is a simulator and contract test, not evidence for any real protocol.

## Verification

Integration tests cover actions without observable properties, exact request/device/component/connection binding, optional source timestamps, old/future/reversed times, unsupported methods, accepted-only requests, missing evidence, rejection, uncertainty, cancellation/deadline after acceptance, composite children, replay after reopening, and storage failures. Runtime acceptance validates CLI completion receipts, composite evidence, restart lookup, and preserved snapshots. A real adapter still needs protocol correlation evidence and model-specific physical acceptance.
