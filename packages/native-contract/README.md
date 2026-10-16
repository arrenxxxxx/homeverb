# HomeVerb native contract

Start with the [Flutter handoff](../../docs/native-ui-contract.md). API version 1.20.0 has 33 callable JSON methods, one separate protected account-connection function, three trusted appliance-approval functions, three trusted LAN-scan functions and three trusted connection-link functions; the JSON envelope schema is 1.0.

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
pause/resume/cancel actions. Each uses empty arguments and the existing trusted
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
