# Test plan and quality gates

Native Mijia tests in `crates/host/tests/mijia_accounts.rs` use temporary SQLite,
fictional secure storage and an encrypted in-process cloud peer. They check
protected connection, named unsupported cards, partial positive IDs, actual power
independent of online hints, provider routing/isolation, local removal replay,
background cancellation and no automatic login replay. The C ABI verifier checks
new startup modes, closed configuration, account scope, buffer clearing and
foreground refusal without vendor traffic.

## Mijia protocol checks

`cargo test -p homeverb-mijia --offline --locked` uses fictional credentials and
local generated TLS peers. Loopback socket access is required; it never contacts
Xiaomi or an appliance. Signing has an independent Python known-answer fixture.
Login/challenge, endpoint admission, encrypted errors, inventory identity,
property correlation and cancellation/deadline/drop are covered. See the
[phase record](development/11-mijia.md) for executed results and pending Core/App
and owner-run physical acceptance.

The expanded library suite also covers bounded home-page unions, private identity
conflicts, pagination cycles and partial failures, exact released MIoT model
selection, readable fan/doorbell/lock/battery mapping and public-spec credential
isolation. No lock action or physical support follows from those checks. Typed event history
is covered separately under ADR-0115 below.

Protected account/Core tests use actual temporary SQLite and an injected fictional
secret store and cloud. They exercise stable IDs/preferences, same-subject reconnect,
unchanged-secret refresh, uncertain-save recovery, exact copied-payload refusal,
cross-account conflicts, metadata corruption, placeholders, partial inventory,
cloud value/cache checks, cancellation/drop lease cleanup, removal and independent
adapter write denial. `cargo test -p homeverb-host --test mijia_startup --offline --locked`
checks empty production Rust composition without credential or provider access.

## Modern exact control checks

`cargo +1.96.0 test -p homeverb-mijia --offline --locked` includes ADR-0104's
continuous-codec property setter, local TLS failure/lifecycle checks, every exact
mapped scalar domain, immutable outlet approval, independent protected admission
and cross-protocol receipt replay prevention. Host integration is covered by
`cargo +1.96.0 test -p homeverb-host --test mijia_accounts --offline --locked modern_host::controls`.
These tests use fictional accounts and no appliance or provider requests.

From apps/mobile, run the affected existing generic UI flows:

```sh
fvm flutter test test/mijia_controls_test.dart test/mijia_settings_test.dart test/mijia_outlet_test.dart test/mijia_auxiliary_test.dart test/session_permissions_test.dart test/approval_test.dart test/device_detail_controls_test.dart test/numeric_history_test.dart test/modern_device_flow_test.dart
```

The suite covers permission preparation, ordinary/elevated controls, consumed or
invalidated approvals, unknown receipts, explicit history and modern card reads.
Passing UI fixtures does not establish a provider acknowledgement or physical effect.

## Modern protected device checks

`cargo test -p homeverb-mijia --offline --locked modern_accounts::devices` uses
actual Core/SQLite with fictional encrypted modern replies and an in-memory secret
store. It checks all pinned scalar mappings, selected-protocol isolation, stable
IDs/organization, payload-3 corruption, split-child identity, stale write denial,
uncertain save/recovery and cancellation/drop. It performs no provider traffic or
physical actuation. Run the broader workspace and native ABI checks after changing
protected payload admission. Host/App publication and real acceptance are separate.

## Modern directory grouping checks

`cargo test -p homeverb-mijia --offline --locked reconciliation` covers the local
kind/parent partition, gateway versus logical-child semantics, duplicate/foreign
identity, cycles, malformed wire metadata, 4096-entry bounds, cancellation and
unchanged request counts. Full Mijia tests include opaque-type serialization
refusal and existing account/authentication reads. This does not prove Core card
publication or physical-device identity; those remain pending under ADR-0099.

## Foreground QR App checks

Run from `apps/mobile` with the installed FVM SDK. Tests use only fictional
accounts and QR material; no Xiaomi account or appliance is required:

```bash
fvm flutter test test/account_qr_test.dart test/modern_account_bridge_test.dart test/account_verification_bridge_test.dart test/account_captcha_test.dart test/account_verification_test.dart test/account_native_test.dart test/native_test.dart test/accounts_test.dart
fvm flutter analyze
```

On macOS, optionally export the fictional painter output and independently decode
it with the system Core Image QR decoder:

```bash
HOMEVERB_QR_FIXTURE_PNG=/private/tmp/homeverb-qr-fixture.png fvm flutter test test/account_qr_test.dart
swift test/fixtures/verify_qr.swift /private/tmp/homeverb-qr-fixture.png
```

The decoder checks one known fictional payload without printing decoded content.
It does not prove real-provider QR acceptance. UI tests cover explicit continuation,
clearing, expiry, background/navigation, credential readiness, late results and
separate refresh/recover/renew actions. Encoding capacity errors fail closed.

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

