# ADR-0100: Protected modern Mijia device bindings and Core reads

Status: Typed protected bindings and Core reads implemented; Host/App publication pending.
Date: October 30, 2026.

## Decision

Use ADR-0099 directory groups to establish stable, system-secret-backed modern
Mijia bindings through explicit typed `refresh_modern_devices` and
`recover_modern_devices` operations. The adapter returns partial AccountDiscovery
for ordinary Core publication, including named unsupported candidates. Existing
QR/login and logical-directory APIs keep their behavior until their Host/App
publication integration is completed separately. No implicit directory request,
authentication, renewal or protocol fallback is added to device reads.

Mijia protected payload 3 retains independent same-subject legacy/modern sessions.
Each binding keeps its existing random local device, connection and identity IDs,
plus explicit read-protocol provenance. Legacy provenance is the default and is
omitted from unchanged version-1/2 encoding. Modern provenance requires protected
modern session material and kind/parent/logical-child evidence. The fixed modern
reference profile selects the reviewed origin and continuous RC4 codec; no saved
URL or arbitrary transport selector is allowed. Payload 3 is written only after
modern device adoption; version-1/2 bytes remain stable until that operation.
Older binaries cannot read version 3 and must not downgrade or reinterpret it.
SQLite schema 5, Meiju payload 3 and native API 1.36 are unchanged.

Registration/account leases cover directory reading, cross-account identity
checks, protected persistence and returned descriptors. A physical root is bound
once. Logical-child IDs remain protected evidence and never get their own card.
Unknown/missing relationships stay unresolved. Reject cross-account root/child
collisions, prior child-to-root promotion, model contradictions and changed known
parent/kind evidence; never auto-merge existing cards or transfer authority.
Historical missing roots/children remain retained because membership is partial.
A confirmed root reuses its local IDs and Core preserves preferences/rooms.

Explicit modern refresh selects modern reads for returned roots, including an
exact existing legacy root; explicit legacy refresh can select legacy reads only
after that protocol returns the root itself. Retain protected modern relationship
evidence across either operation. Reads use the selected protected protocol only,
never fall back after rejection/timeout. This phase does not add modern controls
or history: modern-selected descriptors have no actions, local control/history
inspection returns no options, and consequential adapter entry points independently
refuse them even with stale descriptors and trusted grants. Existing legacy
bindings retain their reviewed controls/history after explicit legacy selection.

Modern mappings use only exact pinned reference scalar definitions. Re-derive
protected mapping evidence before admission; do not inherit arbitrary legacy
public-spec mappings or writable semantics. For unknown models preserve named
unsupported cards. Reads revalidate device/model/component/connection membership,
then use only the current modern session and exact root DID/property pairs.
Logical child property maps are never fused or remapped. Observations retain
cloud source, local receipt time, absent device time and stale=true; Core performs
normal scalar validation before caching. Online hints never become observations.

Persist bindings before publication. Failed/uncertain secret writes publish no
new descriptors and leave vault pending state intact. Explicit recovery validates
the exact stored session with a bounded directory read and confirms the same bytes;
unchanged metadata is not rewritten and authentication is not repeated. Partial
empty inventories keep prior binding/card identity and never set complete time.
Cancellation/deadline covers persistence and publication preparation, and dropping
a future releases account leases. All source fixtures remain fictional.

## Required evidence

Version-1/2 byte stability and version-3 round trips/tamper refusal; restart Core
reads; exact modern request correlation and no legacy requests; stable IDs and
preferences across explicit protocol selection; split-child de-duplication and
cross-account/role/model/parent conflicts; unknown/missing entries; unchanged save
avoidance; uncertain save and exact-byte recovery without login/replay; cancellation
and drop lease release; modern control/history refusal with stale descriptors;
protected markers absent from public JSON and plaintext SQLite; existing legacy,
modern account, Host and native regressions. Host/App card publication and actual
provider/device acceptance remain tracked separately until implemented/tested.

## Executed evidence

The workspace run excluding the pre-existing unrelated Meiju/macOS LAN-alias
failures passed 579 tests with one ignored, including all 12 new modern device
integration tests. These exercise actual SQLite/Core with fictional secret storage
and encrypted responses, all 264 pinned scalar mappings, stable organization,
protocol selection, uncertain persistence and refusal paths. Strict workspace
all-target Clippy passed after an equivalent range-pattern cleanup. The rebuilt
native library passed 36 ordinary methods/57 examples plus all dedicated export
checks, including 62 modern-account C refusal/buffer/lifecycle cases. Documentation,
repository hygiene and reference-projection checks passed. No authenticated provider,
physical phone/appliance, new APK, remote CI or publication was used. The Host/App
login-to-card path remains pending and is not established by these typed Core tests.
