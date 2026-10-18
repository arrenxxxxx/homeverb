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