After dependencies are cached, add `--offline` to Cargo commands. Use a Python virtual environment when system package installation is restricted. `requirements-dev.txt` explicitly includes RFC 3339 validation; contract and runtime checks fail if `date-time` checking is unavailable rather than silently ignoring timestamp formats. The runtime acceptance script loads Linux (`libhomeverb_ffi.so`) or macOS (`libhomeverb_ffi.dylib`) and uses temporary fictional profiles. Its SIGINT case requires ordinary OS signal delivery. A sandboxed failure must be distinguished from an actual run outside the sandbox; never silently skip that case.

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
round trips are exercised; no provider account or appliance is accessed. Fourteen
portable runner tests refuse stale/failed/incomplete reports, wrong process/scope,
missing runtime evidence and unsafe simulator selection/cleanup, including cold
setup timeouts that must fail without installation or app retries. They do not prove
actual Apple runtime behavior. The exact CI job must pass before claiming it.
The test app's exact API-version assertion is checked against the versioned native
contract so a Core upgrade cannot silently leave a stale Simulator expectation.
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

The Flutter foreground read scheduler is tested independently with virtual time:
`cd apps/mobile` then `fvm flutter test test/state_refresh_test.dart`. Its fictional
fixtures cover three-second cadence, filtered/component targets, silent partial
failure, non-overlap, cancelled late replies, user-command priority, lifecycle
resume and absence of automatic activity UI. These tests do not contact providers
or establish physical timing/rate support. See
[ADR-0067](adr/0067-foreground-state-refresh.md) and the
[App verification record](../apps/mobile/verification.md).

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

The foreground App integration is covered by
`fvm flutter test test/dishwasher_permissions_test.dart test/session_permissions_test.dart test/device_detail_controls_test.dart test/approval_test.dart test/state_refresh_test.dart`
from `apps/mobile`, after building `homeverb-ffi`. It checks exact multi-argument
startup grants, trusted-host gating, immutable arguments, malformed/oversized
schemas, native startup and selected-device preparation without sending commands.
Existing widget/native approval and foreground lifecycle regressions remain part
of this focused suite. No provider credentials or household appliances are used.

The optional [refrigerator encoder research](development/12-appliance-command-evidence.md)
uses `scripts/verify_refrigerator_encoders.py` with a local pinned public checkout
and temporary `lupa==2.8` environment. It asserts 13 encoded vectors and seven
expected missing-input errors; these are source experiments, not model support.

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


## Merged Flutter verification on macOS

Build the workspace library, then follow [the app guide](../apps/mobile/README.md)
for `fvm flutter analyze` and `fvm flutter test --reporter expanded`. Native tests
select the host library suffix. Pixel assertions retain the original Linux
reference geometry; macOS rasterization differences are recorded as failures,
not silently accepted. See [current evidence](../apps/mobile/verification.md).

The CLI runtime harness explicitly skips Linux grant-file admission and pipe-size
checks on macOS. Protected terminal input remains unavailable there. Native C ABI
checks run on either host and never require real credentials. Fourteen LAN Rust
fixtures require assigned loopback aliases `127.0.0.2` and `127.0.1.1` on macOS;
without administrator setup they are blocked, not passing tests. Do not weaken
production subnet/endpoint checks to accommodate an unconfigured test host.


## October 22 Meiju reference checks

`python3 scripts/extract_meiju_reference_facts.py --check` verifies the pinned
archive, reviewed scalar constructor fragments, all four compiled AC catalogs
and the 67-operation coverage ledger without bundled-code execution or network.
The Meiju adapter tests exercise exact-reference accepted/unknown writes, replay
and prerequisite failures plus protected three-zone reads, old descriptors and
malformed-read cache preservation. `homeverb-host`'s `ac_feature_grants` tests and
contract fixtures cover independent Boolean startup permissions.

Flutter's `climate_controls_test.dart` and `refrigerator_modes_test.dart` exercise
all three new switches, retained observations, stale callbacks, the read-only
third zone and English/Simplified Chinese rendering. Run them with the existing
compartment, permissions and localization regressions before packaging. See the
[executed results](../apps/mobile/verification.md). Real appliances are not used
by these checks; the complete-interface blockers remain explicit in the
[integration guide](development/13-meiju-reference-integration.md).


### Opt-in modern Meiju read experiment

[ADR-0075](adr/0075-one-shot-meiju-read-compatibility-probe.md) documents the
owner-requested, read-only existing-session experiment. Set
`HOMEVERB_MEIJU_READ_PROBE=1` only while building the separate development AAR.
The ordinary Flutter entry point uses that AAR without any new JSON method or
request parameter. Each process sends at most one aggregate-inventory read
following a legacy home-list read and one empty-command modern Lua-state read
following a legacy appliance-state read. The environment flag is compiled into
the native library; setting it only for Flutter does not enable the probe.

