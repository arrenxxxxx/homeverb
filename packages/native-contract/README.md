# HomeVerb native contract

Start with the [Flutter handoff](../../docs/native-ui-contract.md). API version 1.40.0 has 36 callable JSON methods, one separate protected account-connection function, three trusted appliance-approval functions, three trusted LAN-scan functions three trusted connection-link functions and three trusted account-identity recovery functions and three dedicated interactive login functions and four protected verification functions and four modern account functions; the JSON envelope schema is 1.0.

API 1.19 adds nullable `last_complete_inventory_at` to account summaries. It is
historical local retrieval metadata, not authorization or online status. SQLite
1/2/3/4 migrates with a backup to schema 5; close older clients before upgrading.
Protected credentials and C/JNI exports are unchanged. See [ADR-0060](../../docs/adr/0060-account-inventory-timestamp.md).

API 1.20 adds the separate trusted [connection-link functions](connection-link.json)
and [fictional opted-in examples](connection-link-examples.json). Display exact
scope and potential existing grants, then consume confirmation only after actual
user consent. Production Mock/Meiju refuse linking; see the
[handoff](../../docs/native-ui-contract.md#trusted-connection-linking-api-120).

- [api.json](api.json) is compiled into the `get_api_info` response.
- [examples.json](examples.json) contains fictional requests and actual Mock native responses, including failures and lifecycle states. It is preview/test data, never a production device database.
- [account-examples.json](account-examples.json) provides fictional connection/refresh DTOs and local detail/rename/removal/recovery and saved-inventory inspection request/results; [native-connect.schema.json](../../schemas/native-connect.schema.json) defines nonsecret connection metadata. Secret input is separate native memory, never general JSON.
- [Startup](../../schemas/native-open.schema.json), [requests](../../schemas/native-request.schema.json), and [responses](../../schemas/native-response.schema.json) define the wire contract. Response definitions are keyed by method; `open` describes startup.
- [verify_native_contract.py](../../scripts/verify_native_contract.py) validates every method against the built Linux library and resolves all schema references offline.

API 1.9 adds independent, exact-device `meiju_fan_grants` for the reviewed vendor AC fan codes; see the [startup contract](../../docs/native-ui-contract.md#scoped-experimental-ac-fan-startup). Older libraries reject this field.

Trusted startup can select `midea.meiju` for read-only access to previously connected local accounts; omitted selection remains Mock. Protected native account connection is callable. The Android credential backend has synthetic JNI and isolated AOSP runtime evidence; Flutter input/integration, physical phones and iOS storage remain pending. Explicit recovery validates pending saved Meiju credentials and retrieves cards without another login. API 1.8 also permits fresh same-account login for an explicitly selected pending account with intact protected identity/bindings. Local account removal has durable pending/completed outcomes; API 1.8.1 delegates exact-entry deletion without requiring readable credential bytes; explicit cache/history clearing preserves replay protection. Provider revocation, sharing, arbitrary identity merge and full-profile deletion remain unavailable. Use explicit unavailable states for those production actions; do not infer availability from a mockup. Keep Flutter-only preview repositories separate from the native session.

API 1.10 adds `inspect_account_inventory` for explicit protected local membership inspection. It never contacts the provider or changes credentials; unavailable secrets return an error. Membership is separate from online state and write permission.

API 1.11 adds independent `meiju_mode_grants` for explicit combined AC power-on and mode selection. Negotiate the version before supplying the field; older libraries reject it. See the [activation startup contract](../../docs/native-ui-contract.md#scoped-experimental-ac-mode-activation-startup) for exact values, consent labels and unchanged outcome limits.

API 1.11.1 preserves product mapping evidence only inside protected account payload version 3. Older payloads remain readable without mutation; older binaries cannot open the upgraded payload. Native methods and public DTOs are unchanged. Follow the [migration guidance](../../docs/native-ui-contract.md#protected-product-profile-migration).

API 1.12 adds independent `meiju_temperature_grants` for exact Celsius targets on ACs with reviewed protected product mappings. Negotiate the version before supplying this field. The action schema supplies device-specific write limits; startup grants never establish capability. See the [temperature startup contract](../../docs/native-ui-contract.md#scoped-experimental-ac-temperature-startup). Methods, C functions, SQLite schema 4 and protected payload 3 remain unchanged.

API 1.12.1 fixes explicit AC target reads for the protected single-field temperature profile. It adds no method, schema, permission or storage migration; read-only sessions also benefit. Compound-profile reads retain their previous requirements.

API 1.13 adds [three separate trusted approval functions](approval.json) with
[closed schemas](../../schemas/native-approval.schema.json) and
[fictional inputs](approval-examples.json). The controller must display the
immutable returned scope and obtain explicit consent before execution. Keep
process-local IDs out of persistence and model tools. The 31 JSON methods,
SQLite schema 4 and protected payload 3 are unchanged; the fictional
`approval-required` Mock scenario supports UI integration, while actual Flutter
consent and physical acceptance remain pending; API 1.14 adds the reviewed
dishwasher mappings below.

API 1.14 adds exact `meiju_dishwasher_grants` for reviewed profile-scoped elevated
starts. [The contract](../../docs/native-ui-contract.md#profile-scoped-dishwasher-starts-api-114)
requires separate trusted confirmation and descriptor-specific arguments. The
startup union never establishes model capability. Ordinary methods, C functions
and storage versions are unchanged; physical acceptance remains pending.

API 1.14.1 increases only the exact temperature grant list capacity from 64 to
256 entries, allowing a four-AC configuration with 116 exact candidate targets.
The protected product mapping and action schema still restrict each device; this
is not a universal temperature range. Negotiate 1.14.1 before supplying more than
64 entries. Other grant limits, methods, C functions and storage versions are
unchanged. CLI grant files retain their independent 32 KiB limit.

API 1.15.0 adds independent exact `meiju_dishwasher_cycle_grants` for reviewed
pause/resume/cancel actions. The original profiles use empty arguments and the existing trusted
confirmation functions; program-start permission does not grant a cycle action.
Dishwasher revision 3 adds optional raw operator observations with cloud staleness.
See the [cycle contract](../../docs/native-ui-contract.md#profile-scoped-dishwasher-cycle-operations-api-115).
The 31 methods, eleven exports and storage versions remain unchanged.

API 1.16 adds `inspect_control_options` for a selected saved device/connection.
It projects protected model-specific action definitions before trusted startup
grants, without network access, publication or permission changes. See the
[permission-selection flow](../../docs/native-ui-contract.md#inspect-control-options-before-permission-api-116).

API 1.17 adds `inspect_session_grants` for the selected local device/connection.
It projects current session policy without credentials, provider access or
mutation. Reopening read-only yields empty grants even when saved descriptors
retain actions. See the [permission-display contract](../../docs/native-ui-contract.md#inspect-current-session-grants-api-117).

API 1.18 adds [three trusted LAN-scan functions](lan-scan.json) with
[closed schemas](../../schemas/native-lan-scan.schema.json) and
[fictional examples](lan-scan-examples.json). Platform interface/permission
integration and actual user consent are required. A unique candidate adds no
connection and exposes no endpoint; never register these as ordinary tools.

API 1.21 extends cycle grants with optional exact `arguments` (omission means `{}`).
Product 760Y0026 requires an explicit `additional` value in grant and command;
empty grants never authorize it. Read-only control options expose its domain,
and trusted confirmation must display the chosen setting. Other admitted profiles
still forbid extra arguments. There remain 33 ordinary methods and 16 C functions
plus the Android-only JNI initializer. Storage formats are unchanged.


API 1.22 adds `inspect_device_conflicts`, the 34th ordinary method. It diagnoses
saved cross-account binding candidates from exact local account/device IDs,
including unpublished records, without networking, merging or changing authority.
At most 64 mainland Meiju accounts participate; any inaccessible relevant map
fails the complete query. C/JNI and storage formats remain unchanged. Account
choice and reconciliation remain separate work; see the
[conflict contract](../../docs/native-ui-contract.md#inspect-saved-device-conflicts-api-122).


API 1.23 adds startup-only `meiju_ac_feature_grants` for reviewed optional AC
scalar settings. Inspection can return up to 64 action options; revision-4 AC
properties use `control_requires_observation` to hide unreported optional controls.
Execution independently checks the protected profile and reads required state
before one send. The 34 ordinary methods, C/JNI exports and storage formats are
unchanged. See the [contract](../../docs/native-ui-contract.md#profile-scoped-optional-ac-startup-api-123).

API 1.25 supplies [identity-recovery.json](identity-recovery.json),
[fictional fixtures](identity-recovery-examples.json) and the
[recovery schema](../../schemas/native-identity-recovery.schema.json). Actual
trusted UI consent is required after full device matching. Confirmation seals
the old host and preserves local identity/preferences; reopen read-only. Canonical
inner Meiju session version 3 uses the provider IoT user ID; SQLite and outer
payload formats remain unchanged.

Interactive foreground login uses [account-login.json](account-login.json), its
[fictional examples](account-login-examples.json), and the
[closed schema](../../schemas/native-account-login.schema.json). Resolve these
symbols only after API 1.28 negotiation. Existing one-shot connection remains
available. No interactive method belongs to the ordinary JSON dispatcher.

API 1.29 adds four exact reference fan grant properties and their control options,
without changing the 34 JSON methods or 22 C exports. Negotiate 1.29 before passing
these fields to `homeverb_open`; see the
[fan contract](../../docs/native-ui-contract.md#exact-reference-fan-settings-api-129)
and [startup example](../../examples/native-open.mijia-reference-fan.json).

API 1.30 adds `mijia_setting_grants` for the exact light/purifier controls in
[ADR-0084](../../docs/adr/0084-exact-light-and-purifier-controls.md), with closed
component/property domains, 4,096 exact targets maximum and independent fan
admission. See the [startup fixture](../../examples/native-open.mijia-settings.json).
Negotiate 1.30 before passing the new field; method/export counts are unchanged.


[Account verification](account-verification.json) and its
[examples](account-verification-examples.json) describe API 1.31's dedicated
trusted bridge. A caller-owned 131072-byte output buffer carries protected
captcha/provider-page bytes; metadata JSON never contains the page or ticket.
The existing captcha-only bridge remains available. Browser integration is pending.

API 1.32 additionally exposes the explicit selected-method code request described
by `request_code` and `code_result` in the verification schema. No code is sent
by opening a prompt. Its output reuses the protected prompt buffer and adds only
request-count/cooldown metadata. Existing begin/answer JSON stays unchanged.


API 1.33 adds `inspect_history_options` and `read_history` under ADR-0089. The
first method performs local protected option inspection for an exact device and
connection. The second accepts one closed `history` object matching the domain
history request schema. It reads one explicit historical series without route
fallback, cache writes or execution receipts. Only admitted exact-profile adapters
supply options; Mock and Meiju default to empty options and unsupported reads.
The 26 C exports plus Android initializer and all storage/grants are unchanged.
App, CLI and MCP presentation remain separate integration work.


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


## Native auxiliary settings -- October 30, 2026

API 1.35 implements [ADR-0092](../../docs/adr/0092-exact-auxiliary-reference-settings.md):
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


API 1.36 adds the protected [modern account bridge](modern-account.json),
[closed request/result schema](../../schemas/native-modern-account.schema.json)
and [fictional examples](modern-account-examples.json). QR bytes use the separate
8192-byte output buffer; public results contain only metadata and conservative
logical-entry counts. The four new symbols require explicit API negotiation.
Flutter integration and physical/logical device adoption remain pending.


API 1.37 adds optional `publish_devices:true` to modern QR continuation and
`refresh_devices` / `recover_devices` saved-session operations. Results use
`connected_devices` / `devices` and the existing sanitized account retrieval report,
limited to partial/failed with unique local device IDs. Old logical requests keep
their results. Negotiate the version before sending additions. Ordinary account
refresh/recovery selects inventory from protected stored protocol evidence without
fallback. See [ADR-0102](../../docs/adr/0102-native-modern-device-publication.md).
Flutter now negotiates the opt-in and adopts validated device reports after local
reload; 1.36 retains logical reports. Packaging remains pending, and fictional
native/App evidence does not establish provider, phone or physical-device acceptance.


API 1.40 adds separate exact `mijia_action_grants` for six empty-argument actions
and opt-in `include_no_argument_actions` on control-option inspection. Negotiate
1.40 before either addition. Defaults stay action-free, inspection stays local,
and uncertain non-idempotent receipts are never replayed. See the
[action contract](../../docs/native-ui-contract.md#exact-no-argument-actions-api-140).
Version-negotiated Flutter adoption is implemented. The ab34f21 API 1.40 Android
package has archive checks; real acceptance remains pending.
