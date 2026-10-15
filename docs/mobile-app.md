# Mobile app design

Status: owner-approved functional design, October 14, 2026. The screens and sharing flows below are planned, not implemented or device-verified.

## Independent execution

Flutter UI -> in-process bindings -> Core -> in-process adapters -> appliance LAN or vendor cloud. There is no initial HA/NAS address screen and no hidden dependency on a desktop CLI forwarding requests.

HomeVerb has no application account, registration, sign-in screen, or personal-account center. Users open the app directly; authentication applies only to third-party platforms or device pairing. Local configuration remains accessible without platform authorization.

## Visual direction and navigation

Use Flutter with Material 3, generous spacing, clear typography, restrained card surfaces, consistent line icons, and a small number of accent colors. The device home takes inspiration from the Mi Home room-switching and device-card pattern. Complex dashboards, historical charts, rule editors, and gateway settings are excluded.

The owner-selected default visual direction is **Window Light (01)**, approved October 15, 2026: warm plaster light, diffuse window shadows, cool reflected light, and softly tinted porcelain card surfaces. The [complete Window Light prototype](../output/prototypes/homeverb-window-light-v1/index.html) applies it to 82 screen states, with 21 boards, 14 review journeys, and 198 interaction branches. This is a visual specification, not an implemented Flutter application.

Settings > Appearance includes a skin extension point from the first release, with exactly one available option: **Window Light**, stable ID `window_light`, marked Current. Users can preview it; no alternate skins, dark/system-mode picker, wallpaper importer, or download store are included. Persist the selected skin locally and fall back to Window Light for an unknown ID. Additional reviewed skins can later reuse the same theme tokens and widgets. See the [visual system and Flutter handoff](ui-window-light.md) for implementation and phone-validation gates.

The [functional handoff](ui-functional-spec.md) records all feature groups, recovery paths, available Core interfaces, missing integrations, and development acceptance checks. The [detailed screen inventory](ui-screen-inventory.md) retains every screen ID and branch so prototype functions are not lost during implementation. These documents supersede the earlier atlas's dark-appearance example.

The three primary destinations are **Devices**, **Platforms**, and **Settings**. Use bottom navigation on phones and side navigation on wider layouts. Use **Smart Appliance Platforms** as the platform page title and **platform** or **platform account** in user-facing text, rather than **data source**. Internal Provider and Account identifiers remain unchanged.

Authorization sharing belongs inside Settings, with no primary navigation destination or separate sharing home. Recent Operations is also reached from Settings, with a device-specific entry from Device Controls.

## Screen inventory

| Screen | Entry | Functions |
|---|---|---|
| Devices | Primary navigation; initial screen | Room filters, device cards, search, ordering, favorites, manual refresh, quick controls, and Add Device. The initial empty state offers Add Platform and Receive Authorization. |
| Room Management | Room-bar management control; Settings | Create, rename, reorder, and delete custom rooms; move devices individually or in a selection. |
| Device Controls | Tap a device card | Observed state, supported controls, components, manual refresh, and operation outcomes; links to Device Information and recent operations for this device. |
| Device Information | Device Controls; card menu | Edit the local name and room; inspect model, linked platform, capabilities, and connections; set a connection preference and review ambiguous duplicate-device matches. |
| Add Device | Devices toolbar; empty room | Enumerate an authorized platform account or run supported nearby discovery/pairing; show progress, select devices, resolve identity conflicts, and choose a room. Link to Add Platform when authorization is needed. |
| Smart Appliance Platforms | Primary navigation | Connected platforms and accounts, regions, device counts, and the last authorization check; add a platform, reauthorize, or open details. Support multiple accounts on one platform. |
| Select Platform | Add Platform | Search platforms or protocols; show region requirements, supported authorization methods, and compatibility status. Identify unavailable or unimplemented integrations. |
| Platform Authorization / Pairing | Selected platform; Reauthorize | Adapter-supported browser authorization, verification-code challenge, or device pairing; steps, waiting, cancellation, expiry, and actionable failure reasons. |
| Platform Details | Select a connected account | Masked account details, region, granted permissions, linked devices, and the last check; manually check the connection, reauthorize, or remove local authorization. Offer remote revocation separately only when supported. |
| Recent Operations | Settings; Device Controls | Bounded local device-operation and authorization-transfer records, result details, and recovery actions. Device Controls opens a filtered view. |
| Settings | Primary navigation | Room Management, Authorization Sharing, appearance, this installation's sharing name, connection defaults, system permissions, local-data management, redacted diagnostic preview/export, version, and compatibility information. |

## Rooms and the device home

The toolbar shows HomeVerb, Search, and Add Device. A horizontal room bar starts with **All**, followed by the user's ordered rooms, such as Living Room, Primary Bedroom, and Bathroom. Show **Unassigned** when devices have no room. Room names are user-defined; these examples are not a fixed list.

Each device belongs to at most one room in the local installation. Users can create, rename, reorder, or delete rooms and move devices between them. Deleting a room moves its devices to Unassigned without deleting devices, credentials, or platform accounts. Renaming or moving a device does not change its identity, permissions, or connection routes. Room membership is local organization, not an authorization boundary or automatic synchronization of vendor rooms.

Use a two-column card grid on phones where space allows, adapting to narrow widths and larger text. Each card contains the device icon, local name, key observed state, freshness information, and a supported common action. Tapping the card opens Device Controls; a separate quick-control target sends the specific action. Long-press exposes room assignment, ordering, favorite/pin, and rename options; provide the same options through an accessible menu.