Run `cargo test -p homeverb-midea --lib cloud::read_probe --offline --locked`
and the shared `cloud::tests` TLS/lifecycle suite. Capture only the closed
`HomeVerbReadProbeInventory` and `HomeVerbReadProbeLua` Android markers. Never
capture ordinary provider logs, raw responses or protected storage. Response
business success does not verify complete device semantics. Preserve ordinary
APK/AAR artifacts and restore them afterward. Actual evidence is recorded in
[App verification](../apps/mobile/verification.md) and the
[closed read evidence](development/meiju-read-probe-evidence.json).

## Exact Mijia reference regression

Run `python3 scripts/extract_mijia_reference_facts.py --check` before the Mijia
suite. Every catalog model is published and its readable components exercised
through Core with fictional identities and values. URN/profile conflicts, unsafe
field exclusion, old mapping upgrade, unchanged authority and cloud staleness are
covered. Flutter `test/mijia_reference_test.dart` checks localized read-only
readings at 320 pixels and double text scale. This is not physical acceptance.


## Protected Mijia phone and email protocol

`cargo test -p homeverb-mijia --offline --locked` includes ADR-0085's fictional
phone/email challenges and local TLS one-send transport tests. Coverage includes
initial-cookie continuity, direct/captcha success, subject correlation, cookie
scope/monotonic expiry, signing-key/token requirements, bounded redirects,
provider-confirmation refusal and cancellation/deadline/drop. Compile-fail tests
prevent serializing challenges. Existing Host account tests and
`python3 scripts/verify_native_account_login.py` retain captcha-only compatibility.
These checks neither exercise a trusted browser nor establish real-account or
modern SID authentication acceptance.


Typed account/Host verification regressions additionally cover lease-free waiting,
captcha transition, selected and initially unselected account removal, changed
protected payloads, foreign adapter/subject, changes during the ticket request,
unchanged device identity, secret exclusion from SQLite, session ownership,
combined old/new registry capacity, wrong answer kind, invalid metadata, expiry,
lifecycle cancellation, no replay and ordinary JSON refusal. Use
`cargo test -p homeverb-mijia -p homeverb-host --offline --locked`; native login
verification continues checking the unchanged captcha-only C bridge.


For API 1.31, build `homeverb-ffi`, run its Rust projection tests and run
`python3 scripts/verify_native_account_verification.py`. The native verifier
checks actual C input/output clearing, invalid output capacity, null inputs,
closed metadata, unknown/closed/background host admission and ordinary JSON
refusal. Rust projection tests verify protected bytes are separate from JSON and
cannot be silently truncated. Retain the old account-login and full native
contract verifiers. No browser, vendor or mobile package is exercised by these
host checks.


ADR-0087 typed code-request checks use fictional method info, quotas, send replies
and local TLS only. They cover exact phone/email forms and GET/POST separation,
quota/method refusal, human-risk rejection, monotonic cooldown and original
lifetime, per-method ordinals, total bounds, cancellation/timeout/drop, account
snapshot changes, new one-use handles and ordinary JSON refusal. They do not
send real SMS/email or test a browser. Run the affected Mijia and Host suites.


API 1.32 adds explicit code-request metadata/output checks to
`verify_native_account_verification.py` and the full native verifier. It verifies
all four exports, output clearing, invalid method and buffer capacity, forbidden
recipient/URL fields, unknown/closed handles and ordinary JSON refusal. Synthetic
Host/protocol tests cover success and actual quota/cooldown behavior; the ABI
admission tests do not send real messages.


### App verification continuation

From `apps/mobile`, run `fvm flutter test test/account_verification_test.dart
test/account_verification_bridge_test.dart test/account_captcha_test.dart`.
The ABI test compiles a fictional C shared library with the host `cc` compiler
(macOS/Linux); it performs no networking or platform-store access. The actual
native refusal cases use the built host FFI library and do not send credentials.
These checks do not establish SMS/email delivery or provider acceptance.


### Bounded numeric history fixtures

`python3 scripts/validate_contracts.py` includes the ADR-0089 history options,
request and partial-result examples, plus negative authority/remote-identity,
timestamp, numeric type, bounds and completeness cases. This checks shape only;
Core descriptor/identity, interval ordering, result correlation and lifecycle
invariants require runtime tests when the Core implementation is added. Passing
schema fixtures does not establish a callable history feature.


Run `cargo +1.96.0 test -p homeverb-host --test history --offline --locked` for
Core historical-read admission, explicit connection selection, output invariants,
unknown DTO fields, lock/lifecycle cleanup and no cache/receipt changes. Run
`cargo +1.96.0 test -p homeverb-mijia history --offline --locked` for the protected
exact-model path through Core, encrypted query parameters, reference fallback,
public identity mismatch, unlisted membership and account/device lease cleanup.
These use fictional local accounts and transports, not provider credentials.


