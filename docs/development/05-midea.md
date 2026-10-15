# 5. Integrate Midea and Hualing appliances

The owner selected Midea and Hualing air conditioners, dishwashers, and refrigerators as the first real-device scope. The primary journey uses appliances already bound in Midea Meiju: authorize that account inside HomeVerb, automatically retrieve its accessible devices, and read/control supported capabilities through Midea's cloud at home or away. HomeVerb has no account of its own. Re-pairing, manual IP/Token entry, and a home gateway are not prerequisites.

Daily cloud control is required in the first real integration. LAN is an additional route for the same physical devices, not a substitute for cloud control. The owner has physical products; exact models, firmware, app version, and account region remain to be recorded. This guide records requirements and source research on October 15, 2026, not implemented HomeVerb support or physical verification.

The native read protocols are implemented in [homeverb-midea](../../crates/midea/src/lib.rs): account-scoped home/appliance retrieval, code/data validation, deduplication, partial inventory failure, and bounded cloud-state decoding for the three selected families. They use an injected read transport and synthetic responses with the inspected wire fields. There is no live authentication, signing, HTTPS implementation, registered Core adapter, or enabled UI account method yet. See [ADR-0020](../adr/0020-midea-cloud-inventory.md) and [ADR-0023](../adr/0023-meiju-cloud-state-evidence.md).

The native account vault now persists random local references, composes Linux secure storage, and coordinates session read/update work through account leases. Interrupted or uncertain credential updates remain pending until trusted validation and recovery. This supplies storage for the future provider flow; it does not establish Meiju login, provider identity or session validity. See [ADR-0021](../adr/0021-persistent-account-vault.md) and the [credential integration guide](04-add-adapter.md#integrate-native-credentials).

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

The cloud sources include distinct Meiju and SmartHome login paths and app-specific signing material. Review their meaning and permitted native-client use before enabling login. An upstream hardcoded constant is not proof of public-client authorization. Do not copy shared credentials, default device keys, or a confidential-client secret into HomeVerb to pass the admission gate. Account persistence and refresh use opaque references, system secret stores, and on-demand refresh coordination. Never ask for credentials in chat, command arguments, or ordinary diagnostic output.

The October 15 authentication review found that the official v3 token endpoint requires a developer secret; the Android SDK guide describes obtaining tokens through a third-party server. Those documented paths do not establish the required independent native flow. The community app-facing login uses shared app signing and decoding material. [ADR-0022](../adr/0022-meiju-community-cloud-admission.md) records the primary sources and proposes a narrowly scoped community-protocol exception for owner review. It is not accepted; live authentication remains disabled under the current rules. This is an authentication-admission dependency, not a missing appliance-model prerequisite.

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

CloudState retains account/appliance identity privately and has redacted Debug with no Serialize implementation. `snapshot(local_device_id, component)` is for a trusted adapter after checking the encrypted identity binding and admitted component/property descriptors. It emits allowlisted readings with cloud provenance, a fixed local receipt time, null observation time, and `stale: true`. The inspected response does not establish freshness, so this evidence cannot verify a write. A valid empty response stays empty; it is not an online indicator. See [ADR-0023](../adr/0023-meiju-cloud-state-evidence.md). Authentication approval, actual transport, capability admission, physical testing and native UI account methods remain separate unfinished work.

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
