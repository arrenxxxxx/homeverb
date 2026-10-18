# ADR-0071: Prepare reviewed dishwasher commands in the foreground App

Status: accepted for implementation following the owner's October 18 request to
connect real refrigerator/dishwasher operations. Physical writes require separate
owner authorization; this decision authorizes software implementation and tests.

## Decision

Extend ADR-0065's foreground host preparation to the four already implemented
`vendor.midea.dishwasher` actions: start, pause, resume and cancel cycle. Only
native protected-inventory inspection may supply the exact reviewed action and
parameter domain. Require a host with trusted native approval support. Generate
bounded exact program/additional-option combinations through the existing
`meiju_dishwasher_grants` and `meiju_dishwasher_cycle_grants` startup fields.
Keep each native field's 64-entry bound; never substitute a wildcard grant.

Every operation remains elevated and non-idempotent. Opening a device prepares
authority but sends no command. The user must select every required argument,
review the immutable native preview and confirm through the separate trusted C
function. Cached observations do not select arguments. Ordinary execute, JSON,
CLI and MCP cannot bypass consent. Unknown/high-risk/coupled future schemas are
not admitted by this App mapping. Native profile and membership checks remain
independent of public descriptors and host grants.

Use the existing selected-device preparation, selected-account descriptor refresh,
session replacement and lifecycle cancellation machinery. Native policy inspection
still determines which buttons are usable. Failed setup must not publish apparent
authority; replacement sessions that arrive after cancellation are closed.

## Compatibility and evidence

This changes trusted App preparation policy, not native API 1.23, Core execution,
SQLite, protected payloads, default CLI/MCP policy or real-model compatibility.
The cloud request remains one send through `/mjl/v1/device/lua/control` with the
existing profile-specific payload, accepted/unknown receipts and replay guards.
Pause/cancel establishes neither safe-door state nor physical rollback.

Verify exact combinations, malformed/oversized/foreign definitions, immutable
arguments, current-session display scope, native startup admission, actual trusted
confirmation and navigation/background cancellation using fictional data. Public
upstream source is protocol evidence, not a physical appliance test. Refrigerator
admission remains a separate model/encoder investigation.
