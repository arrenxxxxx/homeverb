# UI functional handoff

Status: owner-approved design scope, October 15, 2026. The Window Light prototype is a visual specification with fictional data, not an implemented mobile application or evidence of real-platform support.

Use the [complete atlas](../output/prototypes/homeverb-window-light-v1/index.html), [82-state inventory](ui-screen-inventory.md), [requirement coverage](../output/prototypes/homeverb-window-light-v1/coverage.md), and [Window Light implementation guide](ui-window-light.md) together. The atlas contains 21 boards, 82 states, 14 review journeys, and 201 action branches. A journey is a review sequence; its adjacent screens are not necessarily a direct application transition. Branch buttons simulate navigation between images, not device actions.

Account-flow amendment, October 15, 2026: Midea Meiju authorization must automatically retrieve existing bound devices and support subsequent cloud control away from home. The October 16 P08 result image and its five action branches supersede the manual Find Devices step. The base result uses the native combined-call contract; connecting, empty, partial, failed and interrupted runtime variants still require separate visual acceptance.

This document expands [mobile requirements](mobile-app.md). The [embedded Core boundary](sdk.md), schemas, and ADRs remain authoritative for execution semantics. Preserve existing Core work; do not infer a new network service from this handoff. The status table below describes this frontend branch against its pinned native API 1.14.1; newer main-branch Core features require separate integration.

The [native UI contract 1.22.0](native-ui-contract.md) supplies 34 callable methods, schemas, actual Mock request/response examples, version negotiation, and screen coverage. It includes account management, explicit pending-account reauthorization with intact protected identity, separate cache/history cleanup, protected saved-inventory/control-option inspection, saved cross-account conflict diagnosis and local session-grant inspection without provider traffic. Three separate trusted scan functions support selected-device/network preview and consuming consent; they require platform interface/permission integration and never add a route. Sharing, identity merge, provider revocation and full-profile deletion remain unavailable; UI previews must stay separate from production native state.

## Product boundary

- Open directly into Devices, without HomeVerb registration, login, or an account center. Only third-party authorization and appliance pairing require authentication.
- Keep exactly three root destinations: Devices, Platforms, Settings. The Platforms page is titled **Smart Appliance Platforms**. Sharing and recent operations are nested in Settings.
- Flutter calls Core through in-process native bindings. No application backend, CLI subprocess, HA/NAS prerequisite, public control API, relay, cloud synchronization, or daemon is introduced.
- All discovery, authorization, reads, writes, and sharing are explicit foreground operations with bounded lifetimes. Opening a page may read local metadata; it must not silently scan, refresh accounts, or start a receiver.
- The explicit Meiju Connect Platform flow includes automatic account-device retrieval after authorization. Daily reads/controls use the vendor cloud at home or away; no HomeVerb backend or LAN prerequisite is introduced.
- The only shipped skin is **Window Light**, stable ID `window_light`. Settings > Appearance contains one selected option and its preview. Future skin extensibility is required; additional skins are not part of this release.

## Feature and recovery contract

