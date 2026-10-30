# ADR-0117: Native event occurrence history

Status: Implemented and synthetic-tested; Flutter and package adoption pending.
Date: October 30, 2026.

## Decision

Native API 1.44 adds inspect_event_history_options and read_event_history to the
ordinary foreground query boundary, using ADR-0116 event-history types unchanged.
Both methods require normal Session admission, an exact local device/connection,
a bounded Operation and the existing read access. Inspection is local; reads
select an admitted event and explicit whole-second UTC window. Results contain
reported occurrence times only, preserve equal timestamps and keep completeness
unproven. No provider identity, raw key, event payload or cursor becomes public.

The new trusted startup Boolean mijia_event_history defaults to false. Only
xiaomi.mijia and homeverb.platforms may enable it; invalid modes/types fail before
profile creation. It enables the reviewed adapter history family, not control
grants, subscriptions or networking. An explicit account connect/refresh publishes
exact event capabilities and event-only components. Existing startup functions
preserve false; new typed MijiaSessionOptions carry the setting together with
existing independent control grants. Merely opening a session neither loads
credentials nor refreshes accounts. Ordinary calls cannot change the startup flag.

Default Mock and Meiju adapters expose empty options and unsupported reads.
Disabled Mijia sessions retain ordinary scalar reads on complete enriched layouts
but cannot read events. Old scalar-only descriptors need explicit refresh before
event options appear. The protected adapter still independently checks exact
account/model/spec/root/membership and selected protocol on every event request.

Background, close, cancellation and deadline stop work and suppress late output;
no subscription, periodic read, renewal, fallback, cache mutation or receipt write
is introduced. No user-supplied principal or write confirmation is accepted.
These methods cannot verify appliance actions or establish current safety.

## Compatibility

The ordinary method count becomes 38. C/JNI exports and signatures, JSON envelope
1.0, SQLite 5 and protected payload 3 remain unchanged. Older native libraries
reject the new methods and startup field; clients must negotiate API 1.44 and
both method names before enabling event history or reopening with the flag.
Existing native clients retain disabled startup. Numeric history and existing
write grants keep their previous semantics. Flutter/CLI/MCP command adoption and
a matching APK are subsequent phases; 85fefd8 remains API 1.43.

## Verification

Check both startup modes, omitted/false compatibility and wrong mode/type refusal
before filesystem creation. Exercise successful protected modern event history
through actual Session dispatch and closed response schemas with fictional QR,
inventory and event peers; keep protocol coverage for both transports in Core.
Test zero networking at inspection, sensitive-field omission, exact targets,
no state/credential/receipt mutation, failure without retry, cancel/background/
close/late reply and unknown startup/request fields. Rebuild the C library and
exercise both new methods, closed argument/response shapes and startup guards
through the actual ABI. These checks do not establish real provider/phone/model
acceptance, event argument interpretation, pagination or live subscriptions.

## Executed evidence

The Rust 1.96 offline/locked workspace suite passed 733 tests with one opt-in test
ignored, excluding the known macOS Meiju LAN-alias package and using four test
threads. Generic and protected Host event-history tests passed, including closed
schema checks, default/opt-in composition and cancel/background/close/drop/late
reply suppression. Strict workspace Clippy, formatting and native FFI build
passed. The actual C ABI verifier passed 38 methods, 86 fictional live examples,
startup mode/type guards and 76 modern-account boundary cases. Contracts passed
26 schemas, 88 positives and 1,966 negatives; reference projection checks passed
all three tests. See the [phase record](../development/14-mijia-reference-integration.md)
for limits. No Flutter/package change, real account/provider/device operation,
phone installation, push or publication is part of this phase.