### App historical reading checks

From `apps/mobile`, run `fvm flutter test test/numeric_history_test.dart`.
Fictional fixtures exercise immutable option admission, historical/live unit
separation, canonical UTC intervals, sorted finite records, partial-data evidence,
identity/source/receipt mismatch refusal, explicit connection selection, no implicit
read/polling, cancellation with late replies, English/Chinese narrow enlarged text
and device-view automatic-refresh suspension/resumption. No provider session or
appliance is used. These tests do not establish real history retention/completeness,
physical model support or mobile packaging.


### Mijia CLI and historical MCP checks

Run `cargo +1.96.0 test -p homeverb-cli --offline --locked`, build the CLI, then
run `python3 scripts/verify_mvp.py`. Rust fixtures exercise complete historical
output through CLI dispatch/MCP, canonical closed requests, zero/partial records,
no cache/receipt mutation, bounded file input, actual FIFO refusal and pending-read
cancellation on deadline, explicit notification, EOF and host shutdown. Real CLI
process tests exercise Mijia empty-profile startup and default profile selection,
no account/network work while idle, presend write/grant/mock-option refusal,
hidden input requiring a terminal, historical options and unsupported reads,
malformed/oversized request files, query override rejection and MCP tool schemas.
These are fictional tests and never use provider credentials or appliance traffic.
Current executed result: 11 Rust tests passed; 37 process cases ran with 30 passed
and seven preexisting platform skips on macOS. See ADR-0090 for remaining acceptance.


### Typed exact outlet checks

Run `cargo +1.96.0 test -p homeverb-mijia --offline --locked`. The outlet fixtures
cover all nine exact targets, separate adapter families, independent grants and
trusted approval, forged public risk metadata, protected mapping/identity/route
refusal, invalidated previews, correlated rejection and unknown outcomes without
replay. Inspection is local and leaves default saved mappings read-only. These
checks use fictional accounts and transports; native/App rollout and physical
acceptance are separate gates under ADR-0091.


### Native exact outlet checks

Run `cargo +1.96.0 test -p homeverb-host --test mijia_outlet_grants --test
mijia_accounts --offline --locked`, build `homeverb-ffi`, then run
`python3 scripts/verify_native_contract.py`. Native fixtures cover all nine exact
targets, duplicate/wrong-family/invalid-scope refusal before profile creation,
local inspection before grants and trusted preview lifecycle, one-use execution
and unknown outcomes. Mijia adapter tests also check both directions of overlapping
purifier/outlet grant isolation. All accounts and transports are fictional.


### App exact outlet checks

From `apps/mobile`, run `fvm flutter test test/mijia_outlet_test.dart`. Fixtures
cover all nine exact targets, separate purifier/outlet startup fields, unexpected
risk/schema/route refusal, API 1.33 refusal before session replacement, and actual
FFI negotiation with a fictional old C library before transmitting the additive
startup field. Both current native platform modes open with fictional grants.
Bilingual narrow enlarged-text reviews name the consequential effects, while
forms require explicit target and cloud selection before preparation. Opening a
device sends no control or approval. Existing `approval_test.dart` covers actual
preview ownership, lifecycle invalidation and lost-reply receipt lookup.


### Exact auxiliary setting checks

Run `cargo +1.96.0 test -p homeverb-mijia --offline --locked` and the Host
`mijia_setting_grants` / `mijia_accounts` integration tests, then build the FFI
library and run `scripts/verify_native_contract.py`. ADR-0092 cases cover all
seven exact targets, wrong model/source/domain/unit/access, independent families
and grants, old saved derivations, local pregrant inspection, zero companion
writes, exact wire codes and unknown outcomes without replay. These are fictional
checks; no physical hood/fan or provider acceptance is established.


From `apps/mobile`, run `fvm flutter test test/mijia_auxiliary_test.dart`. The
auxiliary App fixtures preserve all seven startup targets, actual old-library
negotiation before additive grants, sibling/timer/wrong-domain refusal, current
session-grant intersection and narrow enlarged-text panels in both locales.
The brightness form explicitly sends code 0 or 2, with no percent conversion or
companion power argument. No provider or appliance is used.


## Modern Mijia QR protocol

The `homeverb-mijia` suite includes fictional modern bootstrap/QR/STS/renewal
sequences, exact SID and endpoint rejection, callback-identity consistency,
immutable original cancellation/deadline, a 120-second single-poll ceiling,
pre-cancellation, dropped waits, bounded responses and nonserializable material.
Local generated TLS peers check cookie isolation, duplicate-token rejection,
redirect refusal, no retry and cancelled/deadline/dropped socket closure. They
never contact Xiaomi. Anonymous endpoint-shape research is recorded separately
in ADR-0093 and does not establish account login, renewal or device support.


