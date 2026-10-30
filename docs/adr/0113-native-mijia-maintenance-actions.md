# ADR-0113: Native exact Mijia maintenance actions

Status: Native and Flutter implementation; fictional verification recorded below.
Date: October 30, 2026.

## Decision

API 1.42 admits ADR-0112's four exact no-input/no-output actions through the
existing startup-only mijia_action_grants. Canonical version-four local UUIDs,
closed component/capability/action tuples, duplicate refusal, the shared 64-entry
limit and Mijia/combined-only modes remain mandatory. Validate before creating a
profile. Startup performs no credential access, authentication or device request.

The exact tuples are camera_control-2/vendor.xiaomi.mijia.camera_control with
ptz_calibrate or restart_device, memory_card_management-4/
vendor.xiaomi.mijia.memory_card_management with pop_up, and hood_smart_dry_wash-10/
vendor.xiaomi.mijia.hood_smart_dry_wash with stop_dry_cleaning. Protected model,
complete specification/root identity and selected-protocol evidence still decide
whether the device supports each action. No model or wire selector enters startup.
Formatting, streams/uploads, self-checks, binding changes and cleaning start are
not admitted by these tuples.

All four remain elevated and non_idempotent with unavailable verification. Both
Core and the adapter require independent exact action-family grants, then each
execution requires an immutable trusted preview and actual user consent through
the existing C approval exports. Ordinary plan/execute remain denied. Correlated
acceptance is not safe card removal, restored monitoring, completed calibration,
physical stop or cooling. Unknown outcomes never permit retry, another preview,
protocol fallback, credential renewal or companion writes.

## Separate inspection and compatibility

The new optional include_maintenance_actions defaults to false. It appends only
these reviewed elevated maintenance choices, without networking, descriptor or
credential changes, permission grants or user consent. Existing low-risk and
power-toggle flags keep their original independent scopes. All flags can be
combined without duplicate options, using protected leases and final descriptor/
foreground checks. Granted reopen followed by explicit account refresh publishes
the admitted descriptors; opening a device never prepares a command.

Negotiate API 1.42 before supplying the new flag or startup tuples. Older libraries
reject them; do not silently downgrade explicit maintenance authority. The Rust
InspectControlOptions constructor gains a defaulted Boolean. Response inspection
adds the exact camera, storage and hood components; ordinary method/C export
counts, SQLite 5, protected payload 3 and command/result shapes are unchanged.
The d505ccb APK remains API 1.41. Flutter adoption and package delivery follow
separately; source implementation does not prove provider/phone/model acceptance.

## Verification

Check exact startup/schema/C ABI admission in both allowed modes, foreign tuples,
forbidden fields/actions, malformed IDs, excessive/duplicate lists and non-Boolean
flags. Old inspection must remain unchanged. Fictional Host journeys cover local
inspection before grants, independent granted reopen, explicit refresh, ordinary
denial, immutable preview, actual confirmation, single consumption, dismissal and
background invalidation. Lost/late replies and pending-send cancellation must
preserve unknown receipts across reopen without replay. Existing legacy/modern
Core tests cover protected scope and exact wire identity for all four actions.
No real account, appliance operation, installation, push or publication is needed.

Executed verification: Rust 1.96 offline/locked workspace tests passed 690 tests
with one ignored, excluding homeverb-midea's existing macOS LAN-alias failures.
Strict all-target workspace Clippy and the FFI build passed. The focused Host
account/grant suite passed 68 tests, including all four trusted maintenance paths,
independent inspection scopes and unknown-receipt recovery without replay. The
actual C ABI passed 36 methods and 84 live examples, including exact startup
admission and the new Boolean inspection flag; modern-account exports retained
76 boundary cases. Contract validation passed 25 schemas, 82 positive fixtures
and 1,720 negative fixtures. Documentation, source hygiene, reference projection,
both projection unit tests and diff whitespace checks passed. These are synthetic
software checks; no provider, phone, model or Flutter acceptance is established.

## Flutter adoption

The foreground controller explicitly opts in only with API 1.42 and the trusted
approval bridge. The native client negotiates read-only before supplying any
maintenance grant. Existing low-risk and power-toggle grants retain their
independent versions and inspection flags. The generic descriptor controls use
exact capability/action localization and the existing immutable review; no model
branch, automatic preview, inferred state or companion write is added. Dismissal,
background and duplicate confirmation preserve the existing lifecycle and receipt
rules. A matching Android package and provider/phone/model acceptance are separate.

Executed Flutter verification: all 155 tests across the 17 affected maintenance,
power/action, outlet/settings, permission, control, component, localization,
current-contract, native, approval and foreground-refresh files passed using
Flutter 3.47.7. The maintenance suite contains 15 tests, including all four actions
in both languages at 320 logical pixels and double text size, independent flags,
actual API 1.41 refusal and current native startup in both allowed modes.
Flutter analysis reported no issues; seven shared presentation tests and both
reference projection tests passed. These focused regressions do not resolve the
eight previously recorded unrelated macOS visual/legacy-click full-suite failures.
No live account, appliance write, phone acceptance or remote CI is claimed.