| Area / screens | Required behavior | Validation and recovery |
|---|---|---|
| Launch and navigation: D01, D04, P01, S01 | Restore local organization and the selected root; first launch offers Add Platform and Receive Authorization. Phone bottom navigation becomes a three-item rail on wide layouts. | An empty device list is distinct from a loading error. No authorization requirement for local Settings or Rooms. Returning to foreground does not trigger a refresh. |
| Device home: D01, D02, D09, D11, D12 | All, ordered custom rooms, and Unassigned filters; card count; explicit refresh; separate card-open and supported quick-control targets. Narrow widths and larger text use one column. | Preserve last observations and their age. A stale snapshot is not evidence of being offline. `inspect_account_inventory` can explain saved membership for existing matching cards, without treating membership as online status or creating cards from unpublished bindings. Missing/null state is unknown, never false or zero. Keep room and card identity stable across refresh. |
| Search and card actions: D03, D05, D10 | Local search by alias, adapter label, and room. Clear/cancel search. Accessible menu and long-press provide Rename, Move, Pin, Reorder, Information. | Duplicate aliases remain allowed; actions use exact IDs. Keep selection separate from control. Reorder conflicts reload current IDs; do not silently drop hidden cards. |
| Rooms: R01–R05 | Create, rename, order, delete, and assign one or multiple devices. Suggestions are shortcuts, not a fixed room taxonomy. Deletion moves devices to Unassigned. | Names are trimmed, 1–80 characters, without controls, unique after Unicode lowercasing. Show duplicate/empty/invalid input inline and retain input on failure. All and Unassigned are virtual filters. No change to device authority, credentials, routes, or identity. |
| Device information: D06–D08 | Local alias, room, platform, model, supported capabilities, and available routes; device-specific connection preference. | Send complete current alias/room/favorite metadata when changing a preference. Null alias restores the adapter name. Do not infer a route from platform branding. A local-only selection cannot silently fall back to cloud. |
| Climate: C01–C04, C14 | Supported mode, fan, target input/stepper, power, observed temperature, refresh, information, and device history. | Writable bounds, step and enums come from the exact component action parameters; units come from the property. Readable observations can lie outside write bounds and must remain visible. Use explicit `inspect_control_options` to build permission forms before trusted startup grants; its output is not an executable descriptor or approval. Use `inspect_session_grants` for current local permissions after reopening; saved actions alone do not establish authority, and grants do not replace planning or per-action consent. Validate before planning. Keep requested values separate from observations. Accepted and unknown requests do not replace observed targets. Refresh reads; it does not replay a write. |
| Other controls: C05–C09, C13 | Light brightness/color temperature, fan speed/oscillation, media source/play/mute/volume, fridge/freezer components, cycle programs/actions, and read-only sensors where described by capabilities. | These are fictional examples, not universal brand templates. Hide unsupported controls or explain unavailable conditions. Resolve capabilities by both capability and component. One fridge remains one device, with different compartment limits. |
| Actions and results: C10–C12, S05, S06 | Consequential-action confirmation; show accepted, verified, failed, unknown, and partial with words and evidence. Explicit multi-device actions show per-item results. | Trusted approval must match the intended operation. ADR-0043 supplies separate trusted C prepare/execute/discard functions; Flutter consent UI is implemented with fictional native tests; real-platform acceptance remains pending. Display the immutable preview and retain its temporary ID only in trusted controller memory. Do not convert a UI boolean into a privileged grant. Already-started actions may continue after cancellation. Do not automatically retry non-idempotent unknown writes or the whole partial batch. |
| Add device: A01–A08 | Explicitly retrieve an account's devices again or use nearby discovery; show finite progress/cancel; resolve identity ambiguity and assign rooms. Nearby discovery may require selection and local permission. | Initial Meiju authorization adds supported account devices automatically without this extra selection flow. Cloud retrieval requires no LAN permission. Denied nearby permission has an Open System Settings path. Names do not prove identity; merging requires adapter evidence and a supported safe operation. Add success does not claim fresh state. |
| Platform catalog/accounts: P01–P03, P09, P13 | Multiple accounts per platform, region, local account name, authorization method, granted scope, device count, last check, and compatibility. Search catalog; display unavailable integrations honestly. | Do not enable a platform without native-client admission evidence. A last check is not a live online indicator. Compatibility is specific to model/region/account/protocol, not a broad brand claim. |
| Platform authentication: P04–P08, P10 | Adapter-supported authorization or appliance approval; explicit Continue, Cancel, timeout, expiry, and reconnect. Securely save credentials. For Meiju, automatically retrieve accessible bound appliances and add supported cards within the initiated foreground flow. | Preview account/region scope and retrieval before connecting. Distinguish authorization from retrieval success, empty, partial, or failed results. Offer explicit retrieval retry without unnecessary login; show unsupported devices honestly. Preserve IDs and local organization on reauthorization, resolve ambiguous identity, and require no LAN scan or per-device IP/Token. Keep secrets out of UI persistence, URLs, logs and diagnostics; cancellation cleans resources. |
| Local removal/revocation: P11, P12 | Separate confirmation for removal on this installation and provider-side revocation, only when supported. State the actual affected account and scope. | Local removal does not revoke copies elsewhere. Revocation may affect other installations; never claim it succeeded without provider evidence. Handle local cleanup failures explicitly. |
| Sharing entry/eligibility: S01, H01, H11 | Send and Receive under Settings. Select an eligible account; show why ineligible/expired/device-bound grants cannot be sent and offer independent authorization. | Eligibility requires adapter evidence, including refresh-token behavior. Prefer separate member grants. Do not invent narrower permissions or expiry than the provider supports. Merely opening Settings never advertises. |
| Peer discovery/verification: H02, H03, H05, H13, H14 | Explicit time-limited receive mode; visible local name; finite peer scan; select peer; compare codes on both screens; cancel/decline/mismatch. | Peer names are untrusted labels, not authentication. No credentials before verified peer identity and both parties' consent. End and erase ephemeral material on mismatch/timeout; new attempt uses a new session. LocalSend-style interaction does not imply LocalSend protocol compatibility. |
| Scope and consent: H04, H06 | Sender reviews recipient, platform, account, actual access, expiry and revocation; recipient sees sender/code/account/scope and explicitly accepts. | State when all devices are included. Later device selection and room assignment are local organization, not access restriction. Do not send credentials during preview or before consent. |
| Import/conflict/results: H07–H10, H12, H16 | Distinguish transfer, secure import, and authorization validation. Existing account offers Keep Existing or explicit validated replacement. Secure-store unavailability blocks import. | Keep the working credential on failed replacement. No plaintext fallback. Imported-but-invalid credentials require reauthorization or local removal. Lost final acknowledgement is unconfirmed; consult recipient state before any new transfer. |
| Sharing lifecycle: H09, H12, H15 | Progress/cancel, explicit interruption result, local record. Recipient connects independently after valid import. | Backgrounding, timeout, cancel and completion close sockets/subscriptions. Cancellation is not remote revocation. Forced termination cannot guarantee callbacks; reopening shows known records without automatic resend/listening. |
| Appearance: S02, S11 | One Window Light tile marked Current; preview and return; text follows system accessibility settings. | Resolve a missing/unrecognized local skin ID to `window_light`. No dark/system theme picker, custom image import, download catalog, or placeholder skins. Skin changes never change device/credential state. |
| Local preferences: S03, S10 | Sharing name; connection defaults and system-permission status. Device-specific route choice takes precedence where supported. | A sharing name is advertised only during explicit sessions. Preferences are local; no cloud account or synchronization. Storage failures retain the previous selection and show a retryable local error. |
| Data/history: S04–S06, S09 | Separately clear cached state or retained operations; review full local-data removal. Show profile history, device history, and exact receipt detail. | Current device receipt retention is bounded to 100; absence does not prove no past action. Retired/not-found IDs have no replay action. Sharing history is a separate planned capability. Keep removal scope explicit; preserve execution safeguards and active journals. |
| Diagnostics/about: S07, S08, S12 | Render the Core allowlisted diagnostic report, preview before local save; version, privacy, licenses and evidence-based compatibility. | Export exactly the previewed report, never a raw aggregate of device/state/receipt APIs. No auto-upload. Show truncation, unknown error mapping, and unavailable fields. Counts are local metadata, not online status. |
| Adaptive/accessibility: D12, W01, W02 | One-column large-type layout; wide rail, grid and split-view controls. Accessible semantics, focus order, keyboard actions, labels and touch targets. | Test long room/device names, en-US and zh-CN, text scaling, insets, landscape, keyboard visibility, screen readers, reduced motion and high contrast. Never shrink essential text to preserve a two-column layout. |

