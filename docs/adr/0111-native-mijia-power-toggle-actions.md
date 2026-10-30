# ADR-0111: Native exact Mijia power-toggle actions

Status: Native implementation verified; Flutter adoption pending.
Date: October 30, 2026.

## Decision

API 1.41 extends startup-only mijia_action_grants with the four exact switch
component tuples for ADR-0110's five model/service actions: switch-2, switch-3,
switch-11 and switch-12, vendor.xiaomi.mijia.switch, toggle. Existing canonical
local UUID, closed fields, duplicate refusal, 64-entry bound and Mijia/combined
mode constraints remain. Both Core and adapter receive independent empty-object
grants. Startup does not inspect credentials or contact appliances. Exact
protected model/specification/root/protocol evidence remains mandatory at send.

Every toggle remains elevated and non-idempotent. Grants never imply consent.
Ordinary plan/execute refuse these commands; only an immutable preview followed
by actual trusted UI consent through the existing three C approval functions can
send. The adapter independently enforces Core's private approval marker. Unknown
receipts cannot be replayed or receive another approval, including after reopening
or history retirement. No inferred state, companion write, retry, fallback,
credential renewal or verification read is added.

## Inspection and compatibility

The separate optional include_power_toggle_actions flag defaults to false.
Only explicit true appends these elevated actions. Existing omitted/false
include_no_argument_actions behavior remains; that flag still selects only the
original six low-risk actions. Both flags can be selected without duplicates.
Inspection is local, read-only and independent of grants, with protected leases,
final descriptor validation and foreground cancellation. It neither publishes
new descriptors nor grants permission. Explicit account refresh after granted
reopen publishes admitted actions using existing protected rules.

Negotiate API 1.41 before supplying the new tuples or inspection flag. API 1.40
rejects them; no silent downgrade is permitted for requested power actions.
The Rust InspectControlOptions constructor gains a defaulted Boolean field.
There are still 36 ordinary methods and 30 portable C exports plus Android's
initializer. SQLite 5, protected payload 3, result shapes and scalar grants are
unchanged. The ab34f21 APK remains API 1.40; new native source is not App delivery.

## Acceptance

Check all exact startup tuples through policy/schema and rebuilt C ABI, both
allowed modes, with invalid component/capability/action/UUID/arguments/confirmation
fields and non-Boolean flags refused before side effects. Preserve old inspection
results. Fictional Host tests must inspect before grants, reopen/refresh, deny
ordinary execution, prepare without sending, invalidate on background/dismissal,
confirm once, and preserve unknown receipts without replay after reopen. Exercise
all five model/service mappings. Existing legacy/modern adapter suites cover
protected risk, membership and wire correlation independently.

Flutter must later negotiate 1.41, require the trusted approval bridge, and reuse
the generic elevated action form. Real provider/phone/model acceptance remains
pending. No real account, appliance request, installation, push or publication
is part of this phase.

Executed verification: Rust 1.96 offline/locked workspace tests passed 678 tests
with one ignored, excluding homeverb-midea for its known macOS LAN-alias
limitation. Strict all-target Clippy, rustfmt and FFI rebuild passed. The actual
C ABI verifier passed 36 methods, 76 live examples, 76 modern-account boundary
cases and the existing trusted/protected bridges. The 63 scoped Host/account
tests passed, including all five power-toggle model/service journeys and unknown
reopen without replay. Contract checks passed 25 schemas, 79 positive and 1,547
negative fixtures; reference projection and both projection unit tests passed.
Documentation, repository hygiene, diff and commit checks passed. These are
fictional software checks, not Flutter, package, provider or physical acceptance.
