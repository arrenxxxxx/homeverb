# 11. Mijia native integration

Implementation is in progress under [ADR-0070](../adr/0070-mijia-native-cloud.md).
The owner will perform real-account and device acceptance. Do not access saved
pilot credentials or contact appliances as part of development tests.

Current callable scope includes trusted Rust account/read APIs and native API 1.24
Mijia-only/combined sessions and Flutter platform/account/read views. Fan control
and live acceptance remain pending. Historical phase records below distinguish earlier
protocol-only work from the [protected Core phase](../adr/0072-mijia-protected-core-reads.md).

## Protocol evidence

- MIT `Squachen/micloud`, commit `dc002b87fc09ef36199715cfa10f1eaa2d2d4210`:
  [login and inventory](https://github.com/Squachen/micloud/blob/dc002b87fc09ef36199715cfa10f1eaa2d2d4210/micloud/micloud.py),
  [request encoding](https://github.com/Squachen/micloud/blob/dc002b87fc09ef36199715cfa10f1eaa2d2d4210/micloud/miutils.py).
  HomeVerb independently implements the wire format in Rust; attribution and
  the source license are retained under `third-party/micloud`.
- Pinned [community MIoT client](https://github.com/al-one/hass-xiaomi-miot/blob/a3a07b8504bacfee114a6052ae864d7dd149bf71/custom_components/xiaomi_miot/core/xiaomi_cloud.py)
  was inspected for endpoint and response facts only. No GPL source is included.
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

[ADR-0073](../adr/0073-mijia-native-account-routing.md) adds API 1.24 startup modes
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
