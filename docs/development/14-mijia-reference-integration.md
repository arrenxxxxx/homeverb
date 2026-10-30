# 14. Mijia reference integration and remaining coverage

The October 30 worktree uses API 1.43 and the owner-supplied October 22 Xiaomi Home
11.7.709 analysis. The full official-App feature request remains **incomplete**.
The source archive explicitly does not supply all authentication, protocol or
runtime control contracts. An implemented decoder is not physical acceptance.

## Implemented software

API 1.30 adds 14 independently granted settings for the exact reference light
and purifier under ADR-0084. API 1.32 supplies explicit foreground captcha and
phone/email verification; API 1.33 supplies exact protected daily-energy history
through Core/native/App and now CLI/MCP. These additions and their bounded evidence
are recorded below; they do not establish full reference coverage. API 1.37 adds
modern QR login, protected account/device binding, grouped card publication and
Core reads through Flutter. The [Android package](../../apps/mobile/verification.md#modern-qr-and-device-publication-android-package--october-30-2026)
has clean-source archive checks, without installation or provider acceptance.
ADR-0103 subsequently adds exact modern outlet daily-energy history through the
existing Core/native/App history interface. ADR-0104 adds the 29 reviewed scalar
controls across five exact modern model mappings through existing Core/native/App
contracts. The later 9a6c004 APK includes both additions; it predates numeric
samples. ADR-0105 now exposes 161 exact numeric sample series through Core,
CLI/MCP and version-negotiated API 1.38/App. The
[2e776a9 Android package](../../apps/mobile/verification.md#reported-samples-api-138-android-package--october-30-2026)
includes these samples with clean-source archive evidence. ADR-0106/API 1.39
adds four exact manual/display hood controls, bringing the control count to 33;
that package predates the new hood extension. The later
[503ef1f API 1.39 package](../../apps/mobile/verification.md#exact-hood-controls-api-139-android-package--october-30-2026)
contains all 33 reviewed controls with clean-source archive checks. API 1.40 now
adds six exact no-argument actions through Core/native/App; that APK predates them.
The later [ab34f21 API 1.40 package](../../apps/mobile/verification.md#no-argument-actions-api-140-android-package--october-30-2026)
includes all six actions with clean-source archive checks. The subsequent
[d505ccb API 1.41 package](../../apps/mobile/verification.md#power-toggle-api-141-android-package--october-30-2026)
includes the five elevated power toggles as well, without runtime acceptance.
The later [87f3410 API 1.42 package](../../apps/mobile/verification.md#maintenance-actions-api-142-android-package--october-30-2026)
adds the four trusted maintenance actions with clean-source archive checks.
The later [85fefd8 API 1.43 package](../../apps/mobile/verification.md#switch-and-outlet-api-143-android-package--october-30-2026)
adds seven switch/outlet properties, bringing packaged scalar controls to 40.

- Eight-part MIoT URNs retain their exact extension; malformed/foreign URNs refuse.
- Seventeen exact model/specification catalogs project 264 reviewed scalar read
  properties. Nineteen model records are tracked; scales and watch remain named
  placeholders without a reviewed specification. A fitting specification without
  readable fields also stays a placeholder.
- Fan, air-purifier, light, outlet/switch, gateway, lock/video-doorbell, camera,
  temperature/humidity, water-leak, remote battery and hood scalar settings retain
  separate services and exact wire types. A runtime property error remains absent;
  offline mapping cannot repair a provider's failed response.
- Only equivalent fan/purifier/switch/light on-state uses `power.state`. Indicator,
  camera and lock-adjacent switches retain vendor scope. No online flag becomes
  power, lock state or physical connectivity.
- API 1.29 adds exact-granted included swing angles, vertical swing and ionizer
  settings for `jdjz.fan.2072fc`, preserving the four existing reference fan
  setters. API 1.35 separately admits exact fan quiet and hood illumination settings;
  timing and other unreviewed fields remain read-only.
  Source write access alone cannot create any new action.
- Public pinned specifications can supply exact mappings during explicit account
  retrieval without public-spec networking. Conflicting reported URNs do not use
  this fallback. Old protected derivations remain valid and explicit refresh
  upgrades them while preserving local account/device/connection references.
- Shared English/Chinese labels cover added scalar fields and component selectors.
  The generic App panel keeps raw codes, missing values, units and cloud staleness;
  the initial component prioritizes the actual appliance service over auxiliaries.

The [coverage ledger](mijia-interface-coverage.json) enumerates every source
property/action/event, including exclusions. The [projection script](../../scripts/extract_mijia_reference_facts.py)
checks the archive digest and copies only public specification facts. It never
executes an archived script or copies private observations/identifiers. Verify it:

```sh
python3 scripts/extract_mijia_reference_facts.py --check
cargo +1.96.0 test -p homeverb-mijia --offline --locked
```

ADR-0110 additionally implements five elevated power-toggle actions through typed
Core with independent trusted approval. ADR-0111 adds native API 1.41 admission;
Flutter adoption is implemented. This does
not change the six actions included in the ab34f21 APK.

## Coverage boundaries

| Source item | Current disposition |
|---|---|
| 225 source writable property definitions | 40 exact Core/native/App controls implemented; 185 need separate semantic, risk and runtime admission. A source write flag is not an implemented control |
| 264 scalar readable properties | Exact mapping implemented; synthetic Core reads; provider/physical acceptance remains per property |
| 97 identity/pairing/media properties | Excluded from ordinary observations; no identity leakage or streaming inference |
| 78 non-readable properties | No read operation; write/event-only definitions are not state |
| 28 opaque readable values | Separate typed/business/privacy contracts required |
| 3 network identity values | Excluded from state and exports |
| 40 actions | Eleven exact empty-input actions have Core/native/App support, including five elevated power toggles under ADR-0110/0111; four maintenance/stop actions also have Core/native/App support under ADR-0112/0113; the other 25 actions remain incomplete |
| 78 events | Typed legacy/modern occurrence-time history under ADR-0115; Core/native/App, payload semantics, paging and subscriptions remain incomplete |

The archived successful reads used **SID `mijia`** and
`https://core.api.mijia.tech/app`, with a continuous RC4 stream across encrypted
form fields. Native account/App hosts now support that modern protocol separately
from legacy **SID `xiaomiio`**, reviewed `api.io.mi.com` routes and legacy field
encryption. These are separate session
contracts. Do not move a saved legacy token to the modern origin, silently change
its signing mode, or call modern success evidence proof of independent login.
The archive excludes the existing App session and reports standalone login,
renewal and expiry recovery as unverified. Independent typed Rust QR/renewal and
reference reads are synthetic-tested under ADR-0093/0094; ADR-0095 through
ADR-0102 implement protected modern accounts, grouping, Core reads and native/App
publication. Real account/phone/model acceptance remains pending.

The modern device-page endpoint requests logical split children. The analysis
records two split switch objects and a cloud/cache mismatch (21 shared IDs out
of 22 in each source). These do not prove 23 physical appliances. The current
legacy enumerator retains its bounded account/home union and partial status;
complete inventory and missing-device removal are not claimed. Modern logical
children are grouped by explicit protected parent evidence under ADR-0099/0100;
ambiguous relations refuse and absent historical bindings are retained. No private
identifiers enter this ledger.

The HA source review below supplies protocol evidence used by the implemented
legacy verification flow and exact energy-history profile. Still blocked pending
additional evidence: broader control and historical/event protocols,
authenticated acceptance and specialized paths for
failed cloud properties and BLE devices; broader statistics, event pagination and
foreground subscriptions; health/media/streams; plugin-specific
composite conversion and additional write constraints. Door unlocking stays denied.
No background service, arbitrary RPC, remote MCP or scheduled automation is added.

## Verification

Executed results are appended after the final checks. API 1.27, SQLite 5 and the
protected payload envelopes are unchanged by read mappings. New reference-derived
payload content requires this updated mapping implementation; keep matching native
and App builds. No real account/device operation or remote publication is part of
this phase.

Final source checks on October 25: 321 Mijia/Host Rust tests and compile-fail tests
passed, including all 264 fictional scalar readings through Core. The standalone
Mijia run passed 62 unit and three compile-fail tests. Strict Mijia/Host Clippy,
workspace build and formatting passed. Flutter analysis and the final six-file
account/Mijia/reference/localization/recovery subset passed all 32 cases. Shared
presentation tests passed seven cases; Python tools passed 72 tests. Documentation,
21 schemas with 56 positive/365 negative fixtures, source hygiene and digest-based
reference regeneration passed.

An initial complete Flutter run passed 193 and failed nine. One newly exposed Mock
identity-recovery capability assertion was corrected while preserving Mijia-only
refusal; its actual FFI regression passed in the 32-case rerun. Eight existing
macOS failures remain: button ink, field ink, two control-glyph/pixel tests and
four legacy app/outcome click cases. A final full run is recorded with delivery.
No full-suite pass or physical/device compatibility is claimed.

The Android native AAR cross-build passed arm64-v8a and x86_64, 20 exports per ABI
and 16 KiB ELF alignment using the already-installed NDK 28.2.13676358 and JDK 21.
APK assembly and archive verification are a separate delivery phase; no package
installation or account/device request has occurred.


## Combined Mijia reference APK -- October 25, 2026

Clean application source `87ff5b12a246f46525d7cf50ab154a3bfe154a87` built the
combined `homeverb.platforms` release APK with the FVM-pinned Flutter 3.47.7.
The build used the existing SDK/NDK and persistent development signing location,
with temporary Flutter SDK/JDK configuration. No license or SDK was installed.

Artifact: `apps/mobile/build/delivery/mijia/homeverb-mijia-0.2.0-87ff5b1.apk`.
Size: 74,466,398 bytes. SHA-256:
`e271a2cabd20d5eae87232e0355a084987840608e65d95c30e246fb70ae112e4`.
The adjacent `verification.json` records clean source, AAR/native digests, APK v2
development signature, exactly arm64-v8a/x86_64, API 28 minimum, disabled backup,
no integration-test plugin, 20 Core exports per ABI and 16 KiB ZIP/ELF alignment.
Both native payloads exactly match the current AAR after normal NDK stripping.

Final full Flutter regression: **194 passed, eight failed**. All eight match the
previously recorded macOS glyph/pixel and legacy app/outcome-click cases. The
introduced Mock recovery assertion is fixed and passes. Final workspace strict
Clippy and Flutter analysis passed. This is not a full Flutter test-suite pass.

The recovered worktree moved from `/private/tmp/homeverb-mijia-current` to the
persistent `/Users/zhao/develop/codes/any/homeverb-mijia` directory after packaging.
The old damaged temporary directory's residual files remain untouched; only its
stale Git registration was pruned. Offline Flutter dependency resolution and
analysis passed again after relocation. Mainline remains unchanged. No merge
commit, remote push, installation, account access, real appliance operation,
physical-phone test, telemetry upload or publication was performed.

A full `cargo +1.96.0 test --workspace --offline --locked` attempt on October 25
stopped in the existing Meiju LAN suite: 56 tests passed and 14 failed while
binding synthetic interface addresses (`AddrNotAvailable`, macOS error 49).
The required local interface aliases are absent; host network configuration was
not changed. This run is not a passing workspace test. The separately executed
Mijia/Host suites above remain the scoped evidence.

## Distinct component selectors -- October 25, 2026

Repeated family labels now use their exact localized component labels, so the
reference lock's `battery-4` and `battery-10` remain independently selectable.
Singleton families retain their short label. Unknown component IDs keep a stable
ID fallback. No provider/model branching or change to component identity is added.
The selector also uses the existing control-text layout, fixing the Flutter
`ChoiceChip` dry-layout assertion caused by ordinary `Copy` text.

Executed `fvm flutter analyze` passed. All 18 tests in `test/mijia_test.dart`,
`test/mijia_reference_test.dart` and `test/mijia_controls_test.dart` passed.
The two new regression cases first reproduced the selector failures and then
passed in English/Chinese at 320 logical pixels and double text scale. They
select both batteries, verify independent snapshots and confirm no writes occur.
The previously recorded full-suite failures remain unresolved; this scoped run
is not a new full-suite pass. Native API 1.27, data schemas and grants are unchanged.
The APK from `87ff5b1` predates this UI fix; its earlier archive verification is
valid for that source revision only.

## Component selector APK -- October 25, 2026

Clean source `28cf192c8c7879ba357ce22d7dc83451fb3e742c` built the combined-platform release
APK with the component selector correction:
`apps/mobile/build/delivery/mijia/28cf192/homeverb-mijia-0.2.0-28cf192.apk`.
Size: 74,466,398 bytes. SHA-256:
`07022839234c89cfabaec002c13007a5934388bb1326eacf32865d7e155b88ac`. The adjacent `verification.json` records both ABIs,
20 exports each, current AAR matching, development signing and 16 KiB alignment.
The release archive has no integration-test plugin and application backup is off.

Executed `fvm flutter build apk --release --target-platform
android-arm64,android-x64 --dart-define=HOMEVERB_ADAPTER=homeverb.platforms` and
`scripts/verify_flutter_apk.py` passed. An initial `--no-pub` attempt retained
an integration-test registration generated by the test workflow and failed Java
compilation; normal release dependency/plugin regeneration resolved it. Installed
SDK/NDK/JDK and existing development signing material were reused. No installation,
provider request, real-device operation, upload, push or publication occurred.
This APK supersedes the `87ff5b1` package for local evaluation; it does not close
the protocol gaps or the previously recorded full-suite/physical acceptance limits.

## Home Assistant reuse assessment -- October 25, 2026

The owner asked whether Home Assistant integration can supply missing contracts.
Two distinct integrations were inspected at pinned revisions. The earlier blanket
assessment that all remaining work lacks source evidence was too broad: community
code supplies concrete challenge handling and a legacy history-query candidate.
None of the upstream code was executed, copied into production, or used with an
account. This is source evidence, not live acceptance or a new runtime feature.

### Official Xiaomi Home integration

Reviewed revision: `75b3acb6325a3a8d295688b13904737d6738a6a5`.
The [OAuth client](https://github.com/XiaoMi/ha_xiaomi_home/blob/75b3acb6325a3a8d295688b13904737d6738a6a5/custom_components/xiaomi_home/miot/miot_cloud.py)
exchanges an authorization code and later a refresh token at the HA-specific
`/app/v2/ha/oauth/get_token` endpoint. Both use client ID and redirect URI; the
inspected client requests do not contain a client secret. That does not make the
registered HA application identity available to HomeVerb.
[Constants](https://github.com/XiaoMi/ha_xiaomi_home/blob/75b3acb6325a3a8d295688b13904737d6738a6a5/custom_components/xiaomi_home/miot/const.py)
select `ha.api.io.mi.com`, `ha.mqtt.io.mi.com`, a fixed registered application and
HA callback. This is neither HomeVerb's legacy `xiaomiio` session nor the archived
modern App `mijia` session. Its token refresh does not resolve either session's
independent authentication contract.

The [README](https://github.com/XiaoMi/ha_xiaomi_home/blob/75b3acb6325a3a8d295688b13904737d6738a6a5/README.md)
describes cloud MQTT subscriptions for property changes and events, central-hub
MQTT for local operation, and a separate partial IP-device LAN mode. MQTT event
push is not the missing `/miotspec/event/get` history/pagination contract. A future
HomeVerb subscription would be foreground-scoped and closed on background; HA's
persistent runtime and plaintext token storage cannot be adopted.

The [license](https://github.com/XiaoMi/ha_xiaomi_home/blob/75b3acb6325a3a8d295688b13904737d6738a6a5/LICENSE.md)
explicitly limits the integration and related cloud API interface to non-commercial
Home Assistant use and excludes development of other applications and services.
Direct transplantation into HomeVerb is therefore not admitted under that license.
A separately authorized native-client integration would need its own provider
permission, client registration and callback contract; none was obtained here.

### Community Xiaomi Miot integration

Reviewed revision: `f78158257e19a21902adaf6bcc8440c3ba172a91`.
The root [license](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/LICENSE)
is Apache-2.0, as is the previously cited revision's root license. The earlier
GPL label in the Mijia guide is corrected. Any future code reuse must preserve
applicable notices and check the selected files and dependencies separately.
A Python HA component is not an embeddable Rust Core adapter.

The [cloud client](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/xiaomi_cloud.py)
provides the following evidence:

| Area | Upstream behavior | HomeVerb disposition |
|---|---|---|
| Authentication | Defaults to `xiaomiio`; service login, password digest, captcha and user verification paths | Existing native route has the same service family; explicit protected challenge continuation is a concrete follow-up candidate |
| Expiry | Detects expired sessions, attempts logout/login, requests user reauthentication when necessary | This is relogin, not official OAuth refresh; preserve explicit foreground reauthorization and protected identity |
| History | `user/get_user_device_data` with `did`, `key`, `type`, `time_start`, `time_end`, `limit` | A concrete legacy bounded-read candidate; exact model keys, record semantics, retention and pagination still require review |
| Properties/actions | `miotspec/prop/get`, `prop/set`, `action` using service/property/action IDs | Useful protocol evidence; new commands still need exact model semantics and independent grants |
| Scheduling/retries | HA-managed scheduling and some setter retries or relogin/request replay | Do not transplant automatic uncertain-write replay or background work |

This particular community client's generic SID parameter and non-default SID branch do not
prove `mijia` login, its STS targets, continuous-stream encryption or modern App
host compatibility. No existing token is forwarded to another origin or service.

Next implementation candidates are protected user-driven captcha/verification
continuation and a narrow, exact-model legacy history read. Each requires a bounded
Core contract, sanitized observations, lifecycle/error tests and native/App entry
points before becoming callable. Official HA event push, legacy history reads and
modern App event history must remain separate contracts. Full coverage is still
incomplete, but further community-source investigation can proceed without another
private archive or account access.

## Community captcha protocol phase -- October 25, 2026

[ADR-0081](../adr/0081-mijia-interactive-community-login.md) adds typed Rust
`begin_login` and single-use `CaptchaChallenge::answer`. The original `login`
continues to return `MIJIA_AUTH_CHALLENGE_REQUIRED` without fetching an image.
The new explicit flow retains zeroizing login context, replaces the `ick` cookie
on a rejected answer, and requires a validated STS result before a session exists.
It expires after three minutes, permits at most three answers and retains original
cancellation. No sockets or leases stay open while waiting for input.

Dedicated HTTPS captcha retrieval rejects foreign URLs, redirects, oversized
images, excessive dimensions, wrong content types and missing/duplicate cookies.
Only bounded PNG/JPEG image data can reach a trusted native view. Secrets and
provider URLs are not serializable. The existing native C/App flow has not yet
adopted this typed API; SMS/email verification and history are still pending.
API 1.27, SQLite 5 and existing protected payloads are unchanged in this phase.

Executed Rust 1.96 Mijia tests: 69 unit tests and four compile-fail tests passed.
Strict all-target Mijia Clippy and all three native Mijia account regression tests
passed. Tests cover actual fictional TLS image
responses, request context/cookie continuity, cancellation/drop, lifetime and
attempt limits, hostile URLs/cookies/images, and legacy challenge behavior.
[Community attribution](../../third-party/hass-xiaomi-miot/NOTICE.md) records the
pinned Apache-2.0 evidence. No provider request, real account/device, package build,
remote push or publication was performed.

## Protected interactive account phase -- October 25, 2026

Typed Rust adapter and Host entry points now carry the captcha through protected
account registration. `begin_connection`/`continue_connection` hold no account
lease or registration lock while waiting for an answer. A continuation is bound
to its adapter, immutable label/selected account and the complete saved-account
session fingerprint set. Removal or changed protected sessions invalidate it,
including when the initial login did not select an existing account. Final STS
subject matching and secure-store checks precede persistence and publication.

The Host provides `begin_account_login`, `continue_account_login` and
`discard_account_login` only as trusted Rust methods. Four pending challenges
are allowed; answer consumes the old handle, rejection rotates it, and
background/cancel/close clears the registry. Invalid continuation metadata also
consumes the handle. The result is typed, not generally serializable. Ordinary
JSON, CLI and MCP cannot invoke these methods. The C bridge and Flutter entry
remain the next phase; API 1.27 and the previous APK are unchanged.

Executed `cargo +1.96.0 test -p homeverb-mijia -p homeverb-host --offline --locked`:
339 tests/doc tests passed. Strict all-target Clippy for both crates and Rust
formatting passed. Added checks cover protected identity, account removal during
input, registration/lease release, cross-adapter and cross-host refusal, registry
capacity, expiry, invalid metadata consumption, foreground/cancellation races,
provider isolation and refusal through ordinary JSON. No real account/device,
remote push or package publication was performed.

## Dedicated C login bridge -- October 25, 2026

API 1.28 implements begin/continue/discard through three dedicated C exports under
[ADR-0082](../adr/0082-native-interactive-account-login.md). All 34 ordinary methods
and the 19 prior C functions retain their contracts. Protected input buffers clear
on failure; captcha output carries only bounded UI image data and local metadata.
Android export checks and header-derived iOS checks include the added functions.
No mobile package rebuild or iOS acceptance is established by those list updates.

Executed: offline locked FFI build, FFI projection unit test, strict all-target
FFI Clippy, Rust formatting, complete C ABI verifier (34 methods and 55 generated
runtime examples plus approval, scan, link and identity recovery checks), and the
new login-buffer/lifecycle/refusal verifier. Contract validation passed 22 schemas,
56 positive fixtures and 404 negative fixtures. Documentation and repository
hygiene checks passed. All login material was fictional; no vendor request was
sent. Flutter captcha UI, SMS/email verification and real-account acceptance are
still pending.

## App captcha continuation -- October 25, 2026

The App negotiates API 1.28 before resolving dedicated interactive symbols and
retains one-shot login on older supported hosts. Mijia sign-in and exact-account
reauthorization display a bilingual bounded image prompt. Only explicit user
answers continue the flow. Wrong answers replace the challenge without changing
account selection; completed authentication still reports partial inventory
separately. There is no automatic answer, relogin, password persistence or command
replay. Account/password/answer arrays and native allocations clear on every path.

Cancellation, navigation, background, close and expiry clear input and image
buffers, evict decoded images and invalidate pending challenges. Late results
cannot replace account state or restore a dismissed dialog. Managed immutable
strings and engine-decoded pixels cannot offer a physical-memory zeroization
guarantee; they are not logged or intentionally cached/persisted after dismissal.

Executed: Flutter analysis; 16 new fictional controller, widget and native-admission
checks, including the complete App journey, wrong-answer replacement, exact-account
partial retrieval, late-result refusal, navigation/background/expiry/back dismissal,
API fallback and both locales at 320 pixels with doubled text. The final five-file account, account-native, inventory, identity-recovery and
captcha subset passed all 38 tests. Final analysis and formatting passed after
reviewing native-client background invalidation. Seven shared presentation tests and source hygiene passed.
The prior full-suite macOS pixel/click failures remain unverified by this subset.
New native/mobile packaging and real-account acceptance are separate delivery gates.

## Captcha-enabled Android delivery -- October 25, 2026

Clean source `ea2c6ccf7b024d76cd14e588f11f8e8831700ef2` produced the combined
`homeverb.platforms` release-mode development APK with native API 1.28. Its Mijia
flow includes the protected captcha protocol, account/Host registry, dedicated C
bridge and bilingual App prompt. It does not add SMS/email verification, modern
SID authentication, event history or additional appliance writes.

Artifact: `apps/mobile/build/delivery/mijia/ea2c6cc/homeverb-mijia-0.2.0-ea2c6cc.apk`.
Size: 75,095,050 bytes. SHA-256:
`6fb043337f72ca351019d54c8768fb28a0bfeb722b3d8ef6986e49a35794abf0`.
The adjacent ignored `verification.json` records clean source provenance and hashes.

Executed `scripts/build_android_native.py` for arm64-v8a and x86_64 using installed
Rust 1.99.1, NDK 28.2.13676358 and JDK 21: 23 exports per ABI, valid AAR contents
and 16 KiB ELF alignment passed. An initial Rust 1.96 packaging attempt failed
because that toolchain lacks Android targets; the already-installed stable targets
were then used without downloading a toolchain or changing host interfaces.
Executed `fvm flutter build apk --release --target-platform android-arm64,android-x64
--dart-define=HOMEVERB_ADAPTER=homeverb.platforms` with existing SDK/JDK settings and
the existing development signing identity. `scripts/verify_flutter_apk.py` passed
signature, archive, manifest, minimum API 28, disabled backup, absence of the
integration-test plugin, both ABIs, current AAR byte matching after NDK stripping
and 16 KiB alignment checks.

This supersedes the API 1.27 `28cf192` artifact for local evaluation. No installation,
real-account login, appliance request, physical-device acceptance, iOS package,
upload, push or publication was performed. This is not a store release.

## Further community verification and history evidence -- October 25, 2026

Source inspection at `f78158257e19a21902adaf6bcc8440c3ba172a91` continued after
the captcha delivery. These are implementation inputs, not newly callable features.
The [cloud client](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/xiaomi_cloud.py),
[verification tests](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/tests/test_cloud_auth_verify.py),
[login tests](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/tests/test_cloud_auth_login.py)
and [config flow](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/config_flow.py)
provide concrete distinctions:

- Notification verification starts with the provider's verification webpage and a
  user-entered `verify_ticket`. The inspected flow does not directly implement a
  native send-SMS/send-email endpoint. Do not present a ticket-entry primitive as
  complete native code delivery or assume every ticket is a short numeric code.
- `check_identity_list` changes `fe/service/identity/authStart` to `identity/list`,
  retains `identity_session`, and reads options 4 (phone) and 8 (email).
  `verify_ticket` submits `_flag`, `ticket`, `trust=false` and `_json=true` to
  `/identity/auth/verifyPhone` or `/identity/auth/verifyEmail` with that cookie.
  Upstream may try both methods; HomeVerb must retain explicit user selection and
  bounded, single-use continuation instead of blindly replaying an answer.
- A successful ticket returns a location. Upstream follows redirects, gathers
  cookies, optionally follows a provider `skipUrl` from an `/fe/` URL, and calls
  `/pass/serviceLogin` again to obtain `ssecurity`. A service token without that
  signing key is insufficient. Tests explicitly reject extracting a skip URL from
  arbitrary HTML. HomeVerb must validate each origin/path and cookie scope rather
  than adopt unrestricted redirects or automatic account-setting confirmation.
- The dedicated continuation needs browser/lifecycle handoff, protected cookie
  ownership and complete session/subject correlation. Existing captcha output
  deliberately has no URL field; a webpage handoff requires its own reviewed
  trusted contract. The independent modern `mijia` SID remains a separate gap.

For history, the pinned cloud client admits `did`, `key`, `type`, epoch-second
`time_start`/`time_end` and `limit` at `user/get_user_device_data`. It returns
`result` records; its last-value helper JSON-decodes `value` and takes the first
array item. [Device code](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/device.py)
and [customizations](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/device_customizes.py)
show distinct keys such as `prop_cal_day.power_cost` for `chuangmi.plug.v3`,
`store.powerCost` with day grouping for `zimi.powerstrip.v2`, and
`event.human_visit_details` for the `mxiang.cateye` family. These examples do not
establish the same keys for the reference's `cuco.plug.v3` or `madv.cateye.mi3iot`.
Do not transfer a model's energy conversion or event/media payload to another
model by family name. A bounded query is possible with exact evidence; complete
pagination, retention guarantees and the modern `/miotspec/event/get` contract
are not established by these helpers. No upstream source, account identifiers,
webpage execution or provider traffic was copied into this research record.

## Exact reference fan settings — October 25, 2026

[ADR-0083](../adr/0083-exact-reference-fan-settings.md) is implemented through
protected mapping, Core policy, native API 1.29 startup grants and the App. The
exact reference fan has eight property-set options: its four existing options
and four new included-angle, vertical-swing and ionizer settings. The coverage
ledger records these as legacy-route implementation, not modern SID support.
Other timers, reservations, custom service writes and MIoT actions remain open.

The new settings have ten permitted values in total. Fictional Core tests
check every value's SIID/PIID and scalar wire encoding, reject missing grants,
wrong values and default read-only composition, and retain an unknown receipt
without replay when a reply is lost. Mapping tests reject changed model, service,
property, format, unit, access and domains; old read-only derivations still
validate and upgrade without changing observation definitions.

Executed on macOS:

- `cargo +1.96.0 test -p homeverb-mijia --offline --locked`: 75 unit tests and
  5 compile-fail doc tests passed.
- `cargo +1.96.0 clippy -p homeverb-mijia -p homeverb-host -p homeverb-ffi --all-targets --offline --locked -- -D warnings`: passed.
- `cargo +1.96.0 build -p homeverb-ffi --offline --locked`: passed.
- `python scripts/verify_native_contract.py`: passed all 34 methods and 55
  runtime examples, including all ten new startup values and invalid-value
  refusal before profile creation.
- `fvm flutter analyze`: passed. The four-file subset (`mijia_controls_test`,
  `mijia_test`, `session_permissions_test`, `account_native_test`) passed 33
  tests; the updated 11-test controls suite then passed against the rebuilt
  library, including read-only version probing and granted reopen for both
  Mijia-only and combined sessions.

Repository gates also passed: 22 schemas, 59 positive and 484 negative fixtures;
161 Markdown files and 1,729 local links; 666 maintained text files; 250 existing
Conventional Commits; all seven shared presentation checks; reference regeneration
and `git diff --check`. The public reference profile catalog did not change.

Flutter verifies exact permission choices, API 1.28 refusal before reopening,
English/Chinese controls at 320 pixels with double text scale, and forms that
do not send when opened. No account, appliance, real write or physical
acceptance was used. The previously delivered API 1.28 APK does not contain
these changes. Existing full-suite macOS glyph tests and Meiju LAN alias
limitations remain outside this phase's passing scope.

## Reference fan APK delivery — October 25, 2026

Clean source `71d7ad17dcf31a14072e23cc369b6b96e9a6897f` built the combined-platform API 1.29
release-mode development APK at
`apps/mobile/build/delivery/mijia/71d7ad1/homeverb-mijia-0.2.0-71d7ad1.apk`.
Size: 75,135,482 bytes. SHA-256:
`7e383f87ee1469dfce38c1857f997b9e6b159445671a3fa5bca66cdc9d5efc22`.
The adjacent `verification.json` records the exact source revision, clean status,
AAR and per-ABI library digests and the actual archive checks.

Executed `scripts/build_android_native.py` with installed stable Rust 1.99.1,
NDK 28.2.13676358 and JDK 21 for arm64-v8a and x86_64; both ABIs passed 23 native
exports and 16 KiB ELF alignment. Executed
`fvm flutter build apk --release --target-platform android-arm64,android-x64
--dart-define=HOMEVERB_ADAPTER=homeverb.platforms` with the existing SDK/JDK
configuration and development signing identity. `scripts/verify_flutter_apk.py`
passed signature, ZIP/ELF alignment, API 28 minimum, disabled backup, no test
plugin, and byte matching against the current AAR after normal NDK stripping.

The artifact contains the four new exact reference fan settings and supersedes
the API 1.28 `ea2c6cc` package for local evaluation. It is not installation,
real-account, appliance-write, physical-device or iOS acceptance. No upload,
push or publication occurred. A fresh `git fetch origin` still resolved main
to `ccf2975df836f8e81ccf24fa26c8b2035390c978`; main is an ancestor of this branch
and `git rev-list --count --merges origin/main..HEAD` returned zero.

## Exact light and purifier controls -- October 25, 2026

[ADR-0084](../adr/0084-exact-light-and-purifier-controls.md) is implemented for
`abhome.light.wy0a01` (seven settings) and `zhimi.airpurifier.v6` (seven settings).
The protected mapping requires pinned model/device/service/property identity,
wire format, unit, readable/writable access and exact domain. Existing observation
IDs remain stable; old complete mappings validate and upgrade on explicit refresh.

`with_reference_controls` and trusted `mijia_setting_grants` are independent of
fan admission and grants. Core requires an exact local target; the adapter
rechecks protected evidence before its one property-set send. There are no
companion mode/power changes or automatic uncertain-write retries. A correlated
acknowledgement is accepted without physical verification. Other reference
controls, modern SID authentication, events/history and media/health remain open.

API 1.30 adds the closed setting-grant field and exact control-option component
scopes without changing 34 JSON methods or 22 C exports (23 with the Android
initializer). SQLite schema 5 and protected payload versions are unchanged.
Flutter checks 1.30 before reopening with the new field. It retains 3,930 exact
light targets and 31 purifier targets, including every one of the 3,801 values
from 2700 through 6500 K; no preset-only substitute or range grant is used.

Executed verification:

- `cargo +1.96.0 test -p homeverb-mijia -p homeverb-host --offline --locked`: both
  package suites passed, including 78 Mijia unit tests, five Mijia compile-fail
  doc tests and the Host test of all 3,801 exact color temperatures.
- `cargo +1.96.0 clippy -p homeverb-mijia -p homeverb-host -p homeverb-ffi --all-targets --offline --locked -- -D warnings`: passed.
- `cargo +1.96.0 build -p homeverb-ffi --offline --locked`: passed.
- `python scripts/verify_native_contract.py`: all 34 methods and 55 runtime
  examples passed, including both platform modes with all 14 settings and
  invalid-value refusal before profile creation.
- `fvm flutter test test/mijia_settings_test.dart test/mijia_controls_test.dart
  test/session_permissions_test.dart test/mijia_reference_test.dart`: 30 passed.
  New cases cover both native modes with all 3,961 targets, API 1.29 refusal,
  English/Chinese 320-pixel double-text-scale controls, no send on opening and
  exact 4321 K input with out-of-range refusal. An initial form-test lookup used
  a nonexistent key; the corrected test finds the real submit button.
- `fvm flutter analyze`: passed.
- Contracts: 22 schemas, 62 positive and 626 negative fixtures passed. Docs and
  repository hygiene passed, along with all seven shared presentation tests,
  reference regeneration and `git diff --check`. The pinned public profile
  catalog is unchanged.

No provider/account/appliance requests or physical writes were used. Earlier
full-suite macOS glyph and Meiju LAN alias limitations remain outside this
passing scope. The iOS version assertion is synchronized but no iOS build was
run. The prior API 1.29 APK does not contain this phase until separately rebuilt.

## Light and purifier APK delivery -- October 25, 2026

Clean source `37e9f1c3a9a62cbdf7b3a19f05639231c09328b9` produced the combined-platform API 1.30
release-mode development package:
`apps/mobile/build/delivery/mijia/37e9f1c/homeverb-mijia-0.2.0-37e9f1c.apk`.
Size: 75,177,578 bytes. SHA-256:
`8b7e1fb97ce1625a174d0e49124e0463b36835613ff62c3292ea225bfe7bf2f4`. The adjacent `verification.json`
records clean source, exact AAR/library digests and actual archive checks.

Executed `scripts/build_android_native.py` with installed stable Rust 1.99.1,
NDK 28.2.13676358 and JDK 21 for both arm64-v8a and x86_64. Both native libraries
passed all 23 exports and 16 KiB ELF alignment. Executed
`fvm flutter build apk --release --target-platform android-arm64,android-x64
--dart-define=HOMEVERB_ADAPTER=homeverb.platforms` with the existing SDK/JDK and
development signing identity. `scripts/verify_flutter_apk.py` passed signature,
ZIP/ELF alignment, API 28 minimum, disabled backup, absence of the integration-test
plugin and exact matching against the current AAR after NDK stripping.
The host `cargo +1.96.0 test -p homeverb-ffi --offline --locked` unit test also passed.

This package supersedes API 1.29 `71d7ad1` for local evaluation and contains
the new light/purifier controls. No installation, account/appliance operation,
real write, physical-phone/iOS acceptance, push, upload or publication occurred.
`git rev-list --count --merges origin/main..HEAD` still returned zero. This is
a verified local build, not completion of the remaining Mijia reference scope.

## Protected phone and email ticket protocol -- October 26, 2026

[ADR-0085](../adr/0085-mijia-protected-verification-tickets.md) now has an opt-in
typed Rust implementation. `begin_verification_login` returns a connected session,
a captcha or a non-serializable verification challenge. Captcha answers may
explicitly transition to verification while preserving their original lifetime.
Existing one-shot and captcha-only entry points retain their previous behavior.

The dedicated one-send transport retains bounded initial account cookies, obtains
explicit methods from `identity/list`, submits one protected ticket and follows
at most four validated continuations per completion sequence. Both phone and
email require a fresh legacy SID `xiaomiio` signing key and a consistent numeric
subject. Account cookies never go to STS; captured service tokens are never sent
in verification requests. Cookie expiry uses a monotonic clock after converting
provider expiry once. The challenge consumes one answer, expires after three
minutes and retains no socket, password, account lease or background worker while
waiting. Direct success and captcha success also check exchange-cookie identity.

The provider page is protected protocol material for a future trusted foreground
browser. Tickets need not be numeric codes. There is no SMS/email sending endpoint,
automatic alternate-method attempt, password replay, HTML parsing or implicit
`skipUrl` account-setting confirmation. Extra provider confirmation pages remain
unsupported. Account/Host snapshot binding, native bridge and App browser/ticket
interaction remain pending. API 1.30, 34 ordinary methods, C exports, SQLite 5 and
protected payloads are unchanged; the existing APK does not expose this protocol.

Executed checks use fictional peers and local TLS only. The Mijia suite passed
94 unit tests and six compile-fail documentation tests, including both methods,
direct/captcha success, missing options, malformed/expired cookies, conflicting
subjects, missing signing key/token, rejected tickets, bounded redirects, provider
confirmation refusal, cancellation, expiry and drop. No real account, appliance,
secure-store credential or vendor request was used. Strict Rust 1.96 Clippy for Mijia/Host/FFI, the complete Host package suite,
the FFI build and the existing native login verifier passed. Documentation,
22 schemas with 62 positive and 626 negative fixtures, repository hygiene,
256 existing commit messages and the exact-reference coverage check passed.
No mobile rebuild or full-workspace green result is claimed. Modern
SID `mijia`, native verification interaction and full reference parity remain open.


## Account-bound verification continuation -- October 26, 2026

The opt-in `begin_verification_connection` and `continue_verification_connection`
methods bind the typed protocol to immutable target/label, adapter ownership and
protected payload digests. The account snapshot is checked before sending and
again under the registration lock before saving. Existing same-subject identity,
device mappings and preferences remain authoritative. Waiting retains no account
lease, and a captcha-to-verification transition retains the original lifetime.
Wrong answer kinds consume the attempt without sending. New pending values and
presentation data are non-serializable and have redacted debug output.

The typed Host `begin_account_verification`, `continue_account_verification` and
`discard_account_verification` methods manage single-use session-owned handles.
The new registry and old captcha registry share a four-entry bound. They cannot
consume one another's handles. Expired entries are pruned before new admission;
background/cancel/close clear both, cancel active operations and prevent late
results from reinstating pending state. Connected results use the existing
sanitized account/card publication path. No account is published while waiting.

These methods are not exposed in C, ordinary JSON, CLI, MCP or Flutter. API 1.30,
34 ordinary methods, export counts, SQLite 5 and protected payloads remain
unchanged. Foreground trusted browser/ticket interaction and real account/phone
acceptance remain pending. This phase adds no SMS/email sending or modern SID
`mijia` route and performs no vendor/account/appliance traffic.

Executed checks passed: Mijia's 98 unit and seven compile-fail documentation tests;
the full Host suite including six new verification scenarios and the additional
compile-fail constraint; strict Rust 1.96 Clippy for Mijia/Host/FFI; the FFI build;
and the existing native login verifier. Documentation links, 22 schemas with
62 positive and 626 negative fixtures, source hygiene and exact-reference
coverage passed. This is affected-package evidence, not a full-workspace or
mobile/browser acceptance claim.


## Native protected verification bridge -- October 26, 2026

[ADR-0086](../adr/0086-native-protected-verification-prompt.md) adds three dedicated
C functions in API 1.31. Login/password and answer input remain separately
zeroed; a caller-owned output buffer of exactly 131072 bytes carries protected
captcha/provider-page bytes. Returned JSON contains only closed local metadata.
The old captcha result remains unchanged. A decoded answer selects captcha, phone
or email; the protected adapter still checks the actual kind and offered method.
No C request can inject a provider URL, cookie or alternate account.

The total is 25 C exports plus the Android initializer, with 34 ordinary methods.
Schemas, examples, Android export expectations and the iOS version assertion are
synchronized. No Android/iOS package was rebuilt in this bridge phase, and the
existing API 1.30 APK remains the latest packaged artifact. Flutter/browser
integration, real account acceptance and modern SID `mijia` remain pending.


Executed local checks passed: three FFI unit tests; Rust 1.96 formatting and strict
Host/FFI Clippy; the FFI build; the new C verification verifier including maximum
256-byte login/password and 2048-byte ticket buffers; the unchanged captcha
verifier; and the full native contract suite (34 methods, 55 examples and existing
trusted bridge checks). The new verifier is included in future standard native
contract runs. Flutter `native_test`, `current_contract_test`,
`platform_accounts_test` and `account_captcha_test` passed all 36 tests against
the current host library. Seventeen Android/iOS package-tool unit tests passed;
these are not package builds. Documentation links, 23 schemas with 62 primary
positive fixtures plus dedicated bridge examples and 711 negative fixtures,
repository hygiene and exact-reference coverage passed. No remote publication,
installation, vendor account access or appliance write occurred.


## Public verification page protocol evidence -- October 26, 2026

Anonymous GETs to the public [identity entry page](https://account.xiaomi.com/fe/service/identity/authStart)
and its static assets provide code-delivery facts absent from the pinned community
client. No notification query, account, credential, cookie, SMS or email request was
used. Static JavaScript was inspected as text only; it is not vendored, executed,
or used as HomeVerb application identity. HomeVerb independently implements the
wire facts. This does not change third-party licensing or claim provider support.

The asset base is
`https://cdn.web-global.fds.api.mi-img.com/mcfe--mi-account/static/static/js/`.
Observed assets and SHA-256 values:

| Asset | SHA-256 | Relevant facts |
|---|---|---|
| `main.f09071d0.js` | `db79e6a9040f2d07c4a67891d590503c83c61ee94681406da3571018ed282f2c` | Mobile entry chunk selection |
| `runtime-main.18cf6897.js` | `bb1ed64a8f1dc638d8e8f67fed51e70d4fb6096697fc97f7265e9f825d811da9` | Named, hashed static chunk mapping |
| `mobile.7172b162.chunk.js` | `74cd1a467808584b7b7de45f3db30bc50b5b2115fc2789892c37a6712d43ca23` | Selected-method GET, flag and masked-address/content-type metadata |
| `9500.5c2bceac.chunk.js` | `38e104c0c3539144a556f9afad7052029c7e811fda7e1f26cdc16513455f52f4` | Empty `icode` initial attempt; human risk errors 87001/20031; quota result checks |
| `8467.4534a166.chunk.js` | `1a07a34398305123c033bea8c05d6570e34d47393a064f4910c15c40ae497757` | Phone send POST, quota POST, 60-second UI cooldown |
| `IVPh.b3e6be45.chunk.js` | `7279616944d3cd225b8df111609be54de0561502c6da4bd2270a59a1337772d5` | Phone `retry` ordinal, PH quota, default content type 160010 and optional `data.wt` delay |
| `IVE.9e774d2d.chunk.js` | `272f289c97476ff5ab795654692681f6a1fae5a9e1870acbbf9761abd042c79e` | Email send POST with `_dc` cache-buster, retry ordinal and EM quota |

One referenced shared chunk (`9723.31b42442.chunk.js`) returned HTTP 404. No
missing human-risk implementation is inferred from other chunks. The observed
frontend may run interactive risk checks and repeat; HomeVerb does not emulate
that automatic behavior. Account-settings confirmation, additional identity
stages and real provider acceptance remain incomplete.

## Explicit verification code request protocol -- October 26, 2026

[ADR-0087](../adr/0087-native-mijia-verification-code-delivery.md) implements the
ordinary explicit code-request path as typed Rust. Method inspection and positive
quota precede one selected phone/email send. Initial login/prompt presentation
never sends a code. Acknowledgement means the provider accepted the request, not
that the user received a message. Masked addresses remain discarded provider data.
Existing cookie scope, subject consistency, protected material and original
three-minute lifetime remain mandatory. There is no new secret or developer key.

The challenge retains per-method ordinals, at most three successful requests,
and a monotonic cooldown of at least 60 seconds, extended by a valid provider wait.
No timer sends traffic. Any failure consumes the attempt; no alternate-method
fallback, risk-token generation, implicit retry or password replay occurs.
Typed adapter/Host continuation rechecks account snapshots before/after code
requests, returns a new session-owned handle, and uses existing foreground
cancellation and aggregate pending limits. Native API 1.31 and old C results are
unchanged: a dedicated code-request C operation and App interaction remain next.
No real SMS/email delivery, mobile packaging or account acceptance is claimed.


Executed checks passed: 105 Mijia unit tests and seven compile-fail documentation
tests, the complete Host package suite (including both new code-request cases),
strict Rust 1.96 Clippy for Mijia/Host/FFI, the FFI build and both dedicated native
login verifiers. A separate adapter race test confirms a protected payload change
during the send rejects the new prompt without saving or replaying. Documentation,
23 schemas with 62 primary positive and 711 negative fixtures, source hygiene and
exact-reference coverage passed. Static asset GETs were the only external traffic;
no authenticated API, SMS/email send, device operation or publication occurred.
The unreleased typed `VerificationEndpoint` enum gains five cases; injected
transports with exhaustive matches must handle these for the opt-in code flow.
Existing C/App API 1.31 is unchanged and no package was rebuilt.


## Native explicit code-request bridge -- October 27, 2026

API 1.32 exposes the existing typed code-request operation through one dedicated
trusted C function. Closed metadata admits only the operation, session-owned
challenge and phone/email choice. The output buffer rules are unchanged, and the
result adds count/cooldown metadata to a new verification prompt handle. Existing
API 1.31 begin/answer results keep their shapes; ordinary methods and storage
formats are unchanged. There are 26 C exports plus Android's initializer.

Schemas, examples, Android export expectations and the iOS version assertion are
synchronized. No mobile package or Flutter UI is implied by these source changes;
the latest packaged APK remains API 1.30. No actual SMS/email, account traffic,
installation, device operation or publication occurred.


Executed checks passed: FFI unit tests/build, strict Rust 1.96 Mijia/Host/FFI
Clippy, the dedicated four-export verification ABI verifier and the full native
contract suite (34 methods, 55 examples and existing trusted bridge checks).
Documentation links, 23 schemas with 62 primary positive and 743 negative
fixtures, repository hygiene and 17 Android/iOS package-tool unit tests passed.
These are host/source checks, not rebuilt mobile packages or provider acceptance.
An approval-service quota interruption was resolved before the final checks;
no alternate write path bypassed that service.


## Foreground App verification -- October 27, 2026

The Dart bridge negotiates API 1.32 before resolving the four dedicated C exports.
Caller-owned allocations carry credential/ticket inputs and protected prompts;
all are cleared in finally paths. Only captcha bytes are copied into Dart. The
provider-page buffer is never decoded into a Dart string, persisted or rendered.
Pending objects belong to one client generation and are consumed on answer/send;
malformed or late results discard the returned handle. Waiting holds no operation.

The controller selects the new path for Mijia, preserving exact reauthorization,
partial inventory and older captcha/one-shot fallbacks. The foreground dialog
shows only offered methods and requires an explicit send. Local timers display
cooldown and expire the dialog; they do not generate network work. At most three
requests are offered, while native enforcement remains authoritative. Cancel,
back, navigation, background and expiry clear input and images. UTF-8 input is
bounded by bytes, and unavailable methods/control characters are rejected.

The synthetic C fixture has no network or secret-store code. It exercises
successful captcha, method selection, send and ticket ABI exchange, deliberately
invalid UTF-8 page bytes, malformed metadata, foreign-session refusal and a
controlled late response after cancellation. App tests cover both methods,
partial inventory, old-API fallback, English/Chinese narrow enlarged layouts,
request limits and lifecycle cleanup. Real account access, code delivery,
additional human-risk stages and modern SID `mijia` authentication remain
unverified or unavailable. No account, message send, device write or publication
is part of this phase. Package delivery is recorded separately.


Executed checks: Flutter analysis passed. The full suite passed 249 tests and
failed the same eight previously recorded macOS font/pixel and legacy
app/outcome-click tests; it is not a full-suite pass. After the final Unicode
control-character and cooldown case, the seven-file affected regression subset
passed all 66 tests, including six fictional C ABI cases and 19 new verification
controller/widget/refusal cases. Documentation, 23 schemas with 62 positive and
743 negative fixtures, repository hygiene and commit conventions passed. No
Rust source or C ABI changed in this App phase. No package build, installation,
provider request, real message delivery or physical-device acceptance is implied.


## Verification API 1.32 Android package -- October 27, 2026

Clean source `01a115b68f8441581e211c47e00c0d4a1df5270d` produced the combined-platform
release-mode development APK at
`apps/mobile/build/delivery/mijia/01a115b/homeverb-mijia-0.2.0-01a115b.apk`.
Size: 76,100,350 bytes. SHA-256:
`e4de2f3e8f80f0347eb86ed020a37da7be2d4746280ecebee9675f9f190f6c90`.
The adjacent `verification.json` records the clean source, current AAR/native
hashes and exact stripped AAR matching for arm64-v8a and x86_64.

Executed `scripts/build_android_native.py` using installed stable Rust 1.99.1,
NDK 28.2.13676358 and JDK 21. Both ABIs passed 27 exports and 16 KiB ELF alignment.
Executed `fvm flutter build apk --release --target-platform
android-arm64,android-x64 --dart-define=HOMEVERB_ADAPTER=homeverb.platforms`
with FVM Flutter 3.47.7 and the existing development signing identity.
`scripts/verify_flutter_apk.py` passed signature, ZIP/Core ELF alignment, API 28
minimum, disabled backup, absence of the integration-test plugin, both complete
ABIs and current AAR matching. No SDK installation, APK installation, real
account/message request, appliance write, upload, push or publication occurred.

This package supersedes API 1.30 for local evaluation of the verification UI.
Archive checks do not establish phone, provider or physical-device acceptance;
the eight previously recorded full-suite macOS failures remain unresolved.
Modern SID `mijia`, event/history and remaining functional coverage stay open.


## Exact daily energy source evidence -- October 27, 2026

Further read-only inspection of the pinned Apache-2.0 community revision identifies
an exact statistics path for reference `cuco.plug.v3`. Its
[customizations](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/device_customizes.py)
select key `11.1`, ratio 0.01 and the matching kWh attribute unit. Its
[device implementation](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/device.py)
sends `v2/user/statistics` with `stat_day_v3`, a time window and bounded limit.
The [utility decoder](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/utils.py),
[templates](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/templates.py)
and [statistics tests](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/tests/test_power_cost_statistics.py)
read record timestamps and JSON-string value arrays. They distinguish invalid or
missing values from zero. This corrects the earlier lack of an exact reference
energy candidate; it does not establish the legacy event/history keys previously
discussed, complete paging, modern SID authentication or cloud calendar boundaries.

[ADR-0088](../adr/0088-mijia-exact-daily-energy-history.md) defines a bounded exact
native implementation. A native protocol, Core historical-read contract and App
presentation are separate implementation stages. The sources were fetched as text
only; no upstream executable, HA runtime, account or provider request was used.


## Exact daily energy typed protocol -- October 27, 2026

`MijiaSession::daily_energy_history` implements ADR-0088 with fixed legacy
statistics endpoint, key 11.1, type `stat_day_v3` and limit 31. Its immutable window
is a half-open positive epoch-second interval of at most 32 days, with no future
end admitted at request time. The exact reference model/specification and mainland
session are mandatory; unrelated model statistics cannot reuse this conversion.

The decoder returns sorted, sanitized provider timestamps and kWh records, receipt
time, requested interval, rejected-record count and limit status. Completeness is
always unproven. Invalid numbers, nonnumeric/boolean/multichannel payloads,
underflow to zero, invalid/out-of-window timestamps and all duplicate-timestamp
records are excluded without erasing valid neighbors. Foreign optional device/key
correlation, malformed envelopes and over-limit responses refuse completely.
Zero remains a valid observation only when the provider actually reported zero.
No calendar timezone, complete month sum or missing-day value is inferred.

The operation sends once, preserves encrypted legacy request/response checks, and
honors cancellation/deadline/drop. The unreleased typed `CloudEndpoint` gains
`Statistics`; exhaustive injected transports need to recognize the new variant.
Ordinary C/native API 1.32, JSON/CLI/MCP methods, grants, schemas and saved payloads
are unchanged. Core/adapter admission, host presentation and real acceptance are
next stages; the current APK does not expose this protocol. No provider account,
statistics request, device write or remote publication occurred.


Executed checks passed: all 114 Mijia unit tests and seven compile-fail doc tests,
the complete Host package suite, and strict Rust 1.96 all-target Mijia/Host/FFI
Clippy. The added local TLS case checks the exact statistics route and encryption
headers without contacting Xiaomi. Documentation, 23 schemas with 62 positive
and 743 negative fixtures, repository hygiene and exact-reference regeneration
passed. No native API/export or Flutter change was made in this protocol phase.


## Typed Core and protected daily history -- October 27, 2026

Core `inspect_history_options` and `read_history` implement ADR-0089. Closed
historical DTOs reference existing local device, connection, component, capability
and numeric property. The adapter defaults return no options and refuse reads.
Core validates options against current metadata, bounds the query before traffic,
locks the selected device, admits exactly one connection and validates the entire
returned result including target/unit/source, timestamps, order, counts and limits.
It repeats descriptor admission after I/O and rejects late results. Nothing is
saved to the current-state cache or execution journal.

Mijia implements both hooks using protected account leases and exact profile
admission. Local inspection does not network or write credentials. The read
independently repeats model, mapping, membership, identity and request admission
before the statistics request. A protected reference mapping can supply the exact
URN when inventory omitted it; an unknown or conflicting profile cannot. The
account lease and device lock remain held only during the bounded read and are
released on completion, cancellation, deadline or future drop.

This is a typed library implementation, not an App feature or physical acceptance.
The native API remains 1.32 with 34 ordinary methods and unchanged exports/storage.
Host/CLI/MCP/App integration is still required. No provider account, real history
request, appliance operation or remote publication occurred.


Executed strict Rust 1.96 workspace/all-target Clippy passed. The full workspace
test attempt stopped at 14 previously recorded Meiju LAN fixtures with macOS
`AddrNotAvailable` (56 Meiju library cases passed); required synthetic loopback
aliases are not assigned and host networking was not changed. A complete
`cargo +1.96.0 test --workspace --exclude homeverb-midea --offline --locked` run
passed all included suites and documentation tests, including eight Core history
cases and 118 Mijia unit tests. This is not a full-workspace pass. Documentation,
24 schemas with 65 primary positive and 792 negative fixtures and repository
hygiene passed. No Flutter or mobile package was changed in this layer.


The subsequent timestamp admission correction aligns typed DTO deserialization
with the canonical schema (`...:ssZ`) and refuses offset, fractional and leap-second
forms before normalization. Typed Core also rejects leap-second values constructed
in process. Eight Core history tests, the six-case Mijia history subset and strict
Domain/Core/Mijia/Host Clippy passed after this correction.


## Native bounded historical reads -- October 27, 2026

API 1.33 exposes existing Core history inspection and reading through two closed
ordinary methods. Generic dispatch introduces no brand semantics or privileged
identity parameters. The protected Mijia adapter remains the exact daily-energy
opt-in; default adapters expose no options. There are 36 ordinary methods, with
unchanged C exports, saved formats, write grants and existing method behavior.

The C verifier exercises both new methods against Mock (empty option list and
unsupported read), rejects extra private/authority fields and includes them in
all-method lifecycle checks. The Host tests additionally inject a fictional
historical adapter to verify successful structured results and cancellation on
background, explicit cancel and close, without cache or execution writes.
Schemas/manifests and the iOS acceptance version assertion match API 1.33; this
is not a rebuilt iOS or Android artifact. Existing App account paths remain
compatible; historical UI, CLI/MCP commands and provider acceptance are pending.
No account access, cloud statistics request, real-device write or publication
occurred.


Executed checks passed: FFI host build; strict workspace/all-target Rust 1.96
Clippy; complete Host and FFI tests including ten historical Core/Host cases;
all 36 native methods with 57 regenerated fictional C ABI examples and every
existing trusted bridge verifier; 61 Flutter native/account/captcha/verification
compatibility cases against the rebuilt library; documentation, 24 schemas with
65 primary positive and 792 negative fixtures, and repository hygiene. The earlier
full-workspace Meiju loopback limitation remains recorded; no full-workspace or
mobile/physical acceptance is inferred. The current APK is still API 1.32.


## Foreground App historical readings -- October 27, 2026

The Flutter device-detail dialog now consumes API 1.33 without platform/model
branches. Local connection/series inspection precedes a separate explicit bounded
UTC query. Response validation preserves historical units independently from live
property units, sorted timestamps, zeros, rejected counts, limit status, source,
receipt time and unproven completeness. Missing records remain missing and the
unknown provider calendar timezone prevents complete daily/monthly-total claims.

The dialog cancels on navigation/background/close and discards late results after
identity or session changes. It pauses visible-device state polling for its lifetime;
it introduces no history polling, retries or persistence. CLI/MCP historical tools,
new Android packaging and owner-run provider acceptance remain pending. Modern SID
login, event coverage, additional controls and media/health remain separate open
items. No real account, cloud query, device write or publication was performed.

## Mijia CLI and historical tools -- October 29, 2026

[ADR-0090](../adr/0090-mijia-cli-and-historical-tools.md) adds explicit read-only
Mijia CLI/MCP hosting and generic bounded history options/reads. CLI account
commands reuse the existing Session, with hidden one-shot login and no automatic
challenge sends. Saved cross-account conflicts remain unsupported for Mijia.
Six read-only MCP tools are available without grants, with eight tools for
existing admitted Meiju controls. No write grant or modern SID support is added.
Historical reads preserve Core evidence and cancellation without cache/receipt
writes, polling or retries. Native API 1.33, schemas and protected storage formats
are unchanged. MacOS credential operations remain blocked by its unimplemented
secure-store backend; Linux Secret Service and real-provider acceptance require
the appropriate owner environment. The API 1.33 App APK remains valid because
this phase changes only the CLI host, tests and documentation.


Executed CLI/MCP checks: `cargo +1.96.0 test -p homeverb-cli --offline --locked`
passed 11 tests, including a successful fictional historical adapter, immutable
closed input, partial output, cache/receipt isolation, FIFO/size refusal and
notification/EOF/host/deadline cancellation without replay. Strict workspace
all-target Clippy and the CLI build passed. `scripts/verify_mvp.py` ran 37 process
cases: 30 passed and seven retained documented macOS/Linux-only skips. It verifies
actual empty Mijia CLI/MCP startup, no Internet sockets while idle, hidden-input
refusal without a terminal, presend grant/write refusal, schemas and exact history
inputs. No real provider credentials, cloud query or appliance was used.
Documentation, 24 schemas (65 primary positive and 792 negative fixtures), source
hygiene and diff checks passed. Linux account/secret-store, real-provider history,
physical support and remote CI are not established by these local checks.

## Event-key audit and exact outlet control decision -- October 29, 2026

Further inspection of the pinned community `binary_sensor.py`, `sensor.py` and
customizations confirms that BLE object queries and generic
`user/get_user_device_data` history are distinct from modern event-history RPCs.
The motion/door-magnet examples use their own BLE keys; the lock display fallback
references `event.11`, and `mxiang.cateye` has a separate visit-details key. None
establishes a complete key/payload contract for this reference's
`loock.lock.h01myk` or `madv.cateye.mi3iot`. No cross-model event mapping or new
provider request has been introduced on this evidence.

The same reference supplies four independent settings for exact `cuco.plug.v3`.
The pinned community switch writer confirms single-property dispatch; the existing
native legacy transport can carry it. [ADR-0091](../adr/0091-exact-outlet-control-approval.md)
defines separate outlet grants, elevated trusted confirmation for load power,
power-loss recovery and physical-control locking, and low-risk indicator control.
This is an accepted implementation design, not a callable control feature. Native
API remains 1.33 until the new grants, protected runtime, schemas and inspection
scope are implemented. No physical result, event support or complete parity is
claimed. The current ledger deliberately retains the pending outlet disposition.


## Typed exact outlet controls -- October 29, 2026

[ADR-0091](../adr/0091-exact-outlet-control-approval.md) now has a separate typed Rust
adapter opt-in for four exact `cuco.plug.v3` settings. Power, power-loss recovery
and physical-control locking require Core grants plus actual trusted approval;
indicator control is low-risk. The adapter re-derives risk from protected evidence
and refuses forged low-risk metadata independently of Core. Single-property
requests retain exact values, accepted/unknown outcomes and no automatic replay.

Existing API 1.33 inspection/default account retrieval retain their prior scopes
and read-only outlet derivations. An explicit typed inspection can derive outlet
options locally without enabling execution. Only the typed outlet-enabled refresh
persists new control derivations; older implementations may reject those newer
protected contents, so keep the originating implementation with that profile.
SQLite 5 and payload envelope 1 remain unchanged. Native outlet startup grants,
API 1.34/schema rollout, App confirmation, APK and real-device acceptance are
still pending; the API 1.33 APK does not acquire outlet controls from this phase.

Executed verification: 124 Mijia unit tests and seven documentation tests passed
as part of `cargo +1.96.0 test --workspace --exclude homeverb-midea --offline
--locked` (474 passed, one ignored). The excluded Meiju package has 14 previously
recorded macOS LAN failures from unassigned loopback aliases; this is not a full
workspace pass. Workspace/all-target Clippy with warnings denied passed. Reference
facts, documentation, repository hygiene and contract validation passed (24 schemas,
67 positive fixtures, 798 negative fixtures). No real account or appliance was used.


## Native exact outlet controls -- October 29, 2026

API 1.34 adds closed `mijia_outlet_grants` with exact local device/component/
property/value targets and a 64-entry limit. Only Mijia and combined hosts accept
these startup grants. All four exact outlet options can be inspected locally
before granting authority. Existing light/purifier and fan fields retain their
scopes. Separate adapter policies prevent overlapping physical-lock identifiers
from transferring authority between purifier and outlet grants, even when both
families are enabled.

Power, power-loss recovery and physical-button locking require the separate
immutable trusted preview/confirm bridge for every operation; indicator control
is low-risk. Default startup and CLI/MCP remain read-only for outlets. Ordinary
JSON cannot confirm elevated commands. Unknown replies preserve the request ID
without replay, and backgrounding invalidates previews. C exports (26 plus the
Android initializer), 36 ordinary methods, SQLite 5 and protected payload envelope
1 are unchanged. Explicit account refresh in an outlet-enabled session upgrades
old read-only mappings. Older implementations may refuse those upgraded protected
contents; retain the matching implementation with that profile. App wiring, an
updated APK and real-provider/physical acceptance remain pending.

Native phase verification: workspace/all-target Clippy passed with warnings denied.
The workspace suite excluding `homeverb-midea` passed 481 tests with one ignored;
the previously recorded macOS Meiju LAN alias limitation remains. This includes
126 Mijia unit tests and seven documentation tests, exact Host grant/inspection
and trusted approval fixtures. The built C ABI verifier passed 36 methods and
57 examples, including all nine outlet startup targets and presend refusal.
Contract validation passed 24 schemas, 68 positive and 881 negative fixtures.
Documentation/repository checks passed. No live account or appliance was used.


## App exact outlet controls -- October 29, 2026

The foreground App now negotiates API 1.34 before sending the new independent
`mijia_outlet_grants` field. All nine exact targets for power, power-loss recovery,
physical-control locking and indicator settings are retained. The shared purifier
lock identifier continues to use its separate low-risk reference-setting family.
Opening a device inspects protected options and refreshes only its account to
publish eligible actions; it does not prepare or execute a command.

Elevated outlet forms require explicit target and cloud-route selection, then
the existing immutable trusted review and actual confirmation. English/Chinese
copy describes connected-load start/stop, later power restoration and loss of
local-button operation. Component-scoped presentation labels distinguish switch
power without model/brand branching in the UI. Recovery codes remain numeric.
The indicator uses the ordinary low-risk flow. Existing navigation/background/
session replacement and unknown-receipt handling remain active. The API 1.33 APK
does not include this phase; new packaging and exact physical acceptance remain
pending. Modern SID authentication, event breadth and remaining feature gaps
are unchanged.

Executed App verification: the 12-file affected regression passed 119 tests.
After the final label and old-library negotiation additions, the three affected
files passed 25 tests (including 14 outlet cases); Flutter analysis found no
issues. Seven presentation tests passed. Documentation/repository hygiene and
diff checks passed. The complete Flutter suite was not claimed: previously
recorded macOS font/pixel/legacy-click failures remain separate. No real account,
message, appliance write or phone acceptance was performed.


## Exact outlet API 1.34 Android package -- October 30, 2026

Clean source `9b50a89f80282d5b0786fcd86ecabc9849116ebb` produced the combined-platform
release APK `apps/mobile/build/delivery/mijia/9b50a89/homeverb-mijia-0.2.0-9b50a89.apk`.
Size: 76,783,114 bytes. SHA-256:
`df26923b04abef92a1a368c9b36d4dd48ab0a96fb179eac3365038db55f2ae6c`. The adjacent `verification.json` records clean
source, native/AAR digests, signature, exact two-ABI composition and alignment.

The arm64-v8a and x86_64 native cross-builds passed with 27 exports per ABI
and 16 KiB ELF alignment. APK v2 development signing, ZIP/ELF alignment and
exact current-AAR matching after normal NDK stripping passed. API 1.34 includes
the four exact outlet controls and the App confirmation flow. The initial APK
attempt selected an unrelated Homebrew SDK and failed before assembly; the retry
used temporary explicit Flutter SDK/JDK configuration pointing to the existing
SDK/NDK. No SDK installation or license acceptance was performed.

No installation, provider request, real message or appliance write occurred.
Archive checks do not prove provider/phone/physical acceptance. Modern SID login,
event breadth, additional controls and other ledger gaps remain incomplete.
The latest fetched `origin/main` remains `ccf2975df836f8e81ccf24fa26c8b2035390c978`,
an ancestor of this worktree; there are zero new merge commits and no push.


## Native auxiliary settings -- October 30, 2026

API 1.35 implements [ADR-0092](../adr/0092-exact-auxiliary-reference-settings.md):
the exact `xiaomi.hood.cyp10` illumination switch and brightness enum 0/1/2,
and `jdjz.fan.2072fc` vendor quiet boolean. These seven targets use independent
`mijia_setting_grants`; existing fan and outlet admissions do not enable them.
Only exact protected source evidence permits the properties. No percentage
brightness conversion, companion power request, ventilation, timing or cleaning
authority is added. Each command sends one property and preserves accepted/
unknown outcomes without replay. Local native inspection works before grants.

Existing reference read-only derivations remain valid. Explicit selected-account
refresh upgrades the mapping; use the matching implementation with newer saved
derivations. There are still 36 JSON methods and 26 C exports plus the Android
initializer, with SQLite 5 and protected payload envelope 1 unchanged. Flutter
API 1.35 negotiation and presentation are pending. The delivered API 1.34 APK
does not gain these controls. No real provider/account/device acceptance is claimed.

Executed native auxiliary checks: the workspace suite excluding `homeverb-midea`
passed 483 tests with one ignored; the preexisting macOS Meiju LAN interface
limitation remains. After the final in-memory inspection upgrade, both selected
Mijia auxiliary tests and all 19 native Mijia account tests passed, and strict
workspace/all-target Clippy passed again. The full C ABI verifier passed 36
methods and 57 examples. Contract checks passed 24 schemas, 69 positive and
944 negative fixtures; documentation, repository hygiene and reference regeneration
passed. No provider/account/device traffic was used.


## App auxiliary settings -- October 30, 2026

The App now negotiates API 1.35 before sending the three additional reference
setting tuples from ADR-0092. Exact hood illumination power, brightness 0/1/2
and vendor fan quiet retain all seven targets. Brightness remains an enum rather
than a percentage, and zero is an explicit value rather than missing data.
Current session grants gate the generic component controls. Opening a device
performs no command preparation or control request. English/Chinese narrow
enlarged-text panels use existing localized property labels. Older API libraries
receive only a read-only version probe before refusing the new startup tuples.

No native exports, ordinary methods or storage formats change in this App phase.
New Android packaging and provider/physical acceptance remain pending; the API
1.34 APK is unchanged. Modern authentication, events and other ledger gaps remain
open.

Executed auxiliary App checks: 130 tests passed across the 13 affected control,
permission, approval, localization and history test files, including ten auxiliary
cases. Final Flutter analysis found no issues. Seven shared presentation tests,
documentation validation, repository hygiene and diff checks passed. This is a
scoped regression; the previously documented complete-suite macOS font/pixel/
legacy-click failures are not claimed resolved. No live account or appliance was
used.


## Auxiliary settings API 1.35 Android package -- October 30, 2026

Clean source `98704373d02b30144f884194b5be6faa7753b1ff` produced the combined-platform
release APK `apps/mobile/build/delivery/mijia/9870437/homeverb-mijia-0.2.0-9870437.apk`.
Size: 76,785,758 bytes. SHA-256:
`4c4ee7b02a644b08224cecfcd354e75d82f798237c3ec93d893b25b3e27ce8e7`.
The adjacent `verification.json` records clean source, native/AAR digests,
signature, exact two-ABI composition and alignment.

Both arm64-v8a and x86_64 native cross-builds passed with 27 exports and 16 KiB
ELF alignment per ABI. APK v2 development signing, ZIP/ELF alignment and exact
current-AAR matching after normal NDK stripping passed. Explicit temporary
Flutter SDK/JDK configuration selected the existing Android SDK/NDK. API 1.35
includes all seven auxiliary setting targets, old-API startup refusal and the
existing exact outlet confirmation flow.

No installation, provider request, real message or appliance write occurred.
Archive checks do not prove provider, phone or physical acceptance. Modern SID
login, events, additional controls and other reference gaps remain incomplete.
No push or publication was performed.


## Independent modern QR protocol -- October 30, 2026

[ADR-0093](../adr/0093-mijia-independent-qr-authentication.md) adds typed Rust
`begin_modern_qr_login`, single-use `ModernQrChallenge::finish`, and explicit
`ModernMijiaSession::renew`. The dedicated transport admits only exact reviewed
account, QR, c3 completion and modern STS endpoints. The challenge inherits its
original cancellation and deadline, bounded by 180 seconds; finish makes one
poll capped at 120 seconds and remaining lifetime, then one STS exchange.
No automatic QR regeneration, retry, redirect, background wait or device request
occurs. Protected scan data is intended for local trusted QR rendering only.

Renewal sends passToken only to the exact account service-login endpoint and
requires the same numeric subject, consistent encrypted identity, a returned
signing key and a service token. Optional STS identity cookies must agree too.
Failures preserve the old in-memory session; no assumed 30-day expiry is stored.
Modern sessions cannot be converted into legacy `MijiaSession` values. Native
API 1.35, C/JSON methods, SQLite 5 and protected account payload 1 remain unchanged.

The source review found concrete SID=mijia QR and renewal implementations. Two
anonymous GETs checked bootstrap and QR URL shapes without credentials, scanning,
completion polling, STS exchange or account/device access. These were not account
acceptance tests. Source attribution and exact observed endpoints are in ADR-0093.
Community per-field RC4 at `api.mijia.tech/app` and the archive's continuous RC4
at `core.api.mijia.tech/app` remain separate, unadmitted modern data routes.

Protected persistence, account adoption/identity reconciliation, native challenge
registry, App QR rendering/foreground integration, authenticated login/renewal
acceptance and modern device reads remain pending. The delivered `9870437` APK
contains the auxiliary-setting phase, not these new Rust protocol entry points.

Executed verification: the complete Mijia suite passed 140 unit and nine
compile-fail documentation tests. The workspace suite excluding `homeverb-midea`
passed 499 tests with one ignored; its previously recorded macOS LAN-alias
failures remain excluded. After the final client-format and duplicate-identity
cookie adjustment, all 13 affected modern protocol/TLS tests passed again.
Strict workspace/all-target Clippy and formatting passed. Documentation,
repository hygiene, reference regeneration and all 24 schemas, 69 positive and
944 negative contract fixtures passed. No Flutter flow changed or was retested
for this Rust-only phase. No authenticated provider or physical-device test ran.


## Modern reference-session reads -- October 30, 2026

[ADR-0094](../adr/0094-mijia-reference-session-reads.md) implements the three
reference read routes through the independent typed Rust modern session:
`directory`, `homes` and exact pinned `properties`. These requests use only
`core.api.mijia.tech/app`, continuous RC4 across ordered fields, reference headers
and bounded encrypted gzip decoding. They never send passToken to a data origin,
convert a legacy session, select the community alternate codec/origin, or retry
with new credentials after failure.

Directory traversal retains logical split entries with private IDs, up to 16
pages and 4,096 entries. It returns positive partial results for later ordinary
transport/provider failures, while malformed/ambiguous pages, changed identity,
authentication failure, cancellation and deadline abort. Entries bind to their
protected subject and encrypted identity; another account cannot query them.
They are not published as physical Devices or merged into existing bindings.
Home IDs and owner IDs stay opaque, and neither list exhaustion nor empty results
establish complete physical membership or permission to remove historical cards.

Scalar reads accept one to 90 distinct properties from the entry's pinned source.
All 264 existing scalar mappings are admitted across the reference catalog; the
`njcuk.fitting.ad1204` specification has no readable property and remains an opaque
entry. Unknown models/specs similarly gain no property request authority. Failed,
missing and null readings stay absent; explicit false and zero remain values.
No observation timestamp, live state or physical causality is invented.

The request encoder has an independently calculated Python known-answer vector
that differs from the legacy per-field reset. Raw and inflated responses are each
bounded at 8 MiB; corrupt, truncated, concatenated/trailing gzip, ambiguous encoding,
plaintext success, redirects and untrusted TLS fail closed. The five added Rust
compression packages are locked, use the pure Rust backend, and retain their
[license record](../../third-party/mijia-compression/NOTICE.md).

The complete Mijia suite passed 152 unit and ten documentation tests. Workspace
regression excluding `homeverb-midea` passed 512 tests with one ignored; the known
macOS LAN-alias failures remain excluded. The final cancellation checkpoint and
new synchronous-reply cancellation case passed in all 26 modern protocol/TLS
tests; strict workspace/all-target Clippy passed after that adjustment.

Protected modern account persistence, Core/Host adoption and logical-parent
reconciliation, native C/App entry points, modern controls and authenticated
provider/physical acceptance remain pending. Native API 1.35, its C/JSON method
sets, SQLite 5 and Mijia payload 1 are unchanged. No Flutter flow changed. The
`9870437` APK still contains the earlier auxiliary-setting phase only. This phase
performs no account login, device read/write, message, push or publication.

The latest fetched `origin/main` is `ccf2975df836f8e81ccf24fa26c8b2035390c978`,
already an ancestor of the Mijia branch. No merge commit was introduced.

Additional executed checks: the rebuilt C ABI passed all 36 methods and 57 live
fictional examples. Native API/exports remain unchanged. The installed Rust 1.99.1
stable toolchain passed `cargo check -p homeverb-mijia --target` for both
`aarch64-linux-android` (existing NDK 28.2.13676358) and `aarch64-apple-ios`.
These are target checks, not framework/APK packaging, signed-device execution or
physical acceptance. Rust 1.96.0 supplied the host regression and strict Clippy
checks above. No toolchain, SDK or dependency installation was needed; new locked
compression dependencies were already cached locally.


## Protected modern account sessions -- October 30, 2026

[ADR-0095](../adr/0095-mijia-protected-modern-account-sessions.md) adds explicit
Rust adapter composition for modern QR connection, saved directory/property
reads, renewal and uncertain-save recovery. A version-2 secret-store payload
retains independent legacy and modern slots for one exact numeric subject.
Either login can adopt the matching local account without replacing its other
slot, legacy bindings or label. Ambiguous identities fail closed. Modern logical
entries are account-bound and do not publish physical Device cards.

Legacy-only version-1 bytes stay unchanged. Version 2 requires the modern slot
and a fixed reference profile; older implementations cannot read it and must not
downgrade it. Native API 1.35, C/JSON methods, SQLite 5 and Meiju payload 3 stay
unchanged. Existing legacy routes require their own slot and never fall back to
modern credentials. Default adapter composition does not enable modern requests.

QR completion checks its protected account snapshot before and after provider
exchange; cancellation and the original deadline cover persistence. Explicit
renewal preserves legacy evidence and does not enumerate. Uncertain saves remain
pending until explicit modern-directory validation confirms the exact saved bytes,
without another login, renewal or credential rewrite. No background task is added.

Host lifecycle/registry, C/App QR integration, physical/logical reconciliation,
modern controls and authenticated provider/device acceptance remain pending.
The delivered `9870437` APK contains the earlier auxiliary-setting phase only.
No real account, appliance, message, push or publication was used.

Executed phase checks: the Rust workspace excluding `homeverb-midea` passed 531
tests with one ignored, including 169 Mijia unit tests and 12 documentation tests.
The excluded package retains its 14 previously documented macOS LAN-alias failures;
this is not a full workspace pass. After adding the final secret-write cancellation
case and missing-legacy-slot assertion, all 17 modern account tests passed and
strict workspace/all-target Clippy passed again. No production Rust code changed
after the workspace run. The rebuilt native C ABI passed 36 methods and 57
fictional examples. Contract validation passed 24 schemas, 69 positive fixtures
and 944 negative fixtures. Both reference-projection Python tests, regeneration,
formatting, documentation and repository hygiene checks passed. No Flutter flow,
mobile package, authenticated provider or physical device was tested in this phase.


## Modern Host account lifecycle -- October 30, 2026

[ADR-0096](../adr/0096-mijia-modern-host-lifecycle.md) adds typed foreground Session
methods for protected QR begin/continue/discard and explicit modern account
directory refresh, recovery and renewal. QR handles belong to one Session, are
consumed before continuation admission and share the four-entry pending limit
with captcha and verification. Background, cancel and close clear pending sign-in;
registered work uses existing cancellation/deadlines and late results cannot cross
a lifecycle generation. No automatic polling, retry or background work is added.

Mijia-only and combined native Session composition now registers the reviewed
modern HTTPS transports without contacting them at startup. Core-only CLI/MCP
composition is unchanged. Modern logical directory entries remain opaque typed
results; they do not publish Device cards or grant routes/control authority.
Successful same-subject connection preserves existing legacy cards. Public account
summaries exclude QR data, provider identity and credential references.

This Host-only phase retains API 1.35, ordinary methods/C exports, SQLite 5 and
protected payload versions. Dedicated C buffers/API negotiation, Flutter QR
rendering, logical-parent reconciliation and authenticated provider/physical
acceptance remain pending. The `9870437` APK is unchanged. No live account,
message, device command, push or publication was used.

Executed Host phase checks: the Rust workspace excluding `homeverb-midea` passed
545 tests with one ignored, including the original 12 modern Host cases, the
existing 19 Mijia Host cases, 170 Mijia unit tests and protected-type documentation
tests. The known 14 macOS Meiju LAN-alias failures remain excluded. Two additional
competition/lifecycle cases then passed in all 14 modern Host tests; strict
workspace/all-target Clippy passed after those tests. Production Rust code did
not change after the workspace run. The rebuilt C ABI passed all 36 methods
and 57 fictional examples. Contract validation passed 24 schemas, 69 positive
and 944 negative fixtures. Both reference-projection tests, regeneration,
formatting, documentation and repository hygiene passed. No Flutter, new APK,
real-provider or physical-device acceptance was performed.


## Native modern account bridge (API 1.36) -- October 30, 2026

[ADR-0097](../adr/0097-native-modern-mijia-account-bridge.md) adds four dedicated C
functions for modern QR begin/continue/discard and explicit saved-account refresh,
recovery or renewal. The trusted UI must negotiate API 1.36 before resolving
symbols. QR scan bytes use a caller-owned protected buffer of exactly 8192 bytes;
JSON contains only local challenge metadata, length and remaining lifetime.
Valid buffers clear before admission; invalid lengths are not dereferenced.
Empty/oversized projection discards the pending handle without truncation.

Connected/directory reports expose the public account summary and bounded logical
entry count, with partial/failed status and `complete=false`. They expose no private
membership, credential references, entry names, device IDs or physical cards.
Renewal returns only an account summary and performs no inventory read. Existing
Host cancellation, one-use handles and pending-save recovery remain in force.
No automatic poll, reauthentication, background networking or write authority is
added. Ordinary JSON/CLI/MCP cannot call these functions.

Native API 1.36 retains 36 ordinary methods and adds four C exports for a total
of 30 (31 with the Android initializer). SQLite 5, Mijia payloads 1/2 and Meiju
payload 3 remain unchanged. Older native libraries lack the new symbols; existing
App flows retain their prior behavior. Flutter QR rendering, logical-parent/Core
adoption, mobile packaging and authenticated provider/physical acceptance remain
pending. The delivered `9870437` APK is still API 1.35. No provider login, message,
appliance operation, push or publication was performed.

Executed API 1.36 checks: the workspace suite excluding `homeverb-midea` passed
550 tests with one ignored, including real fictional Host report/schema checks
and protected QR projection tests. The known 14 macOS Meiju LAN-alias failures
remain excluded. The complete rebuilt C ABI passed 36 ordinary methods and 57
fictional examples, including the new four-export verifier's 62 refusal/buffer/
lifecycle cases. Exact host exports match all 30 header declarations; Android's
expected policy contains those 30 plus its initializer. Nine Android and eight
iOS package-unit tests passed using fictional reports, not new mobile archives.

The final sub-millisecond/invalid QR-lifetime refusal passed in all six FFI unit
tests, followed by strict workspace/all-target Clippy. The final rebuilt library
then passed all 62 focused modern C verifier cases. Contract validation passed
25 schemas, 69 primary positive fixtures plus the dedicated modern examples and
1,092 negative fixtures. Both reference-projection tests, regeneration, formatting,
documentation and repository hygiene passed. No Flutter build, new mobile package,
authenticated login, physical-device operation or publication was performed.


## Flutter modern-account client -- October 30, 2026

The typed Flutter client now consumes all four API 1.36 functions under
[ADR-0098](../adr/0098-foreground-mijia-qr-app.md). Version/provider negotiation
precedes lazy symbol lookup; API 1.35 libraries without the new symbols refuse
before lookup. Worker isolates carry only pointer addresses and nonsecret metadata.
Native scan buffers are cleared/freed, owned scan arrays are cleared on consumption,
attempted expired continuation, discard and lifecycle changes; foreign clients
cannot consume them. The pending object itself schedules no timer.

Closed parsers validate modern QR metadata and account/directory outcomes, including
exact selected-account identity, provider, conservative completeness and failed
retrieval. Explicit refresh/recover/renewal remain separate calls; no retry, automatic
poll or cross-protocol fallback is added. Pending challenges share existing App
cancel/background/close handling, and late results cannot restore cancelled state.

Ten fictional C-ABI client tests passed as part of a 67-test affected Flutter
account/native regression. The fixture performs no network, platform-secret or
appliance access. Native API/storage formats are unchanged. Controller/account-page
wiring, local QR rendering, modern account actions, Android packaging and real
account/phone acceptance remain pending. The existing API 1.35 APK is unchanged.


## Foreground Flutter QR account flow -- October 30, 2026

The API 1.36 client is now connected to the sign-in and saved-account pages under
ADR-0098. The explicit QR choice rechecks credential-storage readiness before
requesting a code. The App encodes scan bytes locally with pinned `qr` 3.0.2,
clears the source array after encoding, and owns a clearable module bitmap with
a four-module white quiet zone. Dismissal, expiry, navigation and background clear
owned presentation material. Encoder intermediates have managed lifetimes; no
whole-heap erasure guarantee is made. No URL string, remote image, browser,
clipboard, polling, regeneration or automatic renewal is introduced.

Instructions require Xiaomi Home on another device while HomeVerb stays open,
then one explicit Continue. Controller epochs, foreground state, client ownership
and selected account guard late results. Interrupted sign-in requires a local
account reload because completed secret storage cannot be rolled back. Existing
password/captcha/verification entry points and physical cards remain available.

Saved QR results are separate from legacy inventory: zero entries are displayed
as zero, failed directory reads retain their error, and no logical entry becomes
a physical card or complete-inventory claim. QR directory refresh, saved-session
recovery and renewal each have their own explicit account-page action. Public
summaries do not expose which protected protocol slots exist; the UI explains that
these operations require saved QR authorization, and the native layer refuses a
missing modern slot without fallback.

Executed evidence: 102 affected Flutter account/native tests passed, including
26 new QR cases; final account-page click coverage passed in all 26 QR tests.
After the final QR-specific waiting message, all 35 QR/account-page tests and
Flutter analysis passed. Seven presentation tests, two reference-projection tests,
documentation, repository hygiene and contract validation (25 schemas, 69 primary
positives and 1,092 negatives) passed. A fictional 280-point rendered QR image decoded to the
expected fictional payload using the independent macOS Core Image decoder.
English/Chinese 320-point layouts with doubled text passed. This proves local
software behavior, not provider authentication, phone security or physical-device
support. Native API 1.36, SQLite 5, protected payload versions and grants are
unchanged. Android packaging, real account/phone acceptance, physical/logical
reconciliation and the remaining reference ledger remain open. No provider call,
appliance write, package installation, push or publication was performed.

## Modern directory device groups -- October 30, 2026

[ADR-0099](../adr/0099-mijia-directory-device-groups.md) implements typed, protected directory grouping as the prerequisite for
modern Core adoption. Canonical numeric/text `pid` and optional cloud `parent_id`
remain inside opaque entries. Kinds 0/6/16/18 are separate candidates; kind 21
joins only its explicit ordinary parent with compatible model/spec evidence.
Kind-18 gateway children stay separate, and names or ID suffixes never establish
identity. Unknown kinds, missing parents and nested logical parents are retained
unresolved. Duplicate/foreign identities, cycles and contradictory split metadata
fail before any grouping result. Linear bounded traversal observes cancellation
and preserves each input exactly once, without property-map fusion or networking.

Fourteen new topology cases passed in the final 184 Mijia unit tests, plus 14
compile checks including serialization refusal. Strict Mijia all-target Clippy
passed. The preceding combined Mijia/Host run passed 498 tests before the final
four cases and equivalent traversal cleanup. No native API, protected payload,
SQLite, App report or control authority changes. Existing modern directory reads
now reject malformed kind/parent metadata instead of discarding those fields.
Missing metadata remains readable but cannot establish a physical candidate.

Protected route provenance, stable legacy/modern binding reconciliation, Core
publication/reads and App card adoption remain pending. Typed groups are not
physical device counts, complete membership or evidence that appliances are online.
No provider/account operation, appliance write, mobile package or remote publication
was performed. The reference ledger continues to report partial implementation.

## Modern Core binding checkpoint -- October 30, 2026

[ADR-0100](../adr/0100-mijia-modern-protected-core-reads.md) and the
[Mijia implementation record](11-mijia.md) now cover typed protected device
adoption, payload 3 and Core reads for all 264 pinned scalar mappings with fictional
evidence. Host/App card publication is still pending; the coverage ledger keeps
that distinction and retains the remaining control/event/BLE/LAN/media work.

## Modern daily-energy history -- October 30, 2026

[ADR-0103](../adr/0103-modern-mijia-daily-energy-history.md) admits the exact
cuco.plug.v3 daily-energy profile for modern-selected bindings. The reviewed MIT
source has a separate fixed api.mijia.tech statistics route with per-field RC4,
modern session cookies and plaintext/encrypted JSON or encrypted gzip replies.
The shared exact-model decoder retains ADR-0088 units, limits and incomplete
record semantics. No ID suffix is stripped and no credential/codec fallback is
available. Ordinary native/App history queries reuse existing schemas and UI;
modern controls and broader events/history remain unavailable. The previous APK
predates this implementation. See the ADR for verification and acceptance scope.

## Modern exact scalar controls -- October 30, 2026

ADR-0104 implements the existing 29 reviewed scalar controls for modern-selected
bindings: jdjz.fan.2072fc (9), abhome.light.wy0a01 (7), zhimi.airpurifier.v6 (7),
cuco.plug.v3 (4) and xiaomi.hood.cyp10 (2). A separate fixed modern transport uses
the reference continuous codec at core.api.mijia.tech/app/miotspec/prop/set.
Protected binding selection determines credentials and protocol before I/O.
Legacy bindings keep legacy controls; failures never switch protocols or renew.

Production native hosts register the transport without granting commands. Existing
option inspection, exact startup grants and explicit retrieval expose the admitted
actions through the generic App controls. Default native sessions stay read-only.
Elevated outlet settings independently require Core's trusted immutable approval,
including when public risk metadata is tampered with. One correlated encrypted
acknowledgement can establish accepted or rejected, never physical verification.
Possibly-sent failures retain unknown receipts without replay or companion writes.

Executed evidence: 618 workspace tests passed with one ignored, excluding the
known unrelated Meiju/macOS LAN-alias suite. Strict workspace/all-target Clippy
and the native FFI build passed. All 125 affected Flutter control, permission,
approval, history and modern-device-flow tests passed. The rebuilt native ABI
passed 36 ordinary methods/57 examples and 76 modern-account boundary cases. Tests use fictional material and local TLS only; no provider account or
appliance is accessed. Native API 1.37, ordinary schemas/exports, SQLite 5 and
Mijia payload 3 remain unchanged. Old native libraries expose no modern controls.
The existing dd51c73 APK predates both modern history and these controls.

Broader controls/actions/events, BLE/LAN, health/media, additional model/runtime
contracts, updated packaging and real account/phone/device acceptance remain open.
No remote push, publication, package installation or real-device write occurred.

## Typed numeric property history -- October 30, 2026

ADR-0105 adds experimental typed Rust property-log requests for the 161 numeric
fields in the existing pinned scalar mappings. Legacy and modern sessions use
their own fixed /user/get_user_device_data endpoint and per-field codec. Keys
come from exact SIID.PIID, type is prop, and the protected root identifier is
preserved. Boolean, private/opaque and unmapped fields are not admitted.

One explicit query spans at most 32 days and requests at most 31 records. The
parser retains signed values, original reported units (or no unit), raw numeric
enums and provider timestamps; it does not apply daily-energy scaling. It derives
exact numeric domains from pinned specification evidence without changing stored
read mappings. Foreign supplied correlations fail the response. Invalid/out-of-range
values and every ambiguous duplicate timestamp count as rejected records. No raw
actor identity, unknown metadata, cache write, receipt, credential mutation,
automatic paging, fallback or renewal is added. Completeness remains unproven.

Protocol evidence: 629 workspace tests passed with one ignored, excluding the
known unrelated Meiju/macOS LAN-alias suite; a final legacy cancel/drop test also
passed. Strict Clippy, formatting, FFI, native ABI (36 methods/57 examples and 76
modern-account cases), documentation, schema and projection checks passed. The
preceding Mijia run passed 227 unit and 14 compile-fail tests. See ADR-0105 for
exact scope; no Flutter/mobile/provider acceptance is claimed by this phase.

This is a protocol-library phase. Native API 1.37, public history schemas, Core
options, CLI/MCP and App still expose only the admitted daily-energy series.
Reported-sample contracts, protected Core admission, versioned native/App adoption
and provider acceptance remain pending. The verified 9a6c004 APK is unchanged.
No provider account, appliance, package installation, push or publication is used.

## Reported samples through Core, native and App -- October 30, 2026

ADR-0105 now exposes all 161 pinned numeric history fields through Core on legacy
and modern protected bindings. Exact `(target, aggregation)` options distinguish
reported samples from daily statistics. Samples preserve signed values, source
numeric domains and exact nullable units; daily results remain nonnegative with
string units. Inspection performs no networking. Reads retain account/device
leases and do not change secrets, cache, receipts, descriptors or write authority.

Native API 1.38 adds optional include_reported_samples=false to existing inspection;
older clients remain daily-only. Updated Flutter negotiates 1.38, opts in explicitly,
labels the two types, validates immutable domains and discards results on lifecycle
changes. CLI/MCP reuse the same Core contract. SQLite 5, protected payloads and C
export/method counts are unchanged. Rust enum/nullable-unit changes require source
updates for unpublished consumers; older native libraries refuse new requests.

Executed native checks: workspace tests excluding the known unrelated Meiju/macOS
LAN-alias failures passed 637 tests with one ignored; the final Host history suite
passed 14 tests after an added integer/enum case. Strict workspace Clippy, FFI build
and actual native ABI checks passed (36 methods, 60 examples, 76 modern-account
cases). The affected Flutter history/card suite passed 85 tests and analysis found
no issues. All 13 CLI/MCP tests and 16 Dart FFI tests also passed. Documentation,
source hygiene, 25 schemas with 72 positive/1230 negative fixtures, both reference
projection tests and seven shared presentation tests passed. These use fictional
material and establish no provider/phone/model acceptance. The 9a6c004 APK remains API 1.37; API 1.38 packaging is separate work.

## Exact hood manual controls -- October 30, 2026

ADR-0106 adds four exact xiaomi.hood.cyp10 properties through legacy and modern
Core paths: hood on-state, fan levels 1/2/3/4, display content codes 0/1 and the
independent automatic screen-off Boolean. The 33 reviewed scalar controls still
cover five exact models. Existing vendor IDs and raw codes remain stable; no
percentage conversion, companion power-on, inferred timer or schedule is added.
Core and the protected adapter independently require exact reference-setting
grants; fan/outlet authority does not transfer. Uncertain writes are never replayed.

Native API 1.39 adds four closed startup tuples and three inspection component
scopes. Flutter probes the native version before any granted reopen and preserves
all ten target values. Old illumination-only hood mappings remain valid; explicit
refresh upgrades their domains while keeping identity and organization. Invalid
partial domain mixtures refuse. Methods, C exports, SQLite 5 and protected payload
3 are unchanged. Older native libraries refuse new grants/derivations.

Scoped evidence: 233 Mijia unit and 14 compile-fail tests passed, including all
33 modern controls and every new legacy target. Host account/grant suites passed
53 and 3 tests. The hood/auxiliary/settings/permissions Flutter suite passed 50
tests, the related control/approval/device/FFI suite passed 61 tests, and analysis
reported no issues. All material is fictional. The API 1.38 APK from 2e776a9
predates the four new controls. Final workspace/native checks and API 1.39
packaging are recorded separately; provider/phone/model acceptance remains pending.

Final software verification: the workspace excluding the known unrelated
Meiju/macOS LAN-alias suite passed 642 tests with one ignored. Strict all-target
Clippy and formatting passed. Rebuilt FFI and actual C boundary checks passed
36 methods/60 live examples, including new hood startup acceptance/refusal and
76 modern-account cases. The checked-in examples were regenerated from that run.
Documentation, source hygiene, 25 schemas with 73 positive/1320 negative fixtures,
two coverage-projection tests and seven presentation tests passed. The existing
eight full Flutter font/pixel/legacy-click failures were not rerun or claimed fixed.
No real account, appliance, installation, remote push or publication was used.

## Typed no-argument actions -- October 30, 2026

[ADR-0107](../adr/0107-bounded-no-argument-actions.md) adds six exact typed Rust
MIoT action primitives on legacy and modern sessions: four light actions, one
fan toggle and one hood illumination toggle. Each requires its pinned model,
complete specification and exact SIID/AIID; all inputs and outputs are empty.
The separate action catalog leaves saved property mappings unchanged. These are
non-idempotent operations and are not yet exposed through Core/native/App grants.

Each attempt uses one tracked control send on the selected protocol. A correlated
code-zero result is accepted, never physically verified. Only reviewed
pre-execution argument/identity/missing-action codes prove rejection. Pending,
execution-error, timeout, malformed, foreign and late replies remain uncertain;
there is no retry, renewal or alternate-protocol fallback. Logical children and
foreign modern subjects refuse before I/O. Cancellation and drop release work.

Native API 1.39, C exports/methods, ordinary schemas, SQLite 5 and protected
payload 3 are unchanged. Rust exhaustive CloudEndpoint consumers must handle
Action; ModernControlEndpoint supplies two closed routes. Independent Core action
bindings/grants, persistent non-idempotent receipts, versioned App integration,
remaining actions/events and real provider/phone/device acceptance remain open.
The 503ef1f APK predates these library primitives. No account, appliance,
installation, push or publication is used.

Executed software checks: the workspace excluding the known unrelated Meiju/macOS
LAN-alias suite passed 653 tests with one ignored, including 244 Mijia unit tests
and 14 compile-fail tests. Strict all-target workspace Clippy and formatting
passed. The three projected files, both projection tests, documentation, source
hygiene and 25 schemas with 73 positive/1320 negative fixtures passed. These tests
use fictional data and local TLS only. No Flutter test or new Android package is
claimed for this protocol-only phase; the existing eight full Flutter visual/
legacy-click failures remain outside this change.

Rebuilt native FFI and the actual C ABI check also passed: 36 ordinary methods,
60 live examples and 76 modern-account boundary cases. Existing trusted bridges
retain their guards; no ordinary action grant or method was added.

## Protected Core no-argument actions -- October 30, 2026

[ADR-0108](../adr/0108-mijia-protected-no-argument-actions.md) connects the six exact
ADR-0107 actions to typed Core on legacy and modern protected bindings. A separate
adapter action policy and an independent Core grant are both required; existing
fan/reference/outlet property authority cannot enable them. Local action-option
inspection performs no networking, credential rewrite or permission change.
Explicit opted-in inventory publication adds non-idempotent, empty-argument
vendor actions to existing component/capability scopes without changing properties.

Protected source model/specification/service/action identities and the selected
protocol are rechecked before one send. Public metadata cannot expand arguments
or choose another action. A parent-bearing modern entry is not admitted as an
action root. Missing membership, session, transport or exact policy refuses before
I/O. Accepted is never verified; ambiguous execution replies remain unknown.

Core's existing SQLite journal preserves receipts across reopen and explicit
protocol adoption. Same-ID retries never send again; same-ID command conflicts
refuse, and history retirement preserves replay protection. Revoked Core grants refuse before
journal lookup while retaining the original stored receipt. Cancellation/deadline/
drop release account/device locks and keep unknown outcomes without replay.

This is typed Rust Core integration. Native API 1.39, ordinary schemas, C exports/
methods, SQLite 5 and protected payload 3 remain unchanged. Production native,
CLI and MCP composition does not opt into these actions. Versioned native startup
grants/inspection and Flutter controls are still pending, as are remaining actions,
events and real provider/phone/model acceptance. The 503ef1f APK is unchanged.
No provider, appliance, installation, push or publication is used.

Executed software verification: workspace tests excluding the known unrelated
Meiju/macOS LAN-alias suite passed 661 tests with one ignored, including 252 Mijia
unit tests and 14 compile-fail tests. Strict all-target Clippy passed after an
equivalent Boolean simplification, and the final eight action integration tests
passed again. Formatting, all three generated projections, both projection tests,
documentation, source hygiene and 25 schemas with 75 positive/1328 negative
fixtures passed. No Flutter run, new APK or real-account/device acceptance is
claimed by this typed Core phase; known full Flutter failures remain unchanged.

The rebuilt FFI and actual native C boundary also passed: 36 ordinary methods,
60 live examples and 76 modern-account cases, plus existing protected/trusted
bridge guards. No new native method, startup field or Flutter action is exposed
by this phase.


## Native exact no-argument actions -- October 30, 2026

[ADR-0109](../adr/0109-native-mijia-action-grants.md) exposes the six reviewed
ADR-0108 actions through API 1.40. Independent trusted `mijia_action_grants`
contain only canonical local device IDs and exact component/capability/action
tuples. Invalid grants refuse before profile creation. Both Core and the adapter
receive the exact empty-argument policy; existing property grants imply no action
authority. Default composition remains action-free.

`inspect_control_options` retains its existing result unless the caller explicitly
sets `include_no_argument_actions:true`. Inspection performs no networking or
permission changes and returns saved-inventory evidence with authorization not
checked. Explicit granted reopen and account refresh publish actions on the
protected selected protocol. Non-idempotent receipts preserve uncertain outcomes
without retry, including after cancellation/background and session reopen.

Negotiate 1.40 before supplying the new field or inspection flag. Existing Rust
opening helpers remain compatible; typed Request constructors need the new field.
SQLite 5, protected payload 3, 36 ordinary methods and C exports are unchanged.
Flutter adoption, an updated APK, the remaining actions/events and real account/
phone/model acceptance remain pending. The 503ef1f APK remains API 1.39. No real
account, appliance, installation, remote push or publication is used.

Executed software checks: workspace tests excluding the known unrelated
Meiju/macOS LAN-alias suite passed 666 tests with one ignored. Strict all-target
Clippy, FFI rebuild and the actual C boundary passed: 36 ordinary methods,
68 live examples, 76 modern-account cases and existing protected/trusted bridge
guards. The 58 scoped Host/account tests cover action admission, opt-in inspection,
explicit refresh/execute and uncertain lifecycle recovery without replay.
Documentation, source hygiene, both reference projection tests and 25 schemas
with 76 positive/1451 negative fixtures passed. All data is fictional; no Flutter
run, new package or provider/device acceptance is claimed by this native phase.


## Foreground App no-argument actions -- October 30, 2026

Flutter now negotiates API 1.40 and explicitly requests the six ADR-0109 actions
when preparing a selected device. Exact empty-object definitions become separate
`mijia_action_grants`; property permissions never imply action authority. Trusted
startup probes the version before reopening with grants. Older native libraries
retain property-only inspection and refuse explicitly requested action grants.

The existing descriptor-driven detail buttons and forms display localized action
labels and non-idempotent warnings. Opening/selecting an action sends nothing;
Apply plans and executes once with empty arguments and the exact component.
Pending submission disables duplicate clicks. Unknown receipts remain unknown
without automatic retries or optimistic state changes. Backgrounded inspection
cannot reopen or refresh a device. No new method, storage format or UI protocol
branch is added. Remaining actions/events and provider/phone/model acceptance
are incomplete; the existing 503ef1f APK predates this App integration.

Executed App verification: the 14-file affected Flutter suite passed all 133 tests,
including 11 action cases, and Flutter analysis found no issues. Actual old-ABI
refusal and current Mijia/combined opening use fictional temporary profiles.
Both reference projection tests and all seven shared presentation tests passed.
The known eight full-suite macOS font/pixel/legacy-click failures are not claimed
fixed. No provider account, appliance write, installation or publication occurred.


## No-argument actions API 1.40 Android package -- October 30, 2026

Clean source `ab34f2199305ee991a8eb28055674e8acb57bf1e` produced the combined-platform
release APK `apps/mobile/build/delivery/mijia/ab34f21/homeverb-mijia-0.2.0-ab34f21.apk`.
Size: 79,113,802 bytes. SHA-256:
`d699118223e38341a96a6b9385344188865d6ed58968a05f20340d69a6761040`.
The adjacent `verification.json` records clean source, AAR/native digests,
exact arm64-v8a/x86_64 composition, development signing and 16 KiB alignment.

Both native cross-builds passed with 31 exports per ABI using the installed
stable Rust 1.99.1 Android targets, NDK 28.2.13676358 and existing JDK. An initial
Rust 1.96 attempt failed because that toolchain has no Android targets; no target
or SDK was installed. Native tests use Rust 1.96 and do not imply an Android
runtime test. Flutter 3.47.7 assembled the release APK with temporary SDK/JDK
configuration; the original local properties were restored. Signature, ZIP/ELF
alignment and exact current-AAR code after normal NDK stripping passed.

This APK contains all six independent exact no-argument actions through API 1.40
and Flutter alongside the existing 33 scalar controls and bounded history views.
No installation, real account, appliance request, remote push or publication was
performed. Archive evidence is not provider, phone or model acceptance. The
remaining 34 actions, event support and other reference gaps stay incomplete.


## Typed elevated power-toggle actions -- October 30, 2026

[ADR-0110](../adr/0110-elevated-mijia-power-toggle-actions.md) adds five exact
empty-input switch actions across cuco.plug.v3, yutai.plug.jdls1 and the two pinned
opple wall-switch models. Each protected model/service/action uses the existing
selected legacy/modern protocol and sends once without companion settings.
Eleven exact actions now have protocol/Core support; six are already in native/App.

Power toggles are elevated and non-idempotent. Independent action-family and Core
grants plus actual trusted approval are required. The adapter re-derives risk and
checks Core's private marker even when public metadata is forged low. Unknown
outcomes survive cancellation, deadline, drop and reopen without another approval
or replay. Protected logical children remain refused; wall-switch mode is not
changed and physical load behavior is not inferred from acceptance.

This phase adds typed Rust support only. API 1.40 startup and opted-in inspection
remain limited to the earlier six low-risk actions, with explicit native rejection
of the four new component tuples. C exports/methods, SQLite 5 and protected payload
3 are unchanged. Versioned native/App adoption, the remaining 29 actions, events
and real provider/phone/model acceptance remain open. The ab34f21 APK is unchanged.
No real account, appliance request, installation, push or publication occurred.

Executed verification: the Rust 1.96 offline/locked workspace run passed 672 tests
with one ignored, excluding homeverb-midea for its known macOS LAN-alias
limitation. After adding two final approval-boundary cases, the seven-case power
toggle suite passed separately; no later full-workspace count is claimed. Strict
all-target workspace Clippy, rustfmt and diff checks passed. The rebuilt C ABI
verifier passed 36 methods, 68 live examples and 76 modern-account boundary cases,
including refusal of all new power-toggle startup tuples. Contract validation
passed 25 schemas, 78 positive and 1,463 negative fixtures. Reference projection
and its two unit tests passed; documentation and repository checks passed.
No Flutter tests or new APK were produced in this typed-only phase.

## Native elevated power toggles -- October 30, 2026

[ADR-0111](../adr/0111-native-mijia-power-toggle-actions.md) extends API 1.41's independent mijia_action_grants to the
four exact switch component tuples covering all five ADR-0110 actions. The
separate include_power_toggle_actions flag defaults to false; API 1.40-style
inspection still returns only the original six low-risk actions when opted in.
Inspection remains local and does not grant consent or publish capabilities.

Granted reopen and explicit refresh expose elevated descriptors. Every attempt
requires immutable trusted preview and actual user confirmation through the
existing C approval bridge. Ordinary execution stays denied. Cancellation,
backgrounding, unknown outcomes and reopened receipts never enable replay.
No new C export, method, storage format, companion write or verification is added.
Flutter adoption, a matching APK and provider/phone/model acceptance remain
pending; ab34f21 is still API 1.40. No real account/device operation, installation,
push or publication occurred in this phase.

Executed verification: Rust 1.96 offline/locked workspace tests passed 678 tests
with one ignored, excluding homeverb-midea for its known macOS LAN-alias
limitation. Strict all-target Clippy, rustfmt and FFI rebuild passed. The actual
C ABI verifier passed 36 methods, 76 live examples, 76 modern-account boundary
cases and the existing trusted/protected bridges. The 63 scoped Host/account
tests passed, including all five power-toggle model/service journeys and unknown
reopen without replay. Contract checks passed 25 schemas, 79 positive and 1,547
negative fixtures; reference projection and both projection unit tests passed.
Documentation, repository hygiene, diff and commit checks passed. These are
fictional software checks, not Flutter, package, provider or physical acceptance.

## Foreground App power toggles -- October 30, 2026

Flutter now negotiates API 1.41 before requesting exact elevated power actions or
reopening with their grants. Both the independent power-action inspection opt-in
and a trusted approval bridge are required. Older versions and hosts without that
bridge keep their existing inspection and acquire no power-toggle authority.
The exact switch-2/switch-3/switch-11/switch-12 tuples remain separate from property
and low-risk action permissions; protected native evidence still selects the five
model/service profiles.

Device opening performs no command preparation or send. The existing generic
form requires an explicit cloud route, then displays the immutable trusted preview.
Only a separate confirmation executes once; repeated clicks, cancellation,
backgrounding and unknown results never replay the command. No next-state guess,
optimistic power update or companion setter is added. English/Chinese forms reuse
existing labels and confirmation presentation without model-specific UI branches.
All eleven admitted actions now have Core/native/App wiring. A matching Android
package and real provider/phone/model acceptance remain pending; the ab34f21 APK
still contains API 1.40 and six low-risk actions. The remaining 29 actions, events
and specialized reference paths are incomplete.

Executed App verification: the 16-file affected Flutter regression suite passed
140 tests, including 14 new power-action cases. After adding the specific bilingual
confirmation copy, the power-action/localization/trusted-approval suites passed
23 tests and Flutter analysis again found no issues. Actual C fixtures verify
API 1.40 refusal before any granted open; the rebuilt API 1.41 library opens both
Mijia and combined modes with all four exact startup tuples and no account traffic.
Both projection unit tests, shared presentation tests and generated locale checks
passed. The known eight unrelated full-suite macOS visual/legacy-click failures
are not claimed fixed. No new APK, installation, provider/device operation, push
or publication occurred in this App phase.

## Power-toggle API 1.41 Android package -- October 30, 2026

Clean source d505ccb57bc1ad368e320d41e451b375fdf5bd14 produced the combined-platform
release APK apps/mobile/build/delivery/mijia/d505ccb/homeverb-mijia-0.2.0-d505ccb.apk.
Size: 79,138,514 bytes. SHA-256:
0228a329898b1d9d794f9eeeca63f25e955e22f29db89914e2a5450a305113d6.
The adjacent verification.json records clean source, exact current-AAR code after
NDK stripping, arm64-v8a/x86_64 composition, 31 exports per ABI, development
signature and 16 KiB ZIP/Core ELF alignment. This is archive evidence, not runtime
or distribution acceptance.

Both Android native cross-builds and the universal AAR passed using the existing
Android target toolchain, NDK 28.2.13676358, API 35 stubs and installed JDK.
Flutter 3.47.7 assembled the release APK with temporary SDK/JDK configuration;
the original android/local.properties was restored and source remained clean.
The actual verifier confirmed signature, alignment, exports and exact AAR match.
No SDK installation, app installation, real account/appliance request, push or
publication was performed.

This package contains all eleven exact actions, including five elevated power
toggles with independent startup grants and actual trusted confirmation, alongside
33 scalar controls and bounded history views. The remaining 29 actions, 78 events,
specialized reference paths and provider/phone/model acceptance stay incomplete.

## Typed maintenance and stop actions -- October 30, 2026

[ADR-0112](../adr/0112-exact-mijia-maintenance-actions.md) implements four additional exact no-argument actions through
protocol/Core: xiaomi.camera.083ac1 PTZ calibration, restart and memory-card
ejection, and xiaomi.hood.cyp10 dry-cleaning stop. Complete protected model/spec/
service/action/root evidence selects one legacy or modern request. No property,
recording-mode, power, schedule, storage-format or companion write is added.

All four are elevated, conservatively non-idempotent and require independent exact
Core/action-family grants plus actual trusted approval. Protected risk is checked
independently of public metadata. Accepted is unverified: it establishes neither
safe card removal, restored monitoring, completed calibration nor a stopped/cooled
hood. Uncertain sends retain receipts across cancel/drop/reopen/retirement without
retry, credential renewal or protocol fallback.

Fifteen actions now have typed protocol/Core support; eleven are in native/App.
API 1.41 intentionally retains its earlier startup vocabulary and inspection.
The power opt-in selects switch toggles only, never arbitrary elevated actions.
Native/App adoption of the four maintenance actions, the other 25 actions, all
78 events, specialized reference paths and real acceptance remain incomplete.
The d505ccb APK is unchanged. No real account/device operation, installation,
push or publication occurred in this phase.

Executed verification: Rust 1.96 offline/locked workspace tests passed 686 tests
with one ignored, excluding homeverb-midea for its known macOS LAN-alias
limitation. Strict all-target Clippy, rustfmt, FFI rebuild and diff checks passed.
The rebuilt C ABI passed 36 methods, 76 live examples, 76 modern-account boundary
cases and existing trusted/protected guards; all four maintenance startup tuples
were refused before creating a profile. Contract validation passed 25 schemas,
81 positive and 1,558 negative fixtures. Both reference projection tests, source
hygiene and documentation checks passed. No Flutter run, APK rebuild or real
provider/device operation was performed in this typed-only phase.

The coverage audit now explicitly counts 225 source writable property definitions:
33 are implemented exact controls and 192 still require separate semantic/risk/
runtime admission. This is additional unresolved scope alongside the remaining
actions/events; a source write flag never establishes implemented support.

## Native maintenance actions -- October 30, 2026

[ADR-0113](../adr/0113-native-mijia-maintenance-actions.md) admits four exact ADR-0112 maintenance/stop tuples through
API 1.42's existing independent mijia_action_grants. The separate optional
include_maintenance_actions flag defaults to false. Low-risk and power-toggle
inspection retain their existing scopes; combined flags do not duplicate options.
All startup/inspection work stays local and grants no user consent.

Exact granted reopen and explicit account refresh publish the elevated actions.
Every execution uses the existing immutable trusted preview/confirm/discard bridge;
ordinary plan/execute remain denied. Acceptance establishes neither physical
completion nor safe removal/restored monitoring. Cancelled, late or lost replies
preserve unknown receipts without replay after reopen. No new method/export,
SQLite/protected payload format, retry, companion action or verification is added.

All fifteen admitted actions now have Core/native support; eleven are in App.
Flutter adoption of the four maintenance actions and a matching APK remain pending;
d505ccb stays API 1.41. The other 25 actions, 192 remaining writable definitions,
78 events, specialized reference paths and real provider/phone/model acceptance
remain incomplete. No real account/device operation, installation, push or
publication occurred in this phase.

Executed native verification: 690 Rust tests passed, one ignored, excluding the
known homeverb-midea macOS LAN-alias failures. Strict workspace Clippy, FFI build,
68 focused Host account/grant tests, actual C ABI checks (36 methods, 84 examples,
76 modern-account boundary cases), 25 schemas with 82 positive/1,720 negative
fixtures, documentation/hygiene and both reference projection tests passed.
Flutter and refreshed Android packaging follow as separate phases.


## Foreground App maintenance actions -- October 30, 2026

Flutter negotiates API 1.42 and requires the trusted approval bridge before
requesting include_maintenance_actions or reopening with exact maintenance grants.
The four reviewed camera calibration/restart/card-ejection and hood cleaning-stop
tuples remain independent of low-risk and power-toggle action permissions. Older
native versions retain their previous options and receive no maintenance grants.

Opening a supported device prepares exact grants and refreshes its account to
publish descriptors; it never prepares or sends a command. Generic descriptor
buttons open the existing cloud-route selection and immutable review. Separate
user confirmation executes once. Dismissal/backgrounding invalidates the preview;
duplicate clicks and unknown outcomes do not replay. English/Chinese copy explains
possible motion, interrupted monitoring/recording, card-removal uncertainty and
unconfirmed hood stopping/cooling without inferring a resulting device state.

All fifteen admitted actions now have Core/native/App integration. A matching
Android package follows separately; d505ccb remains API 1.41. The other 25 actions,
192 writable definitions, 78 events, specialized paths and real provider/phone/
model acceptance remain incomplete. No real account/device operation, installation,
push or publication occurred in this phase.


Executed Flutter verification: all 155 tests across the 17 affected maintenance,
power/action, outlet/settings, permission, control, component, localization,
current-contract, native, approval and foreground-refresh files passed using
Flutter 3.47.7. The maintenance suite contains 15 tests, including all four actions
in both languages at 320 logical pixels and double text size, independent flags,
actual API 1.41 refusal and current native startup in both allowed modes.
Flutter analysis reported no issues; seven shared presentation tests and both
reference projection tests passed. These focused regressions do not resolve the
eight previously recorded unrelated macOS visual/legacy-click full-suite failures.
No live account, appliance write, phone acceptance or remote CI is claimed.


## Maintenance actions API 1.42 Android package -- October 30, 2026

Clean source 87f3410c6bbe1f13f5b5894251d0dc6a8110e568 produced the combined-platform
release APK apps/mobile/build/delivery/mijia/87f3410/homeverb-mijia-0.2.0-87f3410.apk.
Size: 79,204,178 bytes. SHA-256:
5d258ebfa6c4ded3dd3802cd9c6b6cdeef008503b83575f8e7dab0e4bcec2fba.
The adjacent verification.json records clean source, exact current-AAR code after
NDK stripping, arm64-v8a/x86_64 composition, 31 exports per ABI, development
signature and 16 KiB ZIP/Core ELF alignment. This is archive evidence, not runtime
or distribution acceptance.

Both Android native cross-builds and the universal AAR passed using the installed
Android target toolchain, NDK 28.2.13676358, API 35 stubs and JDK. Flutter 3.47.7
assembled the release APK with temporary SDK/JDK configuration; the original
android/local.properties was restored and source remained clean. The actual
verifier confirmed signature, alignment, exports and exact AAR match.
No SDK/app installation, real account/appliance request, push or publication was
performed. A fresh git fetch confirmed origin/main remains ccf2975 and is an
ancestor of the worktree, with zero merge commits in origin/main..HEAD.

This package contains all fifteen admitted exact actions, including four elevated
maintenance actions with independent startup grants and actual trusted user
confirmation, alongside 33 scalar controls and bounded history views. The other
25 actions, 192 writable definitions, 78 events, specialized paths and real
provider/phone/model acceptance remain incomplete.


## Exact switch and outlet settings -- October 30, 2026

ADR-0114/API 1.43 adds seven exact property controls: three Opple switch channels
and Yutai outlet power, startup-state code, indicator and local-input child lock.
Protected full model/spec/property/domain evidence and independent outlet grants
remain mandatory. Power, startup state and local-input locking require actual
trusted consent; only the indicator is low risk. No timer, protection threshold,
pairing, opaque payload or companion write is added.

Each absolute target sends one property on the selected legacy or modern protocol.
Acceptance is unverified; unknown receipts survive reopening without retry or
protocol fallback. Old read-only protected mappings remain valid and explicit
refresh upgrades them. API 1.43 adds exact startup tuples without new methods,
exports, storage or outcome formats. Flutter adoption of the new tuples and an
updated package follow separately; the 87f3410 APK remains API 1.42.

There are now 40 implemented Core/native scalar controls across eight models;
33 are in App. Of 225 source writable definitions, 185 remain incomplete, alongside
25 actions, 78 events, specialized paths and provider/phone/model acceptance.
No real account/device operation, installation, push or publication occurred.

Executed native verification: Rust 1.96 offline/locked workspace tests passed
697 tests with one ignored, excluding homeverb-midea's known macOS LAN-alias
failures and using --test-threads=4. The Mijia suite passed 271 unit and 14
compile-fail tests; focused Host account/grant suites passed 68 tests. Strict
all-target workspace Clippy and FFI build passed. Actual C ABI checks passed
36 methods and 84 live examples, including both startup modes and invalid new
tuples; modern-account export checks retained 76 boundary cases. Contracts passed
25 schemas, 83 positive and 1,855 negative fixtures. Documentation, source hygiene,
reference projection, both projection unit tests and diff whitespace checks passed.
An initial default-parallel run exposed a stale supported-model assertion, which
was corrected, and two existing 100 ms deadline tests failed under CPU contention;
both passed in the bounded-concurrency package and workspace reruns. No physical
or provider acceptance follows from these fictional tests.


## Foreground App switch and outlet settings -- October 30, 2026

Flutter now adopts ADR-0114's seven additional property controls. New exact
switch-3/11/12 power, indicator_light-4 and custom-7 input-lock tuples require
API 1.43 negotiation before granted reopen. The existing switch-2 target schemas
remain compatible. Independent outlet permissions, complete native model evidence
and actual trusted confirmation still decide admission; generic UI code does not
infer device support from a model name.

Device opening sends no command. Exact parameter selection and cloud-route review
precede elevated power/startup/input-lock execution. Bilingual confirmation copy
identifies the selected channel or local-button effect. Indicator changes use the
ordinary low-risk path without a companion power write. Backgrounding invalidates
pending setup and previews; repeated clicks and unknown outcomes never replay.
All 40 admitted scalar controls now have Core/native/App integration across eight
models. A matching package follows separately; 87f3410 remains API 1.42. The 185
other writable definitions, 25 actions, 78 events, specialized paths and real
provider/phone/model acceptance remain incomplete.

Executed Flutter verification: 175 tests across 18 affected control, permission,
component, native-contract, localization, approval and foreground-refresh files
passed under Flutter 3.47.7. The new suite has 20 tests, including all seven fields
in both languages at 320 logical pixels with double text size, actual API 1.42
refusal, current native startup in both modes, explicit targets/routes,
single confirmation and background discard. Flutter analysis reported no issues;
seven shared presentation tests passed. These focused regressions do not resolve
the eight previously recorded unrelated macOS visual/legacy-click full-suite
failures. No real account, appliance write, phone acceptance or remote CI is claimed.


## Switch and outlet API 1.43 Android package -- October 30, 2026

Clean source 85fefd8b315692902464dc93ae8468c30675fd17 produced the combined-platform
release APK apps/mobile/build/delivery/mijia/85fefd8/homeverb-mijia-0.2.0-85fefd8.apk.
Size: 79,205,770 bytes. SHA-256:
a4c2c107afe9c937e47f3592770987e435d0b132e04a618fe5b9af5a5b9d3ae8.
The adjacent verification.json records clean source, exact current-AAR code after
NDK stripping, arm64-v8a/x86_64 composition, 31 exports per ABI, development
signature and 16 KiB ZIP/Core ELF alignment. This is archive evidence, not runtime
or distribution acceptance.

Both native cross-builds and the universal AAR passed using the installed Android
target toolchain, NDK 28.2.13676358, API 35 stubs and JDK. Flutter 3.47.7 assembled
the APK with temporary SDK/JDK settings; android/local.properties was restored.
The actual verifier confirmed signature, alignment, exports and exact AAR match.
No app installation, real account/appliance request, push or publication occurred.

This package contains all 40 admitted scalar controls across eight exact models
and 15 admitted actions, including trusted power and maintenance confirmation.
The other 185 writable definitions, 25 actions, 78 events, specialized paths and
real provider/phone/model acceptance remain incomplete.


## Typed reported event occurrences -- October 30, 2026

[ADR-0115](../adr/0115-mijia-reported-event-occurrences.md) adds an exact public
78-event catalog and one-page legacy/modern history requests using the documented
MIoT siid.eiid key. Each request selects one pinned model/specification/event,
at most 31 records and a positive half-open UTC window no longer than 32 days.
Modern history retains its separate community origin and per-field codec.
There is no retry, renewal, protocol fallback, subscription or background work.

Results contain only provider-reported timestamps. Every object row must match
the selected did/key/type; malformed time/value shape is counted as rejected.
Opaque payloads, actors and media are omitted. Equal timestamps remain separate
rows without uniqueness or physical-count claims. Completeness remains unproven,
and a full page is a limit signal, not a continuation cursor or delivery guarantee.
Custom cuco-spec/madv-spec/xiaomi-spec event URNs retain their exact identities;
shared device/property namespace admission is unchanged.

This is typed protocol support only. Core protected membership/policy integration,
Host/native/App adoption, event argument semantics, paging and subscriptions remain
incomplete. API 1.43, saved formats, descriptors and the 85fefd8 APK are unchanged.
No real provider/account/device operation, installation, push or publication occurred.

Executed verification: Rust 1.96 offline/locked workspace tests passed 710 tests
with one opt-in test ignored and the known macOS Meiju LAN-alias package excluded;
all tests used four test threads. This includes 284 Mijia unit tests and 14
compile-fail examples. Strict workspace Clippy, formatting and the native FFI
build passed. All three public-reference projection tests passed, including an
independent comparison of every event against the digest-pinned archive. Contract
validation passed 25 schemas, 83 positive and 1,855 negative fixtures. These are
synthetic/local checks, not real-provider or phone evidence.

The rebuilt actual C ABI passed all 36 ordinary methods, 84 live fictional examples
and 76 modern-account refusal/buffer/lifecycle cases, plus the existing trusted
approval/scan/link/identity boundaries. A fresh origin fetch retained main at
ccf2975, already an ancestor, with zero new merge commits. No mobile source or
package changed during this protocol-only phase.


## Event history domain contract -- October 30, 2026

[ADR-0116](../adr/0116-bounded-event-history.md) defines event history independently
of numeric samples. Public domain/schema targets use component/capability/event,
with an explicit occurrence_times projection, canonical UTC windows and only
reported timestamps. Duplicate timestamps are preserved; opaque payloads, actors,
provider IDs and complete-delivery claims are excluded. Three fictional fixtures
cover options, requests and partial results; empty/full pages retain unproven
completeness. The catalog contains 25 argument-bearing events and several services
without readable components, so future adapter publication must include event-only
services and still leave payload interpretation explicitly incomplete.

This phase implements contracts only. Core cross-field/read-policy validation,
protected adapter membership and descriptor publication, Host/native/App adoption,
payload semantics, pagination, subscriptions and provider/phone/model acceptance
remain unfinished. Native API 1.43 and the 85fefd8 APK do not change.

Executed checks for this contract phase: all four domain tests and all 14 existing
Host numeric-history tests passed with Rust 1.96 offline/locked and four test
threads. Strict workspace Clippy and formatting passed. Contract validation passed
26 schemas, 86 positive fixtures and 1,959 negative cases, including 104 additional
event counterexamples. Documentation and repository hygiene checks passed.
These checks do not establish callable Core event support or new mobile behavior.
