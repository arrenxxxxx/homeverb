# Implementation status and next tasks

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

The subsequent clean-source ab34f21 APK includes API 1.40 and these App actions.
Both ABIs, 31 exports each, development signature, 16 KiB alignment and exact
current-AAR match passed; no runtime/device acceptance is inferred. See the
[package record](../apps/mobile/verification.md#no-argument-actions-api-140-android-package--october-30-2026).

## Native exact no-argument actions -- October 30, 2026

[ADR-0109](adr/0109-native-mijia-action-grants.md) exposes the six reviewed
ADR-0108 actions through API 1.40. Independent trusted `mijia_action_grants`
contain only canonical local device IDs and exact component/capability/action
tuples. Invalid grants refuse before profile creation. Both Core and the adapter
receive the exact empty-argument policy; existing property grants imply no action
authority. Default composition remains action-free.

`inspect_control_options` retains its existing result unless the caller explicitly
sets `include_no_argument_actions:true`. Inspection performs no networking or
permission changes and returns saved-inventory evidence with authorization not
checked. Explicit granted reopen and account refresh publish actions on the
protected selected protocol. Non-idempotent receipts preserve uncertain outcomes
without retry, including after cancellation/background and session reopen.

Negotiate 1.40 before supplying the new field or inspection flag. Existing Rust
opening helpers remain compatible; typed Request constructors need the new field.
SQLite 5, protected payload 3, 36 ordinary methods and C exports are unchanged.
Flutter adoption, an updated APK, the remaining actions/events and real account/
phone/model acceptance remain pending. The 503ef1f APK remains API 1.39. No real
account, appliance, installation, remote push or publication is used.

Executed software checks: workspace tests excluding the known unrelated
Meiju/macOS LAN-alias suite passed 666 tests with one ignored. Strict all-target
Clippy, FFI rebuild and the actual C boundary passed: 36 ordinary methods,
68 live examples, 76 modern-account cases and existing protected/trusted bridge
guards. The 58 scoped Host/account tests cover action admission, opt-in inspection,
explicit refresh/execute and uncertain lifecycle recovery without replay.
Documentation, source hygiene, both reference projection tests and 25 schemas
with 76 positive/1451 negative fixtures passed. All data is fictional; no Flutter
run, new package or provider/device acceptance is claimed by this native phase.

## Protected Core no-argument actions -- October 30, 2026

[ADR-0108](adr/0108-mijia-protected-no-argument-actions.md) connects the six exact
ADR-0107 actions to typed Core on legacy and modern protected bindings. A separate
adapter action policy and an independent Core grant are both required; existing
fan/reference/outlet property authority cannot enable them. Local action-option
inspection performs no networking, credential rewrite or permission change.
Explicit opted-in inventory publication adds non-idempotent, empty-argument
vendor actions to existing component/capability scopes without changing properties.

Protected source model/specification/service/action identities and the selected
protocol are rechecked before one send. Public metadata cannot expand arguments
or choose another action. A parent-bearing modern entry is not admitted as an
action root. Missing membership, session, transport or exact policy refuses before
I/O. Accepted is never verified; ambiguous execution replies remain unknown.

Core's existing SQLite journal preserves receipts across reopen and explicit
protocol adoption. Same-ID retries never send again; same-ID command conflicts
refuse, and history retirement preserves replay protection. Revoked Core grants refuse before
journal lookup while retaining the original stored receipt. Cancellation/deadline/
drop release account/device locks and keep unknown outcomes without replay.

This is typed Rust Core integration. Native API 1.39, ordinary schemas, C exports/
methods, SQLite 5 and protected payload 3 remain unchanged. Production native,
CLI and MCP composition does not opt into these actions. Versioned native startup
grants/inspection and Flutter controls are still pending, as are remaining actions,
events and real provider/phone/model acceptance. The 503ef1f APK is unchanged.
No provider, appliance, installation, push or publication is used.

Executed software verification: workspace tests excluding the known unrelated
Meiju/macOS LAN-alias suite passed 661 tests with one ignored, including 252 Mijia
unit tests and 14 compile-fail tests. Strict all-target Clippy passed after an
equivalent Boolean simplification, and the final eight action integration tests
passed again. Formatting, all three generated projections, both projection tests,
documentation, source hygiene and 25 schemas with 75 positive/1328 negative
fixtures passed. No Flutter run, new APK or real-account/device acceptance is
claimed by this typed Core phase; known full Flutter failures remain unchanged.

The rebuilt FFI and actual native C boundary also passed: 36 ordinary methods,
60 live examples and 76 modern-account cases, plus existing protected/trusted
bridge guards. No new native method, startup field or Flutter action is exposed
by this phase.


## Typed no-argument actions -- October 30, 2026

[ADR-0107](adr/0107-bounded-no-argument-actions.md) adds six exact typed Rust
MIoT action primitives on legacy and modern sessions: four light actions, one
fan toggle and one hood illumination toggle. Each requires its pinned model,
complete specification and exact SIID/AIID; all inputs and outputs are empty.
The separate action catalog leaves saved property mappings unchanged. These are
non-idempotent operations and are not yet exposed through Core/native/App grants.

Each attempt uses one tracked control send on the selected protocol. A correlated
code-zero result is accepted, never physically verified. Only reviewed
pre-execution argument/identity/missing-action codes prove rejection. Pending,
execution-error, timeout, malformed, foreign and late replies remain uncertain;
there is no retry, renewal or alternate-protocol fallback. Logical children and
foreign modern subjects refuse before I/O. Cancellation and drop release work.

Native API 1.39, C exports/methods, ordinary schemas, SQLite 5 and protected
payload 3 are unchanged. Rust exhaustive CloudEndpoint consumers must handle
Action; ModernControlEndpoint supplies two closed routes. Independent Core action
bindings/grants, persistent non-idempotent receipts, versioned App integration,
remaining actions/events and real provider/phone/device acceptance remain open.
The 503ef1f APK predates these library primitives. No account, appliance,
installation, push or publication is used.

Executed software checks: the workspace excluding the known unrelated Meiju/macOS
LAN-alias suite passed 653 tests with one ignored, including 244 Mijia unit tests
and 14 compile-fail tests. Strict all-target workspace Clippy and formatting
passed. The three projected files, both projection tests, documentation, source
hygiene and 25 schemas with 73 positive/1320 negative fixtures passed. These tests
use fictional data and local TLS only. No Flutter test or new Android package is
claimed for this protocol-only phase; the existing eight full Flutter visual/
legacy-click failures remain outside this change.

Rebuilt native FFI and the actual C ABI check also passed: 36 ordinary methods,
60 live examples and 76 modern-account boundary cases. Existing trusted bridges
retain their guards; no ordinary action grant or method was added.


## Exact hood controls Android delivery -- October 30, 2026

The clean 503ef1f source has an archive-checked API 1.39 Android package for
arm64-v8a and x86_64. It includes all 33 reviewed scalar controls, including the
four exact hood settings, and the 161 numeric history fields. The report records
current-AAR matching, 31 exports per ABI, development signing, disabled backup and
16 KiB alignment. See the [package evidence](../apps/mobile/verification.md#exact-hood-controls-api-139-android-package--october-30-2026).
Installation and provider/phone/model acceptance remain pending. Packaging changes
no authority, saved format or functional scope and performs no remote publication.

## Exact hood manual controls -- October 30, 2026

ADR-0106 adds four exact xiaomi.hood.cyp10 properties through legacy and modern
Core paths: hood on-state, fan levels 1/2/3/4, display content codes 0/1 and the
independent automatic screen-off Boolean. The 33 reviewed scalar controls still
cover five exact models. Existing vendor IDs and raw codes remain stable; no
percentage conversion, companion power-on, inferred timer or schedule is added.
Core and the protected adapter independently require exact reference-setting
grants; fan/outlet authority does not transfer. Uncertain writes are never replayed.

Native API 1.39 adds four closed startup tuples and three inspection component
scopes. Flutter probes the native version before any granted reopen and preserves
all ten target values. Old illumination-only hood mappings remain valid; explicit
refresh upgrades their domains while keeping identity and organization. Invalid
partial domain mixtures refuse. Methods, C exports, SQLite 5 and protected payload
3 are unchanged. Older native libraries refuse new grants/derivations.

Scoped evidence: 233 Mijia unit and 14 compile-fail tests passed, including all
33 modern controls and every new legacy target. Host account/grant suites passed
53 and 3 tests. The hood/auxiliary/settings/permissions Flutter suite passed 50
tests, the related control/approval/device/FFI suite passed 61 tests, and analysis
reported no issues. All material is fictional. The API 1.38 APK from 2e776a9
predates the four new controls. Final workspace/native checks and API 1.39
packaging are recorded separately; provider/phone/model acceptance remains pending.

Final software verification: the workspace excluding the known unrelated
Meiju/macOS LAN-alias suite passed 642 tests with one ignored. Strict all-target
Clippy and formatting passed. Rebuilt FFI and actual C boundary checks passed
36 methods/60 live examples, including new hood startup acceptance/refusal and
76 modern-account cases. The checked-in examples were regenerated from that run.
Documentation, source hygiene, 25 schemas with 73 positive/1320 negative fixtures,
two coverage-projection tests and seven presentation tests passed. The existing
eight full Flutter font/pixel/legacy-click failures were not rerun or claimed fixed.
No real account, appliance, installation, remote push or publication was used.

## Reported samples Android delivery -- October 30, 2026

The clean 2e776a9 source now has an archive-checked API 1.38 Android package with
both arm64-v8a and x86_64. It includes the reported-sample UI and existing modern
account, state, daily-history and control flows. The verification report records
exact current AAR matching, 31 exports per ABI, development signature, disabled
backup and 16 KiB alignment. See the
[package evidence](../apps/mobile/verification.md#reported-samples-api-138-android-package--october-30-2026).
Installation and real provider/phone/model acceptance remain pending; no new
functional scope, grants, saved formats or remote action is claimed by packaging.

## Reported samples through Core, native and App -- October 30, 2026

ADR-0105 now exposes all 161 pinned numeric history fields through Core on legacy
and modern protected bindings. Exact `(target, aggregation)` options distinguish
reported samples from daily statistics. Samples preserve signed values, source
numeric domains and exact nullable units; daily results remain nonnegative with
string units. Inspection performs no networking. Reads retain account/device
leases and do not change secrets, cache, receipts, descriptors or write authority.

Native API 1.38 adds optional include_reported_samples=false to existing inspection;
older clients remain daily-only. Updated Flutter negotiates 1.38, opts in explicitly,
labels the two types, validates immutable domains and discards results on lifecycle
changes. CLI/MCP reuse the same Core contract. SQLite 5, protected payloads and C
export/method counts are unchanged. Rust enum/nullable-unit changes require source
updates for unpublished consumers; older native libraries refuse new requests.

Executed native checks: workspace tests excluding the known unrelated Meiju/macOS
LAN-alias failures passed 637 tests with one ignored; the final Host history suite
passed 14 tests after an added integer/enum case. Strict workspace Clippy, FFI build
and actual native ABI checks passed (36 methods, 60 examples, 76 modern-account
cases). The affected Flutter history/card suite passed 85 tests and analysis found
no issues. All 13 CLI/MCP tests and 16 Dart FFI tests also passed. Documentation,
source hygiene, 25 schemas with 72 positive/1230 negative fixtures, both reference
projection tests and seven shared presentation tests passed. These use fictional
material and establish no provider/phone/model acceptance. The 9a6c004 APK remains API 1.37; API 1.38 packaging is separate work.

## Typed numeric property history -- October 30, 2026

ADR-0105 adds experimental typed Rust property-log requests for the 161 numeric
fields in the existing pinned scalar mappings. Legacy and modern sessions use
their own fixed /user/get_user_device_data endpoint and per-field codec. Keys
come from exact SIID.PIID, type is prop, and the protected root identifier is
preserved. Boolean, private/opaque and unmapped fields are not admitted.

One explicit query spans at most 32 days and requests at most 31 records. The
parser retains signed values, original reported units (or no unit), raw numeric
enums and provider timestamps; it does not apply daily-energy scaling. It derives
exact numeric domains from pinned specification evidence without changing stored
read mappings. Foreign supplied correlations fail the response. Invalid/out-of-range
values and every ambiguous duplicate timestamp count as rejected records. No raw
actor identity, unknown metadata, cache write, receipt, credential mutation,
automatic paging, fallback or renewal is added. Completeness remains unproven.

Protocol evidence: 629 workspace tests passed with one ignored, excluding the
known unrelated Meiju/macOS LAN-alias suite; a final legacy cancel/drop test also
passed. Strict Clippy, formatting, FFI, native ABI (36 methods/57 examples and 76
modern-account cases), documentation, schema and projection checks passed. The
preceding Mijia run passed 227 unit and 14 compile-fail tests. See ADR-0105 for
exact scope; no Flutter/mobile/provider acceptance is claimed by this phase.

This is a protocol-library phase. Native API 1.37, public history schemas, Core
options, CLI/MCP and App still expose only the admitted daily-energy series.
Reported-sample contracts, protected Core admission, versioned native/App adoption
and provider acceptance remain pending. The verified 9a6c004 APK is unchanged.
No provider account, appliance, package installation, push or publication is used.

## Modern controls package checkpoint -- October 30, 2026

The clean-source 9a6c004 APK includes API 1.37 modern QR/device reads, ADR-0103
exact daily-energy history and ADR-0104's 29 exact scalar control mappings.
Both ABIs, 31 native exports, APK v2 development signature, current-AAR matching
and 16 KiB alignment passed. See the [package evidence](../apps/mobile/verification.md#modern-history-and-controls-android-package--october-30-2026).
Installation and real provider/phone/model acceptance remain pending; broader
reference parity is still incomplete. No merge commit, push or real-device write
was introduced. Historical phase records below retain their original scope.

## Modern exact scalar controls -- October 30, 2026

ADR-0104 implements the existing 29 reviewed scalar controls for modern-selected
bindings: jdjz.fan.2072fc (9), abhome.light.wy0a01 (7), zhimi.airpurifier.v6 (7),
cuco.plug.v3 (4) and xiaomi.hood.cyp10 (2). A separate fixed modern transport uses
the reference continuous codec at core.api.mijia.tech/app/miotspec/prop/set.
Protected binding selection determines credentials and protocol before I/O.
Legacy bindings keep legacy controls; failures never switch protocols or renew.

Production native hosts register the transport without granting commands. Existing
option inspection, exact startup grants and explicit retrieval expose the admitted
actions through the generic App controls. Default native sessions stay read-only.
Elevated outlet settings independently require Core's trusted immutable approval,
including when public risk metadata is tampered with. One correlated encrypted
acknowledgement can establish accepted or rejected, never physical verification.
Possibly-sent failures retain unknown receipts without replay or companion writes.

Executed evidence: 618 workspace tests passed with one ignored, excluding the
known unrelated Meiju/macOS LAN-alias suite. Strict workspace/all-target Clippy
and the native FFI build passed. All 125 affected Flutter control, permission,
approval, history and modern-device-flow tests passed. The rebuilt native ABI
passed 36 ordinary methods/57 examples and 76 modern-account boundary cases. Tests use fictional material and local TLS only; no provider account or
appliance is accessed. Native API 1.37, ordinary schemas/exports, SQLite 5 and
Mijia payload 3 remain unchanged. Old native libraries expose no modern controls.
The existing dd51c73 APK predates both modern history and these controls.

Broader controls/actions/events, BLE/LAN, health/media, additional model/runtime
contracts, updated packaging and real account/phone/device acceptance remain open.
No remote push, publication, package installation or real-device write occurred.

## Modern exact daily-energy history -- October 30, 2026

ADR-0103 implements the exact cuco.plug.v3 daily-energy query for modern-selected
bindings through existing Core/native/App history contracts. A separate fixed
api.mijia.tech profile uses modern credentials and per-field RC4; current-state
reference reads keep their continuous codec and origin. Legacy bindings retain
legacy statistics. Exact identity/model/specification, bounded windows/records,
unknown completeness and foreground cancellation remain mandatory. There is no
fallback, automatic renewal, credential/cache mutation or new write authority.

Executed evidence: 604 workspace tests passed with one ignored (excluding the
known unrelated Meiju/macOS LAN-alias suite); strict Clippy, FFI build and native
ABI checks passed. All 65 affected Flutter history/device-flow tests passed.
The final four protocol tests also passed after expanded gzip/auth negatives.
This is fictional software evidence, not real provider/device acceptance. The
API 1.37 APK from dd51c73 predates this addition. Native API/schema/storage formats
are unchanged. Modern controls, broader history/events, BLE/LAN, health/media and
additional model/runtime contracts remain open.

## API 1.37 Android package -- October 30, 2026

The native/Flutter modern QR-to-card/read phase is committed as `dd51c73`.
Its clean-source combined-platform APK passed dual-ABI, 31-export, signature,
current-AAR and 16 KiB alignment checks; see the [package evidence](../apps/mobile/verification.md#modern-qr-and-device-publication-android-package--october-30-2026).
The package is built but not installed or provider-tested. Next work remains
modern controls/history, broader events, BLE/LAN, health/media, additional runtime
control contracts and real account/phone/model acceptance. Earlier phase records
below retain their original scope and are superseded where explicitly noted.

## Flutter modern device publication -- October 30, 2026

The API 1.37 Flutter client now requests modern device publication after QR
continuation and explicit refresh/recovery under ADR-0102. API 1.36 retains logical
reports, and 1.35 refuses QR before symbol lookup. Distinct result kinds validate
closed fields, exact account/provider/authentication, partial/failed status,
required errors and at most 4096 unique UUID IDs. Owned parsed reports are immutable.
No successful logical count becomes a device count.

The controller reloads local accounts/cards before adopting a current foreground
report. Existing partial/failure UI displays returned device counts and keeps
historical cards; cancelled navigation/account/lifecycle work cannot restore late
reports. A failed local reload retains the reload requirement without repeating
login. Card taps use existing descriptor-driven Core state reads; modern controls
and history remain unavailable. QR connection wording supports both native
versions. A shared home-header title now wraps within its available width instead
of overflowing at 320 pixels and doubled English text.

The affected 134 Flutter account/native/QR/localization/foreground-read tests
passed, including 20 new ABI/widget/controller cases. They cover actual fictional
C libraries for 1.35/1.36/1.37 and an App QR-to-card click/state-read path, plus
partial/failed/empty results, malformed/private/oversized results, owned-memory
cleanup, cancelled ready replies and local reload failure. These tests use no
provider or appliance. Both shared portrait/landscape reflow tests, Flutter
analysis and all seven presentation tests also passed. Native API 1.37 and protected storage formats are
unchanged. New Android packaging and real account/phone/model acceptance remain
pending; the old API 1.35 APK does not contain these changes.


## Native modern device publication (API 1.37) -- October 30, 2026

[ADR-0102](adr/0102-native-modern-device-publication.md) adds optional Boolean
`publish_devices` to dedicated modern continuation metadata. True returns
`connected_devices` with a sanitized AccountConnection; absent/false retains
logical `connected`. New `refresh_devices` / `recover_devices` session operations
return `devices` with partial/failed retrieval. Old `refresh`, `recover`, `renew`
keep their behavior. No C export or ordinary JSON method is added.

Ordinary account refresh/recovery selects its protocol from protected saved
material before I/O, under the same account/registration leases. Modern-only
accounts and accounts with any modern-selected binding use modern inventory;
otherwise they use legacy. A modern failure never falls back or renews. Returned
roots select that protocol; absent historical bindings retain their identities
and selection. Modern-selected controls/history remain unavailable. Adding only
a modern authentication slot does not change a legacy account's refresh route.

Positive/negative schemas, fictional Host publication/protocol-selection tests and
actual C admission checks cover the additions. The workspace passed 595 tests
with one ignored; strict Clippy and the rebuilt C ABI passed, including 76 modern
account boundary cases. See ADR-0102 for the exact evidence scope. SQLite 5, Mijia payload 3, Meiju payload 3, 36 JSON methods and 30 C
exports (31 on Android) remain unchanged. Flutter opt-in/parsing/card adoption,
a new APK and real provider/phone/model acceptance remain pending. No provider,
appliance write, package installation, remote push or publication is performed.


## Foreground modern Host device publication -- October 30, 2026

[ADR-0101](adr/0101-mijia-modern-host-device-publication.md) adds typed Host QR
continuation with device publication and explicit modern device refresh/recovery.
One bounded directory response is reconciled and persisted under the original
account/registration leases, then published through Core. The Host inherits the
original QR deadline and checks foreground generation before and after publication.
Reports separate successful authentication from partial/failed retrieval and expose
only local device IDs. Logical children stay grouped; unknown models remain named
unsupported cards. Existing logical-only methods keep their behavior.

The new path has fictional Host/Core tests for QR-to-card-to-state, exactly one
directory read, local identity/organization preservation, empty/failed directories,
uncertain authentication/binding storage, explicit recovery, Core publication
conflicts, consumed/foreign handles, cancellation, expiry, drop and late lifecycle
results. The workspace regression passed 591 tests with one ignored; strict Clippy and
the rebuilt native ABI passed. See ADR-0101 for the exact evidence scope.

Native API 1.36, its logical C/App results, SQLite 5 and protected payload versions
are unchanged. The next dependent work is versioned C/Flutter device publication
and generic refresh selection for modern-only/mixed accounts. The existing APK
does not contain this integration. Modern controls/history, broader reference
coverage and real provider/phone/device acceptance remain incomplete. No provider
account, appliance, remote publication or mobile package is used by this phase.


## Modern protected device bindings and Core reads -- October 30, 2026

[ADR-0100](adr/0100-mijia-modern-protected-core-reads.md) adds explicit typed modern
device refresh/recovery returning partial Core descriptors. Protected payload 3
retains root/child evidence, stable local IDs and selected read-protocol provenance.
Versions 1/2 remain byte-stable until explicit device adoption; older binaries
cannot read version 3. SQLite 5, Meiju payload 3 and native API 1.36 are unchanged.

Core can publish these descriptors and read the exact pinned scalar properties
through modern credentials/origin/codec. Explicit legacy or modern inventory can
select its own read protocol; failed reads never fall back or reauthenticate.
Modern-selected bindings suppress controls/history and independently refuse stale
control descriptors/grants. Logical children never receive separate cards; prior
root/child and cross-account collisions require reconciliation rather than merging.
Names, rooms, favorites and device/connection IDs survive exact protocol selection.

Twelve new fictional integration tests cover all 264 mapped scalar readings,
restart, version-3 tampering, stable organization, partial retention, role/model/
parent conflicts, uncertain save/recovery, cancellation/drop and write refusal.
The Rust workspace run excluding the known unrelated Meiju/macOS LAN-alias test
failures passed 579 tests with one ignored. Strict workspace/all-target Clippy and
the native library build passed. The rebuilt C ABI passed all 36 ordinary
methods/57 examples and dedicated-export checks, including 62 modern-account
refusal/buffer/lifecycle cases. Documentation, repository hygiene and the reference
projection tests passed. No real provider/device access or mobile package was used. Host/App login-to-card publication remains the next dependent task;
the existing QR C/App reports still return logical directory summaries only.


## Modern directory device groups -- October 30, 2026

[ADR-0099](adr/0099-mijia-directory-device-groups.md) implements typed, protected directory grouping as the prerequisite for
modern Core adoption. Canonical numeric/text `pid` and optional cloud `parent_id`
remain inside opaque entries. Kinds 0/6/16/18 are separate candidates; kind 21
joins only its explicit ordinary parent with compatible model/spec evidence.
Kind-18 gateway children stay separate, and names or ID suffixes never establish
identity. Unknown kinds, missing parents and nested logical parents are retained
unresolved. Duplicate/foreign identities, cycles and contradictory split metadata
fail before any grouping result. Linear bounded traversal observes cancellation
and preserves each input exactly once, without property-map fusion or networking.

Fourteen new topology cases passed in the final 184 Mijia unit tests, plus 14
compile checks including serialization refusal. Strict Mijia all-target Clippy
passed. The preceding combined Mijia/Host run passed 498 tests before the final
four cases and equivalent traversal cleanup. No native API, protected payload,
SQLite, App report or control authority changes. Existing modern directory reads
now reject malformed kind/parent metadata instead of discarding those fields.
Missing metadata remains readable but cannot establish a physical candidate.

Protected route provenance, stable legacy/modern binding reconciliation, Core
publication/reads and App card adoption remain pending. Typed groups are not
physical device counts, complete membership or evidence that appliances are online.
No provider/account operation, appliance write, mobile package or remote publication
was performed. The reference ledger continues to report partial implementation.


## Foreground Flutter QR account flow -- October 30, 2026

The API 1.36 client is now connected to the sign-in and saved-account pages under
ADR-0098. The explicit QR choice rechecks credential-storage readiness before
requesting a code. The App encodes scan bytes locally with pinned `qr` 3.0.2,
clears the source array after encoding, and owns a clearable module bitmap with
a four-module white quiet zone. Dismissal, expiry, navigation and background clear
owned presentation material. Encoder intermediates have managed lifetimes; no
whole-heap erasure guarantee is made. No URL string, remote image, browser,
clipboard, polling, regeneration or automatic renewal is introduced.

Instructions require Xiaomi Home on another device while HomeVerb stays open,
then one explicit Continue. Controller epochs, foreground state, client ownership
and selected account guard late results. Interrupted sign-in requires a local
account reload because completed secret storage cannot be rolled back. Existing
password/captcha/verification entry points and physical cards remain available.

Saved QR results are separate from legacy inventory: zero entries are displayed
as zero, failed directory reads retain their error, and no logical entry becomes
a physical card or complete-inventory claim. QR directory refresh, saved-session
recovery and renewal each have their own explicit account-page action. Public
summaries do not expose which protected protocol slots exist; the UI explains that
these operations require saved QR authorization, and the native layer refuses a
missing modern slot without fallback.

Executed evidence: 102 affected Flutter account/native tests passed, including
26 new QR cases; final account-page click coverage passed in all 26 QR tests.
After the final QR-specific waiting message, all 35 QR/account-page tests and
Flutter analysis passed. Seven presentation tests, two reference-projection tests,
documentation, repository hygiene and contract validation (25 schemas, 69 primary
positives and 1,092 negatives) passed. A fictional 280-point rendered QR image decoded to the
expected fictional payload using the independent macOS Core Image decoder.
English/Chinese 320-point layouts with doubled text passed. This proves local
software behavior, not provider authentication, phone security or physical-device
support. Native API 1.36, SQLite 5, protected payload versions and grants are
unchanged. Android packaging, real account/phone acceptance, physical/logical
reconciliation and the remaining reference ledger remain open. No provider call,
appliance write, package installation, push or publication was performed.


## Flutter modern-account client -- October 30, 2026

The typed Flutter client now consumes all four API 1.36 functions under
[ADR-0098](adr/0098-foreground-mijia-qr-app.md). Version/provider negotiation
precedes lazy symbol lookup; API 1.35 libraries without the new symbols refuse
before lookup. Worker isolates carry only pointer addresses and nonsecret metadata.
Native scan buffers are cleared/freed, owned scan arrays are cleared on consumption,
attempted expired continuation, discard and lifecycle changes; foreign clients
cannot consume them. The pending object itself schedules no timer.

Closed parsers validate modern QR metadata and account/directory outcomes, including
exact selected-account identity, provider, conservative completeness and failed
retrieval. Explicit refresh/recover/renewal remain separate calls; no retry, automatic
poll or cross-protocol fallback is added. Pending challenges share existing App
cancel/background/close handling, and late results cannot restore cancelled state.

Ten fictional C-ABI client tests passed as part of a 67-test affected Flutter
account/native regression. The fixture performs no network, platform-secret or
appliance access. Native API/storage formats are unchanged. Controller/account-page
wiring, local QR rendering, modern account actions, Android packaging and real
account/phone acceptance remain pending. The existing API 1.35 APK is unchanged.

## Native modern account bridge (API 1.36) -- October 30, 2026

[ADR-0097](adr/0097-native-modern-mijia-account-bridge.md) adds four dedicated C
functions for modern QR begin/continue/discard and explicit saved-account refresh,
recovery or renewal. The trusted UI must negotiate API 1.36 before resolving
symbols. QR scan bytes use a caller-owned protected buffer of exactly 8192 bytes;
JSON contains only local challenge metadata, length and remaining lifetime.
Valid buffers clear before admission; invalid lengths are not dereferenced.
Empty/oversized projection discards the pending handle without truncation.

Connected/directory reports expose the public account summary and bounded logical
entry count, with partial/failed status and `complete=false`. They expose no private
membership, credential references, entry names, device IDs or physical cards.
Renewal returns only an account summary and performs no inventory read. Existing
Host cancellation, one-use handles and pending-save recovery remain in force.
No automatic poll, reauthentication, background networking or write authority is
added. Ordinary JSON/CLI/MCP cannot call these functions.

Native API 1.36 retains 36 ordinary methods and adds four C exports for a total
of 30 (31 with the Android initializer). SQLite 5, Mijia payloads 1/2 and Meiju
payload 3 remain unchanged. Older native libraries lack the new symbols; existing
App flows retain their prior behavior. Flutter QR rendering, logical-parent/Core
adoption, mobile packaging and authenticated provider/physical acceptance remain
pending. The delivered `9870437` APK is still API 1.35. No provider login, message,
appliance operation, push or publication was performed.

## Modern Host account lifecycle -- October 30, 2026

[ADR-0096](adr/0096-mijia-modern-host-lifecycle.md) adds typed foreground Session
methods for protected QR begin/continue/discard and explicit modern account
directory refresh, recovery and renewal. QR handles belong to one Session, are
consumed before continuation admission and share the four-entry pending limit
with captcha and verification. Background, cancel and close clear pending sign-in;
registered work uses existing cancellation/deadlines and late results cannot cross
a lifecycle generation. No automatic polling, retry or background work is added.

Mijia-only and combined native Session composition now registers the reviewed
modern HTTPS transports without contacting them at startup. Core-only CLI/MCP
composition is unchanged. Modern logical directory entries remain opaque typed
results; they do not publish Device cards or grant routes/control authority.
Successful same-subject connection preserves existing legacy cards. Public account
summaries exclude QR data, provider identity and credential references.

This Host-only phase retains API 1.35, ordinary methods/C exports, SQLite 5 and
protected payload versions. Dedicated C buffers/API negotiation, Flutter QR
rendering, logical-parent reconciliation and authenticated provider/physical
acceptance remain pending. The `9870437` APK is unchanged. No live account,
message, device command, push or publication was used.

## Protected modern account sessions -- October 30, 2026

[ADR-0095](adr/0095-mijia-protected-modern-account-sessions.md) adds explicit
Rust adapter composition for modern QR connection, saved directory/property
reads, renewal and uncertain-save recovery. A version-2 secret-store payload
retains independent legacy and modern slots for one exact numeric subject.
Either login can adopt the matching local account without replacing its other
slot, legacy bindings or label. Ambiguous identities fail closed. Modern logical
entries are account-bound and do not publish physical Device cards.

Legacy-only version-1 bytes stay unchanged. Version 2 requires the modern slot
and a fixed reference profile; older implementations cannot read it and must not
downgrade it. Native API 1.35, C/JSON methods, SQLite 5 and Meiju payload 3 stay
unchanged. Existing legacy routes require their own slot and never fall back to
modern credentials. Default adapter composition does not enable modern requests.

QR completion checks its protected account snapshot before and after provider
exchange; cancellation and the original deadline cover persistence. Explicit
renewal preserves legacy evidence and does not enumerate. Uncertain saves remain
pending until explicit modern-directory validation confirms the exact saved bytes,
without another login, renewal or credential rewrite. No background task is added.

Host lifecycle/registry, C/App QR integration, physical/logical reconciliation,
modern controls and authenticated provider/device acceptance remain pending.
The delivered `9870437` APK contains the earlier auxiliary-setting phase only.
No real account, appliance, message, push or publication was used.

## Modern reference-session reads -- October 30, 2026

[ADR-0094](adr/0094-mijia-reference-session-reads.md) adds independent typed Rust
modern directory, home-list and pinned scalar reads at the exact reference origin
with continuous RC4 and bounded encrypted gzip. All 264 existing scalar mappings
are covered; the catalog's no-read specification remains an opaque entry.
Directory pages retain logical children without publishing duplicate physical
Devices. Subject isolation, conservative completeness, no retry/reauthorization,
fixed routes and cancellation remain enforced.

Modern protected persistence, Core/Host adoption, logical-parent reconciliation,
native C/App integration and authenticated provider acceptance remain pending.
API 1.35, SQLite 5, existing protected account payloads and the delivered APK are
unchanged. No live account or appliance operation was performed.


## Independent modern QR protocol -- October 30, 2026

Typed Rust QR login, single-use bounded completion and explicit same-subject
renewal now implement [ADR-0093](adr/0093-mijia-independent-qr-authentication.md).
Protected material remains separate from legacy sessions; exact modern endpoints
reject redirects and foreign URLs, and expiry/cancellation cannot be extended.
Source review plus anonymous URL-shape checks are not authenticated acceptance.

Protected account persistence/adoption, native C/App integration, modern data
origin/codec admission and real login/renewal acceptance remain pending. Native
API 1.35, existing account payloads and the auxiliary-setting APK are unchanged.
No message, account login, appliance request, push or publication was performed.


## Independent modern authentication design -- October 30, 2026

[ADR-0093](adr/0093-mijia-independent-qr-authentication.md) records concrete
community SID=mijia QR/renewal evidence and an anonymous bootstrap/QR URL-shape
check. Implementation and authenticated acceptance remain pending. Exact modern
and legacy sessions, callback hosts and API codecs must remain separate.


## Auxiliary settings package availability -- October 30, 2026

The API 1.35 combined-platform APK from clean source `9870437` includes all seven
auxiliary targets and the existing exact outlet confirmation flow. Both ABIs,
27 exports, development signature, 16 KiB alignment and exact AAR matching passed.
See the [delivery record](../apps/mobile/verification.md#auxiliary-settings-api-135-android-package--october-30-2026).
No installation or real provider/device operation was performed. Modern SID,
events and other reference feature gaps remain open.


## App auxiliary settings -- October 30, 2026

The App now negotiates API 1.35 before sending the three additional reference
setting tuples from ADR-0092. Exact hood illumination power, brightness 0/1/2
and vendor fan quiet retain all seven targets. Brightness remains an enum rather
than a percentage, and zero is an explicit value rather than missing data.
Current session grants gate the generic component controls. Opening a device
performs no command preparation or control request. English/Chinese narrow
enlarged-text panels use existing localized property labels. Older API libraries
receive only a read-only version probe before refusing the new startup tuples.

No native exports, ordinary methods or storage formats change in this App phase.
New Android packaging and provider/physical acceptance remain pending; the API
1.34 APK is unchanged. Modern authentication, events and other ledger gaps remain
open.


## Native auxiliary settings -- October 30, 2026

API 1.35 implements [ADR-0092](adr/0092-exact-auxiliary-reference-settings.md):
the exact `xiaomi.hood.cyp10` illumination switch and brightness enum 0/1/2,
and `jdjz.fan.2072fc` vendor quiet boolean. These seven targets use independent
`mijia_setting_grants`; existing fan and outlet admissions do not enable them.
Only exact protected source evidence permits the properties. No percentage
brightness conversion, companion power request, ventilation, timing or cleaning
authority is added. Each command sends one property and preserves accepted/
unknown outcomes without replay. Local native inspection works before grants.

Existing reference read-only derivations remain valid. Explicit selected-account
refresh upgrades the mapping; use the matching implementation with newer saved
derivations. There are still 36 JSON methods and 26 C exports plus the Android
initializer, with SQLite 5 and protected payload envelope 1 unchanged. Flutter
API 1.35 negotiation and presentation are pending. The delivered API 1.34 APK
does not gain these controls. No real provider/account/device acceptance is claimed.


## Auxiliary reference setting design -- October 30, 2026

[ADR-0092](adr/0092-exact-auxiliary-reference-settings.md) admits the next exact
implementation phase: hood illumination power and three-code brightness, plus
the reference fan's independent custom quiet boolean. Seven exact targets retain
separate grants, no companion writes and no brightness percentage conversion.
Native API 1.35 and App rollout are pending; the current API 1.34 APK is unchanged.


## Exact outlet package availability -- October 30, 2026

The API 1.34 combined-platform APK from clean source `9b50a89` includes exact
outlet grants and App trusted confirmation. Both ABIs, 27 exports, development
signature, 16 KiB alignment and exact AAR matching passed. See the
[delivery record](../apps/mobile/verification.md#exact-outlet-api-134-android-package--october-30-2026).
No installation or live provider/device operation was performed. Modern SID,
events and other reference feature gaps remain open.


## App exact outlet controls -- October 29, 2026

The foreground App now negotiates API 1.34 before sending the new independent
`mijia_outlet_grants` field. All nine exact targets for power, power-loss recovery,
physical-control locking and indicator settings are retained. The shared purifier
lock identifier continues to use its separate low-risk reference-setting family.
Opening a device inspects protected options and refreshes only its account to
publish eligible actions; it does not prepare or execute a command.

Elevated outlet forms require explicit target and cloud-route selection, then
the existing immutable trusted review and actual confirmation. English/Chinese
copy describes connected-load start/stop, later power restoration and loss of
local-button operation. Component-scoped presentation labels distinguish switch
power without model/brand branching in the UI. Recovery codes remain numeric.
The indicator uses the ordinary low-risk flow. Existing navigation/background/
session replacement and unknown-receipt handling remain active. The API 1.33 APK
does not include this phase; new packaging and exact physical acceptance remain
pending. Modern SID authentication, event breadth and remaining feature gaps
are unchanged.


## Native exact outlet controls -- October 29, 2026

API 1.34 adds closed `mijia_outlet_grants` with exact local device/component/
property/value targets and a 64-entry limit. Only Mijia and combined hosts accept
these startup grants. All four exact outlet options can be inspected locally
before granting authority. Existing light/purifier and fan fields retain their
scopes. Separate adapter policies prevent overlapping physical-lock identifiers
from transferring authority between purifier and outlet grants, even when both
families are enabled.

Power, power-loss recovery and physical-button locking require the separate
immutable trusted preview/confirm bridge for every operation; indicator control
is low-risk. Default startup and CLI/MCP remain read-only for outlets. Ordinary
JSON cannot confirm elevated commands. Unknown replies preserve the request ID
without replay, and backgrounding invalidates previews. C exports (26 plus the
Android initializer), 36 ordinary methods, SQLite 5 and protected payload envelope
1 are unchanged. Explicit account refresh in an outlet-enabled session upgrades
old read-only mappings. Older implementations may refuse those upgraded protected
contents; retain the matching implementation with that profile. App wiring, an
updated APK and real-provider/physical acceptance remain pending.




## Typed exact outlet controls -- October 29, 2026

[ADR-0091](adr/0091-exact-outlet-control-approval.md) now has a separate typed Rust
adapter opt-in for four exact `cuco.plug.v3` settings. Power, power-loss recovery
and physical-control locking require Core grants plus actual trusted approval;
indicator control is low-risk. The adapter re-derives risk from protected evidence
and refuses forged low-risk metadata independently of Core. Single-property
requests retain exact values, accepted/unknown outcomes and no automatic replay.

Existing API 1.33 inspection/default account retrieval retain their prior scopes
and read-only outlet derivations. An explicit typed inspection can derive outlet
options locally without enabling execution. Only the typed outlet-enabled refresh
persists new control derivations; older implementations may reject those newer
protected contents, so keep the originating implementation with that profile.
SQLite 5 and payload envelope 1 remain unchanged. Native outlet startup grants,
API 1.34/schema rollout, App confirmation, APK and real-device acceptance are
still pending; the API 1.33 APK does not acquire outlet controls from this phase.


## Exact outlet control design -- October 29, 2026

[ADR-0091](adr/0091-exact-outlet-control-approval.md) defines the next implementation
phase for `cuco.plug.v3`: independent startup grants, three elevated settings with
actual trusted confirmation, and low-risk indicator control. Runtime/native/App
work is pending; API 1.33 and the current APK do not yet provide these controls.
The event audit found no exact current-reference lock/cateye history keys in the
reviewed community examples. Those gaps remain open, without cross-model inference.


## Mijia CLI and historical tools -- October 29, 2026

[ADR-0090](adr/0090-mijia-cli-and-historical-tools.md) adds explicit read-only
Mijia CLI/MCP hosting and generic bounded history options/reads. CLI account
commands reuse the existing Session, with hidden one-shot login and no automatic
challenge sends. Saved cross-account conflicts remain unsupported for Mijia.
Six read-only MCP tools are available without grants, with eight tools for
existing admitted Meiju controls. No write grant or modern SID support is added.
Historical reads preserve Core evidence and cancellation without cache/receipt
writes, polling or retries. Native API 1.33, schemas and protected storage formats
are unchanged. MacOS credential operations remain blocked by its unimplemented
secure-store backend; Linux Secret Service and real-provider acceptance require
the appropriate owner environment. The API 1.33 App APK remains valid because
this phase changes only the CLI host, tests and documentation.


## Historical API 1.33 APK availability -- October 28, 2026

The combined-platform APK from clean source `cdb25bf` includes native historical
reads and the explicit App dialog. Both ABIs, signature, 27 exports per ABI,
16 KiB alignment and exact current-AAR matching passed. See the
[delivery record](../apps/mobile/verification.md#historical-api-133-android-package--october-28-2026).
No installation, real account/statistics request or appliance write was performed.
CLI/MCP history presentation, modern SID login, event breadth and remaining
reference feature coverage are still incomplete.


## App historical readings -- October 27, 2026

The App negotiates API 1.33 and provides a generic device-detail history dialog.
Opening and connection selection inspect local protected options only. Explicit
reads use an offered numeric series and editable bounded whole-second UTC range.
The view retains zero values, individual timestamps, units, source, receipt time,
rejected counts, limit status and unproven completeness; it never fills gaps or
calculates a complete calendar total. Navigation, background, session replacement
and cancellation discard pending results. Device state polling pauses while the
dialog is open. CLI/MCP commands, updated packaging and real-provider acceptance
remain pending. Native contracts, storage, authority and adapter admission are
unchanged. See the [App verification record](../apps/mobile/verification.md#foreground-historical-readings--october-27-2026).


## Native historical reads -- October 27, 2026

API 1.33 adds `inspect_history_options` and `read_history` through the existing C
query bridge, with closed metadata, generic Core dispatch and ordinary session
lifecycle. There are now 36 JSON methods; C exports, SQLite, protected payloads
and grants are unchanged. Default Mock/Meiju adapters have no historical options.
Native schemas, examples and version manifests are synchronized. Flutter, CLI
and MCP presentation and updated mobile packaging remain pending; the current
APK is API 1.32. No live provider account or statistics request was used.


## Core and protected historical reads -- October 27, 2026

Typed Core now inspects and reads bounded numeric history under
[ADR-0089](adr/0089-bounded-numeric-history.md), with closed DTOs, exact local
connection/property admission, device locking, result validation and lifecycle
cleanup. Adapters default to refusal. The Mijia adapter independently admits the
protected exact `cuco.plug.v3` daily energy profile, including a reviewed reference
mapping when inventory omitted the URN. History never writes the snapshot cache,
execution receipts or grants. Native/CLI/MCP/App entry points remain pending;
API 1.32, ordinary methods, C exports, SQLite and protected formats are unchanged.


## Core historical-read contract -- October 27, 2026

[ADR-0089](adr/0089-bounded-numeric-history.md) defines local typed options,
bounded queries and sanitized numeric history results. The schema and fixtures
are implemented; Core/adapter/native/App runtime integration is pending. This
contract keeps historical daily aggregates separate from current state and write
verification. Native API 1.32, ordinary methods, storage and grants are unchanged.


## Exact daily energy protocol -- October 27, 2026

Typed Rust now implements one bounded `v2/user/statistics` read for exact
`cuco.plug.v3` specification 2 and mainland legacy sessions under
[ADR-0088](adr/0088-mijia-exact-daily-energy-history.md). The fixed key/type/limit
are not caller-editable RPC parameters. Sanitized timestamped kWh records retain
missing/invalid/duplicate/limit information and unproven completeness. No current
observation, calendar total, implicit retry or background polling is introduced.
Core historical-read admission, native/App integration and real-device acceptance
remain pending. API 1.32, ordinary methods, saved formats and grants are unchanged.
The packaged APK from `01a115b` predates this protocol and has no energy-history UI.


## Verification APK availability -- October 27, 2026

API 1.32 is now packaged from clean source `01a115b` for arm64-v8a and x86_64.
Local signature, AAR matching, 27 exports per ABI and 16 KiB alignment passed.
This does not establish runtime or real-account/message acceptance. See the
[artifact record](../apps/mobile/verification.md#verification-api-132-android-package--october-27-2026).


## Foreground App verification -- October 27, 2026

The App negotiates native API 1.32 and handles image challenges followed by
explicit phone/email code requests and protected ticket submission. Only offered
methods appear; a user action is required for every send. Cooldown and expiry
are local UI timers, with no polling or automatic resend. Sensitive provider-page
bytes stay in the C output allocation and are cleared without becoming Dart
strings. Input, captcha images and pending handles are cleared on completion,
cancellation, navigation, background or expiry. Older hosts retain captcha-only
or one-shot login. Ordinary methods, exports, grants and storage are unchanged.

English/Chinese widget flows, controller races and a fictional C ABI fixture
cover this integration; real account/message delivery and mobile packaging
remain separate acceptance. The last packaged APK still uses API 1.30. Human-risk
challenges, account-settings confirmation and modern SID `mijia` authentication
remain unsupported. See the [phase record](development/14-mijia-reference-integration.md#foreground-app-verification--october-27-2026).


## Native explicit code-request bridge -- October 27, 2026

API 1.32 exposes one dedicated trusted code-request C function under
[ADR-0087](adr/0087-native-mijia-verification-code-delivery.md). Closed metadata
selects phone/email for an existing session-owned challenge. Successful requests
return a new handle and nonsecret cooldown/count metadata; protected prompt bytes
stay outside JSON. The total is 26 C exports plus Android's initializer, with
34 ordinary methods and unchanged saved formats. Flutter adoption and new mobile
packages remain pending. No real SMS/email, account or appliance was accessed.


## Explicit verification code requests -- October 26, 2026

New public verification-page evidence supports typed native SMS/email code
requests under [ADR-0087](adr/0087-native-mijia-verification-code-delivery.md).
User-selected method inspection and quota checks precede one send, with original
lifetime, bounded attempts, cooldown and account/Host lifecycle checks. Human
risk challenges remain unsupported; there is no automatic resend. Native API
1.31 and existing C/App behavior remain unchanged pending dedicated entry-point
and UI integration. No real messages were sent. See the
[protocol evidence](development/14-mijia-reference-integration.md#public-verification-page-protocol-evidence--october-26-2026).


## Native protected verification bridge -- October 26, 2026

API 1.31 adds three dedicated C verification functions under
[ADR-0086](adr/0086-native-protected-verification-prompt.md). Captcha/provider-page
bytes use a separately cleared caller-owned buffer, never ordinary JSON. Existing
22 C functions, 34 ordinary methods, account/device identities, SQLite 5 and
protected payloads are unchanged. The current total is 25 C exports plus Android's
initializer. New callers must negotiate 1.31; existing App captcha login remains
unchanged. Foreground browser integration, mobile package rebuild and real-account
acceptance remain pending. See the
[phase record](development/14-mijia-reference-integration.md#native-protected-verification-bridge--october-26-2026).


## Account-bound verification continuation -- October 26, 2026

Opt-in typed Rust adapter and Host entry points now bind phone/email verification
to protected account snapshots under
[ADR-0085](adr/0085-mijia-protected-verification-tickets.md). Waiting holds no lease;
changed or removed accounts invalidate continuation, and final persistence checks
the snapshot again. The session registry shares the four-pending-login limit with
the old captcha path and is cleared on background/cancel/close. Native API 1.30,
ordinary methods, C exports and storage formats remain unchanged. C/App browser
and ticket interaction remains pending; this phase supplies synthetic tests only.
See the [phase record](development/14-mijia-reference-integration.md#account-bound-verification-continuation--october-26-2026).


## Protected phone and email ticket protocol -- October 26, 2026

The opt-in Rust protocol in [ADR-0085](adr/0085-mijia-protected-verification-tickets.md)
implements explicit phone/email method discovery, one-use ticket submission and
bounded session completion with signing-key and subject checks. It preserves
initial login cookies and the original three-minute challenge lifetime. Existing
account/Host/C/App paths remain captcha-only: protected account binding and a
foreground trusted browser/ticket bridge are still pending. No native API 1.30,
ordinary method, export or persisted-schema change is introduced. This is
fictional protocol and local TLS evidence, not real-account acceptance or modern
SID `mijia` authentication. See the
[phase record](development/14-mijia-reference-integration.md#protected-phone-and-email-ticket-protocol--october-26-2026).


## Exact light and purifier controls -- October 25, 2026

API 1.30 implements 14 low-risk scalar controls for `abhome.light.wy0a01` and
`zhimi.airpurifier.v6` through protected Core mapping, native startup and the
foreground App. Independent `mijia_setting_grants` retain exact device, component,
property and value; existing fan permissions cannot authorize these settings.
The light's 2700–6500 K domain retains every 1 K target. This is synthetic
implementation evidence, not physical support. See
[ADR-0084](adr/0084-exact-light-and-purifier-controls.md) and the
[phase record](development/14-mijia-reference-integration.md#exact-light-and-purifier-controls--october-25-2026).
The full reference request remains incomplete, including modern authentication,
history/events, other controls and media/health.

## Exact reference fan settings — October 25, 2026

API 1.29 implements [ADR-0083](adr/0083-exact-reference-fan-settings.md):
`jdjz.fan.2072fc` gains horizontal/vertical included swing angles, vertical swing
and ionizer controls, each with independent exact-value startup grants. Core,
the protected adapter and the foreground App retain all identity, route, argument
and uncertainty checks. Old reference mappings remain readable; explicit refresh
derives the additional options. This is synthetic implementation evidence, not
physical-device verification. Other reference controls, modern authentication,
history/events and health/media remain incomplete.

October 25: community-derived typed Rust captcha continuation and protected
account/Host registries, dedicated C entry points and bilingual App challenge UI
are implemented and synthetic-tested. SMS/email verification, historical readings
and real-account acceptance remain pending; see the [phase record](development/14-mijia-reference-integration.md#community-captcha-protocol-phase--october-25-2026).

October 25: 17 exact Mijia reference specifications now supply 264 scalar read
mappings and localized App labels. Additional writes and modern authentication
remain incomplete; see the [full coverage and blockers](development/14-mijia-reference-integration.md).

The October 25 exact light and purifier settings use native API **1.30.0**, following
the October 24 linear mainline update. Earlier Mijia phase versions are historical. See
[contract reconciliation](adr/0079-mijia-mainline-reconciliation.md). Mainline Meiju restoration and controls
remain available alongside Mijia; identity restoration remains Meiju-only.

The October 23 refrigerator presentation removes repetitive Stale labels from
feature cards and temperature-zone captions at the owner's request. Core freshness
flags, timestamps, missing/error states and foreground refresh remain unchanged.

## October 22 refrigerator presentation groups

The App replaces the flat Modes and freshness grid with observed variable-zone,
smart/energy-saving, refrigerator-function, operating-status and settings groups.
Exact capability/property IDs determine presentation; brand/model names do not.
Empty groups are omitted and future unclassified readings remain visible.
English/Chinese and 320-pixel double-text-scale fixtures pass; existing compartment
and localization regressions pass. No appliance requests, new controls, report
services or physical acceptance are supplied by this presentation change. See the
[verification record](../apps/mobile/verification.md#refrigerator-presentation-groups--october-22-2026).

## October 22 Meiju reference integration

The owner's full-integration request is **incomplete**. API 1.26 implements four
exact AC optional catalogs, independent cool-energy-saving/strong-cooling/light-
sensing settings through the existing Lua path, and exact-product refrigerator
third-zone/detail observations in Core and App. Existing shared grants,
prerequisite checks, lifecycle and receipt rules remain mandatory. This supplies
software integration, not new physical compatibility.

The [integration guide](development/13-meiju-reference-integration.md) and generated
67-operation ledger record the remaining work. A subsequent owner-authorized
[read experiment](adr/0075-one-shot-meiju-read-compatibility-probe.md) proved that
the existing protected session and admitted POST signer work for modern Lua-state
and aggregate-inventory reads. The blanket missing-authentication blocker is
withdrawn for those requests; the ordinary App has not yet adopted their response
contracts. Independent modern renewal, endpoint/query/business contracts and some
referenced resources are still absent or untested. The owner confirmed no complete
source directory is available. New TSL/statistical/history/configuration services,
current GX1200Max controls and refrigerator writes remain incomplete.
See the [App verification record](../apps/mobile/verification.md) for actual
offline/runtime/package checks and the existing macOS LAN-test blockers.

## Active Mijia integration: October 18, 2026

The owner requests mainland-China Mijia connection, all returned devices as named
cards, unsupported placeholders, and initial doorbell/door-lock/fan adaptation.
The [integration guide](development/11-mijia.md) tracks protocol, protected
account/Core, native host/App and owner-acceptance phases. Native protocol,
bounded account/home enumeration and selected MIoT readable mappings are
implemented and synthetic-tested. Protected account persistence and stable device
bindings now feed the read-only Core adapter and explicit Rust host composition.
Native API 1.24 now routes Mijia-only and combined Meiju/Mijia sessions through
protected connection and ordinary account/card/read operations. Flutter now has
registered platform selection, provider-scoped sign-in/reauthorization and partial
positive cards. Doorbell, lock, fan and battery readings use descriptor-driven
details. The separately progress-tracked single-property write protocol is
synthetic-tested. Explicit trusted Rust fan admission and local protected option
inspection are implemented under ADR-0078. Native API 1.25 adds exact fan startup
grants and protected option routing. Flutter prepares exact fan permissions and
exposes bounded generic controls, including positive partial-inventory admission.
A dual-ABI combined-platform Android development APK built from `39d6465` has
passed archive/signature/alignment inspection. Owner acceptance remains pending;
the integration guide contains the artifact hash and checklist. See
[ADR-0076](adr/0076-mijia-protected-core-reads.md) and
[ADR-0077](adr/0077-mijia-native-account-routing.md).
Real-account and physical validation belong to the owner. No door-lock write,
background polling, project service or shared application secret is admitted.

## Integrated main: October 16, 2026

The owner resumed development and requested the Flutter UI merge, Core integration
and local Android APK delivery. The former UI branch is now part of `main`.
Native API **1.23.0** provides 34 ordinary methods, 16 C functions plus the
Android-only initializer, SQLite schema 5 and protected payload 3.

The app lives in [apps/mobile](../apps/mobile/README.md). Its protected account
input, local organization, capability-driven controls, trusted command confirmation,
receipt recovery and cleanup call Core in process. Current local build evidence
is recorded in [verification.md](../apps/mobile/verification.md). Historical
Linux/Android UI evidence remains distinct from this merged revision's checks.

Remaining product acceptance includes real account reauthorization, physical
Android/iOS secure-store policy and phone lifecycle, separately authorized
model-specific controls, off-LAN journeys, broader localization/accessibility QA and
pixel equality. LAN route admission, cross-account reconciliation and nearby
sharing remain unavailable. The owner subsequently authorized the selected wall
AC acceptance recorded below; that does not authorize other appliances. Do not
inspect local pilot credentials by default.

Rebuild dependencies on each workstation; SDKs, app databases, secrets and generated
artifacts are not source handoffs. Follow [testing](testing.md) and the
[Android guide](development/08-android-native.md). Remote CI is not implied by
local success; earlier runs were blocked by the repository's billing limit.

## Open-source preparation: October 17, 2026

The owner requested preparation and obsolete design-study removal, then selected
[Apache-2.0](../LICENSE) after the initial license deferral. [Release readiness](open-source-readiness.md) records source
publication evidence and retained third-party notices. The existing Meiju
integration is included in the owner-requested source scope without duplicate
approval. The owner subsequently authorized cleaned-history synchronization and
public conversion; public visibility and enabled private vulnerability reporting
are verified in the [delivery record](publication-status.md).
Application assets and the 82-state prototype remain; obsolete image-generation
studies were removed from the public branch history after a private backup.
The subsequent [source audit](open-source-audit.md) inventories dependencies,
classifies scanner findings, adds the missing Flutter scaffold notice and records
one historical promotional-image rights issue plus a commit-email privacy choice.
No runtime contract, authority or physical compatibility changed.

## Same-account identity recovery: October 18, 2026

Native API 1.25 implements canonical Meiju IoT-user subjects and a separate trusted
legacy restoration preview/confirmation/discard path under
[ADR-0073](adr/0073-meiju-account-identity-recovery.md). An owner-operated diagnostic
confirmed login success followed by changed-login-ID refusal with matching client
identifiers. Complete matching inventory plus explicit owner consent can migrate
an intact legacy binding, preserving local IDs and settings. The old host closes;
control grants and approvals do not survive the transition. Ordinary login still
refuses identity conflicts. SQLite 5 and outer payload 3 are unchanged; canonical
inner sessions use version 3 and cannot be read by older binaries.
Synthetic evidence and pending live restoration are recorded in the
[App verification record](../apps/mobile/verification.md). This is not additional
physical-appliance compatibility or write authorization.

## Task status

The October 18 existing-emulator check retained the saved account and six device
cards, but both explicit state and inventory refresh returned `AUTH_REQUIRED`.
The owner then reported `ACCOUNT_IDENTITY_CONFLICT` for the same account.
The localized explanation is corrected, and static APK inspection distinguishes
login IDs from user IDs. Legacy identity reconciliation remains unresolved;
neither a successful reauthorization nor actual dishwasher-control availability
has been established. See the [identity investigation](development/05-midea.md)
and [runtime record](../apps/mobile/verification.md).

The October 18 appliance-detail revision shares product headers and explicit
buttons across refrigerator compartments and descriptor-driven dishwasher details.
Current route/component permissions gate visible actions and exact temperature
steps. Dishwasher parameters that share names with read-only observations remain
valid arguments to separately confirmed non-idempotent operations, without making
those observations editable. Existing native confirmation and Core validation stay
mandatory. The subsequent [ADR-0071](adr/0071-app-dishwasher-command-preparation.md)
implementation prepares exact reviewed dishwasher grants when opening a device;
each operation still requires separate trusted confirmation. Meiju refrigerator
writes remain unadmitted. See the [App verification record](../apps/mobile/verification.md)
for fictional widget/approval evidence and remaining physical acceptance.

[ADR-0069](adr/0069-profile-scoped-ac-features.md) integrates reviewed optional AC
scalar mappings, exact feature/value startup grants, prerequisite reads and Flutter
reported-state rows. ECO, breezeless, both swing axes, exact directions, display,
sound and profile-specific comfort/fresh-air settings are experimental. Mapping
facts and the remaining unsupported conditional/self-clean/timer functions are
listed in the [parity guide](development/10-meiju-ac-parity.md). Physical optional
feature acceptance and the selected official-App inventory remain open. No new
physical appliance write is authorized by this source integration.

The subsequent owner-requested maintenance revision keeps device refresh, remark
editing, reauthorization and removal confirmation on account details. Refresh
Devices updates the list in place, with one in-button loading indicator and no
page-level activity during refresh; new sign-in also opens details directly.
Cancel/Back dismisses an inline form first, and backgrounding invalidates its
confirmation scope. Only successful removal leaves for the account list.

The owner-requested platform redesign now opens protected sign-in directly from
the catalog. Tappable account cards lead to device-first details; remarks,
reauthorization and removal are grouped under Manage account. Technical inventory
inspection is nested under Troubleshooting, and unsupported revocation has no
production entry. Refresh feedback stays on details; partial retrieval, pending
authorization and interrupted cleanup keep explicit recovery paths.
Masked name fallback remains governed by [ADR-0068](adr/0068-app-account-display-names.md).
See the [App verification record](../apps/mobile/verification.md) for executed checks.

The owner requested silent three-second automatic state refresh while the App is
foregrounded. [ADR-0067](adr/0067-foreground-state-refresh.md) records this narrow
exception to the previous polling prohibition. Native device views now schedule
bounded reads without overlapping cycles or user operations; failures keep prior
evidence without notifications. Non-device navigation/background/disposal stop
the schedule, and it never refreshes inventory, prepares controls or replays a
write. Exact checks and emulator evidence are in the
[verification record](../apps/mobile/verification.md); provider rate limits and
physical-phone timing remain separate acceptance work.

The latest home-card surfaces use white for observed power on and slightly
gray-white for off/unknown, with dark text and outlined high-contrast controls.
Pending commands retain the previous observed color.
Native home cards now omit routine saved-observation/not-read captions and the
Saved locally header while retaining missing-value dashes and unsupported reasons.
The home-card revision extends observed on/off colors and per-button pending
feedback, hides unsupported whole-device shortcuts and adds distinct generated
refrigerator/dishwasher artwork. Card loading inspects only local reviewed
options/session grants and cached state; execution still prepares/rechecks the
current session. The current refrigerator and dishwasher gain no new control
admission. Prompt provenance and exact checks/artifact delivery are recorded in
the [artwork](../apps/mobile/artwork.md) and
[verification](../apps/mobile/verification.md) records. Physical-phone and new
appliance-control acceptance remain open.

The owner unlocked the existing Android emulator, and the ordinary Meiju app
launched successfully. The H5 preview server is stopped. The native app initially
showed no connected accounts and was left at Meiju connection for owner sign-in;
the owner subsequently supplied a real-device control-page screenshot. The next
presentation revision adds observed on/off colors and local pulse feedback and
removes routine accepted/verified bottom cards. Exact executed checks and APK
delivery are recorded in the [app verification record](../apps/mobile/verification.md).

The owner subsequently requested actual operation rather than preview data.
The climate fixture's language button is removed. A current ordinary Meiju APK
has passed build/archive checks and installed in place on the host-GPU Android
emulator. At that earlier handoff, startup was blocked by locked user storage;
the owner has since unlocked it as recorded above. Real H5 account/device
integration remains unimplemented; no temporary
HTTP control bridge was introduced. See the [runtime handoff evidence](../apps/mobile/verification.md#native-runtime-handoff-and-language-button-removal-october-17-2026).

The owner-requested H5 development preview now opens the current shared climate
controls with fictional, memory-only data and three airflow-capability scenarios.
It compiles independently of native FFI and cannot control appliances. The
[app verification record](../apps/mobile/verification.md#h5-development-preview-october-17-2026)
records the executed browser and focused regression checks. This does not add a
browser Core backend or change outstanding native/physical acceptance.

The subsequent scrolling investigation identified forced SwiftShader rendering
as the main emulator bottleneck. Paint isolation and host GPU selection reduced
mean profile raster time from 166.484 to 13.966 ms on the same fictional grid;
physical-phone performance remains open. The initial `flutter drive` command
omitted its retention flag and uninstalled the package after profiling, losing
the emulator's prior local account/device data. The owner was informed; the
ordinary APK is restored but requires account reconnection. The corrected
procedure and incident are recorded in the app verification report.

The October 17 owner-directed Flutter revision restores Window Light materials
and expands only actual locally permitted controls into rows. Swing-only and
fixed-position layouts are independent per axis; unsupported rows are hidden.
Existing power/mode/fan/eligible-temperature commands continue through Core, with
native session-grant inspection separating saved metadata from current authority.
The generic swing/display/sound/comfort paths have fictional widget evidence only;
current Meiju adapters still do not admit those actions. Exact-model mappings and
physical acceptance remain open; see the [app evidence](../apps/mobile/verification.md).
Android 15 arm64 process checks now pass the Native Mock click journey and
expanded-control smoke test, including direct mode/temperature receipts, hidden
unsupported rows and Chinese detail rendering. The ordinary Meiju APK is restored
with saved cards/preferences intact at that earlier checkpoint; its then-current
account returned `AUTH_REQUIRED`. The later profiling incident above supersedes
that emulator-state record. New Meiju detail/control acceptance remains blocked.

The owner subsequently requested complete parity with the selected AC's Meiju
control page. The [AC parity review](development/10-meiju-ac-parity.md) tracks this
unfinished task separately from the completed inline-power/UI revision. The
Flutter panel and generic additional-function controls do not establish new Meiju
controls. Exact-model screen references and protected protocol mappings remain
required for the rest of the feature inventory.

The optional-subtype admission defect is now corrected under ADR-0066. The final
temperature APK was installed as an in-place emulator upgrade with the saved
account and Chinese preference retained. Two owner-authorized commands changed
the selected wall AC target from 28 to 27.5 and back to 28 Celsius. Both receipts
were `accepted`, and separate bounded cloud reads matched each target; neither
receipt is `verified`. The page stayed on the AC controls. This establishes the
selected account's command/read-back path, not physical temperature behavior or
full Meiju parity. See the app verification record for artifact identity and
test limitations.

| ID | Task | Current status | Remaining acceptance |
|---|---|---|---|
| T00 | Documents and contracts | Implemented | Keep references synchronized |
| T01 | Toolchain and workspace | Implemented: Rust 1.96, Cargo workspace | Additional target builds |
| T02 | Domain DTOs and validation | Shared/component-scoped capability definitions, per-adapter descriptor admission, transactional rediscovery identity guards, and stored-metadata operational validation implemented | Expand tests with new capabilities and actual model constraints |
| T03 | Mock and controlled failures | Implemented | Add protocol-specific faults with adapters |
| T04 | Execution and routing | State-match and bounded correlated-completion verification, explicit independent-field decomposition, explicit device batches with whole-list preflight/shared deadlines, scoped authorization, and full-snapshot observation validation with per-property freshness limits and typed single-command elevated user approval and separate trusted native confirmation functions implemented | Real-platform Flutter consent acceptance, reviewed consequential-action mappings, model-specific freshness/correlation evidence and real protocol guarantees |
| T05 | Storage and credentials | SQLite journal/cache, schema-1/2/3/4 backup/upgrade to schema 5, bounded detailed receipts, explicit cache/history clearing, retired-ID protection, and native account vault with persisted references, account leases, explicit Meiju pending-session recovery and same-account reauthorization, recoverable local removal without credential decryption, Linux Secret Service composition, and an Android Keystore/JNI backend with synthetic JVM and isolated AOSP Android 15 runtime verification implemented; iOS Keychain backend has portable/target checks and isolated iOS 26.2 Simulator round-trip/restart evidence | Real provider identity/session acceptance, Android Keystore/phone acceptance, physical iOS Keychain/entitlement policy and provider revocation |
| T06 | CLI | Mock queries/writes and explicit batches; experimental Meiju protected terminal connection, account list/details/rename/removal/refresh/recovery/inventory, local organization, Core reads, exact receipt lookup/history, redacted diagnostics and explicit owner-controlled power, fan, mode activation and profile-scoped temperature grant files implemented | Live CLI login/read/control acceptance and adapter-specific diagnostics evidence |
| T07 | Android embedding probe | Linux native ABI and foreground lifecycle tested; versioned [34-method UI contract 1.22.0](native-ui-contract.md), schemas and native examples implemented; API 28 arm64/x86_64 and combined AAR builds, exports and 16 KiB ELF alignment verified; isolated Android 15 test APK passes native Core/Keystore/lock/restart, grant revocation, trusted approval and uncertain-write/forced-stop recovery checks | Merged APK/runtime evidence in the app verification record; physical Android Keystore and phone acceptance; see the [Android guide](development/08-android-native.md) |
| T08 | Local MCP | Mock/Meiju reads and exact receipt lookup; explicit Meiju startup grants admit command planning/execution with unchanged Core policy; protocol, outcome and shutdown harnesses implemented | Target AI host validation and real Meiju read/control acceptance |
| T09 | First native authentication | Scoped community flow admitted under [ADR-0022](adr/0022-meiju-community-cloud-admission.md); native codec, restricted HTTPS and protected connection tested; one real mainland account login and independent saved-session refresh returned six appliances | Native account connection/list/details/rename/removal/refresh/recovery, protected saved-inventory inspection and read-only control-option inspection before grants and local session-grant inspection are callable; connection uses separate secret buffers and can reauthorize an explicitly selected pending account without resetting intact bindings. Verify real reauthorization identity, physical-phone storage and off-LAN behavior; emulator login and saved-account reuse passed |
| T10 | First real adapter | Default read-only Meiju adapter admits six exact models and checks protected complete-inventory membership before live access and retains private product mapping evidence for reviewed profile-specific temperature contracts; real Core/native reads passed for six cards/seven components; protected product 760Y0026 additionally has synthetic-tested optional read-only additional-option observations. Explicit experimental AC power, vendor fan, power-on/mode and profile-scoped temperature controls, scoped power probe, independent native AC grants, and profile-scoped dishwasher starts and independently granted pause/resume/cancel operations (including explicit additional options for product 760Y0026) through trusted confirmation are synthetic-tested with conservative cloud outcomes; bounded LAN discovery/collection, V3/ordinary packet codecs, standard AC/E1/CA read candidates, one-operation TCP exchanges, account-bound v2 credential lookup, protected candidate reconciliation and composed family reads with message-version negotiation have fictional evidence only | Physical-phone integration, reauthorization identity, explicit cross-account reconciliation (saved candidate inspection is implemented), broader physical AC power acceptance beyond the owner-observed shutdown below; physical AC temperature/preservation and dishwasher start acceptance, remaining unreviewed dishwasher mappings and refrigerator controls, then LAN and model-specific physical/off-LAN acceptance |
| T11 | Flutter room/card UI, platform management, and dynamic device controls | Flutter app merged against Core 1.22 with room/card metadata, protected account input, confirmed cleanup and local inspection; Window Light visual specification retained | Continue the [functional acceptance checklist](ui-functional-spec.md), [82-state inventory](ui-screen-inventory.md), and [one-skin extension contract](ui-window-light.md); physical-phone and visual acceptance remain unverified. No HomeVerb login; rooms preserve device identity |
| T12 | Second protocol family | Planned after T10 | Equivalent capability calls on heterogeneous hardware |
| T13 | Security review | Mock component, argument, route, composite-policy, uncertainty, and diagnostic allowlist/marker tests implemented | Real credential, dependency, live-adapter log and diagnostic review |
| T14 | Release | Planned | License decision, packages, actual remote CI and device evidence |
| T15 | Foreground local-network authorization sharing in Settings | Planned | ADR-0008 contract/security review; peer verification and bilateral consent; secure import; provider eligibility evidence; honest scope and revocation; lifecycle cleanup and interruption tests |

T15 does not establish real-platform sharing support. Its security and compatibility evidence is required before packaging any enabled sharing path.

## Next development order

Use the [progressive guides](index.md). UI development can proceed against the native room/card request contract and locale catalogs. Local organization, schema migration, detailed-receipt retention, and the native account vault with Linux secret-store composition are implemented. The owner selected Midea/Hualing air conditioners, dishwashers, and refrigerators. Follow the [pinned community-source integration plan](development/05-midea.md): secure account connection and automatic cloud device retrieval, cloud reads through Core, authorized cloud writes with physical evidence, then additional LAN connections. The complete mobile journey must work away from home without a gateway or manual IP/Token entry. A read-only stage or LAN-only control does not complete T10.

Research and offline contract/protocol implementation can proceed before exact model details arrive; account/region evidence gates real authentication and model/firmware evidence gates physical compatibility claims. The inspected refrigerator LAN setter is empty, while cloud mappings include compartment temperature settings; assess those routes separately. Android storage has synthetic/emulator evidence; physical-phone acceptance remains pending. The iOS Keychain backend and production framework have isolated Simulator runtime evidence; signed physical-app and phone acceptance remain pending; see the [iOS guide](development/09-ios-native.md). Preserve specific blockers for live access and mobile tooling. Authorization sharing remains gated by ADR-0008.

The [LAN integration sequence and remaining gates](development/05-midea.md#implemented-native-lan-building-blocks)
separate implemented helpers from unfinished endpoint/model admission, trusted
network selection, Meiju connection-link integration and physical acceptance.
The [typed Core/Session linking foundation](adr/0052-trusted-connection-linking.md)
is synthetic-tested, with transactional storage and unchanged rediscovery guards.
Meiju has not opted in: protected route evidence, full-set rediscovery and a
trusted mobile consent flow are still required before publishing a LAN route.
API 1.20 exposes three separate trusted C linking functions under
[ADR-0061](adr/0061-native-connection-link-bridge.md), backed by the existing Core
checks and a bounded session preview registry. This does not satisfy Meiju's
protected endpoint/model evidence or implement the Flutter confirmation screen.
The source's default AC decoder and the six cloud models do not establish a LAN
model allowlist. These gaps do not require reimplementing the completed codecs.

The [trusted foreground scan](adr/0054-trusted-meiju-lan-candidate-scan.md) now
composes selected-device discovery/reconciliation through typed Rust Session
preview and consuming confirmation. Synthetic tests cover lifecycle, identity,
lease and deadline boundaries. It does not fetch keys, read state or publish a
route. API 1.18 adds the [three trusted C scan functions](adr/0058-native-lan-scan-bridge.md)
with bounded, one-use previews and endpoint-free results. Android's
`NativeLanNetworks` now supplies foreground assigned-interface selections and
exact revalidation under [ADR-0059](adr/0059-android-lan-network-selection.md).
Flutter controller/permission integration, consent UI and the remaining
model/endpoint/linking gates are unfinished.

The selected refrigerator's [write evidence review](development/05-midea.md#refrigerator-write-evidence)
establishes model-specific manual bounds but leaves encoder preservation and
temperature step unproven. Keep its existing reads available while researching
that gate; Flutter can implement the read-only compartment view now.

Each completion record names the task, changed interfaces, commands actually run, results, skipped checks, compatibility effects, and external actions. All such records are English.

The owner requires one local commit per completed, verified phase. Pushing or publishing requires separate authorization. The existing Mock MVP is committed as the reusable baseline. Independent Flutter work starts from the [native UI contract](native-ui-contract.md); frontend development must not wait for real-device acceptance to implement the available room/card/control flows. Subsequent account/cloud work adds reviewed contracts and implementation in separate verified phases.

The [iOS package builder](development/09-ios-native.md#2-build-and-check-the-native-code)
passed actual macOS CI on `d4525b1`, including separate device/Simulator slices,
canonical C module metadata, local ad-hoc seals and consumer-link checks. The
[isolated Simulator runner](development/09-ios-native.md#4-run-isolated-simulator-acceptance)
passed Mock/Core and fictional Keychain round trips across separate processes on
`d02b10a` in macOS job `116902983930`. Physical app signing,
Flutter embedding/runtime and Keychain/phone policy acceptance remain open.


### Flutter AC permission integration

The trusted Flutter host now prepares reviewed AC controls automatically when
opening a device, following the owner-approved [default policy](adr/0065-default-app-device-controls.md).
The former session-permission selection/review/revocation UI has been removed.
See the [verification scope](../apps/mobile/verification.md#default-device-controls-owner-directed-revision).
One real account has now logged in and retrieved its devices on the Android
emulator, and the selected AC returned live power-off and temperature readings.
The updated APK preserves this account and Chinese preference. The owner confirmed
physical shutdown after one App power-off command; its receipt remains unknown.
A separate restoring power-on command was accepted and a subsequent read reported
on. See the [bounded evidence](../apps/mobile/verification.md#default-control-apk-and-owner-observed-power-acceptance);
other controls and broad model/phone acceptance remain pending.
The Flutter app now uses slang for en-US/zh-CN with a persisted Settings > Language
choice and system fallback. See the [internationalization guide](../apps/mobile/README.md#internationalization).
The previous emulator sign-in returned `SECRET_STORE_UNAVAILABLE` with no secure
screen lock configured. The owner has now configured the emulator lock, and the
system reports secure/unlocked state. Flutter now checks that prerequisite before
credential entry and submission. Subsequent real login, account persistence across
upgrade and the bounded power test are recorded above; readiness alone was not
provider or Keystore success.

### Refrigerator and dishwasher screenshot parity research

The subsequent appliance panel revision implements compartment temperature cards,
optional read-only refrigerator mode/variable-zone observations, and a dedicated
dishwasher status, program-button and supply-reminder panel. Program buttons
prefill only the selected enum and retain additional-argument/route/approval
requirements. Existing descriptors require an explicit inventory refresh for the
new read capability. This is not complete screenshot parity: refrigerator writes,
current-model dishwasher admission and provider reports remain separate evidence
gates. See the [panel evidence](development/12-appliance-command-evidence.md#appliance-panels-and-optional-refrigerator-readings).

Live inventory refresh retained all six cards and exposed the new refrigerator
readings. Dishwasher read diagnosis identified its empty `operator` sentinel;
the parser fix preserves other valid readings without adding cycle semantics or
control authority. Live delivery results are recorded in the App verification log.

The October 18 owner screenshots expand the appliance-detail scope beyond target
and cycle buttons. Track every control, provider setting and report in the
[screenshot checklist](development/12-appliance-command-evidence.md#owner-screenshot-parity-checklist).
The official APK bridge/cache/transparent transport and a public historical
refrigerator plugin have now been statically traced. The expanded hash-pinned
inspector passed. Public catalog access succeeded without account credentials;
no appliance write occurred. Current exact-model plugin selection, remaining
screenshot feature mappings, dishwasher power admission and physical verification
remain open. Historical demo dispatch and visible official controls do not by
themselves change HomeVerb's capability or grant contracts.

The subsequent [dishwasher power phase](adr/0072-profile-scoped-dishwasher-power.md)
implements profile-scoped on/off requests and App buttons under native API 1.24.
The source comparator checks 18 product/option vectors without executing upstream
code. Exact grants, trusted confirmation and uncertain-write handling apply.
Current-owner product matching and real power acceptance remain pending; this
phase does not close refrigerator parity or the screenshot's provider reports.

The current appliance-controls APK is now built and archive-verified with both
Android ABIs and rebuilt API 1.24 Core. A complete SDK already existed outside the
standard paths; the earlier missing-SDK conclusion is corrected in the
[delivery record](../apps/mobile/verification.md#current-appliance-controls-apk--october-18-2026).
No installation or real-device operation followed. Current refrigerator protocol
identity and physical verification remain open.

Further APK inspection traced the bundled service-center consumables POST,
home/device association and categorical-versus-ratio display. The public
`customer-service` module lookup did not return an artifact. See the
[consumables evidence](development/12-appliance-command-evidence.md#current-apk-consumables-service-trace).
Current-model association, authenticated service responses and maintenance-reset
semantics are still required; no new production endpoint or control is enabled.
