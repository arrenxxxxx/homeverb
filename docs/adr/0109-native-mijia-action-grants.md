# ADR-0109: Native exact Mijia action grants

Status: Implemented native contract and foreground Flutter adoption.
Date: October 30, 2026.

## Native contract

API 1.40 adds trusted startup mijia_action_grants for the six exact no-argument
ADR-0108 tuples. Each entry contains only device_id, component_id, capability and
action. Require a canonical version-four local device UUID, an exact closed tuple,
no duplicates and at most 64 entries. No arguments, wire IDs, values, wildcard,
confirmation or model selector can occur in this startup field. It is accepted
only in xiaomi.mijia and homeverb.platforms modes. Existing fan/reference/outlet
grants never imply action authority. Startup validates grants before creating a
profile and performs no secret access, authentication, inventory or appliance I/O.

Both independent Core and adapter policies receive the exact empty-object command
grant. Default native composition and old Rust opening helpers remain action-free.
The protected adapter still re-derives model/specification/root/protocol evidence
and refuses unmatched devices at execution; syntactic startup acceptance proves
neither device existence nor support. Existing Core request-ID receipts preserve
accepted/failed/unknown and non-idempotent replay protection across lifecycle.

Existing inspect_control_options adds optional include_no_argument_actions=false.
Only an explicit opt-in appends the separately inspected exact action options;
older callers keep their previous result. Inspection is local, retains protected
account leases and final descriptor/lifecycle checks, and changes no authority,
credentials or saved capabilities. Its source remains saved_inventory and its
authorization remains not_checked. Meiju inspection is unchanged. No new ordinary
method or C export is added. Existing response capability/definition shapes fit
the empty parameters, non_idempotent and unavailable-verification metadata.

## Compatibility and delivery

Negotiate API 1.40 before supplying the new startup field or inspection flag.
Older native implementations reject them; do not reopen with widened authority
or silently drop requested actions after a failed version check. Existing property
startup/inspection behavior and non-action Rust helper signatures remain intact.
The public typed Request::InspectControlOptions gains a field, requiring source
updates for Rust enum constructors. Native JSON field omission remains compatible.
SQLite 5 and protected payload 3 are unchanged. The 1.39 APK is not upgraded by
this source change. Flutter permission preparation and generic action forms now negotiate 1.40
and preserve this boundary; real account/phone/model acceptance remains pending.

## Verification

Check all six startup tuples against typed policies, schema fixtures and the
actual C ABI, in both allowed modes. Refuse duplicates, malformed UUIDs, excessive
lists, foreign components/capabilities/actions, setter substitution, extra fields,
wrong adapters and non-Boolean inspection flags before side effects.

Fictional Host journeys must inspect default and opted-in choices before grants,
reopen with exact independent grants, refresh descriptors, plan and execute using
protected protocol bindings. Check no implicit networking from inspection, no
cross-family authority, cancellation/background refusal and persistent unknown
receipts without replay. Rebuild the FFI and regenerate live contract examples.
No real provider account, appliance, install, push or publication is authorized.

Executed software checks: workspace tests excluding the known unrelated
Meiju/macOS LAN-alias suite passed 666 tests with one ignored. Strict all-target
Clippy, FFI rebuild and the actual C boundary passed: 36 ordinary methods,
68 live examples, 76 modern-account cases and existing protected/trusted bridge
guards. The 58 scoped Host/account tests cover action admission, opt-in inspection,
explicit refresh/execute and uncertain lifecycle recovery without replay.
Documentation, source hygiene, both reference projection tests and 25 schemas
with 76 positive/1451 negative fixtures passed. All data is fictional; no Flutter
run, new package or provider/device acceptance is claimed by this native phase.


## Foreground App no-argument actions -- October 30, 2026

Flutter now negotiates API 1.40 and explicitly requests the six ADR-0109 actions
when preparing a selected device. Exact empty-object definitions become separate
`mijia_action_grants`; property permissions never imply action authority. Trusted
startup probes the version before reopening with grants. Older native libraries
retain property-only inspection and refuse explicitly requested action grants.

The existing descriptor-driven detail buttons and forms display localized action
labels and non-idempotent warnings. Opening/selecting an action sends nothing;
Apply plans and executes once with empty arguments and the exact component.
Pending submission disables duplicate clicks. Unknown receipts remain unknown
without automatic retries or optimistic state changes. Backgrounded inspection
cannot reopen or refresh a device. No new method, storage format or UI protocol
branch is added. Remaining actions/events and provider/phone/model acceptance
are incomplete; the existing 503ef1f APK predates this App integration.

Executed App verification: the 14-file affected Flutter suite passed all 133 tests,
including 11 action cases, and Flutter analysis found no issues. Actual old-ABI
refusal and current Mijia/combined opening use fictional temporary profiles.
Both reference projection tests and all seven shared presentation tests passed.
The known eight full-suite macOS font/pixel/legacy-click failures are not claimed
fixed. No provider account, appliance write, installation or publication occurred.
