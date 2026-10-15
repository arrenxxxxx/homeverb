# 5. Integrate Midea and Hualing appliances

The owner selected Midea and Hualing air conditioners, dishwashers, and refrigerators as the first real-device scope. The primary journey uses appliances already bound in Midea Meiju: authorize that account inside HomeVerb, automatically retrieve its accessible devices, and read/control supported capabilities through Midea's cloud at home or away. HomeVerb has no account of its own. Re-pairing, manual IP/Token entry, and a home gateway are not prerequisites.

Daily cloud control is required in the first real integration. LAN is an additional route for the same physical devices, not a substitute for cloud control. The owner has physical products; exact models, firmware, app version, and account region remain to be recorded. This guide records scoped implementation and source evidence on October 15, 2026; no model is physically verified.

The native read protocols are implemented in [homeverb-midea](../../crates/midea/src/lib.rs): account-scoped home/appliance retrieval, code/data validation, deduplication, partial inventory failure, and bounded cloud-state decoding for the three selected families. The inventory/state decoders use an injected read transport and synthetic responses with the inspected wire fields. Native password login, signing, session-key decoding and restricted HTTPS are now implemented under ADR-0022 and tested against fictional responses and a local TLS server. One real mainland account has passed the read-only login/inventory probe below; the exact-model read-only Core adapter is available through `core-inspect`, and native API 1.1.0 startup selection. Native API 1.2.0 also provides protected account connection and local list/refresh; Flutter input and mobile secure stores remain pending. See [ADR-0020](../adr/0020-midea-cloud-inventory.md) and [ADR-0023](../adr/0023-meiju-cloud-state-evidence.md).

The native account vault now persists random local references, composes Linux secure storage, and coordinates session read/update work through account leases. Meiju's private account payload also preserves appliance-to-local-ID bindings with the session. Interrupted or uncertain updates remain pending until trusted validation and recovery. Native API 1.7 provides an explicit [recovery and retrieval flow](../native-ui-contract.md#recover-interrupted-account-storage) that validates saved material without another login. The explicit native connection coordinator now uses this storage and the admitted login client; real account identity and session validity still require live acceptance. See [ADR-0021](../adr/0021-persistent-account-vault.md), [ADR-0024](../adr/0024-meiju-private-device-bindings.md) and the [credential integration guide](04-add-adapter.md#integrate-native-credentials).

The next implemented phase adds an explicitly opted-in AC `power.set` path and
[preview-first control probe](07-meiju-control.md). Its Core grants and TLS send
progress are synthetic-tested. Native API 1.9 also adds independent vendor fan controls with exact-device/code grants.
Real control acceptance, temperature constraints, other settings, dishwasher/refrigerator
writes and Flutter consent UI remain pending.

## Reuse the community protocol work

The Home Assistant integration [Midea Auto Cloud][cloud-integration] demonstrates account-device enumeration and daily cloud control, and lists the `AC`, `E1`, and `CA` families. Its root license is Apache-2.0. The integration [Midea AC LAN][ha] lists Midea and Wahin/Hualing among its brands and includes all three requested families. [midea-local][library] is the Python LAN protocol library extracted from that project; both LAN repositories carry the MIT license. Preserve applicable license notices and attribution when adapting code, including Apache-2.0 modification/NOTICE requirements where applicable. HomeVerb's own release license decision remains separate.

Pinned sources inspected:

- `midea_auto_cloud`: `ce0c4b88224def0f6b53714312c19f71aa9c2a50`.
- `midea-local`: `ef28169926914d15f40fe521b9cc3d3c2f553c59`.
- `midea_ac_lan`: `7c5231ca0588d364aa30e3bb2b887b37d4359719`.

The projects have separate histories and cloud/LAN paths. Check their feature sets independently; a working upstream feature is not a HomeVerb implementation or a model-wide guarantee.

