# 12. Trace refrigerator and dishwasher commands

Reviewed October 18, 2026 in response to the owner's request for actual command
integration. This record separates source/encoder evidence, implemented App
behavior and physical acceptance. No real account, private pilot material or
appliance command was used in this phase.

## Concrete cloud send path

The [pinned cloud client](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/core/cloud.py)
uses `/mjl/v1/device/lua/control`. It sends the private appliance target and a
`command` object containing `control` and optionally `status`. HomeVerb's
`MeijuSession::control_request` already signs this endpoint and sends admitted
`control` fields. It does not presently send a status overlay. Provider acceptance
is not proof of physical completion.

The existing dishwasher encodings are:

| App action | Provider control fields | Additional argument |
|---|---|---|
| Start cycle | `work_status: work`, `mode: <explicit program>` | Explicit `additional` for product 760Y0026 |
| Pause cycle | `operator: pause` | Same profile requirement |
| Resume cycle | `operator: start` | Same profile requirement |
| Cancel cycle | `work_status: cancel` | Same profile requirement |

The protected profile selects the exact program/option domain. Product 760Y0026
admits additional codes 0, 9, 10, 13 and 14; other reviewed cycle profiles use empty
arguments. A display model name does not establish which product profile applies.
Source mappings also mention child lock, power, storage and maintenance settings;
those are not newly admitted controls in this phase.

[ADR-0071](../adr/0071-app-dishwasher-command-preparation.md) connects these existing
Core/adapter operations to foreground App preparation. For the 760Y0026 fixture,
the App constructs 60 exact start grants and 15 exact cycle grants, within the two
independent 64-entry limits. The button opens explicit argument entry, then the
existing immutable native preview and trusted confirmation. Preparation sends no
appliance command. Each operation retains the protected membership/profile checks,
non-idempotent uncertainty handling and no automatic write replay.

## Identify the refrigerator's actual encoder