## Existing integration versus remaining development

Status refreshed October 16, 2026 against source checkpoint `2d4209c`. The
[verification record](../apps/mobile/verification.md) distinguishes native Mock,
contract-fixture, preview and Android emulator evidence. Implemented UI is not
proof of a real provider or physical appliance operation.

| UI requirement | Implemented on this branch | Remaining development or acceptance |
|---|---|---|
| Rooms, cards, search and ordering | Native metadata calls, local search/filtering, create/rename/delete, full-order permutations, atomic assignment and accessible reorder menus; Mock regressions pass | Physical-phone and real multi-device acceptance |
| Descriptor-driven controls | Exact component metadata, typed scalar forms, compartment views, outcome evidence and trusted elevated-action confirmation; Mock/fixture tests pass | Meiju startup-grant UI, real-platform consent and model-specific physical controls; unsupported/high-risk actions remain unavailable |
| Explicit multi-device results | Selection, exact command batches, nested per-item receipts and no automatic replay | Real multi-device behavior; room names never replace explicit device IDs |
| Discovery and identity | Explicit Mock discovery; protected Meiju connection automatically retrieves cards; partial/empty/error fixtures | Real Flutter account retrieval, native LAN permission/consent integration and identity reconciliation; arbitrary identity merge remains unavailable |
| History | Profile/device history, exact receipts, nested outcomes and retired/not-found handling | Physical lifecycle recovery; sharing records remain unavailable |
| Diagnostics | Allowlisted preview and exact-byte export; cancel/save/failure fixtures and Android emulator system-picker evidence | Physical-phone and iOS chooser acceptance |
| Lifecycle | Worker isolates, foreground gating, cancellation, late-reply fencing and uncertainty preservation; Mock and emulator journeys | Physical/OEM behavior, all real-provider flows, iOS and forced termination during actual operations |
| Accounts | Protected input, automatic retrieval, saved-inventory inspection, recovery/reauthorization, metadata and scoped local removal; fixtures/native admission checks | Real Flutter login/Keystore and off-LAN acceptance; browser/code challenges and provider revocation remain unavailable |
| Nearby authorization sharing | Isolated preview states and explicit unavailable gates | ADR-0008 protocol, provider eligibility, peer consent, secure import and interruption handling; no live sharing |
| Appearance and local preferences | One Window Light option, fallback/persistence, sharing name and route preferences | Pixel equality and the committed surface experiment's Android rendering; physical accessibility/performance |
| Cache/history/full-data clearing | Separate cache/history confirmation, local reload and replay-protection tests | Physical acceptance; full-profile deletion remains unavailable |
| Localization and accessibility | Shared English/Chinese catalogs; English Flutter UI, semantics, high contrast and large-text tests | Full Flutter locale wiring, screen-reader acceptance and remaining system-input/contrast cases |