The modern reference-read suite in `crates/mijia/src/modern/reads/tests.rs` adds
an independent continuous-RC4 vector, all 264 pinned scalar query admissions,
opaque no-read entries, account isolation, pagination/conflict/partial-result
cases, missing/false/zero distinctions, bounded encrypted gzip/CRC/trailing-data
refusal and cancellation at synchronous response delivery. Local TLS tests check
reference headers, cookie isolation, no redirects/retries, strict content encoding,
body limits, untrusted certificates and cancellation/deadline/drop socket release.
These tests use fictional data; they do not prove fresh QR credentials work with
the reference origin or that logical entries are distinct physical appliances.


The modern account fixtures in `crates/mijia/src/adapter/tests/modern_accounts.rs`
use real temporary SQLite and fictional secret/auth/read peers. They cover
version-1 byte round trips, version-2 profile/client/subject tamper refusal,
modern-only reopen/read/renewal, adoption in both protocol directions, preserved
legacy bindings and labels, leading-zero ambiguity, unavailable storage, QR
owner/snapshot checks before and after exchange, original/continuation
cancellation, original expiry, removed accounts, uncertain login/renewal saves,
failed/changed-byte recovery, entry subject isolation and lease release on
cancellation/drop. No test scans a real QR code or calls a vendor endpoint.


Modern Host tests in `crates/host/tests/mijia_accounts/modern_host.rs` use real
SQLite, fictional modern/legacy peers and an injected secret store. They cover
protected QR output, same-subject legacy-card preservation, shared login capacity,
expiry pruning, foreign/repeated handles, invalid-metadata and busy-operation
consumption, pending/in-flight lifecycle invalidation, late synchronous results,
dropped futures, lease/operation-ID release, uncertain-save recovery without
reauthentication, ordinary JSON refusal and sanitized account summaries. Existing
native startup fixtures continue to require no credentials or provider requests.


API 1.36 adds `scripts/verify_native_modern_account.py` to the full native verifier.
It loads the real dynamic library and checks all four exports, closed requests,
valid-buffer clearing before invalid host/metadata admission, NULL/invalid-size
refusal without touching memory, ordinary-dispatch refusal and lifecycle forwarding
using Mock only. QR-success/overflow projection tests use protected fictional
bytes; Host success reports are validated against the same closed native schema,
including failed/empty directories and no physical-completeness claim. Android
and iOS package unit tests use fictional archive/symbol reports; they do not
establish a newly built AAR/framework/APK or signed-device acceptance.


The Flutter modern client uses `test/fixtures/modern_account_bridge.c`, a local
fictional ABI compiled during `modern_account_bridge_test.dart`. It has no network
or system-secret access. A separate API 1.35 build deliberately omits modern
symbols to check pre-lookup negotiation; paused ready-response cases check owned
scan-array cleanup and lifecycle refusal independently of native cancellation.
The scoped account/native suite is not full mobile or provider acceptance.


The ADR-0101 cases in
`crates/host/tests/mijia_accounts/modern_host/devices.rs` exercise actual Host/Core
publication from one fictional encrypted QR directory response, then ordinary
state reads. Run `cargo +1.96.0 test -p homeverb-host --test mijia_accounts
--offline --locked`. Cases cover grouped/unsupported cards, sanitized schema-valid
reports, stable organization, partial empty/failing directories, uncertain first
and second saves, exact-byte recovery, Core identity-publication failure, original
QR deadline, foreign/consumed handles, lifecycle changes and dropped/cancelled work.
These checks do not establish a callable C/App device flow or provider acceptance.


API 1.37 adds positive/negative modern device-publication contracts and actual C
checks for optional Boolean publication, the two explicit device operations and
background/closed/unsupported hosts. The existing Host device cases also validate
new dedicated result schemas. Additional ordinary account tests cover modern-only,
mixed-before/after-adoption, pending recovery and unavailable modern transport;
failures never contact legacy routes or authenticate. Run the Mijia/Host workspace
regression and rebuilt `scripts/verify_native_contract.py`. No provider/phone
traffic or new mobile package is exercised by these checks.


From `apps/mobile`, run `fvm flutter test test/modern_device_bridge_test.dart
 test/modern_device_flow_test.dart test/modern_account_bridge_test.dart
 test/account_qr_test.dart`. The new 1.37 C fixture requires publication metadata
and device operation names, refuses malformed result shapes/IDs, and checks late
reply cancellation with the same protected QR cleanup. Widget/controller fixtures
exercise actual card clicks and Core-client state requests, local reload failure
without login replay, partial/failed/empty outcomes and late navigation/account/
foreground results. Retain account/native/localization/foreground-read regressions
and the `screens_test.dart` portrait/landscape reflow checks for the flexible header.
These fixtures perform no provider or physical-device operation.