| Family | Protocol evidence | Initial HomeVerb target | Limits of the evidence |
|---|---|---|---|
| Air conditioner | `0xAC`; cloud integration family list, [HA feature guide][ac-guide], and [library messages][ac-source] | Cloud reads of power, mode, and available temperatures, then supported settings with read-back verification | Multiple message variants and model-specific capabilities exist. Do not apply one model's ranges or state format to every unit |
| Dishwasher | `0xE1`; cloud integration family list, [HA feature guide][e1-guide], and [library device implementation][e1-source] | Cloud operation status, remaining time, door state, and supported warnings; controls only with exact protocol and permission evidence | The LAN public setter handles power, child lock, and storage. A `MessageWork` class or cloud family listing alone does not prove a usable start-cycle flow. Sink dishwashers also appear as a separate `0x34` family |
| Refrigerator | `0xCA`; [cloud mapping][ca-cloud-source], [HA feature guide][ca-guide], and [LAN implementation][ca-source] | Cloud compartment temperatures and supported settings, with separate component capabilities | The cloud mapping declares refrigerator/freezer target temperatures. The LAN `MideaCADevice.set_attribute` is empty at the inspected revision; this limits that LAN path, not the cloud scope. Neither source verifies the owner's models or universal ranges |

Select the adapter and message variant using protocol evidence. The same implementation can serve Midea or Hualing devices using the same protocol, but a brand label is insufficient evidence. Midea and Hualing using one protocol family do not satisfy the separate two-protocol release gate.

## Account authorization and automatic device retrieval

The inspected [Meiju cloud client][meiju-cloud-source] supplies the following source evidence. These are community implementations of app-facing APIs, not proof of official public-client admission or stable public API contracts.

| Operation | Meiju path used by the inspected source | HomeVerb requirement |
|---|---|---|
| List accessible homes | `/v1/homegroup/list/get` | Use the authorized account and region; do not assume one home |
| List bound appliances | `/v1/appliance/home/list/get` | Traverse returned homes/rooms/appliances, retain account scope, and deduplicate by strong identity evidence |
| Query state | `/mjl/v1/device/status/lua/get` | Preserve cloud provenance and evidence age; a new HTTP response can contain old appliance state |
| Send control | `/mjl/v1/device/lua/control` | Map only supported commands; distinguish transport outcome, provider acceptance, and verified device evidence |

The user starts a foreground Connect Platform flow that explains account access and automatic device retrieval. After secure authorization, enumerate the account's accessible devices within a bounded operation and automatically add supported device cards. Do not require a second Find Devices action or per-device selection for this journey. Unsupported returned appliances remain visible with an explanation and no invented controls. A failed or partial enumeration is distinct from an empty account; keep authorization success separate and offer an explicit retry without forcing another login.

Repeated retrieval and reauthorization preserve local IDs, aliases, rooms, favorites, and established connection bindings. New devices start Unassigned. Vendor homes/rooms provide discovery context; they do not overwrite local organization. Ambiguous matches and changes to an existing account/route binding need the explicit identity-resolution flow under [ADR-0013](../adr/0013-preserve-device-connection-identity.md). Partial lists must not silently remove existing devices. This authorization-triggered retrieval does not introduce page-open scans, background synchronization, or periodic polling.

The cloud sources include distinct Meiju and SmartHome login paths and app-specific signing material. Review their meaning and permitted native-client use before enabling login. An upstream hardcoded constant is not proof of public-client authorization. Only the pinned public Meiju app-compatibility constants are admitted under the owner-approved ADR-0022 exception. Do not copy default appliance keys, user credentials, or confidential developer secrets into HomeVerb. Account persistence and refresh use opaque references, system secret stores, and on-demand refresh coordination. Never ask for credentials in chat, command arguments, or ordinary diagnostic output.

The October 15 authentication review found that the official v3 token endpoint requires a developer secret; the Android SDK guide describes obtaining tokens through a third-party server. Those documented paths do not establish the required independent native flow. The community app-facing login uses shared app signing and decoding material. [ADR-0022](../adr/0022-meiju-community-cloud-admission.md) records the primary sources and the narrowly scoped community-protocol exception explicitly approved by the owner on October 15, 2026. Protocol implementation may now proceed; real account and model acceptance remain separate checks. Approval alone did not implement login; the separately tested native login and connection stages below now supply the implementation.

## Native login and restricted HTTPS