The source-only surface checkpoint passed all 84 Flutter tests and static
analysis. It has not been rebuilt or exercised on Android. Main's Core 1.20
integration, merged language assets and fresh native/APK regression are still
required before treating a merged build as the phone-test baseline. Pixel work is
paused by owner decision; that does not waive remaining functional acceptance.

## Loading, empty, error, and navigation rules

API 1.20 adds separate trusted connection-link preview/confirm/discard functions.
Use the [linking handoff](native-ui-contract.md#trusted-connection-linking-api-120)
for future adapter-admitted routes. Mock/Meiju currently return unsupported; do
not enable a linking button from a scan result or treat visual fixtures as native
route evidence. The actual Flutter confirmation screen remains pending.

API 1.19 account summaries supply `last_complete_inventory_at` for platform
details. Label it as the last complete device list, show null as no recorded
time, and retain the device-clock caveat. It is historical acquisition metadata,
not current authorization, appliance online status or successful card publication.

Each data-dependent page needs loading, empty, recoverable-error, and populated states even when one representative state is pictured. Use the same Window Light surface and semantic status components. Local list failures offer explicit reload; a network retry remains a new bounded operation. Disable duplicate submit while a mutation is pending and preserve user input after validation/storage failure. Do not show success before the native result is known.

Back/Cancel must have a deterministic parent or recorded caller. Modal sheets return to their underlying page; authorization returns to the initiating account/platform flow; device history returns to that device. Shared pages such as Rooms preserve the entry source. Leaving an active discovery/authentication/sharing flow cancels its resources; leaving an already-sent write preserves its recorded uncertainty. Dismissing a result does not dismiss the underlying safety obligation.

The 82-state atlas does not exhaust every transport error combination. Native errors must map to actionable text without hiding the original outcome. Sheet variants reuse the same component; loading/empty/save-error variants and system dialogs are acceptance cases below rather than additional product destinations. The provider browser, system permission dialog, secure-store prompt and export chooser are outside app skin control.

## Development acceptance checklist

These are full acceptance gates, not a list of wholly unimplemented features.
They remain unchecked until each complete scope is evidenced. Consult the status
table above and the UI-01 through UI-15 mapping in the [verification
record](../apps/mobile/verification.md) for implemented subsets and remaining gaps.
Images alone never satisfy a gate.

- [ ] **UI-01** Implement all three roots, nested routes and 82 specified states; check all 201 branch destinations and system-owned transitions, plus caller-aware Back/Cancel.
- [ ] **UI-02** Verify first launch, no HomeVerb login, empty rooms, search, card/open versus quick action, and accessible alternatives to long-press/drag.
- [ ] **UI-03** Exercise blank/duplicate/long/control-character room names, duplicate aliases, stale reorder permutations, atomic multi-device assignment and deletion without loss of identity or authority.
- [ ] **UI-04** Drive forms from exact component metadata; test fridge/freezer different ranges, read-only/unknown values, invalid steps, missing capabilities, and permission-denied controls.
- [ ] **UI-05** Verify accepted/verified/failed/unknown/partial, stale/missing observations, separate requested values, per-item evidence, and no blind write replay.
- [ ] **UI-06** Exercise finite discovery, permission denial, cancellation, no results, ambiguous identities, add failures, and local room assignment after success.
- [ ] **UI-07** Establish real native authorization evidence before enabling a provider; test region/account selection, challenge, cancellation, expiry, reauthorization and safe credential storage. For Meiju, automatically retrieve bound devices without re-pairing/manual credentials; test unsupported/empty/partial/failed results, retry without unnecessary login, and preserved IDs/rooms/aliases/favorites.
- [ ] **UI-08** Verify local account removal versus remote revocation and display actual affected grants; test unavailable and partial cleanup outcomes.
- [ ] **UI-09** Complete ADR-0008 review before enabling sharing; verify eligibility, two-sided code matching/consent, scope, conflict handling, secure-store failure, invalid import and acknowledgement loss.
- [ ] **UI-10** Background/cancel/terminate every network flow. Reopen without scan, refresh or automatic resend; inspect uncertainty from durable records. Verify both mobile OSes separately.
- [ ] **UI-11** Render exactly one local skin option, persist its ID, preview without network side effects, restore/fallback safely; later additions must use the same registry contract.
- [ ] **UI-12** Preview and export exactly the allowlisted diagnostic report; exercise cancel/save errors and absence of credentials/live identifiers. Test bounded/retired history and safe clearing semantics.
- [ ] **UI-13** Verify all screens at narrow/regular/wide sizes, 100%/150%/200% text scale, long names, keyboard open, en-US/zh-CN, screen reader, reduced motion and high contrast.
- [ ] **UI-14** Build and run the actual Flutter/native app on supported phones; profile scrolling and modal transitions with Window Light assets. Compare against approved screenshots before claiming visual parity.
- [ ] **UI-15** Verify supported Meiju cloud reads and owner-authorized writes with Wi-Fi disabled, no home computer/gateway, and no LAN discovery permission. Show stale cloud evidence and uncertain writes honestly. Update the P08 retrieval/result variants and atlas transitions before claiming visual coverage of this account journey.

## Scope control

Do not add cloud sync, automation, background listeners, skin downloads, executable theme plugins, a fourth tab, charts or a HomeVerb account to fill apparent gaps in the mockups. Keep [task status](task-breakdown.md) and [compatibility evidence](compatibility.md) explicit. Functional scope changes should update this document, the screen inventory, atlas transitions and acceptance checklist together.


## Owner-directed climate interaction revision

For a device exposing climate and power capabilities, keep the user on its control
page after a power, mode, fan or target-temperature operation. As revised by the
owner on October 17, expand supported controls into rows: mode, fan speed,
horizontal airflow, vertical airflow, then additional settings. Omit unavailable
controls instead of rendering placeholders. Never
silently replace a mode-only operation with a power-on command: disclose the existing
activation effect when that is the action exposed by the adapter.

Show pending work and the retained outcome inline, with optional operation details.
A single bounded foreground read may follow the command; it is not polling or proof
that the command caused the observed state. Preserve unknown outcomes and prevent
replay. Device switching, navigation and background transitions invalidate old
callbacks, drafts and follow-up work. Large text wraps options and row contents.
Quick card power leaves the user on the home page.

The October 17 revision restores the shared Window Light materials and compact
charcoal temperature/power summary. Temperature adjustment remains in place: use only descriptor bounds
and steps, send once on slider release and discard a drag across background/resume.
Never optimistically replace observations. Additional independent low-risk
descriptor setters appear as rows. Inspect current native-session grants locally
when opening the device; saved descriptors alone do not establish control access.
Only permitted exact values on the selected route appear. Keep each grant's full
argument combination intact; do not combine grants to imply extra values. Numeric
fan controls use descriptor steps, while enum fan codes remain presets. Swing-only
devices use switches; fixed-position choices require their own actual action and
values, independently for each axis. A missing observation is not false. Display,
sound and other independent low-risk settings use the same schema and plan/execute
path. Coupled or unfamiliar schemas retain supported explicit forms; high-risk,
read-only and ungranted operations do not become quick controls.

Generic widget support does not add Meiju adapter functions. The [parity record](development/10-meiju-ac-parity.md)
tracks the remaining full-function request and exact-model reference gate.
