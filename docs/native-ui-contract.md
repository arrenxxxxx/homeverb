# Native UI contract 1.12

Status: implemented Linux native contract, October 16, 2026, with Mock, default read-only Meiju startup, protected account connection and scoped experimental AC power, fan, mode activation and profile-scoped temperature grants. Six real devices passed the native read path; phone packaging and complete provider/physical acceptance remain pending. See [ADR-0019](adr/0019-native-ui-contract.md).

## Start here in a Flutter worktree

Branch from a commit containing both the Mock MVP and this contract. Put the Flutter application in `apps/mobile/`; do not copy Rust code into the app. The UI worktree owns Flutter views, routing, accessibility, local UI preferences, Dart FFI wrappers, and mobile build configuration. Core work owns native methods, schemas, capability semantics, storage, adapters, and their fixtures. Agree on schema changes before depending on them. Each verified phase is a separate English Conventional Commit; a local commit does not imply a push.

Read the [UI integration guide](development/03-ui-integration.md) for existing behavior and the [functional handoff](ui-functional-spec.md) for screens. Use [api.json](../packages/native-contract/api.json) as the callable-method catalog and [examples.json](../packages/native-contract/examples.json) as fictional request/response examples. The examples were obtained from the built native library. They contain temporary fictional IDs and observations, not defaults for a production device.

Startup explicitly selects Mock or Meiju; Meiju remains read-only unless the trusted host supplies the independently scoped power, fan, mode activation or temperature grants below. Meiju uses an existing local account or the protected connection function below; opening the library alone does not establish connection or authorization. Preview-only refrigerator, dishwasher, and sharing screens can use isolated UI fixture repositories; never mix those records with a native installation or label them live.

## Startup, ownership, and version negotiation

The seven C functions in [homeverb.h](../crates/ffi/include/homeverb.h) remain the native boundary. There is no HTTP endpoint, port, CLI subprocess, or daemon.

1. Load the platform library and call `homeverb_open` from a worker isolate/thread. Supply schema `1.0`, an absolute OS-private data directory, and an explicit `allow_mock_writes` boolean. `adapter` is optional and defaults to `homeverb.mock`; `midea.meiju` selects saved-account cloud reads. `scenario` defaults to `normal`. Meiju requires `allow_mock_writes: false` and rejects any other scenario before opening the profile. All startup values come from trusted app code, never imported commands. The [startup schema](../schemas/native-open.schema.json) defines fields; Core checks OS path semantics.
2. Decode the open query envelope. On success, retain `data.handle` as an unsigned 64-bit native handle; `data.adapter` confirms `homeverb.mock` or `midea.meiju`. Do not persist handles across app launches. Errors have null data. Free every returned native string exactly once, including errors.
3. Call `get_api_info` with a new operation ID before enabling native features. Require API major version `1`, and check individual methods in `data.methods`. The current `api_version` is `1.12.1`; JSON envelope schema remains `1.0`. Older hosts return `INVALID_ARGUMENT` for this query: show a native-library upgrade requirement. Unknown optional response fields may be retained/ignored, but unknown outcomes must not become success.
4. Load local rooms/cards/history as needed. Opening a page does not discover devices, refresh their state, or contact every account. Explicit discovery uses the selected adapter. Meiju enumerates stored mainland accounts and retrieves their inventories under the request budget.
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
| `clear_cached_state` | None | `{removed_snapshots}` from this profile |
| `clear_execution_history` | None | `{removed_receipts}`; active journals refuse cleanup |
| `list_accounts` | None | Local account summaries; no credentials or network work |
| `get_account` | `account_id` | `{account, device_ids}` from local metadata; no authorization check |
| `rename_account` | `account_id`, `label` | Updated local AccountSummary; no credential or permission changes |
| `remove_account` | `account_id`, `expected_device_ids` | Local cleanup result: `removed` or `pending`; inspect the inner error |
| `inspect_account_inventory` | `account_id` | Saved protected device/connection membership; no network or mutation |
| `refresh_account` | `account_id` | Account and retrieval/publication result; no login |
| `recover_account` | `account_id` | Explicit pending-session validation and card retrieval; same result shape as refresh |
| `list_devices` | None | Device array from local storage |
| `discover` | None | Local Device array after selected-adapter discovery; Mock or model-scoped Meiju |
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
| C01–C14: dynamic controls, history and outcomes | Describe/read/plan/execute/batch/receipt methods | Render exact capabilities; default Meiju discovery has read-only AC/fridge/dishwasher mappings; native grants may admit experimental AC on/off, vendor fan presets, explicit power-on with mode selection or profile-scoped temperature targets. Other consequential real approvals remain unavailable |
| A01–A08: device discovery and addition | Explicit selected-adapter `discover`, including saved-account Meiju cloud retrieval | Real pairing, mobile permissions and safe identity merge remain unavailable. No merge API exists |
| P01–P13: platforms, accounts, authorization and removal | `list_accounts`, `get_account`, `rename_account`, `remove_account`, `refresh_account`, `recover_account`, and separate `homeverb_connect_account` | Flutter protected input, cleanup confirmation/recovery and phone secure stores; provider revocation remains unavailable |
| H01–H16: nearby authorization sharing | No sharing methods in this version | Eligibility, peer discovery, code verification, scope, consent, conflict/import and transfer records require ADR-0008 implementation |
| S02/S11: Window Light appearance | Flutter-owned private preferences | Persist `window_light`, fallback to it, and provide one preview; no native theme API or download capability is needed |
| S03/S10, D07: sharing name and route choices | Flutter-owned private preferences plus explicit Command/ReadState `connection` | Store only nonsensitive display preferences. Resolve device > installation > `auto`; Core still enforces allowed routes. Do not add route fields to `set_device_preferences`. A saved sharing name does not start advertising |
| S04/S09: clear cache/history/all data | `clear_cached_state`, `clear_execution_history`; separate confirmations | Reload local views after completion. Full-profile deletion remains unavailable; never delete SQLite files from Flutter |
| S05/S06: recent operations | Local receipt queries | Appliance receipts work; transfer history remains unavailable |
| S07/S08/S12: diagnostics, export and About | `diagnostics`, API metadata, repository compatibility/licenses | Flutter OS export/chooser, cancel/failure handling, accurate build labels. Export the previewed allowlist only |
| W01/W02 and all screens | Same contract at all sizes | Flutter navigation, text scaling, semantics, focus, system permissions and lifecycle binding |

