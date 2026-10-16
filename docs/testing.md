# Test plan and quality gates

## Local verification

For an isolated Python environment, run `python3 -m venv .venv` and `source .venv/bin/activate` first. Install the declared requirements below before running Python tests.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
python3 -m pip install -r requirements-dev.txt
python3 scripts/verify_mvp.py
python3 scripts/verify_native_contract.py
node packages/presentation/test/i18n.test.mjs
python3 scripts/validate_docs.py
python3 scripts/validate_contracts.py
python3 scripts/validate_repository.py
python3 scripts/validate_commits.py --all
python3 -m unittest discover -s tests -v
```

After dependencies are cached, add `--offline` to Cargo commands. Use a Python virtual environment when system package installation is restricted. `requirements-dev.txt` explicitly includes RFC 3339 validation; contract and runtime checks fail if `date-time` checking is unavailable rather than silently ignoring timestamp formats. The runtime acceptance script currently targets Linux (`libhomeverb_ffi.so`) and uses temporary fictional profiles. Its SIGINT case requires ordinary OS signal delivery. A sandboxed failure must be distinguished from an actual run outside the sandbox; never silently skip that case.

The synthetic `homeverb-host --test meiju_accounts` suite uses paused Tokio time.
Its one-second operation budgets must model the tested deadline, not the runner's
scheduling or SQLite fixture setup latency. Deadline cases explicitly advance the
clock; cancellation, backgrounding and dropped operations retain separate checks.
Recovery setup asserts complete initial retrieval before marking the account
pending. Real TLS and subprocess runtime suites continue to use real time.

## Coverage

Flutter development uses FVM and the version pin in `apps/mobile/.fvmrc`. Build `homeverb-ffi` first, then run from `apps/mobile`:

```sh
fvm install
fvm flutter pub get
fvm flutter analyze
fvm flutter test --reporter expanded
fvm flutter test integration_test/native_smoke_test.dart -d linux
fvm flutter build linux --release
fvm flutter build web -t lib/main_preview.dart
```

The Linux integration can run under Xvfb on a headless host and requires Flutter's Linux development dependencies. Native library packaging is handled by the Linux CMake install step. From the repository root, `python3 apps/mobile/tool/compare_screens.py --require-exact` compares all 82 captures to the untouched prototype and fails on any pixel difference. See [Flutter verification](../apps/mobile/verification.md) for actual results and unverified phone/localization/accessibility gates. Simulated preview branch clicks do not establish real provider behavior.

| Layer | Evidence |
|---|---|
| Core integration | Real SQLite, invalid bounds/steps/enums, grants, hard routing, stale evidence, lost ACK, validation-inclusive and inherited host deadlines, cancellation, persisted/retired request IDs, locks, and composite results preserved across journal/cache failures |
| Descriptor admission | Complete per-adapter validation before saving, schema versions/shapes, unique IDs and references, locally compilable action schemas, foreign connections, ambiguous property names, preserved metadata, and CLI operational refusal followed by valid rediscovery |
| Component scopes | Shared legacy resolution, distinct compartment ranges/steps/freshness, independent grants/risk/routes, verified writes and cache isolation, null/type refusal, duplicate/missing/mixed scopes, invalid stored metadata, rediscovery preservation, and actual CLI scoped descriptors/cache |
| Action decomposition | Explicit independent-setter opt-in, full-argument delivery without it across idempotency categories, lost acknowledgement without field loss or replay, invalid combinations rejected before sends, legacy CLI metadata and rediscovery, and preserved composite policy/results |
| Rediscovery identity | Adapter takeover and each binding-field change rejected, whole-response preflight, metadata updates and reordered connections allowed, preferences/cache/history preserved, competing SQLite inserts, and actual CLI conflict output |
| Receipt lookup | No adapter or journal changes, lookup with invalid metadata/no write grants/device lock, active records outside recent history, exact and long child IDs, missing/retired/blank IDs, cancellation/deadline refusal, and CLI/native/MCP recovery queries |
| Property freshness | Deterministic per-property expiry and equality boundaries, legacy defaults, integer-valued floats and JSON-safe bounds, invalid descriptor limits, relaxed limits with retained stale flags, post-send evidence, and CLI mixed-limit cache reads without snapshot mutation |
| Correlated completion | Non-idempotent actions without observable state, exact identity/connection binding, absent/stale/future/reversed timestamps, accepted-only and method gating, rejection/uncertainty, cancellation/deadline after acceptance, journal failures, and CLI composite/restart evidence without fabricated snapshots |
| Device batches | Whole-list validation/grants/request conflicts, duplicate devices and composite child-ID collisions, four-device concurrency bound, ordered and mixed results, independent failures, shared deadlines/queued cancellation, individual replay, CLI exit/schema checks, and native foreground cancellation |
| Identifier boundaries | Maximum-length ASCII/UTF-8 caller IDs, composite planning and persisted child receipts, offline replay across CLI processes, oversize rejection, long-device writes, cross-instance lock contention, and fixed legacy/digest lock filenames |
| Observations | Full-snapshot version/identity/source/time and component property validation, type/enum/range/step failures, exact numeric target matching across integer/float representations and integer limits, unreadable/ambiguous metadata, read-only vendor values, old-source freshness, preserved cache/acceptance, empty state, clock rollback, cached cancellation, and actual CLI rejection of invalid cache records |
| Organization | Room/name validation, preference persistence, exact-ID permutations, atomic batch room assignment, preserved preferences, unassignment, alias ambiguity, device-history filtering before limits, and no permission escalation |
| Write policy | Component isolation, argument limits, complete-grant matching, granted route intersection/fallback, malformed schemas, composite step authorization, and unchanged high-risk denial |
| Storage | WAL-aware schema-1/2/3/4 backup/upgrade to schema 5, rollback, newer-version refusal, exclusive migration access, detailed receipt pruning, active-write protection, and orphan recovery without replay |
| Local-data cleanup | Real SQLite/Core and C ABI cache/history clearing, preserved metadata, cache miss without reads, active composite journal refusal across store instances, rollback, cancellation/deadline/foreground checks, retired request IDs and reopen without replay |
| Native credentials | Opaque references, nonserializable/redacted secret buffers, unavailable/locked/duplicate error mapping, preflight cancellation/deadline refusal, and uncertain mutations; explicit Linux service probe below |
| Account vault | Persistent profile/account references, profile isolation, lease contention and latest-session rereading, pending/stored transitions, recovery without rewriting, unavailable storage, cancellation, injected SQLite update failures, and absence of session markers in SQLite/WAL; synthetic secrets only |
| CLI runtime | Actual binary output validated against JSON Schemas, exit codes, room/preferences and state persistence, batch rollback after an injected second-device update failure, filtered history, schema-1 upgrade, injected SQLite journal failure, dry run, SIGINT, and forced termination/restart |
| Meiju CLI | Actual binary empty-account discovery, metadata rename/reopen, native response-schema validation, nested refresh/recovery/removal errors and exit codes, exact cleanup scope, isolated adapter defaults, cached cards/receipts, unavailable-keyring preflight and refusal of ungranted writes/Mock/credential arguments and command-only MCP options; isolated PTY tests cover hidden bounded input, cancellation, echo restoration and queued-input clearing. No live CLI login or provider request |
| Meiju CLI grants | Strict schema/native-list parity, bounded file reads, final symlink/FIFO/directory/permission refusal, complete validation before profile creation, explicit request IDs, actual CLI exact-device/state/route plans, independent fan revocation, batch preflight without receipts, pending-account not-sent execution, replay/conflict and receipt lookup without grants; fictional profiles only |
| Android credential storage | Production Java compilation against official API stubs, 54 synthetic encrypted-entry assertions (round trips, randomized ciphertext, profile/reference binding, tampering, key loss, bounded data, locks, symlinks and deletion), and an explicitly run JVM/JNI test for array clearing, status mapping, cancellation/deadlines and uncertain writes; separate emulator evidence below, no physical phone test |
| Android native library | API 28 arm64/x86_64 and combined AAR cross-builds with NDK r28c, JDK 21 and API 35 stubs; production-only archives, seventeen native exports per ABI and 16 KiB ELF alignment checked; six package admission/failure tests; separate test APK evidence below, no release-shrinking test |
| Trusted user approval | Fictional elevated action; exact immutable scope, complete grants, one explicit connection, expiry/wrong Core/metadata/invalidation refusal, nonserializable one-use preview, adapter marker guard despite weakened risk metadata, journal failure/retired IDs/replay, host background/close/cancel and in-flight expiry; ordinary JSON cannot approve and no real appliance action is admitted |
| Meiju temperature control | Protected selector priority and exclusions, normalized categories, whole/half-degree constraints, exact temperature-only encoding, retained evidence, readable historical out-of-range values, descriptor/model/identity/membership refusals, exact-target native/CLI/MCP authority, receipts/replay and foreground cancellation; fictional profiles only, no physical controls |
| Meiju mode activation | Four candidate AC models and five modes, exact combined power-on/mode body, no temperature/fan/cache changes, protected-target and weakened-metadata refusal, independent device/mode/cloud policy, native startup/schema checks, receipts and lifecycle cancellation, CLI/MCP mode-only plans and pre-send failures; fictional profiles only, no physical controls |
| Meiju vendor fan control | Four candidate AC models and six codes, exact fan-only bodies, no implicit power/mode/temperature, descriptor/argument/protected-binding guards, independent exact-code startup policy, accepted/unknown/rejected receipts, no cache changes or replay, lifecycle cancellation and actual Linux ABI/schema startup checks; no physical control |
| Native ABI | Linux library loaded directly, Mock discovery/control, room/card requests, concurrent foreground cancellation and close; Meiju empty-account startup/discovery and invalid mode/grant/scenario refusal without vendor calls. Live read-only evidence is recorded in the native handoff |
| Midea LAN discovery decoder | Independent fictional V2/V3 vectors, AC/E1/CA/unknown hints, minimum/maximum datagrams, all truncation boundaries, checksum/ciphertext corruption, re-signed invalid lengths, padding, ID/port/SSID refusal, ignored reserved ID bytes, actual-source endpoint selection, redacted Debug and compile-fail serialization; no sockets, account access or device admission |
| Midea LAN discovery collector | Fictional loopback UDP peers, exact one-shot probes, selected-subnet/source admission, oversized/invalid responses, duplicate/conflicting hints, candidate/datagram limits, fixed window, inherited cancellation/deadline and socket rebinding after completion/drop; no household traffic or registered LAN route |
| Midea V3 wire codec | Independent OpenSSL/hashlib handshake and encrypted frames, both directions, aligned/unaligned payloads, wrong key/digest/type/length/padding refusal, all truncations, prefix/coalescing bounds, counter exhaustion, redaction and compile-fail handshake/session serialization; no TCP connection, credentials or appliance access |
| Midea V3 TCP exchange | Fictional loopback peer, exact handshake/request bytes, fragmented/coalesced frames, scoped-target/payload/progress preflight, connection/handshake refusal, lost/corrupt replies, cancellation/deadline/future-drop socket closure before and after command send; valid responses never mark acceptance; no real credentials or appliance access |
| Meiju LAN credentials | Scripted fictional cloud with the real vault/SQLite, independent UDP-ID vectors, exact v2 body and selected method, protected binding/session/membership/scope preflight, unique complete home resolution, model/product conflicts, token selection/hex/length/size limits, sanitized errors, cancellation/deadline/drop at every request, unchanged storage and no fallback or LAN traffic |
| Midea LAN packet envelope | Independent OpenSSL/hashlib packets, exact encoding, strict size/type/checksum/PKCS#7 decoding, all truncations, coalesced prefixes, expected six-byte identity and reserved bytes, timestamp/input bounds, maximum V3 fit and independent nested response; no socket, model projection or write outcome |
| Midea AC LAN query/C0 candidates | Independent AA frames and the full upstream CRC table, exact query encoding and explicit sequences, upstream C0 comparison, power/raw codes, signed half/decimal temperatures, unavailable sensors, body-size/units/type/version/checksum refusal, all truncations, outer-packet identity chain and redacted/nonserializable output; no model admission, freshness or device operation |
| Meiju LAN candidate reconciliation | Fictional protected accounts, exact local binding and membership, account isolation, duplicate/reordered hints, selected-ID family/protocol/endpoint conflicts, shared-endpoint IDs, report limits and invalid peers, cancellation/deadline, unchanged storage/cards and redacted/nonserializable results; no cloud or LAN traffic |
| Trusted Meiju LAN scan | Actual Session/adapter/vault/SQLite with scripted discovery: zero calls during preparation, one confirmed collection for AC/E1/CA, retained lease, fixed-network preview, changed metadata/identity/membership, foreign/expired previews, limits/conflicts/invalid reports, cancellation/background/close/deadline/drop, preview-capped deadlines, unchanged secrets/cards/cloud calls and ordinary JSON refusal. No household traffic or mobile permission acceptance |
| Composed AC V3 read | Source-checked appliance query/protocol vectors, independent nested replies and a fictional loopback peer; one handshake and two ordered queries, negotiated message version, separate packet timestamps, both reply identities, malformed/unsupported responses, no partial result/reconnect, preflight refusal and cancellation/deadline/drop at either query; no registered route or physical support |
| Standard E1/CA V3 reads | Independent AA and nested packet vectors, pinned-source field comparison, flag polarity, raw codes, time-extension boundaries and signed/fractional Celsius; strict required prefixes without default values, all truncations/corruption, unsupported kinds/units/layouts, two negotiated loopback conversations, both reply identities and cancellation/deadline/drop without reconnect; no Core Snapshot or model admission |
| Native UI contract | All 34 published methods called through the built C ABI, offline method-specific response validation, saved examples, closed-field rejection, null success, admission errors, all execution outcomes, explicit cancellation and foreground/closed-session refusal |
| Saved device conflict inspection | Fictional accounts with actual vault/SQLite and Session; unpublished/historical/unconfirmed bindings, exact identity matching, stable ordering and reopening, no provider calls or mutations, foreign account exclusion, 64-account limit, unavailable/corrupt/busy scope refusal, cancellation/lifecycle/drop and lease release; C ABI admission and closed schemas |
| Trusted connection-link bridge | Fictional opted-in Session adapter with real Core/SQLite; schema-valid immutable scope/grants, preserved metadata, capacity/expiry, session and consent-kind isolation, one-use failure/concurrency, cancellation/lifecycle/drop and ordinary JSON refusal. Actual C exports verify production Mock/Meiju refusal without credentials or networking |
| Meiju authentication/HTTPS | Fictional login/signing/session vectors, private input/session validation, exact request bodies and account binding, code-first rejection at both login endpoints regardless of challenge data, strict successful payloads and bounded object envelopes, no retries, cancellation/deadlines, redaction and local TLS certificate/redirect/body-limit/socket-closure checks; no real account test |
| Meiju inventory | Synthetic code/data envelopes, nested homes/appliances, candidate and unknown families, deduplication/conflicts, account isolation, malformed/partial/auth-expired responses, shared deadline/cancellation, no retries, and redacted native data; synthetic protocol evidence, not a device claim |
| Meiju state | Synthetic AC numeric/compound and protected-profile scalar temperatures, retained read mappings, malformed-response cache preservation, fridge/freezer separation, vendor-specific dishwasher/AC codes, exact account/appliance/query routing, null/absent readings, malformed/oversized/rejected responses, unproven freshness, cancellation/deadlines, no retries, redaction and nonserializable source state; no live/model acceptance |
| Native account connection | Synthetic login-to-card publication without duplicate inventory, schema-validated summaries/results, same-subject reauthorization and preferences, partial/expired retrieval, saved-session refresh, uncertain binding storage, unavailable storage, wrong-subject/cross-account conflicts, shared operation IDs, cancellation/deadline/background/close and lease release; C ABI caller-buffer clearing and malformed/mode/closed refusal |
| Saved account inventory inspection | Actual synthetic host responses for listed/not-listed/unconfirmed and unpublished bindings, exact account/region isolation, no secret/network mutation, legacy preservation, redaction, unavailable/corrupt/pending/removing/busy errors, cancellation/deadline/background/close/drop and lease release; Linux ABI admission and closed request/response counterexamples |
| Local account metadata | Synthetic host and C ABI detail/rename calls, account/provider isolation, one ID per linked card, unsupported-card inclusion, unavailable secrets, pending-state preservation, lease contention, invalid names, injected SQLite rename failure and reopening; no live account edits |
| Pending Meiju reauthorization | Synthetic protected login with explicit account selection, expired saved-session recovery refusal, same-subject replacement, preserved bindings/preferences/grants, changed/missing/corrupt material, wrong identity, rejected login with unchanged new/stored/pending account state and explicit retry, uncertain persistence, SQLite confirmation failure, partial retrieval and lifecycle/lease cleanup; C ABI rejects forged identity/recovery fields and clears caller buffers |
| Meiju saved-session recovery | Synthetic host validation after uncertain initial/map writes, stable IDs/preferences, exact-byte confirmation without another login, partial/expired reads, corrupt/foreign/missing material, changed-byte races, SQLite failures, cancellation/deadline/background/close/drop and lease release; actual ABI admission without a keyring |
| Account removal | Exact scope and provider/route isolation, unreadable credential cleanup without secret reads, durable intent, unavailable/locked/conflicting/uncertain deletion, explicit retry without duplicate erasure, cancellation/background/close/dropped futures, schema-3 backup, rollback, late publication/cache refusal, retained receipts and rooms; ABI admission and completed-ledger replay without a keyring |
| Meiju connection | Protected session persistence followed by automatic inventory, same-subject reconnect across reopen, separate failed/partial retrieval, explicit refresh without login, changed-subject and foreign-region refusal, unavailable storage and unselected pending accounts before login, profile registration lock and cancellation release; synthetic account evidence only |
| Account inventory timing | Complete/empty retrieval, null initial evidence, unchanged secret bytes, partial/expired/login refusal, uncertain bindings, metadata failure after protected persistence, explicit recovery, cancellation/deadline/background/close/drop, local list/details without secrets, reopen/rename, schema-4 backup/upgrade, clock rollback, corrupt metadata and removal; fictional accounts only |
| Meiju product evidence | Synthetic optional/placeholder fields, canonical subtype parsing, malformed-home refusal, cross-home and stored conflicts, whole-update preservation, version-1/2 read-only open and version-3 update, unchanged local IDs/preferences, reauthorization/recovery, uncertain stores, redacted debug and nonserializable source profiles, and absent markers in native results/SQLite; no live account access |
| Meiju inventory membership | Synthetic full/partial/failed retrieval, legacy payload migration, disappearance/reappearance, preserved IDs/cards/cache/receipts, stale descriptor publication, uncertain persistence and native read/control refusal before transport; no owner profile or appliance access |
| Meiju private bindings | Real SQLite with synthetic protected storage; stable device/connection/identity IDs across reopen and same-subject session replacement, preserved local preferences, partial/empty inventory retention, whole-update conflict refusal, account isolation, corrupt/missing payload refusal, no plaintext identifiers, uncertain-write recovery without regeneration, size limits and cancellation |
| Meiju bound Core reads | Test-only AC/E1/CA descriptors admitted by Core, exact protected connection/model/component preflight, stale cloud values and separate fridge/freezer caches, malformed/undeclared response refusal without cache replacement, cached reads without secret access, blocked writes, shared cancellation/deadline and account-lease release |
| Native power grants | Closed startup schema and C ABI admission/refusal before profile creation, exact canonical UUID/state/cloud grants, unchanged elevated-risk denial, synthetic host plan/execute/replay/receipt, read-only reopen, foreground cancellation/close and lease release; no real appliance operations |
| Meiju AC cloud control | Synthetic exact-model power descriptors, explicit admission and device/state/cloud grants, protected binding/model refusal, code-zero acceptance without verification, authorization rejection, ambiguous provider errors, lost replies, unsent/sent cancellation/deadlines, unchanged cache and replay across Core reopen; local TLS tests track send phases and socket closure without redirects/retries. No physical controls |
| Meiju model adapter | Exact six-model read admission through Core, unsupported visibility, protected model/family checks, partial/empty rediscovery and preferences, historical cross-account conflict refusal, cache preservation, local queries without secrets, cancellation/deadline and lease release; live read-only evidence recorded separately |
| MCP runtime | Actual Mock/Meiju stdio startup, four default/six granted tools, embedded Command-schema parity, local descriptions/cache/receipts, pending-account pre-send refusal, immutable session policy, independent exact-device/value/route grants, replay, default denial and forged fields, no Internet sockets while idle, EOF/SIGINT and output failure/backpressure exit. Fictional adapter tests cover all execution outcomes, command deadlines, cancellation/EOF/signal/output failure after send and persisted unknown receipts |
| Presentation | Exact locale key/placeholder parity, locale fallback, missing keys, unknown observations, and uncertainty display |
| Diagnostics | Actual CLI/native closed-schema reports, omitted private markers from metadata/receipts/nested items, normalized unknown codes, fixed storage errors, bounded receipt/attempt output, no adapter calls, and no journal mutation |
| Repository | Markdown targets, fixtures, English policy with a narrow Chinese catalog exception, and commit conventions |

The separate [Android APK procedure](development/08-android-native.md#run-the-isolated-native-acceptance-apk)
passed 77 assertions across four process invocations on AOSP Android 15 x86_64:
Core/Mock and lifecycle behavior, migration/device/account locks, real Keystore
encryption, process persistence, lock refusal, tampering, deliberately removed
keys, JNI inventory reads and local removal. The runner rejects physical devices
and requires explicit instrumentation success; four Python tests cover its refusal
and false-success paths. This procedure is not part of ordinary workspace tests
and does not use provider accounts or networking.

For native API 1.19, the rebuilt isolated APK additionally passed schema-4 to
schema-5 account migration and historical inventory-time retention on Android.
The run passed 28 unsecured, 19 network and 93 control assertions; secure-lock
phases were not repeated. See the Android guide for the exact evidence boundary.

The [MVP workflow](../.github/workflows/mvp.yml) runs the Linux build/runtime checks
and a separate Android native job. The latter uses pinned SDK/NDK/JDK/Rust versions
for synthetic JVM/JNI storage tests and arm64/x86_64/combined AAR inspection; see
the [Android build checks](development/08-android-native.md#continuous-build-checks).
It does not run Android OS, Flutter or physical-device acceptance. A local run is
not evidence that remote CI ran; inspect the exact commit's job results.

JSON Schema validation uses `jsonschema` 0.45.1 or a compatible patch, with default network-resolution features disabled and the resolved version in `Cargo.lock`. This version includes the [upstream correction for negative `multipleOf` values](https://github.com/Stranger6667/jsonschema/blob/master/CHANGELOG.md). Regression coverage exercises negative fractional temperatures through device constraints, trusted grants, writes, and live/cached observations.

## iOS credential backend checks

`cargo test -p homeverb-secrets --offline --locked ios::` runs portable tests for
explicit namespaces, missing/duplicate/foreign/protection refusal, exact add/read/
replace/delete, copy bounds, OS error mapping, pre-dispatch cancellation/deadlines
and uncertain dispatched mutations without retry. On macOS, the same command also
runs five production Core Foundation/Security query/result tests with fictional objects;
it does not access Keychain. Linux cannot execute those framework-object tests.

The `ios-native` CI job checks the arm64 targets and builds a development
XCFramework using Apple's SDKs and a fictional access-group build value. It
contains arm64 device and arm64/x86_64 Simulator release frameworks with local
ad-hoc seals. `scripts/build_ios_native.py` verifies the extracted package's
platforms, minimum OS versions, dependencies, install names, defined exports,
metadata, canonical header/module map and signatures, then links C consumers for
all three architectures. Eight portable package tests cover wrong slices,
undefined symbols, desktop dependencies, malformed metadata, stale headers,
symlinks/extra fixtures and preservation of prior output after a failed check.
This is not distribution signing or a Keychain/app runtime test. Follow the
[iOS guide](development/09-ios-native.md) for explicit configuration and the
remaining passcode/lock, lifetime, reinstall, backup and Flutter/iPhone gates.

The separate `scripts/verify_ios_acceptance.py` step builds an isolated native app
with the production framework and a Simulator-only Rust secret-store helper. It
creates a new compatible Simulator, verifies embedded Simulator entitlements and
the plain ad-hoc seal, and requires
fresh successful write/read reports with different launch PIDs. Mock Core, SQLite
cache/receipt persistence, explicit lifecycle calls and fictional system Keychain
round trips are exercised; no provider account or appliance is accessed. Twelve
portable runner tests refuse stale/failed/incomplete reports, wrong process/scope,
missing runtime evidence and unsafe simulator selection/cleanup. They do not prove
actual Apple runtime behavior. The exact CI job must pass before claiming it.
Simulator results do not establish physical passcode/lock or signed-device policy.
On `d02b10a`, macOS job `116902983930` passed eighteen object/orchestration tests,
all package/consumer checks and actual iOS 26.2 execution: 40 Core/10 Keychain
checks in the writer, then 41 Core/four Keychain checks in a different process.

## Linux Secret Service probe

This opt-in integration test needs an existing unlocked default Secret Service collection and permission to access the user's D-Bus. It creates only synthetic bytes under fresh HomeVerb namespaces, checks replacement and isolation across instances/profiles, deletes those entries, verifies absence, and checks bounded socket cleanup. It does not enumerate or read existing account secrets. Default `cargo test` ignores it.

```bash
cargo test -p homeverb-secrets --offline --locked --test system_store linux_secret_service_round_trip -- --ignored --exact
```

The probe passed locally on Linux with GNOME Secret Service. Locked collections, interactive service prompts, other implementations, and actual platform credentials were not integration-tested. Unit tests separately exercise sanitized locked/unavailable errors and deadline/cancellation outcomes without keyring access. The test cleans up on returned operation failures; forced process termination can still leave synthetic entries, just as it can interrupt an ordinary credential mutation.

## Meiju native protocol and HTTPS tests

`cargo test -p homeverb-midea --offline --locked` also starts generated local TLS
fixtures. A sandbox that prohibits loopback listeners cannot run those cases;
record the restriction and run with ordinary local socket access, rather than
skipping certificate, redirect, body-limit or cancellation tests. All credentials
in these tests are fictional. The known-answer fixture uses independent Python
hashlib/HMAC and OpenSSL calculations; no test contacts a vendor endpoint.

## Midea account and remote-control acceptance

These are required future checks, not results from the Mock suite. Follow the [integration guide](development/05-midea.md) and record model/firmware/app/region with sanitized evidence.

- Connect an existing Meiju account and automatically retrieve bound devices across accessible homes. Require neither re-pairing nor manual IP/Token entry. Distinguish unsupported, offline, empty, partial, and failed retrieval; never infer fresh state from device-list membership.
- Repeat retrieval and reauthorization without duplicate cards or loss of aliases, rooms, favorites, or established bindings. Test account/region isolation, ambiguous identity, partial-list preservation, credential expiry/refresh, and unavailable secure storage.
- Read and perform explicitly owner-authorized supported writes through cloud with the phone's Wi-Fi disabled, the appliances still connected to Midea, and no home computer/gateway. Repeat on an external network without LAN discovery or local-network permission. Verify each claimed capability independently.
- Exercise stale cloud state, rejection, lost acknowledgements, cancellation/backgrounding, and forced termination. Preserve accepted/unknown outcomes and do not replay uncertain writes, including across routes. After completion or cancellation, leave no polling or background networking.

## Trusted native approval checks

The normal native contract suite also runs
[verify_native_approval.py](../scripts/verify_native_approval.py). It exercises all
three trusted C exports with temporary fictional Mock profiles: exact preview
scope, ordinary denial, consumption/replay, concurrent confirmation, cross-handle
isolation, capacity/dismissal, closed fields, null input, metadata/risk tampering,
background and close. No real account or device is accessed.
`cargo test -p homeverb-host --test user_approval --offline --locked` additionally
covers monotonic expiry/pruning, admission failure, and a deterministic barrier-
controlled background/foreground preparation race. The same tests cover typed
approval send uncertainty and cancellation; no sleep-based expiry test is needed.

## Profile-scoped dishwasher controls

The native contract suite also runs
[verify_native_dishwasher.py](../scripts/verify_native_dishwasher.py). Startup
schema/runtime checks reject invalid grant lists before profile creation, then
use a fictional descriptor with no account to test exact grant scope, ordinary
execution denial and trusted confirmation admission ending safely before any
provider access. Rust adapter tests exercise every admitted program/encoding,
explicit additional options, protected profile/model/family guards despite public
metadata tampering, no implicit reads/cache writes, receipt replay, membership,
cancellation and cloud outcomes. No actual account or appliance is used.

## Profile-scoped dishwasher cycle controls

The same native dishwasher suite verifies independent pause/resume/cancel startup
schema/runtime admission, rejected malformed grants before profile creation,
ordinary denial and separate trusted C confirmation using fictional descriptors
with no account. Rust tests compare the twelve original product/action payloads and fifteen
explicit additional-option payloads, preserve cache and receipts, reject
unreviewed centralized or forged profiles and malformed required options, independently enforce
approval despite public risk changes, and cover pre-send/rejected/unknown outcomes,
replay, membership changes, cancellation/deadline and lease cleanup. Optional
operator observations are raw/stale; malformed recognized values fail decoding.
These checks establish no physical cycle behavior or owner-product match.

## Read-only control-option inspection

Native API 1.16 control-option tests exercise read-only Session inspection through
the actual account vault/SQLite with fictional accounts. They compare reviewed
action definitions to enabled descriptors, preserve exact temperature/program
domains, reject identity/membership/storage failures, retain read-only denial,
recheck metadata after an await and release leases on cancellation/background/
close/deadline/drop. Native C/schema checks cover the new closed method, errors
and fictional response fixtures; no provider or appliance is accessed.

## Current session-grant inspection

API 1.17 tests retain complete rules, exact device/component/action/transport
scopes and valid boolean schemas; invalid schemas confer no authority. Fictional
Session tests reopen read-only with retained writable descriptors and receipts,
refuse secret access and hold the account lease while proving inspection needs
neither. Core risk/capability gates, metadata, receipts and provider calls remain
unchanged. Actual C checks inspect Mock and fictional Meiju startup scopes, revoke
on reopen without discovery, reject request/response authority injection and
unknown IDs/connections, and exercise foreground/closed handles. No real account,
provider, appliance or Flutter worktree is used.

## Android native control flow

The [isolated Android acceptance APK](development/08-android-native.md#exercise-native-controls-without-a-provider)
adds a standalone `controls` phase. It exercises the production C approval exports
and session-grant query on Android, including read-only reopen with retained
actions, ordinary elevated denial, cross-handle/consumed/dismissed previews,
background/close invalidation and accepted receipt lookup. A separate simulated
write reaches its state commit before background cancellation, retains `unknown`
across reopen and returns the same receipt without reapplying the command after
a subsequent state change. Test-only JNI code remains outside production AARs;
there is no Internet permission or provider account. Flutter consent and physical-phone behavior are not inferred from this test.
The separate force-stop phase below tests process recovery on the emulator.

## Android forced-stop recovery

The [isolated force-stop phase](development/08-android-native.md#verify-a-forced-process-stop-after-transmission)
requires matching run/PID evidence from a still-live fictional write, force-stops
only its test APK, verifies process absence, then starts another process. Recovery
checks the unfinalized journal before Core opens, unchanged `unknown` results,
read-only receipt access, released locks, no replay after a later state change,
and request-ID retirement after history clearing. The host treats missing/stale
markers, completed observers and unsuccessful process-state checks as failures;
no exit code or readiness file alone establishes termination. JSON receipts are
compared structurally with every field retained. No Internet permission or
provider account is present, and this phase does not establish phone/OEM behavior.

## Unverified release gates

Physical-device authentication and control, provider account recovery/token rotation,
physical Android/iOS secure-store policy and Flutter phone journeys, distribution
signing/installation, a target AI host, dependency/security review and licensing
remain pending. Isolated Android 15 and iOS 26.2 native apps and their development
packages have runtime/build evidence; they do not establish phone/provider
acceptance. Real writes require explicit device-owner authorization. Two different
protocol families must demonstrate equivalent capability calls before cross-platform
support is claimed.

## Trusted native scan bridge

`cargo test -p homeverb-host --test meiju_accounts lan_scan --offline --locked`
exercises the typed and native Session paths with fictional account/discovery
transports. Native cases cover all four outcomes, immutable scope, schema-valid
outputs, cross-session and consent-kind isolation, single consumption including
concurrent confirmation/refusal, capacity/expiry, background/close, preparation
races, cancellation/deadline/drop and lease release. No household network is used.

`python3 scripts/verify_native_contract.py` includes the three actual C export
parsing/admission checks on empty temporary profiles, plus closed request/result
schema counterexamples. Successful scans use the injected Session tests; the C
suite does not access a provider account or perform UDP discovery. Android AAR
inspection now requires all 16 C functions and credential JNI initialization.
Platform interface/permission selection, Flutter consent and physical acceptance
remain pending.

## Android assigned-interface selection

`scripts/verify_android_store.py` now also compiles and runs
`NativeLanNetworksTest` against the production helper, using fictional metadata
and a monotonic test clock. It covers address/transport eligibility, ambiguous
assignments, exact revalidation, selection ownership, expiry, lifecycle races,
fixed errors and redacted display. The existing credential JVM/JNI checks remain.

The isolated APK's `networks` phase reads actual disposable-emulator connectivity
metadata and exercises selection/lifecycle behavior. It requires a provisioned
virtual interface and checks that `INTERNET` is denied. The builder requires
exactly `ACCESS_NETWORK_STATE`; Python counterexamples reject extra, duplicate,
missing or differently scoped permissions. This does not test appliance traffic,
Android 16/17 socket authorization, Flutter controller wiring or physical phones.

## Trusted native connection-link bridge

`cargo test -p homeverb-host --test connection_link --offline --locked` exercises
the typed operation and session-owned native previews with a fictional opted-in
adapter and real SQLite. It validates immutable descriptor/connection/grant scope,
preserved metadata, consuming failure, capacity, expiry, session isolation and
lifecycle/concurrency boundaries. Scan tests also reject scan/approval IDs in the
link registry without consuming their original scope.

`python3 scripts/verify_native_connection_link.py` loads the actual Linux C exports
and checks closed metadata, ordinary-dispatch refusal and default Mock/Meiju
adapter refusal with fictional local descriptors. It is included by
`verify_native_contract.py`; neither path obtains credentials or opens appliance
connections. Successful links are tested at the injected Session boundary.
Android AAR inspection requires these symbols but does not prove phone behavior.
