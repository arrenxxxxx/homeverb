# Native UI contract 1.0

Status: implemented Linux Mock contract, October 15, 2026. This is the integration baseline for independent Flutter development, not phone packaging or real-provider acceptance. See [ADR-0019](adr/0019-native-ui-contract.md).

## Start here in a Flutter worktree

Branch from a commit containing both the Mock MVP and this contract. Put the Flutter application in `apps/mobile/`; do not copy Rust code into the app. The UI worktree owns Flutter views, routing, accessibility, local UI preferences, Dart FFI wrappers, and mobile build configuration. Core work owns native methods, schemas, capability semantics, storage, adapters, and their fixtures. Agree on schema changes before depending on them. Each verified phase is a separate English Conventional Commit; a local commit does not imply a push.

Read the [UI integration guide](development/03-ui-integration.md) for existing behavior and the [functional handoff](ui-functional-spec.md) for screens. Use [api.json](../packages/native-contract/api.json) as the callable-method catalog and [examples.json](../packages/native-contract/examples.json) as fictional request/response examples. The examples were obtained from the built native library. They contain temporary fictional IDs and observations, not defaults for a production device.

The runtime is still an explicit Mock host. A real provider must not be represented as connected merely because its page exists. Preview-only refrigerator, dishwasher, and sharing screens can use isolated UI fixture repositories; never mix those records with a native installation or label them live.

## Startup, ownership, and version negotiation

The six existing C functions in [homeverb.h](../crates/ffi/include/homeverb.h) remain the native boundary. There is no HTTP endpoint, port, CLI subprocess, or daemon.

1. Load the platform library and call `homeverb_open` from a worker isolate/thread. Supply schema `1.0`, an absolute OS-private data directory, and an explicit `allow_mock_writes` boolean. `scenario` is optional and defaults to `normal`. All startup values come from trusted app code, never imported commands. The [startup schema](../schemas/native-open.schema.json) defines fields; Core checks OS path semantics.
2. Decode the open query envelope. On success, retain `data.handle` as an unsigned 64-bit native handle; `data.adapter` is `homeverb.mock`. Do not persist handles across app launches. Errors have null data. Free every returned native string exactly once, including errors.
3. Call `get_api_info` with a new operation ID before enabling native features. Require API major version `1`, and check individual methods in `data.methods`. The initial `api_version` is `1.0.0`; JSON envelope schema remains `1.0`. Older hosts return `INVALID_ARGUMENT` for this query: show a native-library upgrade requirement. Unknown optional response fields may be retained/ignored, but unknown outcomes must not become success.
4. Load local rooms/cards/history as needed. Opening a page does not discover devices, refresh their state, or contact every account. Explicit Mock discovery is available for development.
5. Execute every blocking FFI call on a worker. Allow lifecycle/cancel calls from another isolate/thread; do not queue cancellation behind the call being cancelled. At most eight active operations belong to one session. Use globally unique operation IDs within that session, and never reuse one until its call finishes.

```json
{
  "schema_version": "1.0",
  "operation_id": "ui-api-info-1",
  "timeout_ms": 10000,
  "request": {"method": "get_api_info"}
}
```

Every request follows [native-request.schema.json](../schemas/native-request.schema.json). Inputs are closed objects: do not add a principal, grant, token, password, `confirmed`, locale, or unused optional fields. Omitted and null are distinct. `timeout_ms` is 1–180000; input JSON is at most 1 MiB. Operation IDs have a 256 UTF-8-byte limit in the host, which differs from JSON Schema's character counting. Core additionally checks names, exact reorder sets, device constraints, identity, permissions, and budgets.

`homeverb_cancel` returns whether an active operation was found, not whether a physical action was reversed. On backgrounding call `homeverb_set_foreground(handle, false)` immediately; returning foreground permits new explicit calls without starting refresh. Disable duplicate submission while pending. Ignore outdated presentation callbacks after navigation/disposal, while retaining execution receipts for recovery. Call `homeverb_close` from a worker when shutting down; closed handles return `HOST_CLOSED` and must not be reused. Forced termination has no guaranteed callback.

## Decode by response kind

