# ADR-0063: Require explicit additional options for centralized dishwasher cycles

Status: accepted for offline implementation; physical acceptance remains deferred.

## Evidence and decision

The pinned E1 mapping at `ce0c4b88224def0f6b53714312c19f71aa9c2a50`, product
760Y0026, declares exactly one centralized field: `additional`. Its reviewed
values are 0, 9, 10, 13 and 14. The cycle mappings use `operator: pause`,
`operator: start` and `work_status: cancel`, respectively. Extend
[ADR-0055](0055-profile-scoped-dishwasher-cycle-controls.md) for this product only,
requiring an explicit additional option on every pause, resume and cancel request.
Do not infer that this is the owner's product from its display model.

The selected option is an explicit setting sent with the operation, not evidence
that the current appliance setting is preserved. Never default it from zero,
read it automatically or copy stale cached state. The user must choose the option
and see it in the immutable confirmation preview. Existing localized option labels
from program start apply to cycle controls as well.

Product 7600016L remains excluded because wash-region and automatic-door domains
are unproven. The four profiles admitted by ADR-0055 retain empty arguments and
unchanged wire payloads. Unknown, missing or conflicting protected product data
remains denied even if public capability metadata is forged.

## Contract and compatibility

Native API 1.21.0 extends each trusted `meiju_dishwasher_cycle_grants` entry with
optional `arguments`. Omission means exactly `{}`, preserving old authorization.
An explicit object may contain only `additional`, with a reviewed integer value.
Each grant matches the exact local device, action and arguments; empty grants
never authorize a nonempty request. Different options may receive separate grants.
Reject duplicate normalized device/action/option tuples and malformed arguments
before opening storage. The existing 64-entry limit remains.

The new profile's cycle-enabled capability has revision 4 and requires
`{"additional":9}` or another admitted value. Other profiles retain revision 3.
Read-only control-option inspection exposes the required domain without granting
permission. Ordinary requests cannot alter startup grants or confirm an action.
Negotiate API 1.21 before sending the new startup field: older libraries reject it.
The unreleased Rust grant struct gains an `arguments` field; callers constructing
it directly must supply an explicit object. No C signature, ordinary method,
SQLite schema or protected-payload format changes.

## Execution and acceptance

Keep elevated risk, non-idempotent execution, independent adapter approval checks,
one cloud attempt, receipt replay protection and foreground cancellation rules.
No implicit read, retry, fallback or optimistic cache update is introduced.
Provider acknowledgement remains `accepted`; possibly-sent failures stay `unknown`.
Pause/cancel does not establish safe-door, draining or physical rollback behavior.

Tests cover all five option values across all three actions, exact payloads,
unchanged legacy payloads, schema and native grant admission, mismatched authority,
required/invalid arguments despite forged public schemas, immutable previews and
cache/receipt preservation. Existing outcome and lifecycle tests remain required.
Only fictional accounts and local fixtures are used; actual product identity,
Flutter confirmation, mobile/off-LAN behavior and physical acceptance remain open.
