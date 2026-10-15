# Native UI contract 1.5

Status: implemented Linux native contract, October 15, 2026, with Mock, default read-only Meiju startup, protected account connection and scoped experimental AC power grants. Six real devices passed the native read path; phone packaging and complete provider/physical acceptance remain pending. See [ADR-0019](adr/0019-native-ui-contract.md).

## Start here in a Flutter worktree

Branch from a commit containing both the Mock MVP and this contract. Put the Flutter application in `apps/mobile/`; do not copy Rust code into the app. The UI worktree owns Flutter views, routing, accessibility, local UI preferences, Dart FFI wrappers, and mobile build configuration. Core work owns native methods, schemas, capability semantics, storage, adapters, and their fixtures. Agree on schema changes before depending on them. Each verified phase is a separate English Conventional Commit; a local commit does not imply a push.

Read the [UI integration guide](development/03-ui-integration.md) for existing behavior and the [functional handoff](ui-functional-spec.md) for screens. Use [api.json](../packages/native-contract/api.json) as the callable-method catalog and [examples.json](../packages/native-contract/examples.json) as fictional request/response examples. The examples were obtained from the built native library. They contain temporary fictional IDs and observations, not defaults for a production device.

Startup explicitly selects Mock or Meiju; Meiju remains read-only unless the trusted host supplies the scoped power grants below. Meiju uses an existing local account or the protected connection function below; opening the library alone does not establish connection or authorization. Preview-only refrigerator, dishwasher, and sharing screens can use isolated UI fixture repositories; never mix those records with a native installation or label them live.

## Startup, ownership, and version negotiation

The seven C functions in [homeverb.h](../crates/ffi/include/homeverb.h) remain the native boundary. There is no HTTP endpoint, port, CLI subprocess, or daemon.

1. Load the platform library and call `homeverb_open` from a worker isolate/thread. Supply schema `1.0`, an absolute OS-private data directory, and an explicit `allow_mock_writes` boolean. `adapter` is optional and defaults to `homeverb.mock`; `midea.meiju` selects saved-account cloud reads. `scenario` defaults to `normal`. Meiju requires `allow_mock_writes: false` and rejects any other scenario before opening the profile. All startup values come from trusted app code, never imported commands. The [startup schema](../schemas/native-open.schema.json) defines fields; Core checks OS path semantics.
2. Decode the open query envelope. On success, retain `data.handle` as an unsigned 64-bit native handle; `data.adapter` confirms `homeverb.mock` or `midea.meiju`. Do not persist handles across app launches. Errors have null data. Free every returned native string exactly once, including errors.
3. Call `get_api_info` with a new operation ID before enabling native features. Require API major version `1`, and check individual methods in `data.methods`. The current `api_version` is `1.5.0`; JSON envelope schema remains `1.0`. Older hosts return `INVALID_ARGUMENT` for this query: show a native-library upgrade requirement. Unknown optional response fields may be retained/ignored, but unknown outcomes must not become success.
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
| `list_accounts` | None | Local account summaries; no credentials or network work |
| `get_account` | `account_id` | `{account, device_ids}` from local metadata; no authorization check |
| `rename_account` | `account_id`, `label` | Updated local AccountSummary; no credential or permission changes |
| `remove_account` | `account_id`, `expected_device_ids` | Local cleanup result: `removed` or `pending`; inspect the inner error |
| `refresh_account` | `account_id` | Account and retrieval/publication result; no login |
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
| C01–C14: dynamic controls, history and outcomes | Describe/read/plan/execute/batch/receipt methods | Render exact capabilities; default Meiju discovery has read-only AC/fridge/dishwasher mappings; native power grants may admit experimental AC on/off. Other consequential real approvals remain unavailable |
| A01–A08: device discovery and addition | Explicit selected-adapter `discover`, including saved-account Meiju cloud retrieval | Real pairing, mobile permissions and safe identity merge remain unavailable. No merge API exists |
| P01–P13: platforms, accounts, authorization and removal | `list_accounts`, `get_account`, `rename_account`, `remove_account`, `refresh_account`, and separate `homeverb_connect_account` | Flutter protected input, cleanup confirmation/recovery and phone secure stores; provider revocation remains unavailable |
| H01–H16: nearby authorization sharing | No sharing methods in this version | Eligibility, peer discovery, code verification, scope, consent, conflict/import and transfer records require ADR-0008 implementation |
| S02/S11: Window Light appearance | Flutter-owned private preferences | Persist `window_light`, fallback to it, and provide one preview; no native theme API or download capability is needed |
| S03/S10, D07: sharing name and route choices | Flutter-owned private preferences plus explicit Command/ReadState `connection` | Store only nonsensitive display preferences. Resolve device > installation > `auto`; Core still enforces allowed routes. Do not add route fields to `set_device_preferences`. A saved sharing name does not start advertising |
| S04/S09: clear cache/history/all data | No cleanup method in this version | Disable destructive native actions until scoped APIs handle active journals, retired IDs, credentials, and partial failures; never delete SQLite files from Flutter |
| S05/S06: recent operations | Local receipt queries | Appliance receipts work; transfer history remains unavailable |
| S07/S08/S12: diagnostics, export and About | `diagnostics`, API metadata, repository compatibility/licenses | Flutter OS export/chooser, cancel/failure handling, accurate build labels. Export the previewed allowlist only |
| W01/W02 and all screens | Same contract at all sizes | Flutter navigation, text scaling, semantics, focus, system permissions and lifecycle binding |

Flutter-owned preferences contain no credentials and grant no authority. Room/card organization remains owned by Core. Route controls must reflect actual connections and action capabilities, not brand labels. Localization uses the shared `en-US`/`zh-CN` catalogs; IDs, arguments and outcomes stay English. Preserve unknown error codes and use a generic localized fallback instead of mapping them to success.

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
vendor calls. Mobile secure stores and Android/iOS packaging remain unimplemented.

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
routes are refused; `auto` can use only cloud. These experimental grants admit no
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
finishes. An explicit retry with the same scope checks the referenced secret before
any deletion; an earlier successful but unconfirmed deletion is not repeated.
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
what happened. Pending sessions require trusted recovery; this API does not reset,
delete or overwrite them. If inventory-binding storage becomes uncertain, the
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

## Verify the handoff

```bash
cargo build --workspace --offline --locked
cargo test -p homeverb-host --test native_contract --offline --locked
.venv/bin/python scripts/verify_native_contract.py
```

The native suite calls all 27 catalog methods, checks their schemas, verifies closed-field rejection, null success, pre-admission errors, composite/batch results, all execution outcomes, cancellation, foreground gating and closed handles. It also validates the saved examples and Meiju startup/account admission guards, caller-buffer clearing, local detail/rename persistence, removal admission/ledger replay and account counterexamples without a live account. `--write-examples` regenerates those examples only from a successful fictional run; review and commit the changes together with the contract. Existing full MVP tests still cover execution and persistence behavior.

This is Linux native verification. Dart FFI allocation, worker scheduling, mobile library packaging, secure stores, system-browser lifecycle, and physical-phone behavior must be tested in the Flutter worktree. No Flutter build has been run for this delivery.