Favorites can be pinned within the current filter without changing room membership. An empty room offers Add Device and Move Existing Devices. A stale snapshot, unknown state, and a failed connection check remain distinguishable; an old snapshot alone does not prove that a device is offline.

Use bottom sheets or dialogs for quick controls, room assignment, action confirmation, sharing-scope confirmation, permissions, and short result messages. Keep the main control prominent and reveal secondary capabilities as needed. Color supplements text and icons rather than being the only status cue.

## Authorization sharing in Settings

The **Authorization Sharing** section offers **Send Authorization** and **Receive Authorization** directly. Follow the LocalSend-style interaction model: both people actively open the app on the same reachable local network, discover the receiving installation, verify the peer, and consent to an encrypted direct transfer. This does not claim LocalSend wire-protocol compatibility or require the LocalSend app.

| Flow | Steps |
|---|---|
| Send Authorization | Choose an eligible platform account -> discover receiving installations -> select the recipient -> compare a pairing code on both devices -> review the actual authorization scope -> confirm and transfer -> show the known result. |
| Receive Authorization | Start a time-limited receive session -> inspect the sender and compare the pairing code -> review the platform, masked account, and actual permissions -> accept -> import into the system secret store -> check authorization -> select devices and assign local rooms. |

Display the installation's sharing name, network/permission requirements, session time remaining, transfer progress, and Cancel. Merely opening Settings must not advertise the device or start discovery. Account conflicts require an explicit choice; never silently overwrite existing credentials. Selecting devices or rooms for display does not reduce the authority of a shared account.

Keep **Transferred**, **Imported**, and **Authorization Validated** distinct. Byte delivery is not proof that the receiving platform session works. If the final acknowledgement is lost, report an unconfirmed result and let the recipient inspect its local result; do not automatically resend credentials. Reopening the app must not restart a transfer.

Enable sending only for adapters with evidence of a safe, supported sharing path. Prefer provider-issued member access or separate grants. Device-bound credentials and unsafe shared refresh-token rotation disable transfer and direct the recipient to their own authorization flow. Show actual scope, provider-defined expiry when available, and revocation limitations; do not offer restrictions or lifetimes that the provider cannot enforce.

After import, the receiving app contacts the platform or appliance independently. The sender is not a relay. Clearing transfer records or deleting local authorization does not revoke another installation's access. See [ADR-0008](adr/0008-foreground-authorization-sharing.md) for the boundary and implementation gates.

## Dynamic controls

Writable booleans use switches; bounded numbers use steppers or inputs; enums use selections; read-only values use status displays; actions use parameterized forms. Units, steps, and disabled conditions come from capability metadata, not hardcoded brand pages. Specialized device views still issue the same Command model.

An unknown power property must not display "off". Show a requested target separately as "Setting..." rather than overwriting the last observation. Display `accepted` as "Accepted; not yet confirmed" and `unknown` as "May have executed; refresh to verify". Do not encourage repeating a start action.

Recent Operations distinguishes `accepted`, `verified`, `failed`, and `unknown`, and shows per-item outcomes for `partial` batches. A verified state match does not necessarily establish physical causality. Controls and permissions must not be inferred from room membership or platform branding.

## Lifecycle

Read, write, and bounded observation happen only for explicit foreground tasks. Backgrounding cancels work and closes subscriptions; already-started appliance behavior may continue. On reopen, display cache age and network only when the user requests it. No keep-alive or system scheduling is added.

Authorization discovery, advertising, and transfer belong to an explicit foreground operation. Completion, cancellation, timeout, or backgrounding releases its sockets and temporary session material. Closing the flow cannot retract credentials already imported by the recipient. Do not promise a callback after forced termination.

## Permissions and credentials

Request local-network or protocol permissions when the relevant feature is used. Explain denial and its consequences; do not silently switch to an unapproved cloud path. Access tokens through platform secret-store interfaces. Flutter UI must not retain unnecessary copies of long-lived secrets. Exclude sensitive data from clipboard operations and default diagnostic exports.

## Cross-platform validation

Start with a mock device to verify dynamic forms, invalid arguments, cancellation, unknown outcomes, repeated calls in one process, and cleanup on backgrounding. Then validate real pairing, LAN access, and capability changes. Android and iOS require independent acceptance; desktop FFI success does not substitute for phone testing.

## UI acceptance criteria

Planned UI acceptance checks cover:

- Opening the app without a HomeVerb account; three primary destinations and no sharing tab.
- Room creation, renaming, ordering, reassignment, and deletion without losing devices or changing their identity.
- Card navigation versus quick-action targets, large text, accessible menus, stale state, and empty rooms.
- Multiple platform accounts, cancelled/expired authorization, and unsupported integrations.
- Explicit receive mode, peer-code mismatch, rejected consent, unreachable peers, and session timeout.
- Unavailable secure storage, account conflicts, invalid imported authorization, and a lost final acknowledgement.
- Provider sharing eligibility, cross-installation refresh behavior, actual scope/expiry/revocation, and independent receiving clients.
- Backgrounding, cancellation, and forced termination without persistent listeners, automatic transfer replay, or credentials in diagnostics.

## No implied guarantees

An app installation does not imply every brand, region, account, model, background state stream, or unattended automation works. Only the compatibility matrix defines verified support.
