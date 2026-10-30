# ADR-0110: Exact elevated Mijia power-toggle actions

Status: Implemented typed protocol/Core contract; native/App adoption pending.
Date: October 30, 2026.

## Evidence and exact scope

The pinned October 22 archive supplies five additional empty-input/empty-output
MIoT toggle definitions. ADR-0107's reviewed community action envelope applies
without inventing new parameters or effects. Preserve complete eight-part URNs.
No source definition or synthetic acknowledgement proves physical acceptance.

| Exact model | Component | Capability | SIID/AIID |
|---|---|---|---|
| cuco.plug.v3 | switch-2 | vendor.xiaomi.mijia.switch | 2/1 |
| yutai.plug.jdls1 | switch-2 | vendor.xiaomi.mijia.switch | 2/1 |
| opple.switch.0010d5 | switch-3 | vendor.xiaomi.mijia.switch | 3/1 |
| opple.switch.0010d6 | switch-11 | vendor.xiaomi.mijia.switch | 11/1 |
| opple.switch.0010d6 | switch-12 | vendor.xiaomi.mijia.switch | 12/1 |

Each action is named toggle, has exactly empty object arguments, cloud transport,
elevated risk, non_idempotent semantics and unavailable verification. It can
change power supplied to an unknown load. Wall-switch mode can affect the physical
result; do not alter mode, guess the next state, add a companion setter or infer
that a cloud acknowledgement means a connected load changed. Logical children
with protected parents remain denied; exact physical root evidence is required.

## Independent approval and authority

Retain both the independent exact action-family policy and Core command grants.
Existing outlet/fan/reference property grants never authorize a toggle. Every
attempt additionally requires Core's immutable trusted user approval. The adapter
must check the private approval marker against protected re-derived action risk,
even if public descriptor metadata is changed to low risk. Scope authorization
can ignore risk only after that private check; high-risk actions remain denied.

Keep selected legacy/modern protocol, complete root identity, exact model/spec/
service/action membership, account/device leases and one tracked send. Accepted
is not verified. Ambiguous replies, late cancellation, timeout or drop after a
possible send retain unknown receipts. No automatic retry, credential renewal,
route fallback, state read or implicit power/mode write is added. Reopen, revoked
authority and history retirement must not permit replay.

## Compatibility and delivery

The first phase adds typed protocol/Core support. Native API 1.40 startup grants
and opted-in option inspection remain restricted to the existing six low-risk
actions. No C/JSON/CLI/MCP power-toggle grant is added in this phase. Native/App
adoption must negotiate a later version, expose these elevated options separately
from old inspection behavior and use the existing trusted confirmation bridge.
SQLite 5, protected property mappings/payload 3 and existing scalar controls are
unchanged. The ab34f21 APK does not acquire these actions through documentation.
Real account/phone/model acceptance remains pending; no real write is authorized.

## Required verification

Project only the five exact public definitions, with unchanged property catalogs.
Run all legacy/modern protocol action correlation and uncertain-code cases across
all eleven pinned actions. Core fixtures must exercise every new model/service on
both protocols: independent grants, ordinary denial, trusted confirmation, no
companion traffic, wrong arguments/scope and root/subject refusal. Tampering with
public risk must still fail before I/O. Test preview invalidation, after-send
cancellation/drop, persistent receipt recovery and retirement without replay.
Retain native API 1.40 refusal and old six-action opt-in behavior. All fixtures
use fictional accounts and local transports; no physical acceptance is inferred.

## Executed evidence

The offline Rust workspace run passed 672 tests with one ignored, excluding
homeverb-midea for its known macOS LAN-alias limitation. The final seven-case
power-toggle suite passed after two additional approval-boundary tests. Strict
Clippy and rebuilt C ABI checks passed, retaining API 1.40 refusal of the new
startup tuples. Contract validation passed 25 schemas, 78 positive and 1,463
negative fixtures. See the [phase record](../development/14-mijia-reference-integration.md#typed-elevated-power-toggle-actions--october-30-2026)
for scope and limitations. No Flutter build or real-provider operation occurred.
