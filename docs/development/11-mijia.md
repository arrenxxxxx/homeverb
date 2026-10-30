# 11. Mijia native integration

October 30: 17 exact Mijia reference specifications supply 264 scalar read
mappings and localized App labels. Modern QR authentication, grouped device
publication and Core reads are integrated through Flutter with fictional evidence.
Exact modern outlet daily-energy history uses the existing history interface.
ADR-0105 exposes numeric property logs for 161 exact fields through Core,
CLI/MCP and API 1.38/App, preserving original signed values and nullable units.
ADR-0104 adds 29 exact modern scalar controls across five models. Broader
controls/history and other reference coverage remain incomplete; see the
[coverage ledger and remaining work](14-mijia-reference-integration.md).

The October 30 integration uses native API **1.38.0**, following the linear
mainline update and dedicated interactive-login bridge. Earlier Mijia phase versions are historical. See
[contract reconciliation](../adr/0079-mijia-mainline-reconciliation.md). Mainline Meiju restoration and controls
remain available alongside Mijia; identity restoration remains Meiju-only.

The experimental integration is implemented under [ADR-0070](../adr/0070-mijia-native-cloud.md).
The owner will perform real-account and device acceptance. Do not access saved
pilot credentials or contact appliances as part of development tests.

Current callable scope includes native API 1.38 Mijia-only/combined sessions,
protected partial inventory and 264 scalar mappings from 17 pinned reference
specifications. Exact fan, light and purifier settings have independent protected
admission and trusted startup grants. Flutter provides account/read/control views,
explicit captcha and phone/email code continuation, and bounded historical reads
for the exact `cuco.plug.v3` daily-energy profile and admitted numeric samples. Historical units and incomplete
record evidence stay separate from live state and execution verification.

