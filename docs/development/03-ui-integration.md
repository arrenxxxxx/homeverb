# 3. Connect the UI and localization

Visual UI design is independent of this Core implementation. There is no production HTTP bridge or CLI-output parsing path. The [Flutter application](../../apps/mobile/README.md) uses an FVM-pinned SDK and direct Dart FFI, tested with the Linux Mock host. Android packaging and physical-phone lifecycle verification remain incomplete. The [Android native package](08-android-native.md) supplies a separate arm64 library builder and Keystore/JNI backend.

The current design handoff is the [complete Window Light prototype](../../output/prototypes/homeverb-window-light-v1/index.html). Use its [functional specification](../ui-functional-spec.md), [82-state inventory](../ui-screen-inventory.md), and [Flutter visual/skin guide](../ui-window-light.md) alongside the native methods below. The application implements the one-option Appearance screen and isolated native-widget preview. Neither establishes platform authorization or nearby-sharing support; [visual equality remains unverified](../../apps/mobile/verification.md).

## Native boundary

API 1.4 adds local `get_account` and `rename_account` for platform details and names.
Use the [account metadata contract](../native-ui-contract.md#local-account-details-and-names)
to show saved linked devices without a refresh or secret-store access. Local names
do not change account identity, credentials or write permissions. API 1.5 adds
[explicit local removal](../native-ui-contract.md#remove-an-account-from-this-installation);
confirm the saved-device scope and render pending cleanup separately from success.
API 1.7 adds [explicit saved-session recovery](../native-ui-contract.md#recover-interrupted-account-storage)
for interrupted credential confirmation, with card retrieval and preserved bindings.
API 1.8 permits [pending-account reauthorization](../native-ui-contract.md#reauthorize-a-pending-account)
through the protected connection function when saved identity/bindings remain intact.

Start new Flutter work against the [versioned native UI contract](../native-ui-contract.md): it provides the callable catalog, offline request/response schemas, actual Mock examples, page-to-interface coverage, and unavailable-feature boundaries. Call `get_api_info` after opening the native host. Do not derive an interface solely from screenshots or issue undocumented requests.

Build `homeverb-ffi`. The C declarations are in [homeverb.h](../../crates/ffi/include/homeverb.h). Linux produces `target/debug/libhomeverb_ffi.so`. A native UI loads the library in its own process.

1. Call `homeverb_open` with trusted startup configuration: `schema_version`, an absolute private `data_dir`, and `allow_mock_writes`. Optional `adapter` defaults to `homeverb.mock`; `midea.meiju` defaults to read-only saved-account access and requires false Mock write permission. API 1.3 optionally accepts exact device/state `meiju_power_grants` from trusted startup; see the [power permission contract](../native-ui-contract.md#scoped-experimental-ac-power-startup). The returned query envelope contains `data.handle` and the selected adapter.
2. Call `homeverb_call` on a worker thread with a unique `operation_id`. Each returned UTF-8 JSON string must be freed exactly once with `homeverb_string_free`.
3. On backgrounding, immediately call `homeverb_set_foreground(handle, false)`. This rejects new operations and cancels active work. Returning to the foreground enables explicit new calls, without automatic refresh.
4. Use `homeverb_cancel` for an individual operation. Call `homeverb_close` when the host closes. Forced termination cannot guarantee a callback; reopen and inspect recorded outcomes.

```json
{
  "schema_version": "1.0",
  "operation_id": "ui-discover-1",
  "timeout_ms": 10000,
  "request": {"method": "discover"}
}
```

Methods: `list_devices`, `discover`, `describe_device` with `device`, `read_state` with `device/component/connection/cached`, `plan` with `command`, `execute` with `command`, `recent_executions`, and `get_execution` with `request_id`. The exact typed request is [HostRequest](../../crates/host/src/session.rs). Execute requires the command's timeout to match the outer timeout and carries the session's original deadline into Core. Queries use `{schema_version,data,warnings,error}`; execute returns the execution envelope directly. Malformed host requests return a query error.

The general JSON bridge never accepts a principal, grant, profile change, password, or model-supplied confirmation in a request. API 1.2 adds local account list/refresh plus a separate protected native connection function; follow its [buffer and result contract](../native-ui-contract.md#account-connection-and-refresh). `allow_mock_writes` is a startup-only grant for the fictional device's `main` power/climate actions over both simulated transports, not a general real-device authorization mechanism. API 1.3 adds separate session-scoped Meiju AC power permissions; ordinary requests still cannot edit grants. Core checks action arguments and route scope before sending. A composite plan includes per-step commands and permitted transports in `items`.

When rendering a component, resolve each referenced capability against that component. `describe_device` may return two definitions with the same capability ID and different `component_id` values. Omitted scope means shared; exact scope means that component only. Core rejects shared/scoped mixing of one ID, so there is no override precedence. Do not pick the first capability by ID or copy one compartment's temperature limits into another. Command and read-state component fields are unchanged. See [component scopes](../adr/0018-component-capability-scopes.md). Older strict readers must be updated before consuming scoped definitions; the C ABI functions do not change.

## Explicit device batches

For explicit appliance selections, `{"method":"plan_batch","batch":...}` returns a query envelope with per-command plans. `{"method":"execute_batch","batch":...}` returns [BatchExecution](../../schemas/batch-result.schema.json) directly after successful admission; admission errors use the query envelope. The [BatchCommand](../../schemas/batch-command.schema.json) contains `schema_version: "1.0"` and 1-32 complete `commands` for distinct exact device IDs. Each command supplies its own request ID, capability, arguments, route, wait condition, and budget. The outer host timeout covers the whole call, including queue time; an individual budget can only shorten execution.

Display ordered device items and their nested composite evidence. The aggregate has no device identity or shared verification. Existing `presentExecution` status/item handling works without new locale keys. A room selection must be resolved into explicit commands by the caller and individually authorized by Core. Foreground cancellation covers the batch; never retry it automatically after interruption. Inspect original item request IDs, since there is no durable aggregate receipt. These additive methods leave the C ABI unchanged; older hosts reject them.

## Local rooms and device cards

These methods use the same host envelope and query response as other queries. They mutate local presentation metadata only, with no appliance request or write grant. MCP does not expose metadata mutations.

| Method | Fields | Result data |
|---|---|---|
| `list_rooms` | None | Ordered Room array |
| `create_room` | `name` | Created Room |
| `rename_room` | `room_id`, `name` | Updated Room |
| `delete_room` | `room_id` | null; devices become unassigned |
| `reorder_rooms` | `room_ids` | Ordered Room array |
| `list_device_cards` | None | Ordered `{device, preferences}` array |
| `set_device_preferences` | `device_id`, `preferences` | Updated DevicePreferences |
| `reorder_devices` | `device_ids` | Ordered DevicePreferences array |
| `assign_devices_to_room` | `assignment` | Updated preferences in requested device-ID order |

For example, the inner request to rename and favorite a device is:

```json
{
  "method": "set_device_preferences",
  "device_id": "dev-mock-ac",
  "preferences": {"alias": "Quiet AC", "room_id": null, "favorite": true}
}
```

Send all of `alias`, `room_id`, and `favorite`; omission is invalid. Null alias restores the adapter label; null room means unassigned. To change only the favorite flag, retain the current alias and room values. Use exact device IDs for metadata mutations. Room deletion preserves devices, aliases, favorites, order, credentials, and grants. Rediscovery preserves local preferences.

Both reorder methods require an exact permutation of all current IDs, including devices hidden by a room/favorite filter. Invalid or outdated lists fail atomically; reload the list before trying a new order. Room names and aliases are trimmed to 1-80 characters without controls; room names must be unique after Unicode lowercasing. Duplicate device aliases remain allowed and require IDs for unambiguous selection. Virtual All/Unassigned filters are presentation concepts, not persisted rooms.

For a selection, use one atomic room assignment instead of replacing each device's full preferences:

```json
{
  "method": "assign_devices_to_room",
  "assignment": {"device_ids": ["dev-mock-ac"], "room_id": null}
}
```

Provide 1-256 distinct exact IDs. Use a room ID returned by `create_room`/`list_rooms` for assignment, or explicit null for Unassigned. An omitted destination is invalid. All devices and the destination must exist in this profile. If validation or storage fails, no memberships change. Success preserves each device's alias, favorite, order, identity, and permissions. This operation makes no appliance calls and creates no appliance execution receipts.

Schemas: [Room](../../schemas/room.schema.json), [DevicePreferences](../../schemas/device-preferences.schema.json), [RoomAssignment](../../schemas/room-assignment.schema.json). User-entered names are stored verbatim after trimming, independent of the selected UI language.

## Device operation history

Settings can call `recent_executions` for the profile. Device Controls calls `{"method":"device_executions","device_id":"dev-mock-ac"}` for one exact device ID. The result is a query envelope containing up to 100 retained receipts, newest first by insertion order, filtered before the limit. Unknown IDs fail; a known device with no retained records returns an empty array. Parent and child receipts are returned as stored, and composite parents contain their item evidence.

To inspect a known operation, call `{"method":"get_execution","request_id":"saved-request-id"}`. This returns one unchanged receipt in the query envelope and works without current write permission or valid device metadata. Parent and child IDs are supported. Display `REQUEST_NOT_FOUND` and `REQUEST_RETIRED` without offering automatic replay; use their locale keys. The usual foreground/closed-session rules still apply. Older hosts reject the additive method, while the C ABI is unchanged.

These are local queries, with no refresh, replay, or background work. Filtering does not preserve more history: the profile still has a shared finalized-receipt retention limit, so older records may already be retired. Do not describe an empty view as proof that the device was never operated.

## Explicit local-data cleanup

Settings can call `clear_cached_state` or `clear_execution_history` after a separate
scope confirmation. Use the [cleanup contract](../native-ui-contract.md#clear-local-cache-or-history)
for counts, active-journal refusal, interrupted calls and replay protection.
Neither operation removes accounts, grants or room/card organization. Full-profile
deletion remains unavailable; do not implement it by deleting SQLite files.

## Diagnostic preview and export

Settings calls `{"method":"diagnostics"}` in the normal host envelope. The query result's data matches [Diagnostics](../../schemas/diagnostics.schema.json). Render a preview, then save that same report only after the user's explicit action. Do not build an export by combining raw `describe_device`, `read_state`, or `recent_executions` results. This method does not accept extra fields or a request to include secrets, and it does not upload anything.

Use the `diagnostics.*` locale keys for the preview/save controls and explanatory notices. Show that this is local metadata, with at most 100 stored receipt summaries and 32 attempts each; `attempts_omitted` makes truncation explicit. `item_outcomes` counts direct children, including partial children as partial, without exporting their private payloads. `OTHER` denotes an unreviewed error code. Room/device/connection counts do not establish online status, and parent/child receipts can represent overlapping work. See [privacy limits](../security-and-privacy.md#local-diagnostic-reports).

This adds a native request method without changing existing payloads, the C ABI, or device storage layout. Older hosts reject the unknown method; the UI must handle that response. Backgrounded/closed hosts continue to reject diagnostic requests.

## Localization contract

Use canonical locales `en-US` and `zh-CN`. `EnUs`/`ZhCN` aliases normalize to these values. Unsupported locales fall back to English. Keep locale selection in the presentation layer; never translate public identifiers, capability IDs, action arguments, or stored outcomes.

[Locale catalogs](../../packages/presentation/locales/en-US.json) share identical keys and placeholders. The only permitted Chinese source text is [zh-CN.json](../../packages/presentation/locales/zh-CN.json). Native frontends can load these JSON resources directly. JavaScript-based designs can use [presentation helpers](../../packages/presentation/src/index.mjs); they have no framework dependency.

Render translation results as text, not raw HTML. Present requested values separately from observations. Missing/null state means unknown. `accepted` is unverified; `unknown` means it may have executed and should be refreshed. Display non-atomic receipts item by item, retaining their evidence and warnings. A partial receipt needs a refresh action when any child is accepted, unknown, or references a retired request. Refresh means an explicit state read, never automatic replay.

The native ABI is verified through a Linux ctypes host and the Flutter Linux application. Both load Core directly; Dart tests cover writes, cancellation, foreground gating and close. This does not establish Android or iOS support.

Next: [Add a real adapter](04-add-adapter.md).
