# ADR-0112: Exact Mijia maintenance and stop actions

Status: Typed protocol/Core implementation verified; native/App adoption follows
[ADR-0113](0113-native-mijia-maintenance-actions.md).
Date: October 30, 2026.

## Evidence and scope

The digest-pinned October 22 reference archive supplies these exact no-input,
no-output MIoT actions. ADR-0107's community action envelope and existing selected
legacy/modern tracked transports apply without new parameters or inferred writes.

| Exact model | Component | Capability suffix | Action | SIID/AIID |
|---|---|---|---|---|
| xiaomi.camera.083ac1 | camera_control-2 | camera_control | ptz_calibrate | 2/1 |
| xiaomi.camera.083ac1 | camera_control-2 | camera_control | restart_device | 2/2 |
| xiaomi.camera.083ac1 | memory_card_management-4 | memory_card_management | pop_up | 4/2 |
| xiaomi.hood.cyp10 | hood_smart_dry_wash-10 | hood_smart_dry_wash | stop_dry_cleaning | 10/1 |

All capability identifiers use vendor.xiaomi.mijia. Preserve the complete device,
service and action URNs, exact protected root identity and the selected protocol.
Every request has exactly empty-object arguments and sends one params object with
in=[]. No recording mode, power, stream, schedule, storage format or companion
setting is sent. Source definitions and synthetic results are not provider or
physical-model acceptance.

## Risk and outcomes

These operations are elevated: calibration can move the camera, restart can
interrupt monitoring, card ejection can interrupt recording, and stopping hood
cleaning can interrupt an appliance operation. They require independent exact
Core/action-family grants and actual trusted consent on every attempt. Protected
risk must check Core's private approval marker even if public metadata is forged
low. A nonempty or altered argument object remains denied after approval.

All four are conservatively non_idempotent with unavailable verification. A
correlated acknowledgement establishes accepted only. It does not establish safe
card removal, restored camera monitoring, completed calibration, a stopped motor,
cooling, physical rollback or readiness for another operation. Do not infer
preconditions from stale observations. Provider rejection is distinct from unknown
execution; ambiguous errors, timeout, cancellation or drop after send never permit
automatic replay, credential renewal, another protocol or a companion action.
Unknown receipts and request IDs survive reopen and history retirement.

Formatting, media streaming/upload, self-check item meanings, remote-control
removal, resets, binding changes and hood cleaning start remain separate work.
The cleaning start has a declared output property and cannot use the empty-output
contract. No destructive/high-risk action is admitted by this decision.

## Compatibility and delivery

This phase implements typed protocol/Core support only. API 1.41 startup remains
limited to the existing eleven actions. Its no-argument flag selects the six
low-risk actions; its power flag selects only the five switch toggles, not every
elevated action. New maintenance action inspection/startup/App admission requires
a later native API version and the existing immutable preview/confirm bridge.
SQLite 5, protected payload 3, command/result shapes and scalar controls remain
unchanged. The d505ccb APK stays API 1.41 and does not acquire these actions.

## Acceptance

Project only the four exact definitions and leave property catalogs unchanged.
Exercise all fifteen no-argument protocol profiles on both credential/codec
selections. Core tests cover exact protected wire identity, ordinary denial,
independent grants, immutable approval, forged public risk/arguments, lifecycle
invalidation, sent uncertainty, released locks, durable receipts and no replay.
Retain native API 1.41 omission/rejection of these actions. Add explicit positive
capability/command fixtures and closed-argument counterexamples. No real account,
appliance operation, installation, push or publication is authorized by tests.

Executed verification: Rust 1.96 offline/locked workspace tests passed 686 tests
with one ignored, excluding homeverb-midea for its known macOS LAN-alias
limitation. Strict all-target Clippy, rustfmt, FFI rebuild and diff checks passed.
The rebuilt C ABI passed 36 methods, 76 live examples, 76 modern-account boundary
cases and existing trusted/protected guards; all four maintenance startup tuples
were refused before creating a profile. Contract validation passed 25 schemas,
81 positive and 1,558 negative fixtures. Both reference projection tests, source
hygiene and documentation checks passed. No Flutter run, APK rebuild or real
provider/device operation was performed in this typed-only phase.

The coverage audit now explicitly counts 225 source writable property definitions:
33 are implemented exact controls and 192 still require separate semantic/risk/
runtime admission. This is additional unresolved scope alongside the remaining
actions/events; a source write flag never establishes implemented support.
