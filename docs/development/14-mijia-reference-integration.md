# 14. Mijia reference integration and remaining coverage

The October 25 worktree uses API 1.30 and the owner-supplied October 22 Xiaomi Home
11.7.709 analysis. The full official-App feature request remains **incomplete**.
The source archive explicitly does not supply all authentication, protocol or
runtime control contracts. An implemented decoder is not physical acceptance.

## Implemented software

API 1.30 adds 14 independently granted settings for the exact reference light
and purifier under ADR-0084; the full-domain native/App path is recorded below.

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
  setters. Timing/quiet and other additional fields remain read-only.
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

## Coverage boundaries

| Source item | Current disposition |
|---|---|
| 264 scalar readable properties | Exact mapping implemented; synthetic Core reads; provider/physical acceptance remains per property |
| 97 identity/pairing/media properties | Excluded from ordinary observations; no identity leakage or streaming inference |
| 78 non-readable properties | No read operation; write/event-only definitions are not state |
| 28 opaque readable values | Separate typed/business/privacy contracts required |
| 3 network identity values | Excluded from state and exports |
| 40 actions | No new action admission; runtime/firmware/composite constraints and risk review missing |
| 78 events | No event-history/subscription body, paging or lifecycle contract; not implemented |

The archived successful reads used **SID `mijia`** and
`https://core.api.mijia.tech/app`, with a continuous RC4 stream across encrypted
form fields. The existing native login uses **SID `xiaomiio`**, its reviewed
`api.io.mi.com` routes and legacy field encryption. These are separate session
contracts. Do not move a saved legacy token to the modern origin, silently change
its signing mode, or call modern success evidence proof of independent login.
The archive excludes the existing App session and reports standalone login,
renewal and expiry recovery as unverified.

The modern device-page endpoint requests logical split children. The analysis
records two split switch objects and a cloud/cache mismatch (21 shared IDs out
of 22 in each source). These do not prove 23 physical appliances. The current
legacy enumerator retains its bounded account/home union and partial status;
complete inventory, logical-parent reconciliation and missing-device removal are
not claimed. No private identifiers enter this ledger.

The HA source review below supplies further community authentication and history
research leads; it does not implement those features. Blocked pending additional
evidence: modern native authentication/renewal;
specialized paths for failed cloud properties and BLE devices; history/statistics,
event pagination and foreground subscriptions; health/media/streams; plugin-specific
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

The community client's generic SID parameter and non-default SID branch do not
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
