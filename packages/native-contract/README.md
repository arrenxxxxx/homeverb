# HomeVerb native contract

Start with the [Flutter handoff](../../docs/native-ui-contract.md). API version 1.14.1 has 31 callable JSON methods, one separate protected account-connection function and three separate trusted approval functions; the JSON envelope schema is 1.0.

- [api.json](api.json) is compiled into the `get_api_info` response.
- [examples.json](examples.json) contains fictional requests and actual Mock native responses, including failures and lifecycle states. It is preview/test data, never a production device database.
- [account-examples.json](account-examples.json) provides fictional connection/refresh DTOs and local detail/rename/removal/recovery and saved-inventory inspection request/results; [native-connect.schema.json](../../schemas/native-connect.schema.json) defines nonsecret connection metadata. Secret input is separate native memory, never general JSON.
- [Startup](../../schemas/native-open.schema.json), [requests](../../schemas/native-request.schema.json), and [responses](../../schemas/native-response.schema.json) define the wire contract. Response definitions are keyed by method; `open` describes startup.
- [verify_native_contract.py](../../scripts/verify_native_contract.py) validates every method against the built Linux library and resolves all schema references offline.

API 1.9 adds independent, exact-device `meiju_fan_grants` for the reviewed vendor AC fan codes; see the [startup contract](../../docs/native-ui-contract.md#scoped-experimental-ac-fan-startup). Older libraries reject this field.

Trusted startup can select `midea.meiju` for read-only access to previously connected local accounts; omitted selection remains Mock. Protected native account connection is callable; Flutter input UI and mobile secure stores remain pending. Explicit recovery validates pending saved Meiju credentials and retrieves cards without another login. API 1.8 also permits fresh same-account login for an explicitly selected pending account with intact protected identity/bindings. Local account removal has durable pending/completed outcomes; API 1.8.1 delegates exact-entry deletion without requiring readable credential bytes; explicit cache/history clearing preserves replay protection. Provider revocation, sharing, arbitrary identity merge and full-profile deletion remain unavailable. Use explicit unavailable states for those production actions; do not infer availability from a mockup. Keep Flutter-only preview repositories separate from the native session.

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