The subsequent [official Android package inspection](#official-android-package-inspection)
adds direct client evidence, including a nonempty `smartProductId` lookup argument
missing from the pinned community request. The phone operating system is not a
prerequisite for cloud-protocol research; public Android assets can be inspected
without a connected phone, account or Android SDK.

The pinned client's `MeijuCloud.download_lua` uses
`/v1/appliance/protocol/lua/luaGet`. Its request contains `applianceSn`,
`applianceType`, `applianceMFCode`, `version`, `iotAppId` and `modelNumber`.
The response provides `fileName` and a download `url`; the upstream code downloads
and decrypts that source. The separate plugin lookup uses
`/v1/plugin/update/getPluginV3` with exact appliance/product/model fields.
These are research entry points, not endpoints currently admitted by HomeVerb.
Do not send invented serials, infer a profile from the display model or copy the
upstream plugin's type-only fallback into identity selection.

The next useful evidence is a securely obtained exact-model protocol filename
and reviewed source, associated with protected product/subtype/manufacturer
metadata. Live serials and account tokens must remain outside ordinary logs and
repository files. Review any returned artifact URL and decryption path before
adding a bounded download; never execute a downloaded plugin in production.
The current workstation has no production macOS account-vault backend, and this
phase did not obtain that exact-model metadata. A connected supported platform
or an owner-provided sanitized protocol artifact is still needed.

Candidate temperature fields are `storage_temperature` and
`freezing_temperature`; modes include `storage_mode` and `freezing_mode`.
The [prior manual and encoder review](05-midea.md#refrigerator-write-evidence)
remains applicable. A working cloud API and known field names do not establish
that a partial change preserves all other refrigerator settings.

## Additional offline encoder comparison

The newer public [midea-local revision](https://github.com/midea-lan/midea-local/tree/904bd5ec74d7e173702ecc2603ddb5afe22aabb6)
contains 15 CA Lua files. Four candidates were executed unchanged using Lupa 2.8,
a fresh Lua 5.4 state for each vector, a fixture-table JSON stub and an integer-only
bit shim. No network, provider session or physical appliance was involved.
The [reproducible harness](../../scripts/verify_refrigerator_encoders.py) checks each
source's SHA-256 before execution. It is optional research tooling, not an App
runtime dependency or support admission.

For 310A1700 version 58, 310A2245 version 66 and the prior 310A1792 candidate,
the same synthetic mode/temperature/power results appear at body offsets 1/2/6:

| Input | Mode | Temperature | Power |
|---|---|---|---|
| Status baseline, length 119 | `ff` | `64` | `1d` |
| Fridge target 5, length 119, no status | `00` | `05` | `00` |
| Fridge target 5, length 119, synthetic status | `ff` | `65` | `1d` |
| Fridge target 5, length 119, only freezer -18 status | `00` | `65` | `00` |

The synthetic status enables all eight packed modes, disables four compartment
powers and supplies fridge 4/freezer -18. This is an encoding vector, not a valid
physical operating combination. 310A1700 emits 120-byte packets even when length
119 is supplied; 310A2245 and 310A1792 emit 131-byte packets. Without that length,
the latter two encounter missing-input errors, while 310A1700 still encodes.
Version 5 encounters uninitialized-field errors for all five vectors. These
failures describe this deliberately limited fixture/runtime, not a claim that the
provider's complete runtime fails.

All 13 produced packets passed length/checksum and selected-byte assertions;
seven expected missing-input errors were reproduced. Fractional conversion,
complete status preservation, valid mode combinations and exact-model association
remain unproven. No refrigerator control is enabled by these results.

To reproduce, use a temporary virtual environment with `lupa==2.8`, obtain the
public checkout at the revision above, then run:

```bash
python scripts/verify_refrigerator_encoders.py /path/to/midea-local
```

Before admitting refrigerator writes, match the exact encoder, establish the
input step and omitted-field behavior, and review preservation against actual
model status. A separately authorized physical test must confirm the intended
change and unchanged settings. The implemented dishwasher path can be tested
independently; its physical acceptance remains pending too.

## Official Android package inspection

After the owner selected Android, a public-package investigation proceeded without
waiting for a connected device. The [official Meiju download page](https://msmart.midea.com/mideaApp/)
and its [page script](https://msmart.midea.com/_nuxt/f68251f.js) expose a public
release lookup at `/meiju-web/v1/apisix/gray/app/getUpgradeStrategy` through the
existing vendor proxy origin. The webpage supplies `os: Android`, `iotAppId: 900`
and `sourceType: H5`; no account credentials are involved. This is a download-page
lookup, not a new HomeVerb production endpoint.

The vendor returned version **11.7.10.6** from `mjdownload-pro.smartmidea.net`.
The 207,629,116-byte APK matched the download response's MD5. Its independently
computed SHA-256 is
`5da32e92638806d55e9ce533b917bbbbba6db0bd7d3a02c45e08ed0dee62d849`.
No APK installation, APK code execution, signature verification, account login or
appliance command was performed. The APK and extracted source remain temporary
research inputs outside the repository; they are not redistributed with HomeVerb.

The archive contains encrypted `assets/lua/T_0000_CA_5.lua` and
`assets/lua/T_0000_E1_3.lua`. The already-pinned community decoding method reads
both; no new APK signing/credential material was extracted or admitted. The CA
file again contains the generic 2-8 fridge and -24 to -15 freezer bounds, with
status applied before control and temperatures/modes/power packed together.
Those bounds are not the selected model's writable contract. Neither decoded
base file contains the `600WUFIPZM` model literal. This is a limited static-search
result, not proof that the App lacks model support or that a device uses the base
file. Neither official Lua file was executed.

Static DEX inspection of `ProtocolControlManager` established:

- `prepareLocalLuaFile` checks cached/local files and bundled assets, constructs
  several increasingly general filename keys and calls `checkDownloadLastLuaFile`.
  A bundled family file alone therefore does not identify the device's actual
  selected protocol.
- `getLastVersion` builds `iotAppId`, `applianceMFCode`, `applianceType`,
  `modelNumber`, `applianceSn` and `version`, and adds `smartProductId` when nonempty.
  The pinned community `download_lua` accepts a similarly named argument but omits
  it from the outgoing request. Preserve the actual protected value when reviewing
  a native lookup; do not invent one or assume omission has equivalent semantics.
- The same method obtains a URL, filename and integrity metadata from its response.
  Request construction is static evidence; this phase did not perform an
  authenticated protocol lookup or download a device-specific file.
- The control callback constructs `deviceinfo` with `deviceSubType`, supplies
  distinct `control` and `status` maps and calls `LuaManager.encodeJsonToBytes`.
  This corroborates the state-overlay dependency found in the public Lua sources;
  it does not establish cloud-state freshness or the selected model's preservation.

A repeatable [static inspector](../../scripts/inspect_meiju_android_protocol.py)
checks the exact APK and community decoder SHA-256 values, decodes the two bundled
files in memory, and reports only hashes, allowlisted field names and selected
method-call checks. It emits no arbitrary DEX strings, proprietary source,
credentials or device identifiers. Use a temporary environment containing
`androguard==4.1.3` and `pycryptodome==4.0.1`:

```bash
python scripts/inspect_meiju_android_protocol.py /path/to/meiju.apk /path/to/midea_auto_cloud
```

The inspector passed against the pinned official package and community checkout.
These checks confirm static structure only. They neither validate field values
from a live account nor execute an appliance-control path. No native API, storage,
App grant, refrigerator capability or physical compatibility claim changes.

The next device-specific input is the actual lookup tuple and returned protocol
identity for this refrigerator. It must come from an authorized account operation
or a sanitized device-specific artifact, not from the Android/iOS choice. A USB
connection can enable device-side inspection but does not by itself grant access
to private App data. The current read-only ADB inventory found zero connected
Android devices; no private data was accessed. The incomplete local Android SDK
blocks building HomeVerb's APK, not this public protocol investigation.

### Control bridge and actual WAN transport

Further static tracing in the same APK established a complete SDK path:

1. `DefaultBridgeDeviceImpl.luaControl` selects the supplied appliance reference
   or the current device; `luaControlByTransport` turns `params` into a map.
2. The bridge also reads a top-level integer `mode`, defaulting to zero. The
   inspected `MideaDeviceManagerProxy.controlDeviceState` overload checks login
   and forwards the device and map, without using that integer. It is not evidence
   of a refrigerator operating-mode argument.
3. `DataTransportHelper.controlDeviceState` queries state only when the device pool
   has no cached state. Otherwise it immediately schedules `SendStateTask`.
4. That task reads the cached state again and passes `getStateMap()` separately
   from the requested change to `getControlStatusProtocol`. It refuses a missing
   cache. The previously inspected callback places these in `status` and `control`
   for Lua encoding. This branch has no demonstrated freshness check.
5. `startSendDevDataByWan` uses `ServerUrls.getTransparentUrl`, converts the encoded
   bytes to a decimal string, encrypts with session data material, and submits
   `applianceCode`, `order`, `timestamp: "true"`, and `isFull: "false"`.

Thus this official bridge uses local Lua encoding followed by transparent cloud
transport. The existing HomeVerb `/mjl/v1/device/lua/control` path instead submits
JSON for server-side encoding. Both are observed protocol designs; do not claim
that every official button uses the same endpoint as the community client. No
session material was extracted, and no transparent command was sent.

### Public plugin lookup and historical refrigerator UI

Retrofit annotations identify four POST aliases:

| Method | Alias | Observed scope |
| --- | --- | --- |
| `getModelPluginDownloadInfo` | `/v1/plugin/update/getModelPlugin` | Module/plugin lookup with `modelList` |
| `getLoginModelPluginDownloadInfo` | `/v1/plugin/update/getLoginModelPlugin` | Logged-in variant |
| `getPluginDownloadInfo` | `/v1/plugin/update/getplugin` | Device plugin lookup with appliance descriptors |
| `getVirtualPluginDownloadInfo` | `/v1/plugin/virtualplugin/get` | Public demo plugin catalog |

The module downloader explicitly chooses the first or second method based on
login state. Despite its name, `modelList` is a list of plugin-ID strings in the
inspected caller; it is not a list of marketing model names. A separate caller
supplies an empty list. The DTO defaults are `iotAppId: "900"`, `clientType: "1"`,
`clientVersion: 201`, and empty `phoneNumber`; `RequestUtils.createBody` serializes
ordinary JSON. Device plugin factories separately use category, model number,
SN-derived product data and other device metadata. These two lookup paths must
not be conflated.

Bounded public requests on October 18 produced:

- An unsigned empty-list module request returned HTTP 200 with business code
  `44001` (missing signature).
- The same request using only the already-approved pinned community signer
  returned HTTP 200 with business code `40400` (invalid business request).
  This does not establish either successful lookup or a login requirement.
- The virtual-plugin POST with an empty body and the same signer returned HTTP
  200, business code `0`, and 19 public demo entries, including a refrigerator.
  No dishwasher entry appeared in this particular response.

The refrigerator entry points to
`http://mjfile-pro.smartmidea.net/T0xCA_plugdemo_20200512.zip`.
An HTTPS attempt failed during TLS negotiation. The exact catalog-provided HTTP
URL then returned 8,122,564 bytes, SHA-256
`9fddb089c412c9662e1a5ff5aa52ab92b4e3acacc9ae040b43f471ef4c21ce27`.
This hash pins the inspected input; it does not authenticate an HTTP download or
establish publisher-signature verification. The archive remains temporary, was
not installed or executed, and is not bundled with HomeVerb.

Its historical `weex.js` contains both a simulated `isDummy` branch and a
non-dummy bridge branch. Static inspection of `setFridgeCtrlModeCmd` and
`setLuaFridgeModeCtrl` finds explicit label-to-command dispatch using
`{operation: "luaControl", params: ...}`. The following are historical UI
mappings, not admissions for the owner's current model:

| Historical UI function | Command field | Values |
| --- | --- | --- |
| Rapid cooling | `storage_mode` | `on`, `off` |
| Rapid freezing | `freezing_mode` | `on`, `off` |
| Light-wave preservation | `preservation_mode` | `on`, `off` |
| Extreme freezing | `acme_freezing_mode` | `on`, `off` |
| Energy saving | `energy_saving_mode` | `on`, `off` |
| Holiday | `holiday_mode` | `on`, `off` |
| Intelligent mode | `intelligent_mode` | `on`, `off`, with a product-specific alternate dispatch |
| Variable compartment | `variable_mode` | Named presets, including `soft_freezing_mode`, `fresh_product_mode`, `zero_fresh_mode` |

The latest APK's bundled scene UI independently labels `storage_mode` as rapid
cooling and `freezing_mode` as rapid freezing. Its bundled E1 Lua declares
`work_status` values `power_on` and `power_off`. This is additional evidence for
investigating the dishwasher power button, not sufficient exact-profile admission.

The historical plugin obtains model-specific UI configuration from
`/fridgesn/v2/getInfo.do`, with an SN-derived product selector and an additional
signed request. Its response controls `fridgeZoneList`, model labels and feature
flags. That signing mechanism was not extracted or admitted. Consequently the
presence of a shared toggle function does not prove that it appears for every
model or that a renamed modern feature uses the same command.

The plugin also separates provider data services from appliance Lua commands.
Historical code references `https://fridge-api.mideav.com/fridge` and
`https://fridge-api2.mideav.com/fridge`, including
`/FridgeStatistics/openDoorRecord`, `/FridgeStatistics/getCleanTime`,
`/FridgeStatistics/intelligentCenter2`, `/FridgeStatistics/getPatternSetting`,
`/FridgeStatistics/setPattern`, and annual-report routes. None of those account
or setting endpoints was called. Their presence identifies further research paths;
it does not prove that current AI features use the historical service.

### Owner screenshot parity checklist

The owner supplied official App screenshots on October 18. They establish the
requested UI scope and visible product features. Screenshots and private displayed
usage values are not copied into the repository. A visible button is evidence of
an official feature; its exact arguments, preconditions and response semantics
still require tracing. All rows below remain in scope, including provider-backed
settings and reports that are not Lua appliance commands.

| Visible feature | Evidence and next implementation dependency |
| --- | --- |
| Refrigerator cold-storage and freezer targets | Existing read mappings; resolve exact encoder and preservation before admitting writes |
| Variable compartment with a maternal/infant preset | Historical variable presets exist; find the current preset's exact code and allowed transitions |
| Rapid cooling and rapid freezing | Current APK scene labels and historical explicit toggle dispatch; bind to the current protected profile |
| Daily mode | Find whether the current plugin sends a preset or a coordinated set of flags |
| Minus-30-degree deep freezing | Historical `acme_freezing_mode` is a lead, not an established mapping for this label or temperature |
| Ice making | Base Lua contains several distinct ice-related fields; identify the current button's exact one |
| Light-wave freshness | Historical `preservation_mode` dispatch is a lead; confirm current model and state overlay |
| Bulk-food freshness | Find its current plugin command and any coordinated temperature/mode changes |
| Long-vacation energy saving | Historical holiday and energy-saving flags are distinct; resolve the current button's actual combination |
| PST+ purification and percentage | Identify status source, available settings and reset semantics |
| Intelligent freshness, noise reduction, energy saving, family protection | Trace each current provider feature's settings, status and lifecycle; historical statistics/pattern endpoints are leads |
| Ambient lighting settings | Base lighting field exists; current subpage may expose additional arguments |
| Door-open notification sound | Find the exact sound-setting command and distinguish it from cloud notification preferences |
| Food inventory | Historical provider food service exists; inspect current operations and account scope |
| Door-open count, cleaning reminder, monthly report, efficiency score | Trace current reporting endpoints and distinguish historical summaries from current appliance state |
| Dishwasher power button | Bundled E1 `power_on`/`power_off` values; review exact centralized profile, required extra options and trusted authority |
| Dishwasher detergent, filter and rinse-aid status | Bundled service-center consumables route traced below; current dishwasher plugin association, authenticated response and maintenance-reset operations remain open |
| Last wash time/mode and water/electricity usage | Trace history/report sources, not synthetic values derived from current state |
| Monthly electricity, water and water savings | Trace provider aggregate report semantics and period |

The dishwasher screenshot shows its powered-off screen. It does not enumerate
all controls displayed after power-on; the existing program/start/pause/resume/
cancel investigation remains in scope. Food-shopping and social navigation do
not themselves establish additional appliance commands.

### Reproducible expanded inspection

The static inspector now additionally checks plugin POST annotations, cache/query
and encoding calls, transparent-transport construction, and the historical demo's
actual toggle dispatch. It never sends these requests, executes plugin JavaScript
or Lua, extracts new signing material, or outputs arbitrary source strings:

```bash
python scripts/inspect_meiju_android_protocol.py /path/to/meiju.apk /path/to/midea_auto_cloud --demo-plugin /path/to/T0xCA_plugdemo_20200512.zip
```

The expanded inspection passed against both pinned archives. The investigation
used public download/catalog requests only: no account login, private App data,
real-device command, repository push or publication occurred. Native API 1.22.0,
SQLite schema 5, protected payload 3 and existing capability/grant admission remain
unchanged. Current-model plugin selection, the screenshot's remaining feature
mappings and physical verification remain open; the historical demo does not
complete those tasks.

Phase checks actually run: the expanded inspector with both pinned archives;
rejection of mismatched APK/demo inputs before decoding; `validate_docs.py`;
`validate_repository.py`; `.venv/bin/python scripts/validate_contracts.py`
(20 schemas, 49 positive and 267 negative fixtures); and
`validate_commits.py --all`. The initial system-Python contract invocation lacked
`jsonschema`; rerunning in the existing repository virtual environment passed.
No Flutter build, Rust runtime test or physical-device test was claimed for this
research-only phase.

## Implemented dishwasher power extension

[ADR-0072](../adr/0072-profile-scoped-dishwasher-power.md) implements explicit
`power_on` and `power_off` for the same five protected product profiles as the
reviewed cycle controls. The exact wire control is `{"work_status":"power_on"}`
or `{"work_status":"power_off"}`, plus the explicitly selected `additional`
for 760Y0026. No program is selected by either request.

The [source comparator](../../scripts/verify_dishwasher_power_mapping.py) reads
only the pinned E1 mapping AST, checks its SHA-256, verifies 18 product/power/option
vectors and keeps 7600016L excluded because its centralized fields differ:

```bash
python3 scripts/verify_dishwasher_power_mapping.py /path/to/midea_auto_cloud
```

The adapter's existing outcome, approval, profile, cache and lifecycle test loops
now include both power actions. The native startup enum and App inspection-driven
grant preparation include them independently. The 760Y0026 fixture prepares 60
start grants and 25 lifecycle grants; old-library inspection without power actions
still prepares only the old 75 grants. All are exact choices, not wildcard scope.

The App uses compact two-column buttons where width/text size permit and a
single column for narrow or enlarged text. Each power button opens explicit
argument/route selection followed by trusted confirmation. The confirmation now
uses the additional-option-specific effect description when applicable.
This completes software admission for these reviewed profiles, not proof that
the owner's device matches one or that physical power behavior has passed.
Native API is now 1.24.0; ordinary methods, C exports, SQLite 5 and protected payload
3 are unchanged. Current-model refrigerator control and the other screenshot
features remain tracked above.

## Current refrigerator artifact acquisition boundary

After the power-button phase, another bounded public lookup used only the
fictional refrigerator entry already returned by the official virtual-plugin
catalog (`appModel: plugdemo`). The old device-plugin update alias
`/v1/plugin/update/getplugin`, signed with the existing approved community
material, returned HTTP 200 and business code `40001` with an explicit missing
access-token error. No account credential, real appliance identity or control
request was used. This establishes a login requirement for that attempted path,
not for every vendor catalog or protocol endpoint. No updated plugin was obtained.

The exact-model official product page was fetched again and inspected for
`smartProductId`, `modelNumber`, `sn8`, product/material-code fields and recognizable
product-code tokens. It contains the display model but none of those candidate
selectors. Public model searches yielded product/manual information, not a
reviewed model-to-encoder association. Do not turn a retail item ID or the public
`plugdemo` selector into a physical-device profile.

A successful bounded ADB inventory check found zero ready and zero unauthorized
devices; device identifiers were not printed and no device data was read. The
remaining artifact dependency is an authorized exact-product selector/source,
for example product/model codes supplied by the owner or inspection on an
available Android device with the owner's existing App session. Passwords,
access tokens and full serial numbers must not be requested through chat. The
owner was asked which source is available. Current-model refrigerator writes,
its renamed modern modes and current provider-report mappings remain incomplete.
These observations do not change the implemented dishwasher controls or any
production API, schema, grant or compatibility claim.

The subsequent full toolchain-location check found an existing complete Android
SDK in another local project's tool directory. Both native ABIs and the current
appliance-controls APK were rebuilt and checked; see the
[package record](../../apps/mobile/verification.md#current-appliance-controls-apk--october-18-2026).
This corrects the earlier build blocker. It does not supply the refrigerator's
protected protocol selector or an authenticated current-model plugin.

## Current APK consumables service trace

Static inspection of the same pinned 11.7.10.6 APK found a concrete provider-data
path in `assets/assetsWeex/service-center/deviceInfo.js`, SHA-256
`1a7c604a6676bb7f911da32029ead338b9d2dda8b4454d3d2f80e37c20dada6c`.
This is the bundled general service-center screen, not a recovered GX1200Max
device plugin. The new evidence narrows the remaining consumables investigation:

1. `queryByHomeGroupList` calls `sendCssHttpRequestWrapper` with
   `/v2/remaining/queryByHomeGroupList`. The helper uses POST and adds
   `plugin_version` and epoch-seconds `tm`; this call explicitly disables the
   optional outer `body` wrapper. Its caller supplies `homeGroupInfoList`, each
   entry containing `homegroupId` and `remainingStatusSet: ["0", "1", "2"]`.
2. The helper invokes native `sendCentralCloundRequest` (the spelling in the
   actual bridge). `DefaultBridgeBusinessNetImpl` uses the MAS request-wrapper
   factory, base body and JSON POST path for the ordinary request branch.
   Alternate new-request/gzip branches also exist. This trace is not evidence
   that HomeVerb's existing transport can send the request unchanged.
3. The caller checks `success`, iterates `data[].applianceConsumablesList`, and
   matches `applianceCode` against the selected device's `iotDeviceId`. It retains
   the containing `homeGroupId` and matched `consumables`. These identifiers must
   remain in protected adapter state if this path is implemented; raw provider
   results must not be exposed to general UI JSON or stored in plaintext.
4. Display distinguishes `remainingValue`, `total`, `remainingStatus` and
   `remainingValueType`. Type 4 renders categorical sufficiency: status 2 is
   sufficient, with a decorative wave height of 0.4; the alternate wave height
   is 0.05. Neither height is a measured percentage. Other branches calculate
   ratios, and missing data has a separate display. Do not convert a categorical
   warning or absent value into an invented percentage or a physical reset.
5. The detail navigation targets `customer-service/materialDetail.js` and passes
   device/product context. The bundled service-center asset does not establish
   this separate module's maintenance-reset request or the screenshot's exact
   filter/detergent association.

The existing E1 read mapping exposes optional `softwater_lack` and `bright_lack`
warning codes. Those fields are not a source for remaining detergent percentage,
filter lifetime, last-wash consumption or monthly totals. The bundled base E1
Lua itself does not contain those two warning names or the screenshot's report
fields; richer protected profiles and provider services must be distinguished.

The static inspector now reproduces the route, request wrapper, response
association, categorical-display and native-call checks. Its output contains
allowlisted names and hashes only. It never evaluates downloaded JavaScript/Lua,
prints arbitrary literals, or admits additional authentication/signing material.
The pinned inspection passed; separate mutations of the route, categorical status
and device-match expression were each rejected by the source checks.

An additional bounded public module lookup used `modelList: ["customer-service"]`
with the already-approved compatibility signer and no account/device data.
The APK's two module-request DTOs both declare `List<String>`. The official
`/v1/plugin/update/getModelPlugin` route returned HTTP 200 with business code
40400 (invalid request); no module was downloaded. This result applies only to
that public request, not to every module format or authenticated route. No
consumables endpoint, real account or appliance command was called.

A subsequent manifest check resolved `customer-service` to module number `30001`
in `assets/weex_packages.json`; the static inspector now verifies this association.
A bounded retry with `modelList: ["30001"]` also returned HTTP 200/business 40400.
Neither request acquired the detail module. The module name and number are public
download selectors, not device-profile evidence. A fresh bounded ADB inventory
still found zero ready and zero unauthorized devices, without printing identifiers.

Next implementation evidence is the actual current device plugin association,
authenticated response/error/units semantics, protected home/device binding and
the service's current authorization path. Any maintenance-reset action also needs
its own effect, confirmation and uncertain-write review. This phase changes
research tooling and documentation only; native API 1.24, existing descriptors,
grants, storage formats and the delivered APK are unchanged. Real consumables,
current refrigerator modes and screenshot parity remain unverified.

Phase validation: the pinned APK/demo inspector; the three changed-source
refusals described above; `validate_docs.py` (146 Markdown files, 1594 local links,
212 JSON files); `validate_repository.py` (624 maintained text files);
`.venv/bin/python scripts/validate_contracts.py` (20 schemas, 50 positive and 291
negative fixtures); and `git diff --check`. No application build, device test,
account access, remote publication or real-device write occurred in this phase.