## Modern exact statistics profile

ADR-0103 adds fictional protocol tests in
`crates/mijia/src/modern/reads/tests/statistics.rs`, real Core/SQLite tests in
`crates/mijia/src/adapter/tests/modern_accounts/history.rs`, and an ordinary Host
history journey after QR publication. Assert per-field encryption/signatures,
exact source origin/cookies/parameters, complete root identity, independent
historical units and bounded plaintext/encrypted/gzip replies. Refusal tests cover
foreign/logical identities, unsupported profiles, malformed/corrupt/oversized data,
limits and late lifecycle results. Mixed-account tests preserve saved protocol
selection, avoid fallback/reauthentication, and release account/device leases.
Run the existing local TLS suite and native ABI checker plus Flutter
`test/numeric_history_test.dart`; they use fictional data and no appliance.

## Reported numeric sample checks

Run `cargo +1.96.0 test -p homeverb-mijia --offline --locked` for protocol and all
161 exact legacy/modern Core sample mappings. Run the Host history suite with
`cargo +1.96.0 test -p homeverb-host --test history --offline --locked` and CLI/MCP
with `cargo +1.96.0 test -p homeverb-cli --offline --locked`. Rebuild FFI and run
scripts/verify_native_contract.py for the API 1.38 explicit opt-in and closed-field
boundary. These checks use fictional credentials and no provider traffic.

From apps/mobile run:

```sh
fvm flutter test test/sample_history_test.dart test/numeric_history_test.dart test/modern_device_flow_test.dart
fvm flutter analyze
```

The sample suite covers version negotiation, signed/unitless values, domain and
privacy refusal, duplicate series, immutable ownership, explicit reads, background
cancellation and English/Chinese narrow enlarged layouts. Existing daily history
and modern card tests retain their prior coverage. Passing this subset does not
claim the known full Flutter visual/legacy-click failures are resolved.

## Exact hood manual controls

Run the Mijia suite, Host mijia_accounts and mijia_setting_grants suites, then
rebuild FFI and run scripts/verify_native_contract.py for API 1.39. Exact new values
are covered on both protocols with independent policy refusal, byte-level request
correlation, lost acknowledgements and no companion writes. Saved-mapping tests
cover old illumination-only upgrades and refusal of partial new-domain mixtures.

From apps/mobile run:

```sh
fvm flutter test test/mijia_hood_test.dart test/mijia_auxiliary_test.dart test/mijia_settings_test.dart test/session_permissions_test.dart
fvm flutter test test/mijia_controls_test.dart test/mijia_outlet_test.dart test/approval_test.dart test/modern_device_flow_test.dart test/native_test.dart test/account_native_test.dart
fvm flutter analyze
```

These include actual old/new ABI startup probes and English/Chinese narrow
forms for every exact value. No real account/appliance is used. Build and verify
a clean matching APK separately before advertising updated Android delivery.


## Typed no-argument action protocol

Run `cargo +1.96.0 test -p homeverb-mijia --offline --locked` for ADR-0107.
Action fixtures cover every six exact tuples on both selected protocol profiles,
independent request decoding/signatures, complete root suffixes, empty input
objects, and strict acknowledgement correlation. Code-zero acceptance is separate
from proven pre-execution rejection and uncertain code 1/execution/return/timeout
errors. Refusal cases include wrong model/spec/query/kind/subject, logical children,
invalid identities, consumed progress, plaintext/duplicate/missing/foreign replies,
nonempty outputs, malformed/bomb/concatenated gzip and size limits.

Both control endpoints run the local HTTPS header/path, redirect, certificate,
size, cancellation, deadline and drop matrices. A consumed progress marker and
an untracked action open no connection. Pending protocol futures release resources
and cannot be replayed; late replies do not mark acceptance. No provider or
appliance is used. Check all three generated projections with
`python3 scripts/extract_mijia_reference_facts.py --check` and
`python3 -m unittest discover -s tests -p test_mijia_reference.py`.

Retain workspace and native ABI regression checks; native API 1.39 and all public
schemas are unchanged. This typed-library phase adds no Flutter control or package.
Core binding/grant/receipt tests and native/App adoption are required in a later
phase before claiming user-callable actions.


## Protected Core no-argument actions

ADR-0108 fixtures are in crates/mijia/src/adapter/tests/modern_accounts/actions.rs.
Run `cargo +1.96.0 test -p homeverb-mijia --offline --locked` for all six actions
on both selected protocols, independent Core/adapter grants and exact empty
arguments. They use real Core/SQLite with fictional protected accounts and injected
transports. Inspection proves no network or credential mutation, and action
publication preserves scalar read/history contracts and existing component scopes.