The catalog's `response_kind` is `query`, `execution`, or `batch_execution`. Validate a method's response against `native-response.schema.json#/$defs/<method>` using the [offline registry](../scripts/native_contract.py); `$defs/open` describes startup. Shared schema URNs resolve only to checked-in schemas, never a network service.

| Returned shape | Meaning | UI handling |
|---|---|---|
| Query, `error: null` | Native query/local mutation completed | Consume `data` and `warnings`; null data is valid for `delete_room` |
| Query, non-null `error` | Query failure or execution admission failure | Preserve input and prior state; display the stable error code through locale resources |
| Top-level execution `status` | Single/composite operation receipt | Render status, attempts, warnings, evidence, and optional items; an embedded error does not erase the receipt |
| Top-level batch `status` with ordered `items` | Per-device results | Render every item; no fabricated aggregate device or durable aggregate request ID |

For either execute method, first check for a top-level `status`; otherwise decode the query error envelope. Do not implement `error != null -> throw away the payload`: failed/unknown executions can contain essential transmission and recovery evidence. `accepted` does not mean completed, `unknown` may have executed, and `partial` needs item-level inspection. A schema-valid response alone does not prove physical execution.

Use a new `operation_id` for each native call. The Command's `request_id` is a different, durable write identity. Never regenerate and replay a write after interruption, timeout, or an uncertain result. Look up `get_execution(request_id)` and/or explicitly read state. Preserve nested child receipts; do not independently execute generated child command IDs.

## Callable methods

All rows use query envelopes except the two execution methods. The schemas and actual examples are authoritative for nested shapes.

| Method | Inner request fields after `method` | Successful result |
|---|---|---|
| `get_api_info` | None | API version, method response kinds, native limits |
| `list_devices` | None | Device array from local storage |
| `discover` | None | Discovered Device array; currently fictional Mock only |
| `describe_device` | `device` | `{device, capabilities}` |
| `read_state` | `device`, `component`, `connection`, `cached` | State snapshot; missing/null observations remain unknown |
| `plan` | `command` | Command, permitted transports, warnings, optional independent steps |
| `execute` | `command` | Execution receipt directly; command timeout must equal the outer timeout |
| `plan_batch` | `batch` | Ordered plans, max concurrency, warnings |
| `execute_batch` | `batch` | Batch execution directly; outer deadline covers all commands |
| `recent_executions` | None | Up to 100 retained receipts across the profile |
| `device_executions` | `device_id` | Up to 100 retained receipts filtered by exact device ID |
| `get_execution` | `request_id` | Exact retained receipt; not-found/retired are errors without a replay action |
| `diagnostics` | None | Allowlisted report for preview and explicit local export |
| `list_rooms` | None | Ordered Room array |
| `create_room` | `name` | Created Room |
| `rename_room` | `room_id`, `name` | Updated Room |
| `delete_room` | `room_id` | Null; devices remain and become Unassigned |
| `reorder_rooms` | `room_ids` | Ordered Room array; supply the complete permutation |
| `list_device_cards` | None | Ordered `{device, preferences}` array, without live state |
| `set_device_preferences` | `device_id`, `preferences` | Updated DevicePreferences; send all alias/room/favorite fields |
| `reorder_devices` | `device_ids` | Ordered DevicePreferences array; supply the complete permutation |
| `assign_devices_to_room` | `assignment` | Preferences for 1–256 exact devices, atomically assigned to explicit room or null |

Prefer exact device IDs even where lookup accepts an alias. Resolve a capability by `(component_id, capability_id)`; shared definitions apply only when no component scope is present. Room filters and search operate on local cards. Fetch cached snapshots separately when displaying observations; a device card alone does not establish online/offline state. A user-requested live read passes `cached: false`. A refresh never resends an action.

## UI function coverage and ownership