`LoginCredentials` consumes protected native account/password bytes. The pinned
community flow uses ASCII input; unsupported, empty or oversized inputs fail before
network access with `MIDEA_LOGIN_INPUT_UNSUPPORTED`. `login` performs exactly one
login-ID request and one password request under the same Operation. Rejection,
additional challenges, cancellation or timeout never trigger automatic login or
replay. Passwords and account spellings are not part of the stored session.

`MeijuSession` holds the access token, decoded session key and an opaque subject
`meiju.login-id.v1:<server-login-id>` in redacted native memory. The server login ID
participates in the successful password proof. It is an exact-match reconnect
guard, not a claim that all account aliases share an identity. Real-account tests
must confirm its stability. Changed IDs require explicit identity reconciliation; saved-session recovery does not reconcile subjects. Never substitute
a nickname, login spelling or guessed `userInfo` field. `encode`/`restore` are private
credential-persistence operations, not public JSON DTOs. `reads` binds a session to
one exact local credential reference for the existing inventory/state protocols.

`MeijuHttps` admits only `https://mp-prod.smartmidea.net` and the five reviewed login,
home, appliance and state endpoints. It exposes no configurable URL, proxy, region
reroute, certificate bypass, control endpoint or executable download. TLS uses
WebPKI roots; redirects and retries are disabled. Each call owns its client with no
idle pool, checks a 1 MiB response limit while streaming, and shares cancellation
and the total deadline. DNS/OS resolution and transient HTTP/TLS buffer copies
remain library/platform responsibilities; no guarantee of complete memory erasure
or preemptible system resolution is made. Provider errors and bodies never enter
public diagnostics.

The public compatibility material and codec are attributed in
[third-party notices](../../third-party/midea-auto-cloud/NOTICE.md), including the
upstream Apache-2.0 license. Independent Python hashlib/HMAC and OpenSSL synthetic
vectors test the port. Local TLS tests cover exact request bytes, sensitive headers,
redirect/retry refusal, certificate validation, oversized/truncated responses and
socket closure on cancellation/deadline. These tests neither contact Meiju nor
establish account, regional, mobile or physical-device support.

## Explicit connection and automatic retrieval

`connect(vault, transport, credentials, existing_account, label, operation)` acquires
a profile registration lock and checks secure-store access before login. With no
explicit account it compares protected subjects in the same provider/region and
reuses a unique exact match. An explicit account retains its lease through login
and refuses a changed subject. Without an explicit account ID, unconfirmed accounts fail before login. An explicitly
selected pending account may use fresh same-subject login under [ADR-0031](../adr/0031-pending-account-reauthorization.md),
after validating its protected identity/bindings and retaining the exact bytes for
a post-login comparison. No reset, deletion or duplicate-account fallback is attempted.