Negative cases exercise absent/wrong/cross-family policies, public argument/action
forgery, connection identity refusal, protected model/specification/service changes,
unlisted membership, missing sessions/transports and parent-bearing modern entries.
Durable receipt cases cover accepted/rejected/not-sent/unknown, concurrent and
sequential duplicates, conflicts, store/Core reopen, revoked grants, history
retirement and explicit legacy-to-modern adoption. Cancel/deadline/drop release
account/device locks and preserve unknown receipts without replay.

Run workspace, strict Clippy, projection/contract validators and rebuilt native
ABI checks. New examples/capability.mijia-action.json and command.mijia-action.json
retain the existing schema shape with non-idempotent semantics; parameter
counterexamples refuse wire IDs, inputs and confirmation flags. Production native
composition remains unchanged. Flutter/App action admission and provider/phone/
model acceptance are separate, unverified work.


## Native no-argument action admission

Run `cargo +1.96.0 test -p homeverb-host --test mijia_action_grants --test
mijia_accounts --offline --locked`, then rebuild FFI and run
`scripts/verify_native_contract.py --write-examples`. API 1.40 cases exercise all
six exact tuples, both allowed modes, 64-entry admission, malformed/duplicate/
excessive lists, cross-family substitutions, extra fields, wrong adapters and
invalid inspection flags. Rejection must precede profile creation. The Host
journey inspects before grants without networking, reopens, explicitly refreshes,
plans and executes, then verifies persistent unknown receipts and background
cancellation without replay. Retain workspace/Clippy and schema validation.
All fixtures are fictional; no provider/phone/device acceptance follows.


For Flutter action adoption run from apps/mobile:

```sh
fvm flutter test test/mijia_actions_test.dart test/mijia_hood_test.dart test/mijia_auxiliary_test.dart test/mijia_settings_test.dart test/mijia_controls_test.dart test/mijia_outlet_test.dart test/session_permissions_test.dart test/approval_test.dart test/modern_device_flow_test.dart test/native_test.dart test/account_native_test.dart test/controls_test.dart test/device_detail_controls_test.dart test/localization_test.dart
fvm flutter analyze
```

The action suite uses actual old/new ABI startup probes, exact six-action grant
fixtures, malformed authority refusal, late background inspection and English/
Chinese 320-pixel double-text-scale controls. Selecting an action is not a write;
Apply plans/executes once, repeated pending clicks do not send, and unknown remains
unknown. No physical behavior is inferred from these fixtures.


## Typed elevated power-toggle actions

Run `cargo +1.96.0 test -p homeverb-mijia --offline --locked` for all eleven
legacy/modern protocol action profiles and the ADR-0110 Core/SQLite journeys in
adapter/tests/modern_accounts/actions/power.rs. Each exact model/service is
checked for ordinary denial, trusted approval and a single correlated send.
Forged public risk, property/foreign grants, cancelled previews, uncertain codes,
after-send cancellation, deadline, drop and reopened/retired receipts must not
permit replay. The tests verify released account/device locks and no extra reads
or companion writes. No provider traffic or physical acceptance is involved.

Run the Host mijia_accounts suite and rebuilt scripts/verify_native_contract.py
to retain API 1.40's six-action limit. Power-toggle tuples are rejected by native
startup before creating a profile and are absent from existing inspection opt-in.
Run projection, contract, documentation and strict workspace checks. New native/
Flutter admission and matching packaging are dependent phases, not established
by these typed Rust tests.

## Native elevated Mijia power-toggle checks

ADR-0111/API 1.41 uses fictional Host model/service journeys in
crates/host/tests/mijia_accounts/modern_host/actions/power.rs. Run the
mijia_action_grants and mijia_accounts integration targets in homeverb-host, then
rebuild homeverb-ffi and run scripts/verify_native_contract.py --write-examples.
Verify separate default-false power inspection, all exact grants in both allowed
modes, no ordinary execution, preview/dismiss/background and unknown reopen without
replay. Startup/schema counterexamples cover foreign tuples and forbidden fields.
The legacy/modern Core tests remain the protected protocol evidence. No provider
traffic, phone, Flutter or APK acceptance follows from these checks.

## Exact typed Mijia maintenance action checks

Run cargo +1.96.0 test -p homeverb-mijia --offline --locked for all fifteen
legacy/modern protocol profiles and the maintenance module under
crates/mijia/src/adapter/tests/modern_accounts/actions/. It checks all four exact
actions, ordinary denial, independent grants, forged public risk/arguments,
protected membership/root changes, trusted approval, cancel/deadline/drop and
unknown receipts after reopen/retirement. Formatting, streams and output-bearing
actions must refuse before transport. Host tests and rebuilt C ABI checks retain
API 1.41 startup rejection and power-inspection isolation. No native/App admission,
Flutter run, package rebuild or real-provider operation follows from these tests.

