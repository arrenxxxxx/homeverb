# ADR-0108: Protected no-argument Mijia actions through Core

Status: Typed Core implementation with fictional integration tests; native/App adoption pending.
Date: October 30, 2026.

## Decision

Expose the six exact ADR-0107 action primitives through typed Core composition.
Use existing component/capability scopes with independent vendor action names:

| Exact model | Component | Capability | Action | SIID/AIID |
|---|---|---|---|---|
| abhome.light.wy0a01 | light-2 | vendor.xiaomi.mijia.light | toggle | 2/1 |
| abhome.light.wy0a01 | light-2 | vendor.xiaomi.mijia.light | brightness_up | 2/2 |
| abhome.light.wy0a01 | light-2 | vendor.xiaomi.mijia.light | brightness_down | 2/3 |
| abhome.light.wy0a01 | light-2 | vendor.xiaomi.mijia.light | toggle_light_mode | 2/4 |
| jdjz.fan.2072fc | fan-2 | vendor.xiaomi.mijia.fan | toggle | 2/1 |
| xiaomi.hood.cyp10 | light-6 | vendor.xiaomi.mijia.light | toggle | 6/1 |

Every action has exactly empty object arguments, cloud transport, low risk,
non_idempotent semantics and unavailable verification. These are bounded light/
fan operations; hood illumination never means ventilation. No implied step,
wrap behavior, companion setter, source-state read or effect is invented.
Action names do not create generic brand-wide admission: protected model, complete
device/service/action specification and SIID/AIID must match the pinned catalog.

The adapter defaults to no action controls. A separate trusted typed
with_action_controls policy opts in; Core independently requires its own exact
command grant. Fan/reference/outlet property policies cannot authorize these
actions. Local inspect_action_options validates protected device/connection and
listed membership without networking, saving mappings or enabling authority.
Descriptors add actions to existing capabilities only after explicit inventory
publication on an opted-in adapter. Protected property catalogs and payloads do
not gain action definitions; existing valid read mappings remain compatible.

Execution holds the account lease and Core device lock, independently re-derives
the action and argument contract, and checks the dedicated adapter policy before
using the protected selected protocol. Legacy uses only its legacy session;
modern requires its registered control transport and bound root kind/subject.
Missing membership/session/transport and forged public metadata refuse before I/O.
No renewal, retry, protocol fallback or physical verification is added.

Core's existing durable request-ID journal applies before sending. Accepted,
explicitly rejected, proven-not-sent and unknown outcomes retain their existing
meaning. A timeout/cancel/drop after possible send leaves an unknown receipt.
Replaying the same command returns the original receipt without another send,
including after store/Core reopen and explicit protocol adoption. Revoked Core
authority refuses execution before journal lookup and preserves the original
stored receipt; revocation never enables a second send.
A conflicting command with the same request ID refuses; clearing visible history
must not remove replay protection. A new request ID is a deliberate new operation,
not recovery of an uncertain previous action.

## Compatibility and delivery

Typed Rust integration only in this phase. Native API 1.39, ordinary startup
schemas, C exports/methods, CLI/MCP registration, SQLite 5 and Mijia protected
payload 3 stay unchanged. Production native adapters do not opt in and cannot
expose these actions through existing property-grant fields. Versioned native
startup grants, local action inspection and Flutter adoption remain dependent
work. Existing APKs remain unchanged. Real provider/phone/model acceptance and
remaining reference actions/events are incomplete.

## Required verification

Use fictional accounts, real Core and SQLite, and injected protocol transports.
Cover every exact action on legacy and modern routes, local inspection without
network/secret mutation, wrong/absent/cross-family grants, exact arguments, forged
public metadata, unlisted/changed protected identity/model/spec and missing
modern transport. Check receipt outcomes, sequential and concurrent duplicates,
request-ID conflicts, reopen/revocation/protocol-adoption replay and history
retirement guards. Pending cancel/deadline/drop must release account/device locks
and retain unknown receipts. Existing read/property/history/native behavior must
regress unchanged; no provider or appliance traffic is used.

## Executed evidence

The [phase record](../development/14-mijia-reference-integration.md#protected-core-no-argument-actions--october-30-2026)
records the executed Core/SQLite, protocol, workspace and contract checks. This
establishes fictional software behavior, not provider or physical-device support.
Native/App action adoption and the remaining reference scope remain incomplete.