Flutter-owned preferences contain no credentials and grant no authority. Room/card organization remains owned by Core. Route controls must reflect actual connections and action capabilities, not brand labels. Localization uses the shared `en-US`/`zh-CN` catalogs; IDs, arguments and outcomes stay English. Preserve unknown error codes and use a generic localized fallback instead of mapping them to success.

## Clear local cache or history

API 1.6 adds two separate no-argument methods under
[ADR-0029](adr/0029-local-cache-and-history-clearing.md). After showing the scope and
obtaining user confirmation, send a normal request envelope with either
`{"method":"clear_cached_state"}` or `{"method":"clear_execution_history"}`.
Neither request accepts a `force` option or resets the installation.

- Cache clearing returns `{"removed_snapshots": N}`. N counts stored device/component
  snapshots, not properties. Clear the UI's matching cached views and show missing
  readings as unknown. Cached reads return `CACHE_MISS`; do not silently fall back
  to the network. An explicit or already-running live read/verification may save
  new snapshots after cleanup.
- History clearing returns `{"removed_receipts": N}`. N counts detailed receipt
  rows, including composite parents and children. Any unfinalized journal returns
  `RESOURCE_BUSY` without mutation; wait for the operation to finish and offer an
  explicit retry. Every removed request ID remains reserved. `get_execution` and
  attempted reuse then return `REQUEST_RETIRED`, never permission to send again.
  Explain before confirmation that detailed uncertain outcomes are also removed.

Both results are query envelopes. On storage failure the corresponding transaction
rolls back. Counts describe that transaction; subsequent work can add data again.
Do not retry automatically after an interrupted caller: reload local views first.
A cancelled/expired operation is refused before mutation, but synchronous SQLite
work cannot be rolled back by a later cancellation or forced process termination.

These operations retain accounts, credentials, cards, rooms, preferences, grants,
retired IDs and account-removal ledgers. Cache clearing retains receipt evidence;
history clearing retains snapshots. Backups and allocated SQLite pages are not
securely erased. No adapter or secret-store call is made. Full-profile deletion
and provider revocation remain unavailable. The seven C symbols and schema 4 stay
unchanged; older libraries reject the new methods.

## Real Meiju read-only startup

After the [local connection probe](development/06-meiju-probe.md) saves an account,
open the same absolute private profile directory with trusted startup configuration:

```json
{
  "schema_version": "1.0",
  "data_dir": "/absolute/private/meiju-profile",
  "adapter": "midea.meiju",
  "allow_mock_writes": false
}
```

Use separate directories for Mock development and real accounts. Version 1.0 hosts
reject the added `adapter` field; an upgraded library is required for this mode.
Existing startup objects without `adapter` keep Mock behavior. The existing C symbols and requests retain behavior. Version 1.2 adds one protected
C function and two JSON methods; JSON envelope version 1.0 and SQLite schema 3 remain unchanged.
No credentials belong in startup JSON or `homeverb_call`.

Use `list_device_cards` for local cards, explicit `discover` to retrieve saved
account inventories, `describe_device` for component-scoped read properties, and
`read_state` with `connection: "cloud"`. Cache reads remain local. Complete empty
inventory does not delete cards. Partial retrieval preserves old cards and returns
`MIDEA_INVENTORY_PARTIAL`; conflicting historical/current cross-account bindings
return `DEVICE_IDENTITY_CONFLICT`. No implicit reconciliation is available.