| UI scope | Current integration | Remaining work and boundary |
|---|---|---|
| D01–D12, R01–R05: cards, search, rooms, local organization | Device/room methods above | Flutter filtering, loading/errors, accessibility and layouts. Metadata writes preserve routes and grants |
| C01–C14: dynamic controls, history and outcomes | Describe/read/plan/execute/batch/receipt methods | Render exact capabilities; non-AC fixtures are previews, not registered real adapters. Consequential real approvals remain unavailable |
| A01–A08: device discovery and addition | Explicit Mock `discover` | Real pairing, cloud retrieval, mobile permissions and safe identity merge remain unavailable. No merge API exists |
| P01–P13: platforms, accounts, authorization and removal | No account methods in this version | Catalog, account persistence, auth challenges, automatic Meiju device retrieval, connection checks, local removal and provider revocation require the next native contract phase |
| H01–H16: nearby authorization sharing | No sharing methods in this version | Eligibility, peer discovery, code verification, scope, consent, conflict/import and transfer records require ADR-0008 implementation |
| S02/S11: Window Light appearance | Flutter-owned private preferences | Persist `window_light`, fallback to it, and provide one preview; no native theme API or download capability is needed |
| S03/S10, D07: sharing name and route choices | Flutter-owned private preferences plus explicit Command/ReadState `connection` | Store only nonsensitive display preferences. Resolve device > installation > `auto`; Core still enforces allowed routes. Do not add route fields to `set_device_preferences`. A saved sharing name does not start advertising |
| S04/S09: clear cache/history/all data | No cleanup method in this version | Disable destructive native actions until scoped APIs handle active journals, retired IDs, credentials, and partial failures; never delete SQLite files from Flutter |
| S05/S06: recent operations | Local receipt queries | Appliance receipts work; transfer history remains unavailable |
| S07/S08/S12: diagnostics, export and About | `diagnostics`, API metadata, repository compatibility/licenses | Flutter OS export/chooser, cancel/failure handling, accurate build labels. Export the previewed allowlist only |
| W01/W02 and all screens | Same contract at all sizes | Flutter navigation, text scaling, semantics, focus, system permissions and lifecycle binding |

Flutter-owned preferences contain no credentials and grant no authority. Room/card organization remains owned by Core. Route controls must reflect actual connections and action capabilities, not brand labels. Localization uses the shared `en-US`/`zh-CN` catalogs; IDs, arguments and outcomes stay English. Preserve unknown error codes and use a generic localized fallback instead of mapping them to success.

## Next native phase: account connection

The next slice is the required Meiju journey: connect an existing account, automatically retrieve bound devices, then cloud reads and authorized controls. UI code should depend on a separate platform repository interface, not send invented method names to `homeverb_call`. This phase must define account summaries with opaque local IDs, region, masked identity, actual permission scope, authorization state and last-check time; connection progress with separate authentication/retrieval results; and device-retrieval results with supported, unsupported, ambiguous and partial entries.

Required screen states are `unavailable`, `connecting`, `awaiting_user`, `retrieving_devices`, `connected`, `partial`, `reauthorization_required`, `cancelled`, and `failed`. These are planned presentation states, not additional runtime wire enums. A successful authorization must continue to bounded device retrieval without another Find Devices step. Retrying a failed retrieval must not unnecessarily repeat login. Existing IDs and room/card organization survive reauthorization. Final auth method fields depend on the reviewed native provider flow; passwords and challenge secrets do not enter this general request channel.

Sharing and destructive cleanup need their own reviewed contracts after account identity, permissions and secure persistence exist. Keep their page layouts developable with explicit unavailable/preview states. Do not publish provisional account/sharing JSON as a callable v1 method just to satisfy a visual flow.

## Verify the handoff

```bash
cargo build --workspace --offline --locked
cargo test -p homeverb-host --test native_contract --offline --locked
.venv/bin/python scripts/verify_native_contract.py
```

The native suite calls all 22 catalog methods, checks their schemas, verifies closed-field rejection, null success, pre-admission errors, composite/batch results, all execution outcomes, cancellation, foreground gating and closed handles. It also validates the saved examples. `--write-examples` regenerates those examples only from a successful fictional run; review and commit the changes together with the contract. Existing full MVP tests still cover execution and persistence behavior.

This is Linux native verification. Dart FFI allocation, worker scheduling, mobile library packaging, secure stores, system-browser lifecycle, and physical-phone behavior must be tested in the Flutter worktree. No Flutter build has been run for this delivery.
