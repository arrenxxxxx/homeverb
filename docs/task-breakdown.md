# Implementation status and next tasks

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
synthetic-tested; protected fan admission, native grants and App controls remain
the next phase. See
[ADR-0072](adr/0072-mijia-protected-core-reads.md) and
[ADR-0073](adr/0073-mijia-native-account-routing.md).
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

## Task status

The October 18 appliance-detail revision shares product headers and explicit
buttons across refrigerator compartments and descriptor-driven dishwasher details.
Current route/component permissions gate visible actions and exact temperature
steps. Dishwasher parameters that share names with read-only observations remain
valid arguments to separately confirmed non-idempotent operations, without making
those observations editable. Existing native confirmation and Core validation stay
mandatory. Ordinary App startup still does not grant elevated dishwasher actions;
Meiju refrigerator writes remain unadmitted. See the [App verification record](../apps/mobile/verification.md)
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