Only the six [observed model/family pairs](compatibility.md#observed-cloud-read-models)
are experimental; other models remain visible as unsupported. Default discovery
advertises no actions or writable properties. Cloud observations have no proven source timestamp
and remain stale, including after a successful live request. Show them as recent
cloud responses with unknown freshness, not as verified physical state.

On October 15, 2026 the built Linux C ABI opened this mode, preserved existing
preferences on discovery, described six real devices, and read/cached all seven
components through schema-validated responses. No appliance controls were sent.
The automated suite separately checks empty-account startup/discovery, invalid
mode/grant/scenario combinations, lifecycle gating and closed handles without
vendor calls. The [Android native package](development/08-android-native.md) now supplies Keystore integration and a library build path; Android OS/APK/phone and iOS acceptance remain pending.

The separate [AC developer probe](development/07-meiju-control.md) can persist
experimental writable descriptors in a shared profile. Metadata is not authority:
a default native host has no Meiju write grants and its adapter refuses controls.
A read-only discovery replaces those descriptors. No `allow_meiju_writes` switch
or ordinary request-supplied approval flag is admitted.

## Scoped experimental AC power startup

API 1.3.0 adds the optional startup field under
[ADR-0027](adr/0027-native-power-grants.md). This fictional configuration authorizes
only turning off one local AC through cloud:

```json
{
  "schema_version": "1.0",
  "data_dir": "/absolute/private/homeverb-profile",
  "adapter": "midea.meiju",
  "allow_mock_writes": false,
  "meiju_power_grants": [
    {"device_id": "30000000-0000-4000-8000-000000000001", "state": false}
  ]
}
```

Use the actual random local device ID obtained from this profile's cards. Each
entry permits exactly `main / power.set {state: <value>}` over cloud. To permit
both on and off, supply two distinct entries. The closed schema accepts at most 64
pairs; UUIDs are canonical lowercase v4 and duplicate pairs are invalid. Missing or
empty lists preserve read-only behavior. Mock cannot receive nonempty Meiju grants.
There is no arbitrary action/argument schema or wildcard. Invalid lists fail before
opening/creating the profile. Opening never logs in, refreshes or sends a control.

Flutter's trusted controller must obtain the user's device/state permission before
supplying this list. Never derive it from a model-controlled request or react to
`POLICY_DENIED` by silently adding a grant. Ordinary requests cannot select or edit
permissions. The list is immutable for this handle and is not saved in SQLite.
Revoke/change it by stopping submissions, closing the old handle and opening a new
one with the intended list. Backgrounding cancels work but does not revoke the
handle's grants; close it when revocation is intended.

Read `get_api_info` on the default read-only session and require API 1.3 or later.
Older native libraries reject this new field. After deliberately opening a granted
session, perform an explicit `refresh_account` or `discover`, describe the selected
device and call `plan` before `execute`. The existing command methods and payloads
are unchanged. Unsupported models, ungranted states/components/actions and LAN-only
routes are refused; `auto` can use only cloud. These power grants admit no
temperature, dishwasher, refrigerator or elevated/high-risk controls.

An accepted code-zero cloud reply is not verified physical state. For
`wait_for: verified`, show `accepted` plus `VERIFICATION_PENDING`. Lost or ambiguous
replies remain `unknown`; no automatic new request ID or retry is permitted. Use
`get_execution` to inspect retained receipts, including after reopening read-only.
Handle foreground cancellation/close as specified above. Consent UI, phone builds,
and real control acceptance remain unverified; this phase used synthetic controls
only because the owner deferred household operation.

## Local account details and names

API 1.4.0 adds `get_account` and `rename_account` through the existing JSON call.
They use exact local account UUIDs, never provider login identifiers. Both work
without opening the secret store or contacting Meiju, including when the account
is pending or the keyring is unavailable. Mock refuses both with
`AUTH_FLOW_UNSUPPORTED`; missing accounts and accounts outside this host's admitted
provider/region return `ACCOUNT_NOT_FOUND`.

`get_account` returns `account` in the same safe summary shape as `list_accounts`,
plus sorted, distinct `device_ids` from saved device connections referencing this
account. It includes unsupported and previously retrieved cards, with each device
listed once even if several connections reference the account. It excludes other
accounts and unassociated devices. The array length is the local linked-card count,
not the current vendor inventory or permission scope. Join these IDs with
`list_device_cards` for names, rooms and display order. Account and device metadata
are separate local reads, not a frozen snapshot across concurrent changes; neither
result establishes online status or current authorization.

`rename_account` accepts a local display `label`, trims surrounding whitespace and
requires 1–80 Unicode characters without embedded controls. Duplicate labels are
allowed; always select by account ID. The account lease prevents simultaneous
session updates or renames; contention returns `RESOURCE_BUSY` without waiting or
retrying. Only the label changes. Account/credential/device identities, pending or
stored state, device organization, receipts and session write permissions remain
unchanged. The result is the updated AccountSummary, still
`authorization: "not_checked"`. Local read/rename uses normal foreground, operation
limit and cancellation admission. A synchronous SQLite commit cannot be rolled back
by cancelling the client afterward; reread local details after an interrupted UI
callback instead of assuming the name stayed unchanged.

The [account examples](../packages/native-contract/account-examples.json) include
both request/result shapes. Libraries before 1.4 reject these new methods; check
the method catalog before enabling the corresponding UI actions. The API 1.4
metadata additions preserved existing payloads, seven C symbols, SQLite schema 3
and appliance execution.
API 1.5 adds local removal below. Provider revocation remains unavailable.

## Remove an account from this installation

API 1.5.0 adds `remove_account` under [ADR-0028](adr/0028-local-account-removal.md).
Use `get_account` to show the account and saved linked cards. Explain that local
credentials, the account, those cards, their preferences and snapshots will be
removed; rooms, operation receipts and replay-protection records remain. Other
installations and provider authorization are unaffected. After user confirmation,
send the exact reviewed set in `expected_device_ids`:

```json
{
  "schema_version": "1.0",
  "operation_id": "ui-remove-account-1",
  "timeout_ms": 10000,
  "request": {
    "method": "remove_account",
    "account_id": "00000000-0000-4000-8000-000000000001",
    "expected_device_ids": ["00000000-0000-4000-8000-000000000002"]
  }
}
```

The set may be empty, contains at most 4096 distinct nonempty local IDs, and never
expands implicitly. `ACCOUNT_SCOPE_CHANGED` requires reviewing and confirming the
new scope; do not automatically replace the set and retry. A linked card with
routes outside this account/provider returns `ACCOUNT_REMOVAL_UNSUPPORTED` without
mutation. Account/registration lease contention returns `RESOURCE_BUSY`. Mock
refuses the method; Meiju accepts only its admitted provider/region.

Query data contains `account_id`, sorted `device_ids`, `status`,
`remote_revocation: "not_attempted"`, and `error`:

| Status | Meaning and next UI action |
|---|---|
| `removed`, null inner error | Local cleanup is complete. Reload cards/accounts; retain room and receipt views. |
| `pending`, non-null inner error | Cleanup intent is saved, but credentials or metadata may remain. Display the error and offer **Retry local cleanup** explicitly. |
| Outer query error | Admission or intent persistence failed; do not display success. Reload metadata after an interrupted caller. |

After intent the account appears as `credential_state: "removing"`; ordinary session
use and rename are refused. Local details remain readable without secrets. Pending
cleanup may block new connections or cross-account inventory validation until it
finishes. Since API 1.8.1, removal and explicit retries delegate exact-reference
inspection to the backend without reading/decrypting credentials. Missing keys or
corrupt contents can therefore be removed when the backend can identify and delete
that exact entry. Locked, unavailable or ambiguous storage still leaves cleanup
pending; unreadable never means absent. A confirmed absent entry causes no deletion.
See [ADR-0033](adr/0033-delete-unreadable-account-credentials.md). Earlier libraries
can remain pending when the credential pre-read fails; offer a library update.
Repeating completed removal returns its persisted result without opening the keyring.
Startup never resumes deletion automatically. Background/cancel/close retains intent;
forced termination promises no callback. Late discovery/cache saves cannot resurrect
removed cards. Existing appliance receipts retain their outcomes and may still finish
journal persistence after the account lease is released.

Reconnecting after removal creates new local IDs, without recovering old aliases,
room assignments or write grants. Removal is not secure erasure of backups, retained
receipt metadata or OS storage pages. No provider-revocation option is supported.
The [fictional examples](../packages/native-contract/account-examples.json) cover
complete and pending results; English/Chinese catalogs provide scope and retry text.

This version migrates SQLite 1/2/3 to 4 with a verified backup. Close older clients
before opening the profile and update native/UI code together: schema-3 binaries
refuse the upgrade, and strict older readers reject `removing`. The seven C symbols
and appliance command/receipt schemas retain their meaning.

## Account connection and refresh

API 1.2.0 adds two ordinary JSON methods and one separate native function under
[ADR-0026](adr/0026-native-account-connection.md). It does not add a general JSON
login method. Check the API minor version and method catalog before resolving the
new symbol; older native libraries lack it. Mock lists no accounts and refuses
connection/refresh with `AUTH_FLOW_UNSUPPORTED`. Meiju currently admits mainland
China only. No HomeVerb account or project server is involved.

`list_accounts` returns local `id`, `provider`, `region`, `label`, `credential_state`
(`pending`, `stored` or `removing`) and `authorization: "not_checked"`. It never opens the secret
store or contacts the provider. A stored session is not proof of current authorization.
No provider subject, credential reference, guessed masked login, permission scope
or last-check timestamp is exposed. Display the local label as the account name.

`refresh_account` takes one local UUID. It retrieves that account's inventory with
the saved session, validates historical/current cross-account identities, and
publishes admitted cards through Core. Other accounts' protected bindings may be
checked for ambiguity, but their inventories are not refreshed. It never logs in.
The returned `authentication` is `not_attempted`. Unknown IDs return
`ACCOUNT_NOT_FOUND`; callers cannot substitute a provider identifier or credential.

### Protected connection function

```c
char *homeverb_connect_account(uint64_t handle, const char *metadata_json,
                              uint8_t *login, size_t login_length,
                              uint8_t *password, size_t password_length);
```

The [metadata schema](../schemas/native-connect.schema.json) accepts only:

```json
{
  "schema_version": "1.0",
  "operation_id": "ui-connect-1",
  "timeout_ms": 180000,
  "label": "Meiju account",
  "account_id": null
}
```

Omitted/null `account_id` matches the authenticated provider subject or creates a
new local account. Supply an existing UUID for explicit reauthorization; a changed
subject is rejected without replacing that account. The label is a local display
name for a new account, not a login identifier. Existing labels/preferences survive.

Run on a worker and pass disjoint writable native byte buffers of 1–256 bytes.
Lengths exclude any NUL terminator. Current Meiju input validation requires bounded
ASCII input. Native code copies valid buffers into zeroizing storage and clears
those caller buffers before parsing metadata or starting work, including ordinary
error returns. It never dereferences NULL/oversized inputs. The wrapper must still
clear/free every allocation in `finally`, including invalid input paths, and clear
UI controllers after submission. Do not log, persist, serialize, send to analytics
or put credentials in general JSON. This does not promise erasure of copies made by
Flutter, the keyboard, OS or managed strings. No secret belongs in an operation ID.

This function shares the normal eight-operation registry, deadline and lifecycle
cancellation. Use a unique `operation_id` across both entry points. Backgrounding
or closing cancels active networking. Returning foreground does not retry anything.
There is no staged-progress callback; show a single connecting/retrieving state
until completion. The UI collects credentials before the native budget starts.

### Interpret account results

Validate connect output with `native-response.schema.json#/$defs/connect_account`
and refresh output with `$defs/refresh_account`. Both use the normal query envelope.
The [fictional account examples](../packages/native-contract/account-examples.json)
cover successful, partial, expired and cancelled retrieval. Actual synthetic host
responses are separately validated against these schemas in Rust integration tests.

After confirmed login and secure session storage, connect automatically retrieves
and publishes cards without another Find Devices call or duplicate inventory query.
Its data has `account`, `authentication: "succeeded"`, and `retrieval`:

| Retrieval status | Fields and UI action |
|---|---|
| `complete` | `device_ids` lists the current retrieved cards, including unsupported models; `error` is null. Load `list_device_cards` and describe/read supported components. An empty list does not delete older cards. |
| `partial` | Empty `device_ids`, fixed error `MIDEA_INVENTORY_PARTIAL`; existing cards remain. Offer explicit `refresh_account`, not another login. |
| `failed` | Empty `device_ids` and a fixed error; handle the cause. `AUTH_REQUIRED` needs explicit reauthorization; identity conflict needs reconciliation; cancellation must not automatically retry. |

Top-level query errors mean no completed connection report is available. After
`SECRET_STORE_WRITE_UNKNOWN` or interruption, reload local summaries before deciding
what happened. Pending sessions require explicit `recover_account` or the
[scoped reauthorization flow](#reauthorize-a-pending-account) in API 1.8. Refresh and
connection without an existing account ID do not reset or overwrite them. If inventory-binding storage becomes uncertain, the
returned account state is pending. Even when authentication succeeded, callers must
inspect `retrieval.error`; an outer `error: null` does not mean devices were loaded.
Core validates the complete descriptor batch before storage, but a later storage
failure can leave earlier cards saved. Reload cards after errors; do not imply rollback.

Connection, refresh and local metadata methods grant no appliance writes or provider
revocation. Only explicit removal cleans this installation's account. Sharing, mobile
secure stores and phone lifecycle remain unverified.

On October 15, 2026 a Linux C ABI run listed the owner's saved account and used
`refresh_account` to publish six existing cards without login. All seven component
read/cache pairs passed response-schema validation; preferences were unchanged.
This is saved-session acceptance only. Successful secret-input login-to-card flow
is synthetic-tested; real reauthorization through the new symbol remains unverified.

## Recover interrupted account storage

API 1.7 adds `recover_account` under [ADR-0030](adr/0030-explicit-meiju-account-recovery.md).
After `SECRET_STORE_WRITE_UNKNOWN` or a lost connection result, reload `list_accounts`.
For `credential_state: pending`, offer **Recover saved authorization** as an explicit
foreground action. Explain that it uses the saved authorization to retrieve cloud
inventory and preserves established local organization; it sends no appliance controls.

```json
{
  "schema_version": "1.0",
  "operation_id": "ui-recover-account-1",
  "timeout_ms": 10000,
  "request": {
    "method": "recover_account",
    "account_id": "00000000-0000-4000-8000-000000000001"
  }
}
```

The result uses `$defs/recover_account`, the same query data as `refresh_account`:
`account`, `authentication: not_attempted`, and `retrieval`. Complete retrieval with
stored credentials permits reloading the device cards. Inspect both credential state
and retrieval error: validation can fail while pending, a new binding-map save can
become pending again, or publication can fail after credentials become stored.
An outer null error alone is not a complete recovery result.

Recovery validates the protected local account/session association, obtains complete
inventory, then rereads and confirms those exact credential bytes without rewriting
them. New appliances use the normal binding-save path. Previously saved IDs, rooms,
aliases and favorites remain; inventory is reused for publication without a second
cloud retrieval. Cloud inventory acceptance does not prove physical device state or
independently establish a new account subject. Summary authorization stays `not_checked`.

If already stored, the request performs ordinary explicit refresh. `removing` accounts
return `ACCOUNT_REMOVAL_PENDING`; missing/foreign account IDs are refused. Missing,
malformed, changed or rejected credentials are never reset. Repeated recovery cannot
repair an expired session. API 1.8 allows explicit reauthorization of the original
account when protected identity/bindings are intact. Missing/corrupt protected data
remains refused; preserve local data and explain the limitation. Removing that
account and connecting again is a separate user-confirmed path that loses its old
organization and grants.

Cancellation, deadline, backgrounding or close stops networking and leaves the last
persisted credential state. Forced termination has no guaranteed callback. Reload local
summaries after interruption; never run recovery automatically on startup or page open.
Mock refuses the method, older hosts reject it, and live recovery/mobile acceptance
remain unverified. No new C symbol or database migration is required.

## Reauthorize a pending account

API 1.8 extends the existing `homeverb_connect_account` function under
[ADR-0031](adr/0031-pending-account-reauthorization.md). Require native version 1.8.0
or later before offering this action for pending credentials. It uses the same
protected input buffers and connection response; there is no new JSON method.

On the selected account page, offer **Sign in to the original account** after
expired recovery or as an explicit alternative to saved-session recovery. Explain
that successful identity validation retains existing device organization. Supply
that exact local UUID as `account_id` in the connection metadata; never substitute
null or silently create a new account if reauthorization fails. A fresh password
login is initiated only by this explicit submission.

Native code validates the protected payload/session association before login,
requires the newly authenticated subject to match it, and rereads the exact bytes
before replacement. Missing, malformed or foreign material refuses before login;
`ACCOUNT_IDENTITY_CONFLICT` refuses a different authenticated account. A changed
entry during login returns `ACCOUNT_SESSION_UNCONFIRMED`. None permits reset or merge.

An outer connection error provides no completed report: reload local account
summaries. Uncertain storage or failed final metadata confirmation leaves pending;
offer saved-session recovery rather than automatically submitting credentials again.
After successful persistence, `authentication: succeeded` and stored credentials
are independent of the nested inventory/publication result. Inspect both, as in the
ordinary connection flow. Labels, IDs, rooms, aliases, favorites and existing grants
remain; login does not broaden control permission. Cancellation/backgrounding follows
the same foreground rules. Only synthetic login/account evidence exists for this
extension; real reauthorization and mobile acceptance remain unverified.

## Scoped experimental AC fan startup

API 1.9.0 adds independent `meiju_fan_grants` under
[ADR-0034](adr/0034-scoped-meiju-ac-fan-control.md). Negotiate the version on a
read-only handle first; older libraries reject this field. Trusted startup can
supply up to 64 unique pairs:

```json
{
  "schema_version": "1.0",
  "data_dir": "/absolute/private/homeverb-data",
  "adapter": "midea.meiju",
  "allow_mock_writes": false,
  "meiju_fan_grants": [
    {"device_id": "30000000-0000-4000-8000-000000000001", "wind_speed": 40}
  ]
}
```

Each pair permits only `main / vendor.midea.ac.set_fan_speed` with that exact
`wind_speed` and cloud route. The reviewed codes are 20, 40, 60, 80, 100 and 102,
corresponding to the pinned source's silent, low, medium, high, full and auto
presets. These are vendor codes, not percentages or universal fan enums. Only the
four admitted AC models can expose the action after explicit discovery/refresh.
See the fictional [capability](../examples/capability.meiju-fan.json) and
[command](../examples/command.meiju-fan.json); a readable different code does not
become a writable preset.

Obtain explicit user consent before supplying grants. Fan permission never grants
power/mode/temperature changes; power grants never grant fan changes. Empty lists
grant neither action. Neither ordinary requests nor descriptors can provide or
persist authority. Use the existing close/reopen procedure to change permissions.

Render the action from capability metadata. The presentation catalogs provide
`vendor.midea.ac.set_fan_speed`, `vendor.midea.ac.wind_speed` and code-suffixed value
labels; unknown observations retain a fallback value. UI code must not infer
controls from the brand or add other codes. Sending changes only the fan field,
without powering on, changing mode or copying cached state. Devices may ignore it
in off/auto/dry or other managed modes; do not send a second command to force it.
Keep accepted/unknown receipts unverified and let the user explicitly refresh.
Upstream's usual entity path supplies cached `command.status`; HomeVerb omits it.
Physical acceptance must check that the vendor encoder preserves power, mode and
temperature without that context. No physical fan control has been tested,
including away from home.

## Scoped experimental AC mode activation startup

API 1.11.0 adds independent `meiju_mode_grants` under
[ADR-0039](adr/0039-explicit-meiju-mode-activation.md). Negotiate API 1.11 or later
on a read-only handle before supplying this field; older binaries reject it.
A fictional configuration granting only one combined action is:

```json
{
  "schema_version": "1.0",
  "data_dir": "/absolute/private/homeverb",
  "adapter": "midea.meiju",
  "allow_mock_writes": false,
  "meiju_mode_grants": [
    {"device_id": "30000000-0000-4000-8000-000000000001", "mode": "cool"}
  ]
}
```

Each entry authorizes `main / vendor.midea.ac.activate_mode {mode}` over cloud:
**turn on the AC and select the mode** in one provider request. Allowed vendor
values are `cool`, `heat`, `auto`, `dry` and `fan`; `off` belongs to the independent
power action. Limit each list to 64 distinct canonical local UUID-v4/mode pairs.
An omitted/empty list grants nothing. Power and fan grants cannot authorize mode
activation; mode activation grants cannot authorize independent power/fan changes,
temperature changes or another mode. Ordinary requests cannot modify these grants.
Changing trusted startup authority requires closing and reopening the session.

After explicit discovery/account refresh, the four admitted AC models can expose
capability revision 3 with this action. Render the exact descriptor and its mode
enum; do not infer availability from brand or readable mode values. See the
fictional [capability](../examples/capability.meiju-mode.json) and
[command](../examples/command.meiju-mode.json). The presentation catalogs provide
`vendor.midea.ac.activate_mode`, mode-value labels and
`vendor.midea.ac.activate_mode.confirm` with `{device}` and `{mode}` placeholders.
Both the control label and consent must make the power-on effect visible.

The adapter sends only `power: on` and the selected `mode`; no separate power
request, cached status, temperature or fan value is included. One request does
not establish physical atomicity: `atomic` and `independent_fields` remain false.
A code-zero reply is `accepted`, never physical verification; possible sends stay
`unknown`, with no automatic read, cache update, retry or restoring command.
Source evidence does not establish model/firmware behavior or preservation of
unrelated settings without cached status context. Physical acceptance remains
deferred. The 31 JSON methods, seven C functions and SQLite schema 4 remain
unchanged. See the protected product-profile migration below before downgrading
a library.

## Scoped experimental AC temperature startup

API 1.12.0 adds independent `meiju_temperature_grants` under
[ADR-0041](adr/0041-profile-scoped-ac-temperature.md). Negotiate API 1.12 or later
on a read-only handle before supplying this field; older binaries reject it.
A fictional configuration granting one exact target is:

```json
{
  "schema_version": "1.0",
  "data_dir": "/absolute/private/homeverb",
  "adapter": "midea.meiju",
  "allow_mock_writes": false,
  "meiju_temperature_grants": [
    {"device_id": "30000000-0000-4000-8000-000000000001", "target_temperature": 26.5}
  ]
}
```

Each entry authorizes only `main / climate.set {target_temperature}` over cloud
for that local device and exact Celsius value. At most 64 distinct canonical local
UUID-v4/target pairs are allowed. The startup schema admits 16–30 in half-degree
increments as the union of reviewed targets; **this is not a universal device
range**. No other temperature, power, mode or fan change is authorized. Empty or
omitted lists grant nothing. Ordinary requests cannot edit authority. Obtain
user permission in the trusted controller and close/reopen to change grants,
following the same lifecycle and receipt rules as the other control lists.

After explicit discovery/account refresh, climate revision 2 exposes `set` only
for one of the four admitted AC models with a supported protected product profile.
The adapter selects subtype first, then exact product code, then an explicitly
supported category; missing evidence and unsupported higher-priority mappings
never become a generic family default. Exact mapping ranges, steps and exclusions
are listed in ADR-0041. Profile evidence remains private: Flutter renders the
returned action, without branching on brand, model or product code.

Read `actions.set.parameters.properties.target_temperature` for writable minimum,
maximum and `multipleOf`, together with the property's Celsius unit. Do not use
read-property limits to construct this input: a valid historical observation may
be outside today's writable range. Validate before `plan`, keep requested values
separate from observations, and hide the setter when the action is absent.
API 1.12.1 also decodes the selected single-field temperature profile on explicit
reads, including read-only sessions: its `temperature` value does not require or
combine with `small_temperature`. Other profiles keep the two-field contract;
missing product evidence never infers a scalar target. Read bounds/steps remain
independent from write constraints, and observations retain unproven freshness.
A grant cannot supply missing capability or permit a half-degree value on a
whole-degree profile. See the fictional [capability](../examples/capability.meiju-temperature.json)
and [command](../examples/command.meiju-temperature.json). Existing standard climate
labels in both presentation catalogs apply; no vendor-specific screen is needed.

The adapter reselects the protected profile and checks arguments before sending.
One request contains only `temperature` and, where required, `small_temperature`;
it never rounds an invalid input, sends power/mode/fan fields, copies cached status,
or performs an automatic read, retry, fallback or restoring command. Both
`atomic` and `independent_fields` are false. Code-zero acknowledgement remains
`accepted`, with `VERIFICATION_PENDING` when requested; possible transmission
remains `unknown`. Cached observations are unchanged. Preservation of other settings
without upstream status context, model/firmware behavior and off-LAN operation
still require physical acceptance. No real device was operated in this phase.
The 31 JSON methods, seven C functions, SQLite schema 4 and protected payload 3
are unchanged.

## Verify the handoff

```bash
cargo build --workspace --offline --locked
cargo test -p homeverb-host --test native_contract --offline --locked
.venv/bin/python scripts/verify_native_contract.py
```

The native suite calls all 31 catalog methods, checks their schemas, verifies closed-field rejection, null success, pre-admission errors, composite/batch results, all execution outcomes, cancellation, foreground gating and closed handles. It also validates the saved examples and Meiju startup/account admission guards, caller-buffer clearing, local detail/rename persistence, removal admission/ledger replay, explicit cache/history clearing, retired-request refusal, recovery admission and account counterexamples without a live account. `--write-examples` regenerates those examples only from a successful fictional run; review and commit the changes together with the contract. Existing full MVP tests still cover execution and persistence behavior.

This is Linux native verification. Dart FFI allocation, worker scheduling, mobile library packaging, secure stores, system-browser lifecycle, and physical-phone behavior must be tested in the Flutter worktree. No Flutter build has been run for this delivery.

## Missing devices and legacy account inventory

API 1.9.1 gates live Meiju reads and controls on the protected last complete account
inventory under [ADR-0035](adr/0035-meiju-inventory-membership.md). The public card
and connection shapes are unchanged. Keep historical cards, rooms, favorites,
cache and receipts; do not infer connectivity from a card or a refresh result.

- `MIDEA_DEVICE_NOT_LISTED`: the last complete retrieval omitted this device. Show
  the localized explanation and offer explicit `refresh_account`. This does not
  establish physical removal, offline status or provider revocation.
- `MIDEA_INVENTORY_REQUIRED`: an older protected binding has no complete-inventory
  membership evidence. Offer the same explicit refresh, without another login.
- Partial/failed retrieval does not change established membership. A complete
  retrieval can restore the same immutable device identity without losing local
  organization. Do not automatically retry a control after a refresh.

Local lists, cached reads, descriptions and plans do not consult the protected
membership. A valid plan is not a promise that execution will be admitted. A live
read returns a query error; a control returns `failed` with a `not_sent` attempt.
Neither error starts network work or background retries. A late descriptor cannot
restore private membership. No cached availability field is added in this phase.

Protected payload version 1 opens without migration or network access, but live
work requires explicit complete retrieval. API 1.11.1 preserves versions 1 and 2
on reads; later explicit protected updates write version 3 as described below.
Preserve the profile and upgrade the library; never reset credentials to downgrade.
SQLite schema 4, 31 JSON methods and seven C functions remain unchanged. No owner
profile was migrated during development.

## Inspect saved account inventory

API 1.10 adds `inspect_account_inventory` under
[ADR-0036](adr/0036-local-account-inventory-inspection.md). Check method availability
before exposing it. This foreground operation reads the selected account's saved
protected bindings and does not contact Meiju, refresh state or change credentials.
Unlike `get_account`, it requires readable protected material and an available
account lease. Use the existing account recovery/removal flows for pending accounts.

```json
{
  "schema_version": "1.0",
  "operation_id": "inspect-saved-devices",
  "timeout_ms": 10000,
  "request": {
    "method": "inspect_account_inventory",
    "account_id": "00000000-0000-4000-8000-000000000001"
  }
}
```

Successful `data` contains `account` (the existing public summary, with
`authorization: not_checked`) and `devices`, sorted by `device_id`. Each entry has
only `device_id`, `connection_id` and `membership`:

| Membership | Meaning | Suggested presentation |
|---|---|---|
| `listed` | Present in the last complete saved inventory | Show saved membership; do not infer online status, physical support or permission |
| `not_listed` | Absent from the last complete saved inventory | Explain that live requests are blocked and offer explicit account refresh |
| `unconfirmed` | No complete saved inventory has established membership | Offer explicit account refresh; legacy entries are not migrated by inspection |

Join only matching device and connection IDs on existing cards. Historical and
unsupported bindings remain present; partial retrieval may retain an unpublished
binding. Never create cards, merge identities or change grants from this response.
An empty saved list is not proof that the cloud account has no devices. There is
no trustworthy retrieval timestamp in this payload version; do not invent one.

Locked/unavailable/corrupt secrets, pending/removing accounts and lock contention
return query errors, not an empty list. There is no automatic cloud fallback.
Opening the app does not inspect all protected accounts; use the explicit selected
account action and avoid polling. Ignore outdated callbacks after navigation,
cancellation or backgrounding. A returned `listed` value can become outdated;
Core rechecks protected membership and other admission rules before a live request.
See the [fictional three-state example](../packages/native-contract/account-examples.json).
The seven C functions and SQLite schema 4 are unchanged. Product mapping evidence
is deliberately absent from this public response.

## Protected product-profile migration

API 1.11.1 adds protected product mapping evidence under
[ADR-0040](adr/0040-protected-meiju-product-profiles.md). Explicit account retrieval
can retain the provider product code, subtype and category inside the protected
account entry. These values are not device identifiers for Flutter to select,
are never returned in ordinary results and grant no additional capability.
Existing controls and all 31 native methods keep their contracts.

Payload versions 1 and 2 open without writes or invented evidence. A subsequent
explicit session/binding update writes version 3. Older native libraries cannot
read version 3: upgrade the library rather than clearing credentials, resetting
bindings or falling back to plaintext. SQLite schema 4 is unchanged. Missing
profile fields do not erase known evidence; conflicting values return
`DEVICE_IDENTITY_CONFLICT` before binding/card publication. Malformed fields make
that home's inventory invalid; incomplete retrieval preserves existing cards and
membership. Uncertain protected saves use the existing pending/recovery flow.
API 1.12 uses this protected evidence for the reviewed temperature mappings above.
Physical temperature acceptance and appliance-cycle support remain outstanding.