Successful login stores the session and then immediately retrieves/binds the
inventory. `ConnectReport` separates account persistence from `retrieval`, which may
be failed or partial. `MeijuAccount::retrieve` supports a later explicit retry with
the saved session, checks the encoded subject against the protected account and
retains the original account lease. Neither operation registers capabilities,
publishes Core cards, deduplicates appliances across accounts or authorizes writes.
The original coordinator remains a Rust call; the additive native 1.2 account entry points are described in the [Flutter contract](../native-ui-contract.md#account-connection-and-refresh).

The [local read-only probe](06-meiju-probe.md) exposes this flow to an interactive
Linux terminal with hidden input and no credential arguments. Its output contains
only local account references, family counts and fixed outcomes.

On October 15, 2026, the owner ran the hidden-input connect flow and reported a
stored session with complete retrieval: four air conditioners, one dishwasher,
one refrigerator, no other families and zero failed homes. A subsequent independent
probe process was executed against that exact local account with `refresh`; it
returned the same six-device counts and stored account reference without another
login. This establishes one real-account persisted-session inventory path on Linux.
Account names, local IDs, appliance identifiers and credentials are omitted here.

The subsequent explicit `inspect` probe returned mapped state for all six devices.
Four ACs supplied power, current/target temperature, vendor mode and wind-speed
code. The refrigerator supplied power and current/target temperature for both
compartments. The dishwasher supplied all eight currently mapped vendor fields.
No source observation timestamp was established; every projected value remained
stale. See the exact [cloud-read evidence](../compatibility.md#observed-cloud-read-models).

This evidence does not establish complete model/firmware support, physical state/control,
relogin subject stability, phone packaging or operation away from the home network.
Device binding/preference preservation remains covered by the synthetic suite;
real reauthorization and off-LAN acceptance remain to be recorded.

## Native account identity and device persistence

`MeijuAccount` owns an AccountVault lease for one bounded operation. After admitted native authentication validates an account subject and session, `create(vault, region, label, subject, session, operation)` reserves a fresh local account and stores a private version-1 payload. For an existing account, acquire its lease and use `open`; it never initializes or resets an absent/corrupt map. `record()` exposes local metadata, and `session(operation)` returns protected native bytes. These remain low-level Rust calls; the native UI exposes reviewed projections and operations through API 1.2 without returning private payloads.

The opaque subject must come from reviewed provider evidence, not a nickname or mutable login spelling. The native login flow below binds a successful password proof to the exact server-issued login ID; the storage layer itself does not infer a provider field. `replace_session` requires that exact subject and keeps the bindings. A mismatch returns `ACCOUNT_IDENTITY_CONFLICT` without changing stored state. Neither opening a payload nor matching the subject proves live authorization.

After retrieval, `bind_inventory` validates the inventory's credential reference and complete identity evidence before saving. It returns local device, cloud-connection and identity-reference UUIDs in inventory order only after confirmed persistence. Reuse those IDs when building descriptors. `resolve_device` returns a private appliance ID only for a device bound to that leased account. Keep the same lease for the future signed transport; do not acquire a second lease inside a request.

Repeated inventories preserve IDs. Missing entries never remove mappings, including after a complete empty list. Names, vendor rooms and online hints do not affect identity. Known family/model conflicts reject the whole update; an absent model can become known. Unknown families receive bindings without gaining capabilities. Unchanged identities and membership require no secret-store write. This storage helper does not publish Core descriptors. The read-only adapter checks protected current and historical bindings across accounts before publishing; ambiguity fails without silently merging devices.

The protected payload contains the immutable local account binding, subject/session bytes and private appliance map. Limits are 1 MiB total, 1024 subject bytes, 65536 session bytes, 4096 bindings, and 256 UTF-8 bytes per model. The system secret store protects the payload; SQLite contains no provider identifiers or session data. Debug/errors are redacted. Pending writes need exact-byte recovery through the vault before `open` can succeed; never retry by resetting or recreating the account. Tests use synthetic secrets and real SQLite, including unchanged local preferences after descriptor rediscovery. The Linux connect/refresh probe has exercised real account persistence; reauthorization and phone support remain pending.

For a Core adapter's default state read, use `read_bound_state(transport, device, component, connection, clock, operation)` on the leased account. It checks the exact local device and full cloud connection against the protected map, known model evidence, confirmed membership in the last complete inventory, and the requested component before issuing one request. The private appliance target comes from the map, not from UI arguments. Use an operation-scoped transport that already resolves the correct session under this lease; do not reacquire the account lock inside the transport. This helper does not discover capabilities or turn read properties into actions.

Core must admit the model's descriptor before using this path and validate all resulting observations before caching. The test-only adapter supplies fictional read-only descriptors for AC/E1/CA; it does not register real models. Undeclared fields fail Core validation, malformed fields fail decoding, and both preserve the previous cache. Cached reads remain local even when the secret store is unavailable. Cancellation or expiry drops the pending read and releases its lease when the adapter operation exits. AccountLock now requires Send + Sync so these immutable account reads satisfy Core's existing Send future contract. See [ADR-0025](../adr/0025-bound-meiju-core-reads.md).

## Model-scoped read-only Core adapter

`MeijuAdapter::new(vault, transport, clock)` performs no I/O. Explicit Core discovery
loads saved mainland Meiju accounts, retrieves complete inventories, and publishes
read-only descriptors for the six exact [observed model/family pairs](../compatibility.md#observed-cloud-read-models).
Unknown or mismatched models remain visible with unsupported status and an empty
`vendor.midea.inventory` capability. Brand is unknown until there is evidence;
names do not select semantics. AC mode/fan codes and all dishwasher fields remain
vendor properties. Refrigerator fridge/freezer components share one device ID.
There are no actions, writable properties, inferred enums or temperature limits.

Discovery holds the profile registration lock and all selected account leases
through retrieval and identity comparison. Current plus historical private bindings
must be unique across accounts. Conflicts return `DEVICE_IDENTITY_CONFLICT` without
merging. Any incomplete inventory returns `MIDEA_INVENTORY_PARTIAL` before the
adapter returns descriptors; Core keeps existing cards/preferences/cache. Private
bindings saved before an error remain stable, not rolled back. Complete empty
inventory also does not remove existing cards, but blocks live access to their
protected bindings under [ADR-0035](../adr/0035-meiju-inventory-membership.md). Cross-account reconciliation and
explicit retirement remain future work.

Reads check the exact admitted model/family against protected evidence and then
use the full bound connection/component validation above. Core validates and saves
the complete snapshot; failed responses preserve the previous cache. Every value
remains stale because provider observation time is unproven. Cached reads, local
cards and descriptions do not need credentials or network work. All operations
retain the caller's deadline and cancellation token; there is no login, refresh or
polling while idle.

The [core-inspect probe](06-meiju-probe.md#publish-read-only-cards-and-inspect-through-core)
exercised the production adapter on October 15, 2026: six real cards and seven Core
component reads succeeded with zero failures. No controls were sent. Unit/integration
tests separately cover exact and unsupported models, forged admission, partial and
empty retrieval, historical cross-account conflicts, preserved preferences/cache,
secure-store unavailability, cancellation/deadlines and lease release. Native UI
API 1.1.0 adds explicit Meiju startup while preserving the Mock default and existing
22 request methods. SQLite schema 3 is unchanged; see the [Flutter handoff](../native-ui-contract.md#real-meiju-read-only-startup).

## Daily cloud control and the additional LAN route

HomeVerb connects directly to Midea's servers for cloud reads and writes; the appliance must remain reachable through that vendor service. The phone can be on mobile data or another external network. No HA installation, home computer, NAS, project backend, VPN, LAN scan, or local-network permission is required for this cloud journey. A viable independent native authentication path is still required.

The [LAN setup documentation][ha] describes cloud onboarding to retrieve Token/Key material, followed by local operation. The [library connection code][device-source] uses a token/key handshake for V3; its V2 path does not invoke that handshake. Discovery, transport version, and appliance-message version are separate concerns. LAN-only credential import may support a separate local workflow; it does not satisfy the required account/cloud journey.

Use one physical Device with cloud and, when established safely, LAN connections. Preserve existing [routing rules](../routing-and-execution.md): explicit cloud works independently of LAN, explicit LAN never falls back, and Auto chooses a permitted action-supported route with LAN preferred when available. Never race writes or replay a possibly-sent write on another route. A cloud API success is not physical verification; keep accepted or uncertain outcomes honest when fresh evidence is unavailable.

## Fit the embedded Core

Reuse or port reviewed cloud requests, capability mappings, and LAN codecs into in-process Rust adapters. Keep the existing Core, capability contracts, and host architecture. There is no requirement to install HA, run a Python companion service, or call HA's control API.

The library's `MideaDevice` inherits from a Python thread and contains reconnect and refresh loops. Its CLI also documents plaintext credential and token/key caches. Those runtime and storage choices are unsuitable for direct reuse here. Replace them with the existing bounded Operation, cancellation, system secret store, and opaque local identity references. Do not copy raw packet or device-identifier logging.

The cloud integration also downloads protocol files and can run Lua locally. Do not transplant downloaded executable protocols or automatic polling into HomeVerb. Version one uses reviewed compile-time mappings. The inspected cloud control helper treats any non-`None` response as success; HomeVerb must separately establish provider acceptance and completion evidence. Do not copy optimistic attributes, default state values, or implicit command retries as verified behavior.

Every HTTP request and socket belongs to the current operation. A foreground read may authenticate, query, and close. A write uses Core's transmission phases and must never be replayed after an uncertain send. Do not declare independent setters without protocol evidence.

## Development sequence and acceptance

1. **Secure account and cloud enumeration:** review native login feasibility and implement bounded authentication/session handling, secure persistence, and automatic home/device retrieval. Test synthetic multi-home, empty, unsupported, partial/error, account/region isolation, repeated retrieval, expiry, and cancellation cases. This foundation does not require exact owner models.
2. **Cloud reads through Core:** map supported appliance data into capability descriptors and timestamped state. Keep refrigerator compartments on one Device and unavailable readings unknown. Test stale responses, offline devices, missing fields, identity preservation, and cleanup. Add compile-time registration and CLI/native entry points when usable.
3. **Cloud writes and physical acceptance:** record exact model/firmware/app/region, verify account retrieval and reads, then test explicitly authorized supported writes with independent read-back. Test provider rejection, expired sessions, lost acknowledgements, and no uncertain replay. A read-only prototype is an intermediate milestone, not the completed first integration.
4. **Additional LAN route:** port required framing/checksums/version handling and bounded sessions, including V3 where required. Use synthetic malformed/fragmented response tests before physical access. Add the route through reviewed identity confirmation without duplicating cards or weakening binding guards.
5. **Mobile acceptance:** implement the phone's secure store and native integration, then repeat the account-to-control flow with Wi-Fi disabled and no home computer/gateway. Test foreground cancellation and recovery. Desktop tests and protocol fixtures do not replace this acceptance.

The complete flow must demonstrate an existing Meiju account's devices appearing without re-pairing or manual device credentials, preserved organization after reauthorization, honest unsupported/offline/partial states, and permitted cloud reads/writes away from home. Cloud failure must remain visible; cached state or a working LAN path alone does not pass the remote-control gate.

Default priority is air conditioner, dishwasher, then refrigerator; this is an implementation order, not a model-support claim. Exact model information may change the order or reveal an unsupported protocol. Do not block source research, offline protocol tests, or adapter foundations solely because model details are pending. Preserve specific blockers for actual credentials, network access, and physical verification.

## Develop the native inventory stage

Run `cargo test -p homeverb-midea --offline --locked`. `ReadTransport` admits only the reviewed home-list, home-appliance and appliance-state read endpoints, with the exact account credential reference and caller's Operation. A future signed HTTPS transport must validate its fixed vendor destination, cap received bytes before allocating an unbounded body, and return response bytes through `SecretBytes`. It must not retry or reauthenticate implicitly. The protocol layer additionally rejects bodies larger than 1 MiB and applies the caller's shared deadline.

Inventory and `CloudId` are private native data: no Serialize implementation, no raw identifiers or appliance metadata in Debug/errors, and no direct SQLite/native JSON projection. Add encrypted identity mapping before converting them into stable local Device/Connection references. The returned family is only a protocol candidate. Preserve unknown families, and never translate `online_hint` into a verified state snapshot. Errors on individual homes remain explicit; session expiry stops subsequent requests; cancellation discards the return value without applying a partial replacement.

## Develop cloud state reads

`read_cloud_state` performs one explicit state query. `StateQuery::Default` sends an empty query object; `IndoorTemperature` sends the inspected AC query type and is rejected for other families. A future signed transport wraps that object as `command.query` beside the exact `applianceCode`, following `MeijuCloud.get_device_status` at the pinned revision. It must handle the new `ReadEndpoint::ApplianceState` variant. No automatic follow-up query, local Lua fallback or write is performed.

The native [state decoder](../../crates/midea/src/state.rs) uses these limited source mappings:

| Family / decoder component | Fields and projection | Boundary |
|---|---|---|
| [AC][ac-cloud-source] / `main` | `power` -> `power.state`; `indoor_temperature` -> current temperature; `temperature` plus `small_temperature` -> target temperature | Both target parts are required. Mode and wind-speed codes remain `vendor.midea.ac.*`; no universal fan enums or setter ranges |
| [CA][ca-cloud-source] / `fridge`, `freezer` | `storage_*` / `freezing_*` power and target fields; `refrigeration_real_temperature` / `freezing_real_temperature` -> current values | Two decoder slots for one local device, not proof of physical compartment support. Other zones remain unmapped |
| [E1][e1-cloud-source] / `main` | Program `mode`, `work_status`, `doorswitch`, lack flags, temperature and remaining minutes under `vendor.midea.dishwasher.*` | Preserve model-specific codes. In the inspected `7600016L` mapping, door code 1 means closed; do not manufacture a universal door-open or cycle-running boolean |

Finite numeric strings are normalized inside the adapter. Missing/null values remain absent; malformed recognized fields reject the whole response. Unknown fields, including undocumented timestamps, are not exported. Explicit non-Celsius temperature units are unsupported until their wire semantics are established. Defaults and mappings alone do not authorize capabilities or controls.

CloudState retains account/appliance identity privately and has redacted Debug with no Serialize implementation. `snapshot(local_device_id, component)` is a low-level projection and does not validate a local binding. The account's `read_bound_state` performs that identity check before requesting and projecting state; Core must still admit component/property descriptors. The projection emits allowlisted readings with cloud provenance, a fixed local receipt time, null observation time, and `stale: true`. The inspected response does not establish freshness, so this evidence cannot verify a write. A valid empty response stays empty; it is not an online indicator. See [ADR-0023](../adr/0023-meiju-cloud-state-evidence.md). Authentication approval, actual transport, model capability admission, physical testing and native UI account methods remain separate unfinished work.

[cloud-integration]: https://github.com/sususweet/midea_auto_cloud/tree/ce0c4b88224def0f6b53714312c19f71aa9c2a50
[meiju-cloud-source]: https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/core/cloud.py
[ca-cloud-source]: https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/device_mapping/T0xCA.py
[ac-cloud-source]: https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/device_mapping/T0xAC.py
[e1-cloud-source]: https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/device_mapping/T0xE1.py
[ha]: https://github.com/georgezhao2010/midea_ac_lan/blob/7c5231ca0588d364aa30e3bb2b887b37d4359719/README.md
[library]: https://github.com/midea-lan/midea-local/tree/ef28169926914d15f40fe521b9cc3d3c2f553c59
[ac-guide]: https://github.com/georgezhao2010/midea_ac_lan/blob/7c5231ca0588d364aa30e3bb2b887b37d4359719/doc/AC.md
[e1-guide]: https://github.com/georgezhao2010/midea_ac_lan/blob/7c5231ca0588d364aa30e3bb2b887b37d4359719/doc/E1.md
[ca-guide]: https://github.com/georgezhao2010/midea_ac_lan/blob/7c5231ca0588d364aa30e3bb2b887b37d4359719/doc/CA.md
[ac-source]: https://github.com/midea-lan/midea-local/blob/ef28169926914d15f40fe521b9cc3d3c2f553c59/midealocal/devices/ac/message.py
[e1-source]: https://github.com/midea-lan/midea-local/blob/ef28169926914d15f40fe521b9cc3d3c2f553c59/midealocal/devices/e1/__init__.py
[ca-source]: https://github.com/midea-lan/midea-local/blob/ef28169926914d15f40fe521b9cc3d3c2f553c59/midealocal/devices/ca/__init__.py
[device-source]: https://github.com/midea-lan/midea-local/blob/ef28169926914d15f40fe521b9cc3d3c2f553c59/midealocal/device.py

## Native account handoff

API 1.2 uses the same adapter for Core reads and native account operations. After
`connect_devices`, its already-retrieved inventory is checked against all protected
historical account bindings and published through `Core::publish_discovery`.
`refresh_devices` retrieves only the selected account, without login, and reports
the credential state after any binding-store mutation. An uncertain mutation stays
pending and blocks reuse. Partial retrieval preserves cards; recognized expired
sessions retain `AUTH_REQUIRED` even when the failure occurred within a home list.

The host returns public summaries and separate authentication/retrieval results.
It does not expose private `AccountRecord.credential_ref`, provider subjects or
appliance IDs. Native secret input, buffer ownership and frontend handling are
specified in [ADR-0026](../adr/0026-native-account-connection.md) and the
[account contract](../native-ui-contract.md#account-connection-and-refresh).