Native API 1.32 supplies the user-triggered code-request bridge under
[ADR-0087](../adr/0087-native-mijia-verification-code-delivery.md); Flutter now
consumes it without automatic message sends or retries. The API 1.33
[Android package](../../apps/mobile/verification.md#historical-api-133-android-package--october-28-2026)
includes this account flow and historical UI. The [API 1.37 package](../../apps/mobile/verification.md#modern-qr-and-device-publication-android-package--october-30-2026)
also contains modern SID `mijia` QR login, grouped cards and read-only state views.
ADR-0103 adds exact modern outlet daily-energy history after that APK revision.
ADR-0104 adds exact modern controls after that APK revision.
Human risk challenges, account-settings confirmation, broader controls/history,
broader event/control coverage and real-provider/device acceptance remain open. Earlier
phase records below retain their original implementation and evidence scope.

## Protocol evidence

- MIT `Squachen/micloud`, commit `dc002b87fc09ef36199715cfa10f1eaa2d2d4210`:
  [login and inventory](https://github.com/Squachen/micloud/blob/dc002b87fc09ef36199715cfa10f1eaa2d2d4210/micloud/micloud.py),
  [request encoding](https://github.com/Squachen/micloud/blob/dc002b87fc09ef36199715cfa10f1eaa2d2d4210/micloud/miutils.py).
  HomeVerb independently implements the wire format in Rust; attribution and
  the source license are retained under `third-party/micloud`.
- Pinned [community MIoT client](https://github.com/al-one/hass-xiaomi-miot/blob/a3a07b8504bacfee114a6052ae864d7dd149bf71/custom_components/xiaomi_miot/core/xiaomi_cloud.py)
  was inspected for endpoint and response facts only. Its pinned root license is
  Apache-2.0; the earlier GPL description was incorrect. No source was copied.
- [Xiaomi's HA client](https://github.com/XiaoMi/ha_xiaomi_home/blob/main/custom_components/xiaomi_home/miot/miot_cloud.py)
  demonstrates a distinct registered OAuth flow; its application identity is not
  HomeVerb's authorization and is not reused.

The native password flow requests a login signature, submits the transient
password MD5 digest, then exchanges an allowlisted STS location for a user session.
Cloud requests use per-request random/time nonces, SHA-256 derived nonce keys,
SHA-1 signatures and RC4 with the first 1024 stream bytes discarded. These legacy
algorithms implement vendor compatibility within authenticated TLS; they are not
chosen for new security protocols. No shared developer secret is needed.

## Development phases

The owner selected mainland China and requested all returned devices as named
cards, including unsupported placeholders, with doorbells, door locks and fans
first. Device online hints must not become power or lock state. Fan controls need
actual readable/writable MIoT evidence and independent grants; locks and doorbells
initially expose only reviewed readable properties. Unlocking remains denied.

1. Bounded protocol, protected login/session, inventory and MIoT property decoding,
   with independent fictional vectors and failure/cancellation tests.
2. Protected bindings, exact model/spec evidence, Core adapter and account lifecycle.
3. Versioned native host and Flutter connection/read flows, then reviewed controls.
4. Owner acceptance of real login/challenges, complete regional inventory, same-account
   reconnection, device readings, phone lifecycle and separately authorized writes.

Only executed local checks will be recorded as passing. No model is verified by
source research or synthetic tests. Regional support and challenge handling must
be checked with the owner's account before a supported-account claim.

## Protocol phase evidence

On October 18, 2026, `cargo test -p homeverb-mijia --offline --locked`
passed 16 fictional tests, including independently calculated signing/encryption
vectors, login failure/challenge checks, inventory and property correlation,
verified local TLS, redirect/certificate/body/cookie rejection and pending-socket
cleanup on cancellation/deadline/drop. The initial sandbox run passed 12 cases
and blocked four socket cases; the subsequent ordinary local-socket run passed
all 16. No vendor, account, secure store or appliance was accessed.
`cargo clippy -p homeverb-mijia --all-targets --offline --locked -- -D warnings`
passed. Core registration, complete-home inventory, account persistence, App and
device semantics remain later phases rather than protocol-test conclusions.

Repository checks passed for Markdown/JSON links, 20 schemas, 49 positive and
267 negative fixtures, and English/source hygiene using the existing `.venv`.
System Python lacked the schema dependency, and creating a new temporary venv
could not download packages under restricted networking; the already-provisioned
repository venv supplied the required dependencies without installation.
Workspace Clippy with Rust 1.96 passed. Rust 1.99 workspace Clippy stops on four
existing Meiju `chunks_exact_to_as_chunks` warnings; the new crate passes on 1.99.
The workspace test run stops at 14 existing Meiju LAN cases because macOS lacks
the documented `127.0.0.2` / `127.0.1.1` aliases (`AddrNotAvailable`); these are
blocked, not passing tests. No privileged network configuration was changed.

The Rust 1.96 workspace build and formatting check passed. Existing runtime checks
also passed: 35 CLI/MCP cases with seven documented macOS skips, 34 native methods
with 55 live fictional examples plus trusted bridge suites, 68 Python unit tests,
seven presentation tests and 186 existing commit messages. These regressions do
not make the new Mijia library available through any host yet. No remote push,
publication or real-device operation was performed.

## Inventory and specification phase

The native library now unions the legacy account device list with accessible
home device pages. Limits are 300 homes, 16 pages per home, 300 entries per page
and 4096 unique devices within one inherited deadline. Duplicate identity with a
different model/specification is refused. Home failures retain positive results
with sanitized issues, while authorization failure and cancellation stop the
operation. Community home-list coverage is not proven globally complete, so the
result cannot remove historical bindings or claim a complete-inventory timestamp.

Explicit public specification lookup uses the
[MIoT index](https://miot-spec.org/miot-spec-v2/instances?status=all) and exact
instance URNs. Index responses are capped at 8 MiB/100000 entries; instance and
account responses remain capped at 1 MiB. No account cookies or encryption headers
go to the public specification origin. There is no startup/background fetch.
Only exact released model matches and unambiguous selected versions are admitted.

Reviewed wire facts were inspected in public specifications for
`zhimi.fan.za4` revision 1, `loock.lock.v1` revision 1 and
`linp.doorbell.g03` revision 1 on October 18. These are protocol examples, not
owner-device identification or support certification. The code projects only
matching standard service/property URNs with actual `read` access:

- Fan power is a Boolean `power.state`. Fan level, mode, horizontal swing and
  angle retain `vendor.xiaomi.mijia.fan` semantics and reported types.
- Lock `status` remains a raw numeric `vendor.xiaomi.mijia.lock.status`; a vendor
  code such as `255` does not become an unlocked/locked Boolean. No lock writes.
- Doorbell volume and readable battery level remain vendor properties. Neither
  implies a doorbell power switch. No camera stream, event history or notification
  subscription is added.
- Identity/serial properties, event-only fields, arbitrary vendor extensions and
  other device families are not mapped. They will remain unsupported placeholders.

Capabilities currently have no actions and all properties remain read-only.
Reported fan write domains are retained internally as evidence only, with type,
range, step and enum checks; future controls still need independent startup grants
and execution validation. Per-service component IDs prevent merging distinct units.

`cargo +1.96.0 test -p homeverb-mijia --offline --locked` passed 28 fictional tests;
`cargo +1.96.0 clippy -p homeverb-mijia --all-targets --offline --locked -- -D warnings`
passed. Added cases cover model/version/refusal, fan/lock/doorbell distinctions,
identity/event exclusion, exact home union, pagination cycles/conflicts, partial
errors, cancellation and actual TLS proof that spec requests omit account cookies.
Core/vault registration and native/App connection are still pending.

Public spec response SHA-256 values at inspection (source data is not vendored):

| Source | SHA-256 |
|---|---|
| Fan `urn:miot-spec-v2:device:fan:0000A005:zhimi-za4:1` | `cab2d8b2de82ca17db5abc7c701fb589fb294253afeb08304d0a064f9a1ca859` |
| Lock `urn:miot-spec-v2:device:lock:0000A038:loock-v1:1` | `e4e25ddb4a2ce037414ebb95e7f0092df71f2914e3f973291af1596a94ea8233` |
| Doorbell `urn:miot-spec-v2:device:doorbell:0000A03B:linp-g03:1` | `aa4bf6dc423e22d764568d2d868b6be3467d089a110b526cf61930b01477e732` |

Each instance is available from
`https://miot-spec.org/miot-spec-v2/instance?type=<exact-URN>`. Tests use independent
fictional specifications; public-model facts do not establish a physical test.

## Protected account and Core phase

`MijiaAdapter::connect_devices` consumes protected credentials and persists only
provider-validated session material. `refresh_devices` reuses it without login;
`recover_devices` explicitly validates exact pending bytes before confirming them.
Reconnection by the same protected subject retains local IDs and preferences.
`accounts`, `account`, `rename_account`, `inspect_inventory` and `remove_account`
operate on exact local scopes; rename/removal need no provider request.

The system-store payload is version 1, namespaced by provider and region and bound
to the AccountVault's random account/credential references. Private vendor IDs,
session material and original MIoT source/derived mappings stay out of SQLite and
ordinary host JSON. Saved mappings must re-derive identically from their protected
source. A copied, corrupt, foreign or uncertain payload fails closed.

Explicit account connection/refresh returns positive descriptors plus partial
coverage/issues for a trusted host to pass to `Core::publish_discovery`. Unsupported
models remain named cards; optional specification failure preserves placeholders.
Generic Core discovery refuses populated Mijia accounts before networking, because
its return type cannot express partial coverage. Missing historical bindings and
complete-inventory timestamps are never changed based on this partial enumeration.

The Core adapter rechecks protected model/component/account/connection identity,
holds the account lease through the read, and returns correlated scalar observations
with cloud source, local receipt time and stale/unknown-source-time evidence.
Core rejects malformed values before replacing cache. No action is admitted, even
when public metadata and a Core grant claim otherwise. Native/App composition and
separately admitted fan controls are the next implementation work.

On October 18, `cargo +1.96.0 test -p homeverb-mijia -p homeverb-host --offline --locked`
passed 286 checks across 25 targets/doc-test suites, including all 43 Mijia tests,
three Mijia serialization compile-fail tests and the production empty-startup
test. Fictional secret-store failure/uncertainty, subject/route/model conflicts,
copied/corrupt payloads, stable preferences, partial retention, optional spec failure,
Core cache refusal, cancellation/drop, cleanup and forged-action refusal passed.
Workspace build, formatting and strict Clippy passed on Rust 1.96; docs/contracts
and language checks also passed. The prior full-workspace macOS LAN limitations
remain documented above; this scoped passing run does not waive them.

Existing runtime regressions also passed: `scripts/verify_mvp.py` completed 35
CLI/MCP cases with seven documented platform skips, and
`scripts/verify_native_contract.py` passed 34 native methods, 55 fictional examples
and the trusted approval, dishwasher, scan and connection-link bridge suites.
No real account, vendor traffic, physical device, push or publication was used.


## Native account routing phase

[ADR-0077](../adr/0077-mijia-native-account-routing.md) adds API 1.24 startup modes
`xiaomi.mijia` and `homeverb.platforms` with the existing protected native connection
function. The combined mode shares one vault/Core with Meiju while preserving exact
Meiju startup grants. Optional provider metadata chooses an already registered
connection flow. Ordinary account methods route using saved account ownership;
completed removal retries consult each provider-bound ledger before any mutation.

Mijia partial retrieval publishes positive device IDs instead of hiding named
cards. Failed publication returns an error and no result IDs; callers reload local
cards because storage errors do not promise rollback. Inventory inspection remains
`unconfirmed`: partial positive bindings do not establish membership in a complete
inventory. Source online hints never become actual power state.

The native contract has 34 ordinary methods and the same C exports. SQLite 5,
Mijia protected payload 1 and Meiju protected payload 3 are unchanged. Flutter and
reviewed fan control are the next phases. No real account or appliance has been
used for this phase's fictional tests.

On October 18, `cargo +1.96.0 test -p homeverb-host --offline --locked` passed
243 tests across 24 targets/doc-test suites, including three new encrypted fictional
Mijia account tests and the existing Meiju/Mock regressions. Workspace build and
strict all-target Clippy passed on Rust 1.96. The actual C ABI verifier passed all
34 methods, 55 fictional examples, the existing trusted bridge suites and new
Mijia/combined startup, provider admission and buffer/lifecycle checks. CLI/MCP
runtime verification passed 35 cases with seven documented platform skips.
Contracts passed 20 schemas, 49 positive fixtures and 278 negative fixtures;
document links and source hygiene passed. These checks used temporary fictional
profiles only. Mobile package builds and real accounts/devices were not tested in
this phase; iOS acceptance's expected API version is updated but was not executed.


## Flutter account and read phase

The platform catalog displays providers registered in the native session. Mijia
sign-in and same-account reauthorization pass only the selected public provider
through native metadata; credentials remain owned, cleared buffers. Account
remarks, masked-name fallback, removal and recovery retain the existing lifecycle
rules. Partial retrieval shows its positive named cards and coverage warning;
unknown models remain placeholders. The general account-retrieval entry opens the
account list so a user chooses the account to refresh instead of invoking generic
multi-adapter discovery, whose result cannot represent Mijia partial coverage.

Generic device details render reported fan values, raw lock status, doorbell
volume and battery levels without guessed state or actions. Unsupported native
control-option inspection leaves read-only details accessible. Primary appliance
components precede auxiliary batteries so a battery service cannot replace fan
power in a home-card summary. Percentage/degree units render as symbols without
changing numeric values or ranges. Lock and doorbell symbols are decorative only.

Use a native library built from this branch and the pinned Flutter SDK:

```sh
cd apps/mobile
fvm flutter run -d DEVICE_ID --dart-define=HOMEVERB_ADAPTER=homeverb.platforms
```

Combined mode reuses the existing app-private `native-meiju` profile. Explicit
`xiaomi.mijia` uses `native-mijia`; default Mock keeps `native`. There is no profile
copy, secret export or migration. Rebuild the Android AAR or signed iOS framework
from this checkout before running on a phone; desktop macOS has no production
credential backend and is suitable only for the fictional native tests.

Owner acceptance should start at Platforms > Add Platform > Mijia. Confirm the
mainland account, named returned devices and unsupported placeholders, then open
fan/doorbell/lock cards and compare the actual readable values with the vendor app.
Online hints do not establish power; a missing field must remain unknown. The
current community password flow cannot complete CAPTCHA/verification challenges;
`MIJIA_AUTH_CHALLENGE_REQUIRED` is an incomplete login, not a saved successful
connection. Shared-device coverage, real authentication and physical readings are
still unverified. No lock write is available. Fan control is a subsequent phase.

This App phase passed 52 scoped Flutter checks and a subsequent 23-check Mijia/
state-refresh/permission run after adding the delayed-inspection cancellation
case. The analyzer passed; fictional catalog/sign-in/account captures were visually
inspected. Full Flutter regression had 157 passes and the eight previously
recorded macOS glyph/legacy-click failures, not a full-suite pass. Rust Mijia/host
regression passed 290 checks across 26 suites, plus workspace build and strict
Clippy. See the [App verification record](../../apps/mobile/verification.md) for
commands, evidence scope and remaining phone/package acceptance.

## Single-property write protocol foundation

The protocol crate now has a separate `send_control` path for the fixed
`/miotspec/prop/set` endpoint. Read transports refuse that endpoint, and transports
without explicit write support refuse without falling back to `send`. One scalar
property is sent once under the inherited operation. Synchronous preflight occurs
before the possibly-sent marker; subsequent cancellation, deadline, DNS/TLS/HTTP
failure or malformed response preserves uncertainty. There is no retry, redirect,
automatic login or alternate-route send.

An encrypted response must contain exactly one matching device/SIID/PIID result.
Its zero code means accepted with no completion evidence; a correlated nonzero
item code is explicit rejection. Plaintext/global codes and missing, duplicate or
foreign results do not establish acceptance or rejection. HTTP 200 alone never
advances the write to accepted. Existing outcome schemas are unchanged.

This low-level Rust protocol primitive is not a grant. The production Core adapter
still denies execution; protected fan admission, native grants and App controls
remain pending. No control is available through native JSON, CLI, MCP or Flutter.
No real account or appliance was used.

Rust 1.96 `cargo test -p homeverb-mijia --offline --locked` passed 53 unit tests
and three compile-fail doc tests, including nine new protocol/transport cases.
The initial sandbox run could not bind loopback sockets; the successful rerun
used permitted local TLS fixtures. Strict all-target Mijia Clippy passed.

## Protected fan adapter phase

[ADR-0078](../adr/0078-mijia-fan-control.md) adds explicit trusted Rust composition
with `MijiaAdapter::with_fan_controls()`. It still requires independent Core grants.
Protected model/spec evidence supplies power, level, mode, horizontal swing and
horizontal angle actions only when their exact properties are readable/writable
and have representable domains. Each setter writes one property. The default
adapter and current native/App composition remain read-only.

Local option inspection checks the protected binding and projects choices without
networking, descriptor publication or authority changes. Execution rechecks the
account lease, exact route/identity, component/action and protected argument domain
before selecting SIID/PIID. Enlarged public schemas and forged actions cannot
bypass this check. Uncertain replies and cancellation retain unknown receipts;
repeated request IDs do not send again. Acceptance has no verification evidence.
Doorbell/lock actions remain absent.

The phase passes 58 Mijia unit tests and three compile-fail tests on Rust 1.96,
including protected option inspection, exact Core grants, forged public domains,
identity-mutation refusal, accepted/rejected/unknown receipts and cancellation.
All evidence is synthetic; no real fan was controlled. Native grant contracts,
Flutter preparation and phone packaging remain subsequent work.

## Native fan grant phase

API 1.25 adds exact startup-only `mijia_fan_grants` and routes protected
`inspect_control_options` by the saved registered provider. Mijia-only and combined
sessions independently compose fan adapter admission and Core policy. Malformed
or foreign-provider grants fail before profile creation; ordinary requests cannot
change authority. Default constructors remain read-only. Protected option
inspection uses no network and publishes no descriptors. Strict clients must
negotiate the new `fan-SIID` response scope before enabling controls.

The native contract and examples retain 34 ordinary methods and unchanged C
exports. SQLite 5, protected Mijia payload 1 and Meiju payload 3 are unchanged.
Flutter preparation and installation packages are the next phase. Native physical
and account acceptance remain owner work.

Native phase verification: 246 host tests/doc tests passed on Rust 1.96, including
three new exact-grant cases and protected Mijia inspection through the response
schema. Workspace build and strict all-target Clippy passed. The actual C ABI
verifier passed 34 methods, 55 fictional examples, trusted bridge regressions and
new Mijia grant/provider denial checks. Contracts passed 20 schemas, 51 positive
fixtures and 291 negative fixtures. No real account, appliance or mobile runtime
was used in this phase.

A subsequent exact-value review keeps the original JSON scalar in Core's grant
instead of normalizing it through floating point. Only duplicate detection uses
numeric normalization; it never changes authority. The four focused grant tests
pass, including an integer above the exact binary64 integer range. This does not
admit a new device value: the protected MIoT domain remains independently required.

## Flutter fan controls

The foreground controller derives exact fan startup permissions from protected
option inspection, retaining each `fan-SIID` component and actual boolean/enum/
range values. It negotiates API 1.25 for these grants, reopens the native session
and refreshes only the selected account. A partial inventory may complete device
setup only if its positive published IDs include that exact device. Missing target
publication refuses setup. Preparation sends no appliance write.

Generic detail buttons and argument forms expose power, level, mode code,
horizontal swing and angle only where described and granted. English/Chinese
labels preserve native mode codes without inventing labels or ranges. Submission
retains existing planning, observation, receipt and lifecycle behavior. Opening a
form or editing a value sends nothing. Doorbell/lock/battery remain read-only.
The owner will validate real login, readings, controls and phone lifecycle.

Flutter fan verification passed 48 focused checks and static analysis. Full
regression passed 165 with the same eight documented macOS baseline failures;
it is not a full-suite pass. See the [App record](../../apps/mobile/verification.md)
for the commands and exact failure scope. No real account/device testing occurred.

## Android package and owner acceptance

The combined-platform development APK was built from clean source revision
`39d646546a6ea0c07bc6a4154212b1c5a9d1ee74` on October 18, 2026. The dedicated
branch is `feature/mijia-integration`; all development phases are local commits.
No merge, push, installation, account login or physical-device operation was
performed for this delivery.

Artifact: `apps/mobile/build/delivery/mijia/homeverb-mijia-0.2.0-39d6465.apk`
(73,254,278 bytes). The adjacent `verification.json` records the APK/AAR/native
hashes and exact source revision. SHA-256:
`59d0925a8ebfa981a5760695cf411cf912ac55441f0e4f289e1477b1c95ace00`.
Generated artifacts are ignored build outputs, not repository source files.

Executed build and inspection:

- `python3 scripts/build_android_native.py --ndk NDK --android-jar ANDROID_JAR --java-home JDK --abi arm64-v8a --abi x86_64`:
  passed on installed Rust 1.99.1, NDK 28.2.13676358 and JDK 21; both libraries have
  17 exports and 16 KiB ELF alignment. The AAR was rebuilt after the exact-value fix.
- `fvm flutter build apk --release --target-platform android-arm64,android-x64 --dart-define=HOMEVERB_ADAPTER=homeverb.platforms`:
  passed with pinned Flutter 3.47.7 and a temporary SDK configuration. The existing
  persistent development signing key was used. This remains a development APK,
  not a store-signed release.
- `python3 scripts/verify_flutter_apk.py --apk APK --sdk SDK --java-home JDK --adapter homeverb.platforms --report REPORT`:
  passed signature, ZIP and Core ELF alignment, exactly two complete ABIs, 17 Core
  exports per ABI, release manifest/API 28 minimum, disabled backup and absence of
  the integration-test plugin. This inspects the archive, not running-device behavior.
- Presentation tests: seven passed. Documentation, contracts, hygiene and commit
  conventions passed. Full Flutter baseline failures remain as recorded above.

The principal changed files are the [Mijia adapter](../../crates/mijia/src/adapter.rs),
[fixed cloud transport](../../crates/mijia/src/transport.rs),
[MIoT mapping](../../crates/mijia/src/spec.rs),
[native grant policy](../../crates/host/src/mijia_fan_grants.rs),
[native startup bridge](../../crates/ffi/src/lib.rs),
[App grant preparation](../../apps/mobile/lib/session_permissions.dart),
[App controller](../../apps/mobile/lib/controller.dart) and
[versioned startup contract](../../schemas/native-open.schema.json).

Owner validation can proceed without a HomeVerb service:

1. Install the APK on Android 9 or later. It uses the existing HomeVerb package ID;
   installation replaces an older build and combined mode reuses its private Meiju
   profile. This delivery does not automatically install or remove application data.
2. Open Platforms, choose Add Platform and Mijia, then sign in with the mainland
   account. Password/session material stays in native protected buffers and Android
   Keystore. CAPTCHA/additional verification currently returns
   `MIJIA_AUTH_CHALLENGE_REQUIRED`; challenge completion is not implemented.
3. Compare returned names with the Mijia app. Unsupported devices should retain
   cards. Partial coverage is explicit; do not interpret absent devices as removed.
4. Open the doorbell, lock and fan. Compare available power/level/battery/volume or
   raw lock status readings. Missing fields remain unknown. These services are
   model-dependent; doorbell event/history/streaming and lock unlocking are absent.
5. For a fan, only the model's writable options should appear. If choosing to test
   a control, submit one safe value and compare the resulting state with the vendor
   app/device. Accepted is not physical verification; unknown outcomes must not
   trigger automatic resend. Background/foreground and restart should preserve
   account/card identity while cancelling in-flight work.

Real-account authentication, shared-device completeness, exact physical models,
Android phone lifecycle and iOS packaging/runtime remain unverified. The owner
explicitly owns real-device validation. Do not upload credentials, private device
IDs or raw provider replies when reporting a mismatch.

## Mainline reconciliation verification -- October 25, 2026

Fetched `origin` and rebased the ten Mijia commits onto mainline `ccf2975`.
`git log --merges origin/main..HEAD` is empty. The old temporary worktree had lost
its Git pointer/source files; its residual outputs were preserved and a new
worktree was created at `/private/tmp/homeverb-mijia-current`.

API 1.27 resolves independent version collisions and retains both providers,
Meiju identity recovery, dishwasher permissions and exact reference profiles.
Mijia cannot enter the Meiju identity-restoration path. Branch-only decisions are
now ADR-0076/0077/0078; ADR-0079 records the combined contract.

Executed: Rust 1.96 Mijia/Host tests (316 passed), workspace build and strict
all-target Clippy; the final identity-recovery subset (8 passed); Flutter analysis
and the six-file Mijia/control/recovery/permission/foreground suite (42 passed).
The actual C ABI verifier passed 34 methods, 55 examples and all trusted bridge
suites. CLI/MCP passed 35 cases with seven existing platform skips. Documentation,
21 schemas with 54 positive/365 negative fixtures, repository hygiene and seven
presentation checks passed. Generated locale catalogs were regenerated together.

Initial integration failures (missing provider fields, stale verifier API literal,
and fixtures without selected account metadata) were corrected and rerun. No
physical account/appliance, Android/iOS package, remote CI, push or publication
was used. Full workspace LAN and full Flutter suites were not run in this phase.
The October 22 reference feature audit remains subsequent work.

## Exact reference fan settings

API 1.29 adds protected horizontal/vertical swing ranges, vertical swing and
ionizer settings for `jdjz.fan.2072fc`, with native grants and localized App
forms. The earlier read-only statements above record earlier implementation
phases. See [the current phase evidence](14-mijia-reference-integration.md#exact-reference-fan-settings--october-25-2026).

## Native light and purifier settings

API 1.30 implements 14 exact reference settings across protected Rust mapping,
independent native startup grants and foreground App controls. See the
[verification and coverage record](14-mijia-reference-integration.md#exact-light-and-purifier-controls--october-25-2026).
Older read-only statements above describe historical phases. Physical acceptance
and the remaining reference features are not established by this implementation.


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

## Modern protected device bindings and Core reads -- October 30, 2026

[ADR-0100](../adr/0100-mijia-modern-protected-core-reads.md) adds explicit typed modern
device refresh/recovery returning partial Core descriptors. Protected payload 3
retains root/child evidence, stable local IDs and selected read-protocol provenance.
Versions 1/2 remain byte-stable until explicit device adoption; older binaries
cannot read version 3. SQLite 5, Meiju payload 3 and native API 1.36 are unchanged.

Core can publish these descriptors and read the exact pinned scalar properties
through modern credentials/origin/codec. Explicit legacy or modern inventory can
select its own read protocol; failed reads never fall back or reauthenticate.
Modern-selected bindings suppress controls/history and independently refuse stale
control descriptors/grants. Logical children never receive separate cards; prior
root/child and cross-account collisions require reconciliation rather than merging.
Names, rooms, favorites and device/connection IDs survive exact protocol selection.

Twelve new fictional integration tests cover all 264 mapped scalar readings,
restart, version-3 tampering, stable organization, partial retention, role/model/
parent conflicts, uncertain save/recovery, cancellation/drop and write refusal.
The Rust workspace run excluding the known unrelated Meiju/macOS LAN-alias test
failures passed 579 tests with one ignored. Strict workspace/all-target Clippy and
the native library build passed. The rebuilt C ABI passed all 36 ordinary
methods/57 examples and dedicated-export checks, including 62 modern-account
refusal/buffer/lifecycle cases. Documentation, repository hygiene and the reference
projection tests passed. No real provider/device access or mobile package was used. Host/App login-to-card publication remains the next dependent task;
the existing QR C/App reports still return logical directory summaries only.


## Foreground modern Host device publication -- October 30, 2026

[ADR-0101](../adr/0101-mijia-modern-host-device-publication.md) adds typed
`continue_modern_device_connection` and Host `continue_modern_device_login`.
The existing logical QR continuation is unchanged. The new continuation reuses
its one directory read while retaining original account locks and lifetime through
protected binding persistence. Host publication uses the remaining QR deadline.
`refresh_modern_account_devices` and `recover_modern_account_devices` explicitly
publish saved modern bindings without logging in. Ordinary Core state reads use
the protected selected protocol. Unknown models remain named cards and logical
split children never become duplicate physical cards.

AccountConnection reports distinguish authentication from partial/failed device
retrieval; uncertain binding persistence publishes nothing and requires explicit
recovery. Core publication failure returns no result IDs without promising rollback
of local storage. Background/cancellation/drop fences publication and releases
leases. The new typed results use the existing account-report schema; native API
1.36 C/App results remain logical-only. Versioned C/Flutter wiring and generic
refresh selection are next, followed by packaging and owner acceptance. No complete
inventory, modern control/history support or physical acceptance is claimed.

The Rust workspace regression excluding the documented unrelated Meiju/macOS
LAN-alias failures passed 591 tests with one ignored, including all 12 new Host
publication cases. Strict workspace/all-target Clippy and the native library build
passed. The rebuilt ABI passed 36 ordinary methods/57 examples and all dedicated
export checks, including 62 modern-account refusal/buffer/lifecycle cases. Contracts
passed 25 schemas, 69 primary positives and 1,092 negatives; documentation, repository
hygiene, the generated coverage check and both reference-projection tests passed.
These are fictional software checks, not provider, phone or physical-device
acceptance. No Flutter build, APK, provider account, appliance write, remote CI,
push or publication was performed.


## Native modern device publication (API 1.37) -- October 30, 2026

[ADR-0102](../adr/0102-native-modern-device-publication.md) adds optional Boolean
`publish_devices` to dedicated modern continuation metadata. True returns
`connected_devices` with a sanitized AccountConnection; absent/false retains
logical `connected`. New `refresh_devices` / `recover_devices` session operations
return `devices` with partial/failed retrieval. Old `refresh`, `recover`, `renew`
keep their behavior. No C export or ordinary JSON method is added.

Ordinary account refresh/recovery selects its protocol from protected saved
material before I/O, under the same account/registration leases. Modern-only
accounts and accounts with any modern-selected binding use modern inventory;
otherwise they use legacy. A modern failure never falls back or renews. Returned
roots select that protocol; absent historical bindings retain their identities
and selection. Modern-selected controls/history remain unavailable. Adding only
a modern authentication slot does not change a legacy account's refresh route.

Positive/negative schemas, fictional Host publication/protocol-selection tests and
actual C admission checks cover the additions. The workspace passed 595 tests
with one ignored; strict Clippy and the rebuilt C ABI passed, including 76 modern
account boundary cases. See ADR-0102 for the exact evidence scope. SQLite 5, Mijia payload 3, Meiju payload 3, 36 JSON methods and 30 C
exports (31 on Android) remain unchanged. Flutter opt-in/parsing/card adoption,
a new APK and real provider/phone/model acceptance remain pending. No provider,
appliance write, package installation, remote push or publication is performed.


## Flutter modern device publication -- October 30, 2026

The API 1.37 Flutter client now requests modern device publication after QR
continuation and explicit refresh/recovery under ADR-0102. API 1.36 retains logical
reports, and 1.35 refuses QR before symbol lookup. Distinct result kinds validate
closed fields, exact account/provider/authentication, partial/failed status,
required errors and at most 4096 unique UUID IDs. Owned parsed reports are immutable.
No successful logical count becomes a device count.

The controller reloads local accounts/cards before adopting a current foreground
report. Existing partial/failure UI displays returned device counts and keeps
historical cards; cancelled navigation/account/lifecycle work cannot restore late
reports. A failed local reload retains the reload requirement without repeating
login. Card taps use existing descriptor-driven Core state reads; modern controls
and history remain unavailable. QR connection wording supports both native
versions. A shared home-header title now wraps within its available width instead
of overflowing at 320 pixels and doubled English text.

The affected 134 Flutter account/native/QR/localization/foreground-read tests
passed, including 20 new ABI/widget/controller cases. They cover actual fictional
C libraries for 1.35/1.36/1.37 and an App QR-to-card click/state-read path, plus
partial/failed/empty results, malformed/private/oversized results, owned-memory
cleanup, cancelled ready replies and local reload failure. These tests use no
provider or appliance. Both shared portrait/landscape reflow tests, Flutter
analysis and all seven presentation tests also passed. Native API 1.37 and protected storage formats are
unchanged. New Android packaging and real account/phone/model acceptance remain
pending; the old API 1.35 APK does not contain these changes.

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