## Native Mijia maintenance admission checks

Run the homeverb-host mijia_action_grants and mijia_accounts targets for ADR-0113
exact scope, independent inspection and trusted maintenance journeys. The dedicated
maintenance module exercises all four actions and retained unknown outcomes after
reopen, with dismissal/background/single-consumption checks. Rebuild homeverb-ffi
and run scripts/verify_native_contract.py --write-examples for actual API 1.42
startup/Boolean-field acceptance and invalid-input refusal before profile creation.
Preserve existing 1.40/1.41 inspection results. Fictional evidence does not prove
Flutter, package, provider or physical acceptance.

## Flutter maintenance action adoption

From apps/mobile, run fvm flutter test test/mijia_maintenance_actions_test.dart
with the rebuilt current native library. The suite covers all exact tuples,
independent inspection flags, unsafe metadata/unknown-action refusal, actual old
ABI refusal before any granted opening, current Mijia and combined startup,
late inspection after background, descriptor buttons, cloud-route selection,
immutable bilingual review at 320 logical pixels with double text size, explicit
single consumption, unknown receipts and background discard. Run the existing
power/action, permission, native, control and approval suites and Flutter analysis
as regressions. No provider, account, phone or appliance is used by these tests.

## Exact additional switch and outlet controls

ADR-0114 adds seven-field legacy/modern fixtures and all fifteen possible targets,
including exact source mutations, old read-only mappings, native inspection,
independent startup grants, forged public risk, protected identity changes,
preview/foreground invalidation and uncertain receipts after reopen. Run the
Mijia tests, Host mijia_accounts/mijia_outlet_grants suites, native C ABI and
contract checks with rebuilt FFI. On this workstation use --test-threads=4 for
Rust regression runs to avoid CPU contention in existing 100 ms deadline cases.
These checks use fictional material and establish no provider/physical support.

Run fvm flutter test test/mijia_switch_outlet_test.dart from apps/mobile with
rebuilt FFI to cover new-tuple startup/version negotiation and all seven exact
fields through bilingual controls. Include existing outlet/action/maintenance,
permission, control, native, approval and foreground-refresh regressions, plus
Flutter analysis. The native fixtures use fictional identities and no provider
traffic; UI tests do not establish physical-device support.


## Typed reported Mijia event history

Run the following with fictional data and local TLS peers only:

```sh
python3 scripts/extract_mijia_reference_facts.py --check
python3 -m unittest discover -s tests -p test_mijia_reference.py -v
cargo +1.96.0 test -p homeverb-mijia event_history --offline --locked -- --test-threads=4
cargo +1.96.0 test -p homeverb-mijia transport::tests --offline --locked -- --test-threads=4
```

ADR-0115 tests every one of the 78 exact source tuples on each protocol, preserving
vendor namespaces and refusing foreign model/spec/event selections before send.
Verify request signatures and separate origins/codecs; malformed/foreign/missing
correlations; positive time-window, response/count and decompression bounds;
opaque payload omission; same-second rows; unproven empty/full pages; auth failure;
late results; cancellation, deadline and future drop without retry. These typed
checks do not establish protected Core membership, native/App support, payload
interpretation, pagination, subscriptions or real-provider retention/delivery.


## Event history domain contract checks

Run cargo +1.96.0 test -p homeverb-domain --offline --locked for the ADR-0116
round-trip, canonical UTC/leap-second, remote selector/authority and event-payload
refusal tests. Run python3 scripts/validate_contracts.py for the event-history
schema's three fictional fixtures and negative boundary cases, including empty,
partial/full pages, equal timestamps and option/record bounds. The separate
numeric history schema must continue rejecting event results. These checks do not
exercise Core event admission, adapter bindings, native/App methods or devices.
Core cross-field time/count/identity checks remain a required implementation phase.


## Protected Core event-history checks

Run cargo +1.96.0 test -p homeverb-host --test event_history --offline --locked --
--test-threads=4 for generic Core fixtures: declaration/identity/window admission,
same-second and empty/full/rejected pages, malformed adapter result refusal,
unchanged snapshots/receipts, metadata changes and lock cleanup. Run
cargo +1.96.0 test -p homeverb-mijia event_history --offline --locked --
--test-threads=4 for all 78 exact legacy/modern Core requests, full old/enriched
layout compatibility, event-only services, independent protected model/root and
membership checks, unavailable secure storage, cancellation/deadline/drop/late
reply cleanup and no credential/state/receipt mutation or protocol fallback.

All checks use fictional accounts and local transports. They do not establish
Host/native/CLI/MCP/App availability, decoded event arguments, subscriptions,
provider delivery/retention or phone/model acceptance. Run the broader workspace
and native ABI regressions when changing protected descriptor admission.
