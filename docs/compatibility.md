# Platform and device compatibility matrix

> The fictional `homeverb.mock` adapter is implemented and Mock-tested on Linux. The six listed Meiju models have experimental cloud reads; real-platform rows do not claim verified physical support. One owner-observed wall-AC shutdown is recorded below; this does not establish protocol-level verification or broad model support.

| Candidate | Potential route | Evidence still required | Project status |
|---|---|---|---|
| Midea air conditioners, dishwashers, and refrigerators | Required Meiju cloud enumeration/read/control; additional AC/E1/CA LAN routes | Exact model/app/firmware, physical cloud reads/controls and off-LAN tests | One mainland account and six exact model mappings exercised through read-only Core; physical acceptance pending |
| Hualing devices in the selected appliance scope | Meiju cloud and LAN paths only when actual account/protocol evidence matches | Confirm account visibility, family/version and supported cloud/LAN capabilities per device; do not infer from brand | protocol-researched / models unverified |
| Samsung televisions | Candidate LAN protocol | Pairing, firmware, wake behavior, capabilities, authentication | planned / unverified |
| SmartThings | Vendor cloud API | Backend-free native authorization, region, API permissions and cost | planned / auth gate pending |
| Hisense devices | Candidate account platform/API | Regional platform, app, account, protocol, native authorization | planned / unverified |

Brand names express user interest, not protocol boundaries. `midea-local` and SmartThings CLI provide protocol/API references, but do not prove native authentication, mobile embedding, or model support in this project. [R5 and R6](references.md)

The owner selected Midea/Hualing air conditioners, dishwashers, and refrigerators on October 15, 2026, with automatic retrieval of existing Meiju-bound devices after authorization and daily cloud control at home or away. The [integration guide](development/05-midea.md) pins separate cloud and LAN sources. Source research is sufficient to start offline work; it does not establish HomeVerb support. Refrigerator writes are absent from the inspected LAN implementation, while cloud mappings declare compartment settings. Neither a mapping nor a dishwasher message class proves a verified model-specific action. A model-scoped read-only Meiju adapter is available through the explicit developer probe and native API 1.1.0 startup selection; omitted selection remains Mock.

`homeverb-midea` now implements the Meiju home/appliance inventory protocol behind an injected read transport, verified with synthetic responses. Synthetic protocol coverage is supplemented by the scoped live evidence below; full account/model/physical acceptance remains pending. Existing native API version 1.0.0 and Mock behavior are unchanged. See [ADR-0020](adr/0020-midea-cloud-inventory.md).

The library also decodes selected AC/E1/CA cloud state fields under [ADR-0023](adr/0023-meiju-cloud-state-evidence.md). Returned cloud observation time is unproven; projected readings are stale and cannot verify writes. Vendor-specific mode, fan and dishwasher codes retain their meaning without invented common controls. The unreleased Rust endpoint enum gains an appliance-state variant; native API 1.0.0, SQLite schema 3 and registered adapters are unchanged. Synthetic decoding does not establish support for a physical model.

The Mock climate capability is revision 2 and explicitly permits independent-field decomposition. Rediscovery updates existing profiles. Older descriptors without the declaration keep complete argument objects together; request-ID replay remains unchanged. See [ADR-0012](adr/0012-explicit-action-decomposition.md). No real-platform compatibility follows from this change.

Rediscovery preserves established connection bindings and refuses identity changes with `DEVICE_IDENTITY_CONFLICT`. Stable Mock identities require no migration. Adding/removing routes and changing accounts need a future explicit confirmation flow; initial real-device identity and deduplication still require adapter evidence. See [ADR-0013](adr/0013-preserve-device-connection-identity.md).

Core now accepts bounded adapter-correlated completion evidence under [ADR-0016](adr/0016-correlated-completion.md). The `correlated-completion` Mock scenario changes declared verification methods on discovery and returns fictional evidence. Normal Mock behavior, JSON result fields, and SQLite schema remain compatible. The unreleased Rust `AdapterOutcome` adds `completion`; existing adapters set it to `None`. No real-platform correlation or device support follows from this change.

Explicit device batches are additive Core/CLI/native functionality under [ADR-0017](adr/0017-bounded-device-batches.md). Existing single-command payloads, SQLite records, and C ABI functions are unchanged. Older hosts reject `plan_batch` and `execute_batch`. The built-in Mock host still exposes one fictional appliance; multi-device concurrency is exercised by a dedicated integration adapter, not a real-platform support claim.

Component-scoped capability definitions are supported under [ADR-0018](adr/0018-component-capability-scopes.md). Existing shared definitions retain their JSON representation; old strict readers reject the new scope field. Update consumers before loading scoped descriptors, and never discard scope to claim compatibility. SQLite and C ABI functions are unchanged. The fridge/freezer fixtures demonstrate Core semantics only; no real refrigerator model is verified.

The [native UI contract](native-ui-contract.md) adds `get_api_info` and publishes API version 1.0.0 under [ADR-0019](adr/0019-native-ui-contract.md). Existing valid JSON requests, C symbols and stored data are unchanged. No-argument methods now reject previously ignored extra fields, enforcing the existing closed-request policy. Older hosts reject the new query; new UIs must require an upgraded library before assuming method availability. The callable catalog describes interfaces, not real-platform support or write permission.

The native Meiju account layer under [ADR-0024](adr/0024-meiju-private-device-bindings.md) adds protected subject/session/device-map persistence through the existing vault. Synthetic evidence covers stable local bindings across reopening, session replacement, partial inventories and descriptor rediscovery. It neither registers a real adapter nor establishes provider identity, cross-account deduplication or physical support. SQLite schema 3, native API 1.0.0 and its 22 methods remain unchanged.

[ADR-0025](adr/0025-bound-meiju-core-reads.md) adds account-bound state reads with exact connection/model/component preflight and Core cache-validation coverage using fictional descriptors. The unreleased AccountLock Rust trait now requires Send + Sync; custom implementations must meet that bound. That bound-read phase alone did not register a real adapter or add controls, native methods or a SQLite migration; the current model-scoped adapter is described below.

The owner-approved community login codec and restricted HTTPS client are now
implemented and synthetic-tested under ADR-0022. Only the mainland Meiju endpoint
is admitted; SmartHome, automatic region routing and additional login challenges
are not implemented. The protected native connection coordinator and local read-only probe are also synthetic-tested, including reconnect identity and separate retrieval failures. One owner-run real login/inventory probe and an independently executed saved-session refresh returned four air conditioners, one dishwasher and one refrigerator on October 15, 2026. This is Linux account/inventory evidence only. No public account method,
control permission or mobile support follows from that native connection phase. Native API 1.0.0 and SQLite schema 3 are unchanged.

## Observed cloud-read models

The explicit read-only probe on October 15, 2026 returned the following mapped
fields from the owner's six appliances. Model strings below use JSON escapes for
non-ASCII characters to preserve the repository language policy. No account/device
identifiers, raw responses or private state values are retained here.

| Exact model string | Family | Observed mapped fields |
|---|---|---|
| `KFR-35G/N8HE1\u2161Pro` | AC | Power, current/target temperature, vendor mode and wind-speed code |
| `KFR-35G/WF2` | AC | Same mapped fields |
| `KFR-72G/N8HA1Pro` | AC | Same mapped fields |
| `KFR-26G/N8VJC3` | AC | Same mapped fields |
| `BCD-600WUFIPZM(E)` | Refrigerator | Separate fridge/freezer power and current/target temperature |
| `GX1200Max \u5c0a\u4eab\u7248` | Dishwasher | Vendor mode, work status, door code, water/salt/brightener codes, temperature and remaining minutes |

This is exercised cloud-read evidence, not `verified` physical-model support.
Firmware, official-app comparison, trustworthy observation time, controls,
reauthorization and off-LAN/mobile acceptance remain unverified.

The subsequent production `MeijuAdapter` admits these exact model/family pairs as
experimental read-only descriptors. The explicit `core-inspect` run published all
six cards and read seven component snapshots through Core with zero failures.
Every observation remains stale. Unknown/missing/mismatched models stay visible as
unsupported without read requests or actions. Complete discovery checks current
and historical protected bindings across accounts and refuses ambiguous duplicates.
Partial retrieval preserves existing cards.

Native API 1.1.0 subsequently adds optional startup `adapter` selection. Existing
startup objects preserve Mock behavior; explicit `midea.meiju` opens saved-account
read-only access and rejects Mock write grants/fault scenarios. Version 1.0 hosts
reject the new startup field. The six C symbols, 22 request methods and SQLite
schema 3 are unchanged. A live Linux C ABI run described six devices, read/cached
seven components and preserved existing preferences. That 1.1 host phase did not add UI login methods or controls.

Native API 1.2.0 adds `list_accounts`, `refresh_account` and the separate
`homeverb_connect_account` C symbol with bounded writable secret buffers. Existing
startup/requests and SQLite schema 3 remain compatible; older libraries lack the
new symbol/methods. Native account results separate authentication from retrieval;
partial lists, expiry, cancellation, identity conflict and uncertain credential
updates preserve their distinct outcomes. Synthetic host tests validate actual
results against published schemas; C ABI tests verify secret-buffer clearing and
admission refusal. No controls, mobile credential store or physical verification
follows from these account operations.

The new `list_accounts` and `refresh_account` methods were exercised on the same
real Linux profile on October 15, 2026: six cards, seven successful read/cache
pairs, preserved preferences, no repeated login and no controls. Reauthorization
through the protected C symbol and phone acceptance remain unverified.

## Experimental AC control implementation

A trusted Rust host can now opt into `power.set` for the four exact AC models above.
The separate [control probe](development/07-meiju-control.md) defaults to preview
and grants only one chosen local device/state over cloud. Core/TLS synthetic tests
cover protected binding/model refusal, scoped policy, code-zero acceptance,
authorization rejection, unknown provider errors, lost replies, cancellation,
deadlines, cache preservation and request-ID replay without resending. None of
these tests operates a real appliance. A real saved-account preview subsequently
passed inventory/plan validation for `KFR-35G/WF2` with zero controls sent.
Physical and off-LAN control acceptance remain unverified; dishwasher/refrigerator writes and other AC settings are pending.

Native API 1.2.0, SQLite schema 3 and result schemas remain unchanged. Default native
Meiju sessions still have no control admission or write grants. The unreleased Rust
`CloudEndpoint` adds `ApplianceControl`; exhaustive transport implementations must
handle or refuse it. `CloudTransport::send_control` defaults to refusal, never to
`send`. The HTTPS read path independently refuses control endpoints. Opted-in AC
power capability revision 2 is cloud-only and has unavailable verification. A
read-only rediscovery returns revision 1; neither revision authorizes a write.

## Native power permission contract 1.3

API 1.3.0 adds optional startup-only `meiju_power_grants` for exact local AC
IDs/states over cloud. Default behavior remains read-only. The seven C symbols,
24 methods, JSON command/result contracts and SQLite schema 3 are unchanged;
older libraries reject the new startup field. Grants never authorize arbitrary
commands, other models or other transports, and are not persisted. See
[ADR-0027](adr/0027-native-power-grants.md).

Synthetic host tests exercise actual plan/execute/receipt paths, denied states and
routes, read-only reopening, cancellation/background/close, and account-lease
release. Linux ABI tests validate startup and reject malformed/forged permission
fields before profile creation. No real account or device was used in this phase:
the owner explicitly deferred household operation. Physical control, off-LAN and
mobile acceptance remain pending.

## Local account metadata contract 1.4

API 1.4.0 adds `get_account` and `rename_account`, taking the ordinary method count
to 26. Local details list saved linked IDs including unsupported cards; rename
changes only the trimmed local display label. Neither accesses protected sessions
or the vendor. Pending state and default read-only permissions are preserved.
Synthetic host/ABI tests cover provider/account isolation, duplicate route counting,
secret-store unavailability, lease contention, invalid input, SQLite update failure,
reopening and closed/background admission. No live account was changed.

Existing requests, seven C symbols, SQLite schema 3 and execution schemas remain
compatible. Older libraries reject the two new methods. Check the catalog before
enabling them; see the [local metadata contract](native-ui-contract.md#local-account-details-and-names).
Account removal, provider revocation and mobile acceptance remain pending.

## Local account removal contract 1.5

API 1.5.0 adds `remove_account` with an exact reviewed device-ID set, bringing the
ordinary catalog to 27 methods. SQLite upgrades from schema 1/2/3 to 4 with a
verified backup. Close older clients before migration; schema-3 binaries refuse
the upgraded profile. Strict older readers reject the new `removing` credential
state. Upgrade UI/native code together; C symbols and appliance receipts are unchanged.

Synthetic tests cover scoped cleanup, shared-route refusal, unavailable/uncertain
secret deletion, explicit recovery across reopen, cancellation and dropped futures,
transaction rollback, late discovery/cache refusal, retained rooms/receipts and
completed-removal replay without secret access. ABI tests check admission failures
and saved completion replay; the suite does not use the real keyring. No live
account was deleted or appliance operated in this phase. Local cleanup does not
revoke provider access. Remote revocation, shared-route detachment and mobile
acceptance remain pending. See [ADR-0028](adr/0028-local-account-removal.md).

## Status definitions

The native account vault under [ADR-0021](adr/0021-persistent-account-vault.md) adds SQLite schema 3 and internal Rust account APIs. Schema 1/2 upgrades preserve existing data and create verified backups; close older clients first. Schema-2 binaries reject the upgraded profile. Native UI API 1.0.0, its 22 methods, C symbols and execution semantics are unchanged. No public account operation or real-provider authentication is enabled by locally storing references.

The Flutter application in `apps/mobile` is Mock-tested through the Linux native library and an actual Linux Flutter process. It pins Flutter 3.47.7 through FVM, packages the Linux library, and now incorporates Core API 1.14.1 and SQLite schema 4; Flutter account input, retrieval/recovery and local removal are synthetic-tested; real account/phone acceptance remains pending. Its isolated 82-state preview is fictional. Screenshot comparison still reports differences; Android APK packaging and x86_64 emulator Mock journeys are verified; iOS packaging, actual Keystore, physical-phone lifecycle, full localization/accessibility and real-platform support are unverified. See the [Flutter verification report](../apps/mobile/verification.md). No platform row is promoted by the UI implementation.

Foreground authorization sharing is also planned and unverified for every candidate. Login support alone does not establish transferability. Before enabling sharing, record provider-specific evidence for permission scope, device binding, refresh-token behavior across installations, expiry, and revocation under [ADR-0008](adr/0008-foreground-authorization-sharing.md).

| Status | Meaning |
|---|---|
| planned | Candidate requirement only |
| protocol-researched | Traceable protocol evidence exists |
| mock-tested | Fictional/redacted contract fixtures pass |
| experimental | Implementation exists with limited verified paths |
| verified | A specific model and firmware passed physical acceptance |
| blocked | Authentication, networking, licensing, or another hard gate is unmet |
| unsupported | Explicitly outside the current scope |

## Required verified record

Brand, exact model, category, firmware, region, vendor app, account type, transport, adapter version, verification date, reproducible steps, tested reads/writes, verification method, unsupported features, disconnect tests, and duplicate-execution tests. Remove accounts, IPs, MAC addresses, and serial numbers before committing evidence.

## Independent native-client gate

The October 15 Meiju review found a developer-secret requirement in the official v3 API and shared app material in the pinned community login path. The owner explicitly approved the narrow community-material exception in [ADR-0022](adr/0022-meiju-community-cloud-admission.md) on October 15, 2026. The official developer-secret path remains excluded. Approval permits implementation of the scoped community flow; it establishes neither official approval nor physical support.

Do not skip authentication feasibility merely because another feature works. Label cloud-only operation `cloud-required`. Separate cloud-dependent onboarding from daily LAN operation. Platform support does not imply every product sold under that brand is supported.

## Explicit local-data cleanup contract 1.6

API 1.6.0 adds `clear_cached_state` and `clear_execution_history`, bringing the
ordinary catalog to 29 methods. The seven C symbols and SQLite schema 4 are
unchanged. Older hosts reject the new methods; check the published catalog.
Synthetic SQLite/Core and actual Linux C ABI tests cover cache misses, preserved
metadata, active/composite journal refusal, atomic rollback, retired-ID protection,
reopening and lifecycle admission. This does not add device controls, full-profile
removal, credential revocation, secure erasure or phone acceptance.

## Explicit Meiju account recovery contract 1.7

API 1.7.0 adds `recover_account`, bringing the ordinary catalog to 30 methods.
It validates interrupted saved credentials with complete cloud inventory, confirms
unchanged bytes, and republishes through the existing identity guards. Stored
accounts use ordinary refresh. Synthetic host tests cover uncertain initial/map
writes, preserved organization, unchanged-secret recovery, rejected/partial reads,
corrupt/foreign material, exact-byte races, SQLite failures and host lifecycle.
C ABI guards use fictional metadata without a keyring. Seven C symbols, SQLite
schema 4 and protected payload version 1 remain unchanged. This phase did not
implement expired pending-session reauthorization; the API 1.8 extension is below.
Live recovery and mobile acceptance remain unverified.


## Pending Meiju reauthorization contract 1.8

API 1.8.0 permits protected fresh login for an explicitly selected pending account
with intact saved identity/bindings. Exact subject matching and unchanged-byte
checks precede replacement. Missing/corrupt material and unselected pending accounts
remain refused. Synthetic host tests cover success, preserved organization/grants,
identity rejection, failed login, byte races, persistence uncertainty and lifecycle.
Actual ABI tests cover input guards and clearing, not successful provider login.
The 30 methods, seven C symbols, JSON shapes, SQLite schema 4 and protected payload
version 1 remain unchanged. Older libraries refuse pending reauthorization; check
the version before enabling it. Real identity stability, mobile and physical
acceptance remain unverified. See [ADR-0031](adr/0031-pending-account-reauthorization.md).


## Android native credential backend

The Android Java/JNI backend uses per-profile Keystore keys and authenticated
ciphertext under the application no-backup directory. Synthetic JVM/JNI tests and
Java compilation against official API stubs passed. The complete API 28 arm64 AAR
built with NDK r28c; archive contents, native exports and 16 KiB ELF load alignment
passed inspection. This is library build evidence, not application acceptance.
Native API 1.8, seven C functions, 30 methods, SQLite schema 4 and protected Meiju
payloads are unchanged; Android adds a trusted platform-only JNI initializer.
Default Linux behavior is unchanged; iOS still fails closed without a backend.
Android OS/Keystore, APK, physical-phone and real account/control acceptance remain
unverified. The initial unreadable-entry removal limitation is corrected in 1.8.1
below. See [ADR-0032](adr/0032-android-native-credential-storage.md) and the
[Android guide](development/08-android-native.md).

## Local cleanup correction 1.8.1

Explicit account removal delegates exact-entry inspection/deletion to the backend
without decrypting credentials. Missing keys or damaged contents no longer block
cleanup when the backend can delete that exact entry. Locked/unavailable/ambiguous
storage and uncertain deletion still retain pending intent and metadata. Synthetic
host and JVM tests cover these cases; no owner account or device was changed.
The method count, request/response shapes, seven C symbols, schema 4 and protected
payload are unchanged. Older libraries may remain pending on a failed credential
read. No force flag, automatic erasure or provider revocation is added. See
[ADR-0033](adr/0033-delete-unreadable-account-credentials.md).

## Experimental AC fan contract 1.9

The four exact AC models in the read table have a separate, opt-in vendor fan
action for codes 20, 40, 60, 80, 100 and 102 from the pinned default AC mapping.
This is an experimental candidate mapping; exact firmware behavior, managed modes
and physical/off-LAN effects remain unverified. It is not brand-wide support or a
universal fan-speed enum. Unsupported models gain no action.

Native `meiju_fan_grants` permits exact-device/code cloud requests independently
of power grants. Existing startup, 30 methods, seven C functions, execution
outcomes and SQLite schema 4 are unchanged. Older libraries reject the new field;
negotiate API 1.9 before supplying it. Synthetic tests cover exact fields, scope,
identity, immutable permissions, uncertainty, replay and lifecycle behavior. No
household action was sent. See [ADR-0034](adr/0034-scoped-meiju-ac-fan-control.md).

## Inventory membership correction 1.9.1

Complete Meiju retrieval now records protected membership separately from retained
historical bindings. Missing targets refuse new reads/controls before transmission;
partial and failed retrieval preserve prior membership. Same-identity reappearance
restores admission without replacing IDs or local preferences. This is synthetic
account/Core/native evidence, not tested physical revocation behavior.

Version 1 protected payloads remain readable with unconfirmed membership; an
explicit complete refresh establishes it. Subsequent protected updates write
version 2, which older libraries refuse. No downgrade/reset or live-profile
migration was performed. Public response shapes, 30 methods, seven C functions
and SQLite schema 4 stay unchanged. New localized errors are documented in the
[native handoff](native-ui-contract.md#missing-devices-and-legacy-account-inventory).
No public online-state field, background refresh or new control is introduced.

Local validation for this correction passed: 256 Rust workspace tests (two explicit
system probes ignored), Clippy with warnings denied, formatting/build checks,
24 Mock runtime cases, the 30-method/50-example Linux ABI suite, 23 Python tests,
five presentation tests, and docs/contracts/repository checks. The Android arm64
AAR was rebuilt and passed export/archive/16 KiB alignment checks. No remote CI,
Flutter/APK, physical-phone, actual Keystore or appliance test was performed.

## Saved inventory inspection 1.10

`inspect_account_inventory` adds one native JSON method (31 total). It returns
sorted local device/connection IDs and saved membership for one selected account,
including historical, unsupported and unpublished bindings. It reads protected
material under a lease, without cloud work, mutation, descriptor publication or
permission changes. It never upgrades listed membership to online/authorized.

Existing metadata-only `get_account` behavior, seven C functions, SQLite schema 4
and protected payload version 2 remain unchanged. Legacy bindings are returned as
unconfirmed without migration. Older libraries reject the method; negotiate it
through the catalog. Synthetic host and Linux ABI coverage is described in
[testing](testing.md). Flutter rendering and physical/provider acceptance remain
unverified. No real account was inspected during this phase.

Local verification passed 261 Rust workspace tests (two opt-in system probes
ignored), Clippy/format/build checks, 24 Mock runtime cases, all 31 Linux native
methods with 51 examples, 23 Python tests, five presentation tests and contract,
document and repository checks. The rebuilt Android arm64 AAR passed export,
archive and 16 KiB alignment inspection; no APK, physical-phone, actual Keystore
or remote CI acceptance is claimed.

## Android emulator ABI packaging

The Android builder additionally accepts `--abi x86_64`, or both supported ABIs
for one combined AAR. The default arm64 command and artifact path remain stable.
The x86_64 target is for compatible devices/emulators; no 32-bit support is added.
Native API 1.10.0, its 31 methods and seven C functions, SQLite schema 4 and
protected payload version 2 remain unchanged. Use one AAR dependency per app.

Local arm64, x86_64 and combined release builds passed with Rust 1.96.0 and NDK
r28c at API 28, with eight exports per ABI, correct ELF architecture, 16 KiB load
alignment and production-only archive checks. All 29 Python tests, including six
package admission/failure tests, and document/contract/repository/commit checks
passed. No Flutter worktree, real account or appliance was accessed by this phase.
Emulator startup, APK integration, Android Keystore and physical/off-LAN acceptance
remain unverified; no emulator was available in the inspected SDK. See the
[Android build and startup guide](development/08-android-native.md).

## Android file-lock portability

The first Android 15 x86_64 app launch exposed unsupported standard-library file
locking in Rust 1.96. An isolated Android executable reproduced `Unsupported` for
shared, exclusive and unlock operations. The storage implementation now uses
Bionic `flock` only on Android, retaining nonblocking migration, device and account
leases and release on close. Other targets retain their existing locking calls.
All 13 storage tests passed on Linux and an Android 15 x86_64 emulator, including
shared/exclusive contention, release, migrations, rollback and replay retention.
There is no lock-free fallback or change to SQLite schema 4, the native API, lock
filenames, write authority or outcome semantics. See the app's
[verification report](../apps/mobile/verification.md) for runtime results.

## Meiju CLI account and read entrypoint

The CLI accepts explicit `--adapter midea.meiju` and reuses the native account
session for protected Linux terminal connection, listing/details, rename/removal,
saved membership inspection, refresh and recovery. Device reads and local
organization/history/diagnostics use the same Core. The default Meiju profile is
separate from Mock; explicit profile selection can reopen existing native data.
Account and credential mutations retain the existing identity, lease, pending
state and uncertainty rules. No password, provider identifier or session material
is accepted in CLI arguments, piped input or ordinary JSON.

This phase supplied no Meiju appliance grants and refused writes before profile
startup; the following scoped-control phase adds explicit admission. Mock behavior
and its MCP surface remain compatible. No native API,
capability, SQLite or protected payload version changes; native API 1.10.0 still
has 31 methods and seven C functions. Terminal login requires Linux; this does not
provide mobile input or storage. See the [CLI contract](cli.md).

Local verification passed 265 Rust tests (two opt-in system probes ignored),
Clippy with warnings denied, formatting/build checks, 28 runtime cases, the
31-method/51-example native ABI suite, 29 Python tests, five presentation tests,
and document/contract/repository checks. The final input-buffer bound was rechecked
with the CLI unit suite and Clippy. Tests use fictional profiles, local PTYs and
an unavailable test D-Bus address. No owner profile, vendor login/read, device
write, Flutter worktree, phone or remote CI acceptance was performed by this phase.

## Explicit Meiju CLI control grants

The Linux CLI now accepts an explicitly selected `--meiju-grants` file containing
exact local-device power states and independent vendor fan codes. It reuses the
existing native grant validation, Core execution and protected adapter checks.
Default startup remains read-only; a file cannot admit elevated-risk actions or
other appliance controls. Whole-file validation, ownership/mode checks, final
symlink/nonregular-file refusal and a 32 KiB bound precede profile startup.
See [ADR-0037](adr/0037-cli-meiju-control-grants.md) and the
[CLI workflow](cli.md#scoped-meiju-controls).

Single Meiju writes and previews require explicit request IDs. Existing batch
files retain per-command IDs, complete preflight and individual receipts.
Discovery/account refresh explicitly publishes the admitted actions; planning
does not refresh them or contact the provider. Control still distinguishes
not-sent failures, acceptance and uncertainty without automatic retry or physical
verification. Receipt lookup remains available without grants.

Mock commands and MCP are unchanged. This adds a CLI option, one schema/fictional
fixture and an additive Rust host helper; native API 1.10.0 still has 31 JSON
methods and seven C functions. SQLite schema 4, protected payload version 2,
capabilities and execution outcomes are unchanged. No mobile artifact rebuild is
required by the CLI option.

Local verification passed 268 Rust tests (two opt-in system probes ignored),
Clippy with warnings denied, formatting and workspace build, 30 actual CLI runtime
cases, the 31-method/51-example native ABI suite, 29 Python tests and five
presentation tests. Contract validation passed 17 schemas, 30 positive fixtures
and 105 negative fixtures; document/repository/commit checks also passed. The
runtime grant cases use fictional SQLite profiles, pending accounts and an
unavailable test D-Bus endpoint. No owner profile, vendor request, appliance
control, Flutter worktree, phone or remote CI was tested by this phase. Physical
model, off-LAN and real CLI acceptance remain pending.

## Meiju local MCP and scoped command tools

Explicit Meiju MCP startup reuses the connected profile and embedded Core. Default
sessions expose the four existing read tools; nonempty owner-controlled startup
grants add `command_plan` and `command_execute`. Each command includes its own
request ID, bounded timeout, route and wait condition. Core permissions, exact
model admission, protected account/membership checks and conservative outcomes
remain unchanged. Tool parameters cannot select credentials, profile, principal
or new permissions. No account/discovery/batch tool is added. Mock MCP stays
read-only. See [ADR-0038](adr/0038-scoped-meiju-mcp.md) and the [MCP guide](mcp.md).

This supersedes the earlier Meiju MCP startup refusal. MCP now rejects formerly
ignored global request/deadline/dry-run and nondefault route/wait options. Query
tools preserve their envelopes; command execution returns the existing Execution
envelope with outcome-aware `isError`. Plans do not send or journal. RPC IDs and
durable command IDs remain separate. Grant-file edits take effect only after
session restart. Native API 1.10.0, its 31 methods/seven C functions, SQLite schema
4 and protected payload version 2 are unchanged.

EOF/SIGINT, input/output failures and a two-second output-backpressure deadline
cancel active operations and drain their cleanup. Synthetic sent operations retain
unknown receipts after reopening; no automatic replay or physical rollback occurs.
The existing bounded CLI runtime shutdown also prevents blocked stdio workers
from keeping the process alive. Actual pipe closure/backpressure tests passed.

Local verification passed 271 Rust tests (two opt-in system probes ignored),
Clippy with warnings denied, formatting/workspace build, 32 runtime acceptance
cases, the 31-method/51-example native suite, 29 Python tests, five presentation
tests and document/contract/repository/commit checks. The sandboxed SIGINT case
timed out; the focused and complete runtime suites passed with ordinary process
signal access. Tests use fictional profiles and no owner credentials, vendor
requests or appliance controls. No target AI host configuration, Flutter worktree,
phone, physical/off-LAN behavior or remote CI was tested by this phase.

## Explicit AC power-on and mode activation

Native API 1.11.0 adds `meiju_mode_grants` for
`main / vendor.midea.ac.activate_mode`. Each exact local-device/mode grant covers
both turning on the AC and selecting `cool`, `heat`, `auto`, `dry` or `fan` in one
cloud request. Existing power and fan grants remain independent; default startup
is read-only. The four previously admitted AC models expose this candidate action
only after explicit opt-in and discovery/refresh. The mode-enabled vendor
capability has revision 3. See [ADR-0039](adr/0039-explicit-meiju-mode-activation.md)
and the [native handoff](native-ui-contract.md#scoped-experimental-ac-mode-activation-startup).

The formal CLI and MCP accept the same optional grant list. Ordinary commands
cannot change permissions. The action sends only power-on and the selected mode;
no cached status, temperature or fan values and no separate power request are
sent. Acceptance remains unverified; possible sends retain unknown outcomes
without retry, cache mutation or a restoring command. UI labels and consent name
both effects. Physical behavior and preservation of unrelated settings remain
unverified, including the missing status-context limitation from ADR-0034.

Existing Rust host helpers, 31 JSON methods, seven C functions, command/result
schemas, SQLite schema 4 and protected payload version 2 remain compatible.
Older binaries reject the new startup field; negotiate API 1.11 before supplying
it. The English/Chinese catalogs include mode labels and explicit power-on consent.

Local verification passed 278 Rust tests (two opt-in system probes ignored),
Clippy with warnings denied, formatting/workspace build, 33 runtime cases,
the 31-method/51-example native ABI suite, 29 Python tests and five presentation
tests. Contract checks passed 17 schemas, 32 positive and 111 negative fixtures;
document/repository/commit checks passed. The full Rust/runtime suites ran with
ordinary local socket and signal access. The API 28 arm64/x86_64 combined AAR was
rebuilt with NDK r28c, JDK 21 and API 35 stubs; archive contents, eight exports per
ABI and 16 KiB ELF alignment passed. This does not establish APK, emulator,
Android Keystore or phone acceptance.

Tests used fictional profiles, synthetic protected storage and local protocol
fixtures. This phase accessed no owner account or appliance and did not modify
the separate Flutter worktree. Physical/off-LAN acceptance remains deferred;
model-specific AC temperature bounds, dishwasher/refrigerator controls and the
complete mobile journey remain incomplete. Remote CI was not verified.

## Protected product mapping evidence 1.11.1

The existing Meiju inventory now retains optional `sn8`, `modelNumber` and
`category` as private product mapping evidence under
[ADR-0040](adr/0040-protected-meiju-product-profiles.md). Numeric subtype
representations are canonicalized, absent/placeholder fields remain unknown,
and conflicting nonempty evidence refuses the whole binding update. Malformed
fields invalidate that home's response; partial retrieval preserves cards and
established membership. No additional endpoint, full serial-number decoding,
Lua/plugin download, device control or automatic refresh is introduced.

Protected account payload version 3 stores this evidence with existing immutable
bindings. Versions 1 and 2 open without writes or invented values; a later explicit
protected mutation writes version 3. Missing later fields, session replacement,
pending reauthorization and recovery retain established evidence and local IDs.
Uncertain storage keeps the account pending. Older binaries cannot reopen version
3; preserve the profile and upgrade the library instead of resetting accounts.
Native API 1.11.1 identifies this storage change, with the same 31 JSON methods,
seven C functions, public DTOs and SQLite schema 4. Source inventory gains an
opaque, nonserializable `ProductProfile`; external Rust literal constructors must
initialize its new field. No new model or control is admitted by this evidence.

Local verification on October 16, 2026 passed 287 Rust tests (two opt-in system
probes ignored), Clippy with warnings denied, formatting, the workspace build,
`cargo build --workspace --examples --offline --locked`, 33 runtime cases,
the 31-method/51-example native ABI suite, 29 Python tests and five presentation
tests. Contract checks passed 17 schemas, 32 positive and 111 negative fixtures;
document/repository/commit checks passed. Rust/runtime suites used ordinary local
socket and signal access. Native session tests cover protected evidence capture,
unchanged descriptors, pending reauthorization, exact-byte recovery without
relogin, and absent profile markers in public responses and SQLite/WAL.

The API 28 arm64, x86_64 and combined Android AARs were rebuilt using NDK r28c,
JDK 21 and API 35 stubs; production archive contents, eight exports per ABI and
16 KiB ELF alignment passed. No APK, emulator, Android Keystore, phone, real
account/profile, appliance or remote CI acceptance was performed. The separate
Flutter worktree was untouched. Model-specific temperature/program constraints,
physical/off-LAN controls and the complete mobile journey remain incomplete.

## Profile-scoped AC temperature 1.12

Native API 1.12.0 adds independent `meiju_temperature_grants` for one exact local
AC and Celsius target under [ADR-0041](adr/0041-profile-scoped-ac-temperature.md).
Native, CLI and MCP use the same startup-only policy; ordinary commands cannot
change authority. The startup target union never overrides the selected action's
bounds/step. Existing Rust host helpers remain compatible wrappers around the
new borrowed `MeijuControlGrants` bundle. Older native binaries reject the new
field. The 31 JSON methods, seven C functions, SQLite schema 4 and protected
account payload 3 remain unchanged.

Only the four existing experimental AC models with reviewed protected product
evidence expose `main / climate.set {target_temperature}` after explicit refresh
or discovery. Selection preserves subtype/product/category priority, normalized
category spellings and explicit unsupported overrides, including fresh-air product
26096947. Missing evidence never selects a generic family default. Retained
profiles survive later omitted inventory fields and are used without another
provider request. Public descriptors contain selected constraints, not raw
product evidence. Writable limits do not suppress valid historical observations.

The adapter reselects the protected mapping before transport and sends only exact
whole/half-degree temperature field(s). It does not round invalid input, include
power/mode/fan or cached status, update optimistic state, read after acceptance,
retry, fall back or restore settings. Cloud acknowledgement remains accepted;
uncertain transmission remains unknown. Source mappings do not prove physical
support or preservation of unrelated settings without upstream status context.

Local verification on October 16, 2026 passed 296 Rust tests (two opt-in system
probes ignored), formatting, Clippy with warnings denied, workspace and example
builds using `--offline --locked`, 34 runtime cases, the 31-method/51-example
native ABI suite, 29 Python tests and five presentation tests. Contract checks
passed 17 schemas, 34 positive and 117 negative fixtures; document, repository
and commit checks passed. Rust/runtime suites used ordinary local socket and
signal access. Tests cover all reviewed selector groups, missing/conflicting
admission conditions, exact encoding, weakened descriptors, independent policy,
protected membership, receipts/replay and foreground interruption.

The API 28 arm64, x86_64 and combined Android AARs were rebuilt with NDK r28c,
JDK 21 and API 35 stubs. Production archive contents, eight exports per ABI and
16 KiB ELF alignment passed. This phase used fictional profiles and synthetic
storage/transport only. No owner account, credential profile or appliance was
accessed; the separate Flutter worktree was untouched. APK/emulator/Keystore/phone,
physical/off-LAN controls, dishwasher/refrigerator writes and remote CI remain
unverified. The full real-device MVP remains incomplete.

## Profile-aware scalar AC readings 1.12.1

The prior decoder required `small_temperature` for every AC target. This dropped
a valid `temperature` reading for the reviewed single-field mapping even though
API 1.12 could send its scalar setpoint. API 1.12.1 uses the existing protected
selector for reads as well: product code 23096245 with a known non-overriding
subtype reads `temperature` alone, matching the pinned climate getter. Subtype
524, compound profiles and missing evidence retain the two-field contract.
No extra provider request, implicit refresh, control opt-in or raw-profile export
is introduced. Missing/null values remain absent, invalid scalar values refuse
the response without overwriting cache, and historical values are not constrained
by write bounds or steps. Cloud freshness and write outcomes remain unchanged.

The new regression was executed against the prior committed decoder and failed
because the expected target 26 was absent. After the fix, local verification on
October 16, 2026 passed 299 Rust tests (two opt-in system probes ignored),
formatting, Clippy with warnings denied, workspace and example builds using
`--offline --locked`, 34 runtime cases, the 31-method/51-example native ABI suite,
29 Python tests and five presentation tests. Contract validation passed 17 schemas,
34 positive and 117 negative fixtures; document/repository/commit checks passed.
Tests cover scalar/compound selection, subtype priority, numeric strings,
malformed/nonfinite targets, missing fields, unproven freshness, retained protected
evidence after sparse inventory, read-only Core sessions and cache preservation.

All three API 28 Android AARs (arm64, x86_64 and combined) were rebuilt using
NDK r28c, JDK 21 and API 35 stubs; archive contents, eight exports per ABI and
16 KiB ELF alignment passed. Native methods, C functions, public schemas,
SQLite schema 4 and protected payload 3 are unchanged. No real account,
credential profile or appliance was accessed; the separate Flutter worktree was
untouched. Physical/off-LAN, phone/Keystore/APK and remote CI acceptance remain
unverified. Dishwasher and refrigerator controls remain incomplete.

## Typed trusted single-command approval foundation

[ADR-0042](adr/0042-trusted-single-command-approval.md) adds Rust Core/Session
preparation and one-use execution for one elevated action. A complete matching
grant, one explicit transport and exactly one registered connection remain
required. The opaque preview binds its command, descriptor, Core instance,
approval generation and 60-second monotonic lifetime. Preparation performs no
adapter/credential/network work and does not constitute consent. Trusted UI code
must display and confirm the immutable view before consuming it. Wrong-host,
expired, changed or invalidated scopes refuse before sending; high-risk, batch
and independently decomposed actions remain excluded.

Core supplies an approval marker to adapters only on this validated path.
Consequential adapters must check it independently of public risk metadata.
Session background/close invalidates pending previews and cancels active tokens;
a possible send still becomes unknown without replay or rollback. Existing
journaling, device locking, retained receipts and request retirement remain
in force. A fixed warning records approval for the exact journaled command.
`APPROVAL_INVALID` and `APPROVAL_EXPIRED` are explicitly unsent in the shared
receipt schemas and have English/Chinese presentation messages.

This is a Rust integration foundation, not a new appliance capability. Native
API 1.12.1, 31 JSON methods, seven C functions, startup grants, SQLite schema 4
and protected payload 3 are unchanged. Existing JSON/C/CLI/MCP entry points cannot
prepare or consume approval. The trusted C/Flutter bridge, UI consent and
model-specific dishwasher/refrigerator preservation/control mappings remain
incomplete; this phase does not admit household operation.

Local verification on October 16, 2026 passed 309 Rust tests (two opt-in system
probes ignored), formatting, Clippy with warnings denied, offline locked workspace
and example builds, 34 runtime cases, the 31-method/51-example native ABI suite,
29 Python tests and five presentation tests. The new eight-case Rust suite also
validates actual typed receipts against the shared JSON schema. Tests cover exact
scope/grants, risk and route refusal, ownership, expiry, descriptor changes,
revocation, duplicate preparation/replay, retired IDs, journal failures, adapter
marker enforcement, host lifecycle and uncertainty. Two compile-fail checks
prevent cloning/serialization. Actual C calls reject invented approval methods
and forged command fields. Contract checks passed 17 schemas, 37 positive and
122 negative fixtures; document/repository/commit checks passed.

All three API 28 Android AARs were rebuilt with NDK r28c, JDK 21 and API 35 stubs;
archive contents, eight exports per ABI and 16 KiB alignment passed. Only fictional
devices, temporary profiles, synthetic storage and local protocol fixtures were
used. No owner account or appliance was accessed, and the separate Flutter
worktree was untouched. Phone/Keystore/APK, physical/off-LAN and remote CI
acceptance remain unverified. The full real-device MVP is still incomplete.

## Trusted native single-command approval 1.13.0

[ADR-0043](adr/0043-native-trusted-approval-bridge.md) extends the typed approval
foundation with three separate trusted C functions: prepare an immutable preview,
execute it after actual user consent, and discard it locally. Session retains at
most eight previews with the existing 60-second monotonic expiry. Confirmation
consumes a matching ID before admission; cross-handle, reused, discarded or pruned
IDs fail. Background/close clears previews and cancels active work, and a
lifecycle generation check rejects preparation spanning background/foreground.
No registry lock spans execution or networking. Normal journaling, non-replay,
uncertain outcomes and original-request receipt recovery remain unchanged.

Native API 1.13.0 has ten C functions and still 31 ordinary JSON methods. Approval
IDs are temporary trusted-controller state, never durable data, model arguments
or proof of consent. No general JSON/CLI/MCP confirmation path was added. Existing
clients can use the previous exports; new clients must negotiate 1.13 before
resolving the added symbols. SQLite schema 4 and protected account payload 3 are
unchanged. The new closed approval definition bundle, fictional inputs and native
contract describe query errors versus admitted Execution receipts.

The explicit Mock-only `approval-required` scenario adds a fictional elevated
cycle and exact grant alongside the existing simulated setters. Its adapter
requires Core's private approval marker even if cached risk metadata is weakened.
The scenario is refused for Meiju before profile creation. Every real Meiju
permission and mapping remains unchanged; no real account was opened or appliance
operated. Actual Flutter consent UI, real consequential mappings, mobile secure
store/runtime and physical/off-LAN acceptance remain pending.

Validation performed on October 16, 2026:

- `cargo fmt --all -- --check`, workspace Clippy with warnings denied, workspace
  build and example build passed, using locked offline dependencies.
- `cargo test --workspace --offline --locked`: 313 passed, 2 ignored. The approval
  suite has 12 tests, including native capacity/pruning, retained expiry,
  admission-failure consumption, cross-session isolation, concurrent confirmation
  and a deterministic background/foreground preparation race.
- `scripts/verify_mvp.py`: 34 runtime cases passed with local socket/signal access.
- `scripts/verify_native_contract.py`: 31 methods and 51 live fictional examples
  passed, plus actual calls of all three trusted exports. Dedicated checks cover
  scope, duplicate/concurrent consumption, dismissal/capacity, closed input,
  lifecycle, metadata changes, adapter marker enforcement and receipt lookup.
  `scripts/verify_native_approval.py` also passed the Meiju scenario refusal.
- Python tooling: 29 tests passed; presentation catalogs: 5 tests passed.
  Contract validation: 18 schemas, 39 positive fixtures and 135 negative fixtures.
  Documentation, repository hygiene and Conventional Commit checks passed.
- The combined, arm64-v8a and x86_64 Android AARs were rebuilt using NDK r28c,
  JDK 21 and API 35 stubs. Production archive contents, eleven exports per ABI
  (ten C functions plus JNI), ELF architectures and 16 KiB alignment passed.
  These are library artifacts, not APK, emulator, Keystore or phone acceptance.

## Profile-scoped dishwasher starts 1.14.0

[ADR-0044](adr/0044-profile-scoped-dishwasher-start.md) adds experimental elevated,
non-idempotent `vendor.midea.dishwasher.start_cycle` on `main`. Admission requires
the existing exact dishwasher model, protected known subtype/product evidence,
explicit adapter opt-in, an exact startup argument grant and trusted single-command
user confirmation. Ordinary plan/execute/CLI/MCP cannot approve it. The adapter
checks Core's marker before account access, then independently checks protected
family/model, identity, inventory membership, product mapping and arguments.
Public descriptor edits cannot supply missing authority or product evidence.

Five reviewed product codes select short, suffixed or reduced program lists.
760Y0026 requires an explicit additional-setting code in the confirmed command;
other admitted profiles reject that field. The start list excludes its cancellation
option. 7600016L remains unavailable because complete wash-region/door-preservation
domains have not been established. Unknown/missing products do not use the family
default. The owner's protected product-code match was not inspected in this phase.

Native API 1.14 adds `meiju_dishwasher_grants` with at most 64 distinct exact
local UUID-v4 / arguments pairs. Empty/absent lists preserve read-only startup.
The startup union does not establish any device's program support. Existing AC
authority is independent; CLI/MCP grant files have no elevated dishwasher field.
The unreleased Rust grant bundle gains a `dishwasher` slice; existing helper
functions preserve behavior and callers can use default fields. The 31 ordinary
methods, ten C functions, SQLite schema 4 and protected payload 3 are unchanged.
English/Chinese vendor-program, additional-option and confirmation labels were
added; the public identifiers stay English.

One cloud request contains only work status, the reviewed mode and any explicitly
required additional option. No implicit pre/post read, cached-state copying,
optimistic cache update, retry, fallback or restoration occurs. An acknowledgement
is accepted, possible transmission remains unknown, and original request receipts
prevent automatic duplicate starts. Protocol encoding and existing cloud outcome
handling do not establish physical completion or preservation of unrelated settings.

Validation performed on October 16, 2026:

- Formatting, workspace Clippy with warnings denied, workspace build and example
  build passed with locked offline dependencies.
- `cargo test --workspace --offline --locked`: 321 passed, 2 ignored. Six new
  dishwasher adapter tests cover every admitted program/encoding and additional
  option, read-only defaults, retained profiles, forged public metadata,
  protected model/family/product gates, host opt-in, approval enforcement, cloud
  outcomes, cancelled possible sends, receipt replay and inventory membership.
  Two host policy tests cover exact grant scope, invalid/duplicate lists and
  refusal before profile creation.
- `scripts/verify_mvp.py`: 34 runtime cases passed with local socket/signal access.
- `scripts/verify_native_contract.py`: 31 methods, 51 live fictional examples,
  existing trusted-approval checks and new native dishwasher startup/confirmation
  checks passed. New native tests use a fictional descriptor with no account:
  confirmation stops with `ACCOUNT_NOT_FOUND` before credential/provider access.
- Python tooling: 29 tests passed; presentation catalogs: 5 tests passed.
  Contract validation: 18 schemas, 41 positive fixtures and 140 negative fixtures.
  Documentation, repository hygiene and Conventional Commit checks passed.
- Combined, arm64-v8a and x86_64 Android AARs were rebuilt using NDK r28c, JDK 21
  and API 35 stubs. Production archives, eleven exports per ABI, ELF architectures
  and 16 KiB alignment passed. APK/emulator/Keystore/phone runtime remains untested.

No real account or appliance was accessed. Physical/off-LAN dishwasher operation,
remaining product mappings, refrigerator writes, Flutter consent and mobile
acceptance remain outstanding; this stage does not complete T10 or the MVP goal.

## Refrigerator evidence review and presentation correction

On October 16, 2026, the [refrigerator review](development/05-midea.md#refrigerator-write-evidence)
compared the exact-model manufacturer manual with the pinned community mapping
and centralized setter. This documents a write-admission blocker, not implemented
refrigerator control. Existing read descriptors and observation ranges are unchanged.

The presentation helper now resolves numeric observation labels by exact property
and value, so the existing AC code 102 displays Auto. Unknown codes still use
locale number formatting; typed values and evidence remain intact. A regression
test reproduced the numeric-label failure before the fix. All six presentation
tests and 29 Python tooling tests passed; documentation, contracts and repository
hygiene checks passed. These changes do not alter the native API, C exports,
SQLite schema or protected account format. No Rust/Android build, Flutter/mobile
runtime or real account/device test was performed for these changes.

## Android file locks and isolated native APK acceptance

The first isolated AOSP Android 15 x86_64 run failed to open a native profile.
A minimal target executable showed Rust 1.96 `File::try_lock_shared` and
`File::try_lock` returning `Unsupported`, while Bionic `flock` succeeded. Android
storage now uses nonblocking Bionic locks for migration, device and account leases;
other platforms keep standard-library locking. File names, ownership lifetimes,
contention errors, native API 1.14.0, SQLite 4 and protected payload 3 are unchanged.
The existing locked `libc` version is now an explicit Android storage dependency.

The [test-only APK](development/08-android-native.md#run-the-isolated-native-acceptance-apk)
embeds the production AAR with no requested permissions, including Internet.
On October 16, 2026, image revision 2 and Emulator 37.2.12 passed unsecured, write,
locked and read phases with 17/17/16/27 assertions. Evidence covers native Mock
execution/lifecycle, independent lock contention, Android Keystore round trips and
nonexportable keys, persistence in a different process, tampered ciphertext,
deliberately removed keys without regeneration, and Rust/JNI inspection/removal
of a fictional protected account. A failed lock-state precondition is not store
evidence; both OS transitions were confirmed before successful reruns.

Formatting, Linux workspace and Android-storage Clippy passed with warnings denied.
Workspace and example builds passed; Rust tests: 321 passed, 2 ignored. The MVP
runtime suite passed 34 cases; native checks passed 31 methods/51 live fictional
examples plus trusted approval/dishwasher cases. Python tooling: 33 passed;
presentation: 6 passed. All three AAR variants were rebuilt and inspected; the
x86_64 test APK passed signing, permission and 16 KiB ZIP-alignment checks.

Physical/OEM Keystore behavior, successful provider login/session replacement,
Flutter APK journeys, release shrinking, actual 16 KiB-page execution and real
appliance operation remain unverified. No owner account, real device, separate
Flutter worktree or published package was used in this phase.

## Temperature grant capacity correction 1.14.1

The original 64-entry startup limit rejected a four-AC configuration containing
29 exact candidate targets per device (116 pairs). A host regression reproduced
that refusal before the correction. API 1.14.1 permits 256 temperature pairs in
native startup and explicit CLI/MCP grant files. Other grant limits remain 64;
the CLI file limit remains 32 KiB. Each device's action constraints and protected
product mapping still apply independently; candidate targets do not establish
model support. There are no wildcard grants or changes to write outcomes.

On October 16, 2026, the native C ABI accepted 116 and 256 entries and rejected
257 distinct entries before profile creation. Planning all 116 candidate pairs
across four fictional descriptors preserved different minimum and whole/half-degree
step constraints. Actual CLI/MCP processes accepted the four-device configuration,
retained independent power denial and refused a pending-account execution before
sending. CLI also accepted 256 entries and rejected 257 before profile creation.

Formatting, workspace Clippy, workspace/example builds and local validation passed:
322 Rust tests with two explicitly ignored integration tests; 35 MVP runtime cases;
31 native methods and 51 live fictional examples plus trusted approval/dishwasher
checks; 33 Python tooling tests; six presentation tests; 18 schemas, 41 positive
fixtures and 141 negative fixtures. All three Android AAR variants were rebuilt
with eleven exports per ABI and 16 KiB ELF alignment checks. The prior isolated
Android runtime acceptance was not rerun for this capacity correction.

Clients must negotiate API 1.14.1 before supplying more than 64 temperature pairs;
older libraries reject the larger configuration, so never silently truncate it.
JSON shapes, methods, C exports, SQLite schema 4 and protected payload 3 are
unchanged. No real account, appliance operation, Flutter worktree change, mobile
runtime acceptance or package publication was performed in this phase.

## Bounded LAN discovery decoding

[ADR-0045](adr/0045-bounded-lan-discovery-decoding.md) adds the fixed Midea probe
and an offline V2/V3-wrapped discovery decoder. Eight independently generated
positive vectors and fifteen negative vectors cover family hints, minimum and
maximum sizes, checksums, padding, lengths and field validation. Every truncation
boundary is rejected. The six-byte ID excludes adjacent reserved bytes; the
endpoint uses the actual supplied datagram source. Candidates have redacted Debug
and no serialization implementation, and retain no serial, MAC, SSID or model.

The public wire key/checksum do not authenticate a device. No sockets, account
access, pairing, identity merge, LAN route registration or control are implemented
by this module. Native API 1.14.1, its 31 methods and eleven C/JNI exports, SQLite
schema 4, protected payload 3 and cloud execution semantics remain unchanged.
Bounded collection, protected reconciliation, V3 session authentication, TCP/model
codecs and authorized physical/off-LAN acceptance remain separate gates. This
phase does not complete T10 or establish support for a physical product.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
local validators passed. Rust tests: 326 passed, two explicitly ignored; runtime
acceptance: 35 passed; native contract: 31 methods/51 fictional examples plus
approval checks; Python tooling: 33 passed; presentation: six passed. All three
Android AAR variants passed build/export/16 KiB ELF checks. No real account,
appliance, Flutter worktree or new Android runtime test was used in this phase.

## Bounded LAN collection

The follow-up to [ADR-0045](adr/0045-bounded-lan-discovery-decoding.md) adds a native
UDP collector for an explicitly selected private/link-local subnet. It binds to
the supplied local interface, sends one probe per reviewed port with TTL 1, and
uses a fixed five-second window within the existing Operation. All datagrams
count toward the 256-response limit; 64 distinct candidates also ends collection
with `limit_reached`. Identical hints are deduplicated while conflicts remain
untrusted candidates. No result establishes a complete inventory or permission.

Six loopback tests exercise real UDP sockets with fictional responses, including
invalid scope/source refusal, malformed and oversized packets, duplicate and
conflicting hints, both limits, deadline/cancellation and rebinding after normal
completion or future drop. The public scope rejects loopback; destination and
scope overrides are private test fixtures. No household scan was performed.

Native API 1.14.1, its methods/exports, SQLite schema 4, protected payload 3 and
cloud execution are unchanged. The collector is not registered in Core or any
entry point. Host network selection/consent, protected identity reconciliation,
V3 authentication, TCP/model codecs and physical LAN acceptance remain pending.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
local validators passed. Rust tests: 333 passed, two explicitly ignored; runtime
acceptance: 35 passed; native contract: 31 methods/51 fictional examples plus
approval checks; Python tooling: 33 passed; presentation: six passed. All three
Android AAR variants passed build/export/16 KiB ELF checks. No Android network
runtime, real account, appliance operation or separate Flutter worktree was used.

## Bounded V3 LAN wire codec

[ADR-0046](adr/0046-bounded-v3-lan-codec.md) adds protected native handshake and
session codecs. Independent OpenSSL/hashlib vectors cover handshake key
confirmation and twelve aligned/unaligned encrypted request/response frames.
Malformed, truncated, oversized, wrong-direction and failed-digest inputs expose
no plaintext. Outgoing counters stop before wrap; the 4096-byte frame limit and
credential sizes are scoped admission limits, not physical compatibility claims.

The encoder includes digest bytes in the declared length even when no random
padding is required. Keys and plaintext use protected buffers and enabled CBC/AES
cleanup features; constant-time digest checks and OS randomness reuse existing
locked dependencies. No dependency version changed. Native API 1.14.1, JSON
methods, C exports, SQLite schema 4 and protected payload 3 are unchanged.

No TCP connection, live credential retrieval, persistence, route admission or
appliance operation is implemented by this codec. A valid digest/sequence does
not establish device identity, freshness, anti-replay or physical execution.
Protected identity/credentials, bounded TCP transport, inner packet/model checks
and physical acceptance remain separate gates.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
local validators passed. Rust tests: 340 passed, two explicitly ignored; runtime
acceptance: 35 passed; native contract: 31 methods/51 fictional examples plus
approval checks; Python tooling: 33 passed; presentation: six passed. All three
Android AAR variants passed build/export/16 KiB ELF checks. No Android network
runtime, real account, appliance operation or separate Flutter worktree was used.

## Operation-owned V3 TCP exchange

The transport follow-up to [ADR-0046](adr/0046-bounded-v3-lan-codec.md) adds one
scoped TCP connection, handshake and command exchange within the inherited
Operation. Header-first reads bound allocation and preserve frame boundaries.
Preflight rejects out-of-scope/self/broadcast/network endpoints, port zero,
invalid payload sizes and reused transmission markers. There is no reconnect,
retry, pooling, background worker or connection returned to the caller.

Eight loopback tests cover exact fictional handshake/command bytes, fragmented
and coalesced frames, connection and handshake refusal, malformed/lost responses,
and socket closure before/after command transmission on cancellation, deadline
and dropped futures. Handshake failures leave the command not sent; starting its
write marks possible transmission, which errors cannot clear. A valid response
does not mark acceptance or prove identity, freshness or physical execution.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
local validators passed. Rust tests: 348 passed, two explicitly ignored; runtime
acceptance: 35 passed; native contract: 31 methods/51 fictional examples plus
approval checks; Python tooling: 33 passed; presentation: six passed. All three
Android AAR variants passed build/export/16 KiB ELF checks. Android network runtime
and physical device acceptance were not exercised.

Native API 1.14.1, JSON methods, exports, SQLite schema 4, protected payload 3 and
cloud behavior are unchanged. No adapter/entry point invokes this low-level
helper yet. Protected identity/credentials, current membership, grants, model
admission and inner response evidence must gate future real LAN access. No real
account, household endpoint or separate Flutter worktree was accessed.

## Account-bound Meiju LAN credentials

[ADR-0047](adr/0047-account-bound-lan-credentials.md) adds an explicit native
credential lookup from the exact protected local device binding. Last complete
membership, region, session and six-byte ID checks precede any request. Fresh
home listings must yield one target with consistent identity/product evidence;
partial, conflicting or ambiguous results never trigger a token request. One
reviewed Meiju v2 request selects exactly one UDP-ID entry and returns protected
V3 handshake material. No scan, LAN peer access, persistence of Token/Key,
default key, v1 fallback or automatic alternate method is added.

Successful explicit login now saves protected inner session version 2, retaining
the original login client identifier required by the complete token request.
Version-1 sessions remain usable for existing cloud functions and are never
rewritten implicitly. Only the new lookup requires explicit same-account
reauthorization when that identifier is absent. Tests preserve the account and
device bindings through this upgrade and keep the identifier out of SQLite and
public metadata. Older binaries refuse inner version 2; do not reset bindings.

Native API 1.14.1, all 31 JSON methods/11 C and JNI exports, SQLite schema 4 and
outer protected payload 3 are unchanged. The helper is not registered as a LAN
route and requires cloud connectivity; endpoint reconciliation, permissions,
inner packet/model codecs and physical/off-LAN acceptance remain unfinished.
No real account, appliance, existing pilot store or Flutter worktree was accessed.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
local validators passed. Rust tests: 359 passed, two explicitly ignored; runtime
acceptance: 35 passed; native contract: 31 methods/51 fictional examples plus
approval checks; Python tooling: 33 passed; presentation: six passed. All three
Android AAR variants passed build/export/16 KiB ELF checks. No fresh emulator,
phone, live credential, physical or off-LAN acceptance was performed.

## Ordinary LAN packet envelopes

[ADR-0048](adr/0048-bounded-lan-packet-envelopes.md) adds pure native encoding,
bounded prefix lengths and exact decoding for ordinary `5a5a` packets. The codec
checks the expected six-byte ID, salted MD5, declared length and PKCS#7 padding
before returning protected plaintext. Its 4040-byte envelope cap fits V3's
4096-byte frames. The explicit outgoing UTC timestamp has no freshness meaning.

Seven independent positive vectors and sixteen negative vectors use fictional
data and Python hashlib/OpenSSL. The unchanged pinned upstream PacketBuilder
class, with a fixed clock and OpenSSL cipher substitute, independently matches
all seven positive packets. Rust tests cover every truncation, concatenated frame
boundaries, corrupt/unsupported packets, wrong IDs and reserved bytes, input/time
limits, maximum V3 fit and an independently encrypted nested V3 response.

No native JSON method, route, connection, credential lookup, storage format or
write outcome changes. API 1.14.1, 31 methods/11 C and JNI exports, SQLite schema
4, outer payload 3 and inner session formats 1/2 remain unchanged. A valid packet
is not an authenticated peer, model observation or acknowledgement. Inner model
codecs, protected endpoint admission and physical acceptance remain pending.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
local validators passed. Rust tests: 365 passed, two explicitly ignored; runtime
acceptance: 35 passed; native contract: 31 methods/51 fictional examples plus
approval checks; Python tooling: 33 passed; presentation: six passed. All three
Android AAR variants passed build/export/16 KiB ELF checks. No real account,
household endpoint, Flutter worktree or fresh Android runtime was accessed.

## Standard AC LAN query and C0 candidates

[ADR-0049](adr/0049-ac-lan-query-and-c0-decoding.md) adds a pure standard AC query
encoder with explicit protocol/sequence inputs and a strict AA query/C0 decoder.
The native result contains power, raw mode/fan codes and candidate Celsius values;
missing sensors remain unknown. Extended-body decimal rules preserve signed
truncation, invalid decimals fail, and explicit Fahrenheit is unsupported.

Four query vectors match the unchanged pinned query/AA classes, eight positive
C0 vectors match its unchanged field parser, and all 256 CRC lookup entries
match the bitwise port. Twenty-four negative fixtures and truncation/context
tests reject malformed or unrelated messages. The packet-to-C0 chain independently
checks expected device identity and inner protocol context without a socket.

No model, route, control, observation freshness or physical support is admitted.
The pinned source documents stale C0 temperatures for product 22013279; exact
protected model/profile selection remains necessary. Native API 1.14.1, JSON
methods, C/JNI exports, SQLite schema 4, outer payload 3 and inner sessions 1/2
are unchanged. No account, appliance or independent Flutter worktree was accessed.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
local validators passed. Rust tests: 371 passed, two explicitly ignored; runtime
acceptance: 35 passed; native contract: 31 methods/51 fictional examples plus
approval checks; Python tooling: 33 passed; presentation: six passed. All three
Android AAR variants passed build/export/16 KiB ELF checks. New emulator, phone,
live credentials and physical/off-LAN behavior were not tested in this phase.

## Protected LAN candidate reconciliation

[ADR-0050](adr/0050-protected-lan-candidate-reconciliation.md) adds a native
account helper that compares a selected protected local binding with an explicit
network and discovery report. It requires saved listed membership, supported
family and six-byte identity. It returns missing, incomplete or ambiguous hints,
or borrows one unique candidate without creating a device/connection or sending
credentials. Matching public fields is not peer authentication, current account
authorization, model compatibility or globally complete discovery.

Seven fictional account test groups cover the three families and V2/V3 hints,
duplicate/reordered entries, selected identity/address/protocol/family conflicts,
foreign IDs at one endpoint, unrelated conflicts, report limits/counters and
invalid peer scopes. Exact local-reference and cross-account checks, unknown or
absent membership, unsupported identities/families, unavailable secrets and
cancelled/expired operations fail without fallback. Storage/card preservation,
redacted Debug and a compile-fail serialization check are also verified.

Native JSON API 1.14.1, C/JNI exports, SQLite schema 4, protected payload/session
formats and cloud descriptors/execution remain unchanged. No public schemas or
new error codes are introduced. Model-specific endpoint admission and composed
LAN routes remain unfinished; T10 is not complete.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
local validators passed. Rust tests: 379 passed, two explicitly ignored; runtime
acceptance: 35 passed; native contract: 31 methods/51 fictional examples plus
approval checks; Python tooling: 33 passed; presentation: six passed. All three
Android AAR variants passed build/export/16 KiB ELF checks. No real account,
appliance, separate Flutter worktree, fresh Android runtime or physical/off-LAN
acceptance was used in this phase. Remote CI is verified separately after push.

## Composed AC V3 read with message-version negotiation

[ADR-0051](adr/0051-bounded-ac-lan-read-sequence.md) adds strict appliance query
encoding/protocol extraction and `query_ac_v3`. The native helper accepts an
already-admitted endpoint, protected handshake and expected target. It performs
one handshake, obtains the message version, then sends one AC status query with
that version on the same TCP connection. The injected clock supplies a timestamp
for each packet. Both responses require expected target and inner context checks.
Errors, cancellation, deadline and drop close the connection without retry,
reconnect or partial state. Existing one-command write tracking is unchanged.

Three appliance query vectors and twelve protocol-field vectors match unchanged
classes from the pinned upstream source. Sixteen negative AA cases and all
truncations reject malformed or unrelated replies. Ten independently assembled
nested packet/V3 responses drive a fictional loopback peer that checks both
ordered request payloads, negotiated version, separate timestamps and request
counters. Reply counters deliberately differ: echo correlation is not claimed.
Lifecycle tests stop at either query; malformed negotiation stops before status.

The result remains native C0 candidates, not fresh state or physical support.
No automatic endpoint admission, account access, credential lookup, route/card
publication or public command is added. Exact model/profile rules and the full
mobile LAN journey remain unfinished. Native JSON API 1.14.1, C/JNI exports,
SQLite/protected formats and cloud behavior remain unchanged.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
local validators passed. Rust tests: 385 passed, two explicitly ignored; runtime
acceptance: 35 passed; native contract: 31 methods/51 fictional examples plus
approval checks; Python tooling: 33 passed; presentation: six passed. All three
Android AAR variants passed build/export/16 KiB ELF checks. No real account,
household endpoint, separate Flutter worktree, new emulator/phone or physical
acceptance was used. Remote CI is inspected independently after push.

## Trusted native connection linking

[ADR-0052](adr/0052-trusted-connection-linking.md) adds a typed Core/Session
preview and consuming confirmation for one same-identity connection. Adapter
evidence is rechecked within the bounded operation; SQLite compares effective
metadata atomically before appending the route while preserving raw labels and
local organization, cache and history. Existing potential transport grants are
visible in the preview; cloud-only grants remain cloud-only. Ordinary discovery
retains its strict connection-set guard.

Eleven integration test groups and three compile-fail checks cover proposal
scope, opaque previews, adapter refusal/revalidation, lifecycle/expiry/deadline,
locks, competing metadata/alias/account removal, repeated confirmation, restart,
preserved records, policy effects and ordinary JSON refusal. Mock and Meiju do
not opt in. Meiju protected route/model evidence, complete linked-set discovery,
mobile consent and physical acceptance remain unfinished.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds,
documentation/contracts/repository/commit validators passed. Rust: 399 passed,
two explicitly ignored; runtime: 35 passed; native: 31 methods/51 fictional
examples plus trusted approval checks; Python tooling: 33 passed; presentation:
six passed. All three Android AAR variants passed export and 16 KiB ELF checks.
Native JSON API 1.14.1, eleven C/JNI exports and storage versions are unchanged.
No real account, household appliance, separate Flutter worktree, new emulator,
phone or physical/off-LAN acceptance was used. This phase is committed and pushed
separately; remote CI results are checked after that push.

## Standard dishwasher and refrigerator LAN query candidates

[ADR-0053](adr/0053-dishwasher-refrigerator-lan-reads.md) adds standard AA E1/CA
query encoders, selected general-prefix decoders and composed V3 reads. Both
families share AC's bounded handshake/version-negotiation/query sequence. Native
candidates retain raw E1 status/program/option/temperature codes and time/shortage
bits, or CA main-compartment power and Celsius readings. No optional defaults,
Core Snapshots, public routes, cloud changes or physical support are added.

Eight query encodings and 19 field projections match unchanged pinned source
classes. Twenty-one positive and 40 negative AA vectors include two CA lengths
that exercise only the admitted prefix, avoiding upstream optional-field indexing.
All truncations and single-byte corruption, flag polarity, extension boundaries,
identity/context checks and redaction are tested. Independent nested vectors and
loopback peers verify two complete conversations and 23 invalid response paths,
plus cancellation/deadline/drop at either query without reconnecting. Existing AC
query and write-transmission tests continue to pass.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
local validators passed. Rust: 410 passed, two explicitly ignored; runtime: 35;
native: 31 methods/51 fictional examples plus approval checks; Python tooling: 33;
presentation: six. All three Android AAR variants passed export/alignment checks.
API 1.14.1 and storage formats remain unchanged. Exact-model admission, network
consent, linked-route integration and physical/off-LAN acceptance remain pending.
No real account, appliance, separate Flutter worktree, new emulator or phone was
accessed. Remote CI is inspected independently after the phase push.

## Trusted foreground Meiju LAN candidate scan

[ADR-0054](adr/0054-trusted-meiju-lan-candidate-scan.md) composes protected local
preflight, an immutable 60-second Session-owned device/network preview and one
consuming confirmed collection. The account lease spans discovery and protected
reconciliation. Only the selected native candidate or a non-unique outcome can
return, after identity, membership, descriptor and lifecycle revalidation. The
production collector keeps its bounded two-probe behavior; no cloud, token lookup,
TCP/state query, route publication, write or automatic retry is introduced.

Eight integration groups exercise the actual Session, adapter, vault and SQLite
using fictional accounts and scripted discovery, including AC/E1/CA, invalid and
conflicting reports, changed identity/metadata, retained leases, foreign/expired
previews, cancellation/background/close/deadline/drop and unchanged secrets/cards.
Three new compile-fail checks enforce nonserializable results and non-cloneable
previews. Ordinary JSON has no scan method; existing UDP loopback tests remain
the collector evidence. Mobile interface selection, permission/consent, the
bridge, endpoint/model admission, linked routes and physical acceptance remain
pending. Private address validation is not interface ownership evidence.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
documentation/contracts/repository/commit validators passed. Rust: 421 passed,
two explicitly ignored; runtime: 35; native: 31 methods/51 fictional examples
plus trusted approval checks; Python tooling: 33; presentation: six. All three
Android AAR variants passed eleven-export and 16 KiB alignment checks. Native
JSON API 1.14.1, C/JNI exports and storage formats remain unchanged. No owner
account, appliance, separate Flutter worktree, emulator or phone was accessed.
Remote CI is checked independently after the phase commit and push.

## Profile-scoped dishwasher cycle operations

[ADR-0055](adr/0055-profile-scoped-dishwasher-cycle-controls.md) adds independent
cloud pause/resume/cancel permissions for the existing exact experimental E1
model and protected products 7600020L, 76006481, 7600649Q and 7600V1E7. The clean
pinned community source establishes twelve action payloads and empty centralized
field lists. Products 760Y0026 and 7600016L remain excluded from these operations;
their existing start admission is unchanged. No owner's product code was read
or assumed. Optional raw operator observations remain stale and read-only.

Native API 1.15.0 adds `meiju_dishwasher_cycle_grants`, with at most 64 distinct
exact device/action pairs and no start authority. All actions require separate
trusted confirmation and independently enforce the approval marker in the
adapter. Their arguments are empty and no setting is copied from cache. Core
retains accepted/failed/unknown semantics, durable receipts and replay guards;
pause/cancel are not physical safety or rollback guarantees.

Six adapter integration groups, two native-policy groups and a state-decoding
group cover twelve exact payloads, protected model/profile/family and host
admission, centralized-profile rejection, malformed arguments and metadata/risk
tampering, independent permissions, unchanged observations, replay, membership,
pre-send/rejected/unknown results, cancellation/deadline and lease cleanup. Actual
C calls verify all three permissions through separate confirmation, and reject
invalid startup before profile creation using fictional descriptors and no
account. Explicit rediscovery upgrades old descriptors while preserving identity;
undeclared new observations fail without replacing the cache beforehand. No
CLI/MCP elevated permission is enabled.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
documentation/contracts/repository/commit validators passed. Rust: 430 passed,
two explicitly ignored; runtime: 35; native: 31 methods/51 fictional examples
plus approval and cycle-control checks; Python tooling: 33; presentation: six.
All three Android AAR variants passed eleven-export and 16 KiB alignment checks.
The ten public C functions plus Android JNI initialization, JSON schema 1.0 and
storage formats are unchanged. Older libraries reject the new startup field.
Flutter UI, owner-product matching, physical/off-LAN behavior and refrigerator
writes remain pending. No real account, appliance, separate Flutter worktree,
emulator or physical phone was accessed. Remote CI is inspected after push.

## Read-only control-option inspection

[ADR-0056](adr/0056-read-only-control-option-inspection.md) adds native API 1.16.0
`inspect_control_options` for one exact saved device/connection. Default read-only
sessions can project reviewed AC and dishwasher action definitions from protected
listed membership and model/profile evidence before user permission selection.
The same constructors/selectors supply enabled descriptors and execution limits;
unsupported models/profiles and refrigerators have no write options. Nothing is
published, granted, approved, sent or cached by inspection. Provider authorization
and physical compatibility are not inferred from saved evidence.

Seven synthetic host test groups cover exact profile domains and descriptor
parity, unchanged read-only denial and explicit later grant selection, no account
or Core mutation, protected identity/membership/storage failures, unrelated-account
isolation, exact local IDs, metadata races, cancellation/deadline/background/close/
drop and lease cleanup. Populated AC and empty refrigerator UI examples have
closed response schemas, including negative authority/private-field fixtures.
Actual C calls exercise the new method without credentials; successful protected
inspection uses the synthetic Session/vault/SQLite tests.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
local documentation/contract/repository/commit checks passed. Rust: 437 passed,
two explicitly ignored; runtime: 35; native: 32 methods/52 fictional examples plus
trusted approval/dishwasher checks; Python tooling: 33; presentation: six. The
native suite initially found a stale 1.15.0 assertion in the dishwasher checker;
it now compares the advertised API catalog and the rerun passed. All three Android
AAR variants passed eleven-export and 16 KiB ELF checks. The additive method keeps
JSON schema 1.0, ten public C functions plus Android JNI initialization, SQLite
schema 4 and protected payload/session versions unchanged. Negotiate the new
method before use; existing write gates and outcomes are unchanged. No real
account, appliance, separate Flutter worktree, new emulator or phone was accessed.
Remote CI is verified separately after the phase commit and push.

## Current session-grant inspection

[ADR-0057](adr/0057-session-grant-inspection.md) adds native API 1.17.0
`inspect_session_grants` for one exact local device/connection. It projects
immutable Core policy for the selected transport without adapters, credentials,
account leases or mutation. Complete argument rules remain separate, invalid
schemas are excluded, and configured grants never establish action support,
provider authorization or per-invocation approval. Read-only reopening reports
empty rules immediately even when saved descriptors retain writable actions.
Existing policy, risk, model/profile, membership and execution gates are unchanged.

Three Core test groups cover exact scopes, complete argument combinations,
invalid/boolean schemas, detached response mutation, unchanged capability/risk
refusals, exact IDs and cancellation/deadline. A fictional Meiju Session test
retains actions and a receipt across revocation with secret access disabled and
the account lease held; inspection changes neither metadata nor provider calls.
Actual C calls cover Mock and fictional Meiju grants, read-only reopen, retained
actions, closed authority fields, unknown IDs/connections and lifecycle guards.

On October 16, 2026, formatting, workspace Clippy, workspace/example builds and
local documentation/contract/repository/commit checks passed. Rust: 441 passed,
two explicitly ignored; runtime: 35; native: 33 methods/54 fictional examples plus
trusted approval/dishwasher checks; Python tooling: 33; presentation: six. The
initial C assertion compared a serialized descriptor to its unnormalized seed;
it now compares the actual before/after descriptors and the rerun passed. All
three Android AAR variants passed eleven-export and 16 KiB ELF checks.

The additive method retains JSON schema 1.0, ten public C functions plus Android
JNI initialization, SQLite schema 4 and protected payload/session versions.
Older hosts reject the query; negotiate its advertised version/method. The UI
handoff and English/Chinese catalogs distinguish local grants from capability
metadata and protected control options. No real account, appliance, separate
Flutter worktree, new emulator or physical phone was accessed. Remote CI is
verified separately after the phase commit and push.

## Android native control acceptance

The isolated Android APK now includes an independent `controls` phase for the
production session-grant and trusted approval boundaries. A fresh disposable
AOSP Android 15 x86_64 instance passed 93 assertions without a secure lock and
again while locked. Read-only reopening immediately removes grants while retaining
actions; ordinary elevated execution is denied; confirmation is one-use and
handle-bound; dismissal/background/close invalidate previews. A simulated
post-transmission interruption persists `unknown` across reopening, and receipt
replay does not overwrite a later simulated state change.

On October 16, 2026, the rebuilt test APK passed Java/C compilation, production
library export inspection, signature, exact ABI contents, no-permission and
16 KiB ZIP-alignment checks. The original four Android credential phases passed
17/17/16/27 assertions, including separate-process reads and actual emulator
Keystore/lock behavior. All 33 Python tests and documentation, contract, repository
and commit checks passed. Commit `5b7bddf` documentation and MVP CI runs were independently confirmed
successful.

Only the test harness and handoff documentation changed. Native API 1.17.0,
production Java/Rust code, ten C exports plus JNI initialization, storage and
permission semantics are unchanged. Test-only JNI wrappers do not enter the AAR.
No provider account, appliance or separate Flutter worktree was accessed. The
emulator installation and dedicated ADB server were cleaned up after acceptance.
Actual Flutter consent, physical phones, forced process termination during control
and real/off-LAN appliance behavior remain pending. Remote CI for this test phase
is inspected after commit/push.

## Android forced-stop journal recovery

The isolated APK's `force-stop` phase now demonstrates actual process termination
after a fictional state commit and before acknowledgement. A unique run/PID marker
is accepted only while the matching instrumentation/application process is live.
The host stops only the test package and verifies process disappearance before
starting recovery. Neither a readiness file nor an observer exit code alone is
accepted as termination evidence.

On October 16, 2026, AOSP Android 15 x86_64 passed 27 recovery assertions: the
pre-open journal remained unfinalized and `unknown`; read-only startup finalized
it without changing the receipt or fictional appliance state; released Bionic
locks allowed a subsequent write; replay did not repeat the interrupted command;
history clearing preserved request-ID retirement. Existing controls/unsecured
phases passed 93/17 assertions. An initial test compared JSON text with different
key order; structural equality fixed the assertion while retaining every field.

The rebuilt APK passed Java/C compilation, signature, ABI/no-permission and
16 KiB ZIP-alignment checks. Six new host-side test groups cover stale/malformed
markers, normal completion, process identity/absence and delayed termination;
all 39 Python tests and documentation/contract/repository/commit checks passed.
Both CI workflows for the previous `e2009aa` commit were confirmed successful.
The disposable APK, emulator and dedicated ADB server were cleaned up.

Production Rust/Java code, API 1.17.0, C exports, grants, storage versions and
execution semantics are unchanged. No real account, appliance, Flutter worktree
or physical phone was accessed. Flutter interaction, physical/OEM process behavior
and real/off-LAN controls remain separate acceptance gates. This phase's remote CI
is checked after its commit and push.

## Trusted native selected-device scan bridge

[ADR-0058](adr/0058-native-lan-scan-bridge.md) adds API 1.18.0 and three separate
C prepare/confirm/discard functions around the existing typed Meiju scan. The
controller supplies platform-assigned interface information outside ordinary
JSON and obtains actual user consent to immutable device/network scope. Session
retains at most eight previews for 60 seconds, consumes confirmation before
admission, and clears previews on background/close. Scan and appliance-approval
IDs cannot alias across their separate consent registries.

The returned four-state summary contains only local IDs and explicit false
authentication/route flags. No candidate endpoint, provider identity or retained
candidate handle is exported. Preparation does no discovery; confirmation
reuses bounded collection, protected account leases and identity checks.
The 33 ordinary methods, CLI/MCP, cloud grants/routing, storage versions and
appliance write outcomes are unchanged. Thirteen C functions plus Android
credential JNI initialization are now required by archive export inspection.

On October 16, 2026, six new injected Session tests passed, covering four outcomes,
schema-valid immutable previews/results, cross-handle and consent-kind isolation,
capacity/expiry, duplicate confirmation during a scan, failure consumption,
membership changes, lifecycle/preparation races and cancellation/deadline/drop.
All 14 typed/native scan tests passed. The actual Linux C ABI suite checks
metadata, network syntax, ordinary-dispatch refusal and background/closed
admission without accounts or UDP traffic; successful scans are exercised through
the injected Session path, not a live C-triggered household scan.

Verification passed: 447 Rust tests with two explicitly ignored system/account
probes, formatting, Clippy, workspace/example builds, 35 runtime checks, all 33
native methods/54 fictional examples plus trusted approval/dishwasher/scan checks,
39 Python tests, six presentation tests and document/contract/repository/commit
validation. Android arm64, x86_64 and combined AAR builds passed architecture,
all 14 exports and 16 KiB alignment inspection. No dependencies were added.
Both CI workflows for the previous `6415f79` commit were confirmed successful.

No real account, household network/device, Flutter worktree, emulator or physical
phone was accessed by this phase. Mobile interface/permission selection, Flutter
consent, endpoint/model admission, connection linking and physical/off-LAN
acceptance remain pending. This phase is committed and pushed after local checks;
its remote CI is inspected separately.

## Android assigned-network selection

[ADR-0059](adr/0059-android-lan-network-selection.md) adds the production Java
`NativeLanNetworks` helper to all Android AAR variants. It reads assigned
Wi-Fi/Ethernet IPv4 link properties on explicit foreground calls and returns
instance-owned, immutable, expiring selections. `requireCurrent` rechecks the
exact network/interface/address/prefix/transport tuple before native preparation
and confirmation. Background/close invalidates selections without waiting for
platform calls. Conflicting local-address assignments, unsupported transports
and an active default VPN cannot select a route. Platform failures export only
fixed codes; Selection and helper `toString` output is redacted.

The helper owns no socket, listener, permission prompt or worker. It provides
metadata evidence, not socket authority, peer authentication or scan consent.
The production manifest adds normal `ACCESS_NETWORK_STATE` alongside existing
`INTERNET`. Flutter still owns controller/lifecycle wiring and applicable Android
16/17 runtime permission under the linked official guidance. Native API 1.18,
33 ordinary methods, 13 C exports, Rust/storage/grant/cloud semantics are unchanged.

On October 16, 2026, the Linux JVM passed 70 fictional network-selection assertions
and the existing 54 encrypted-entry assertions; the explicit Rust/JNI credential
test passed. All three AARs compiled production Java and passed ABI, 14-export,
archive and 16 KiB ELF checks. Package tests now reject a missing platform helper
without replacing a previous archive.

The rebuilt isolated APK permits exactly `ACCESS_NETWORK_STATE`, without
`INTERNET`. It passed Java/C compilation, signature, ABI and 16 KiB ZIP checks.
A fresh AOSP Android 15 x86_64 emulator passed 19 actual network-metadata
assertions, followed by the existing 17 unsecured and 93 control assertions.
Initial attempts had no usable selection; emulator diagnostics showed only
loopback and no active network. Enabling virtual Wi-Fi and waiting for its
assigned address satisfied the test precondition without changing production
selection rules or adding app polling. The test package, emulator and dedicated
ADB server were cleaned up. No household discovery or account access occurred.

Regression checks passed: 447 Rust tests with two explicitly ignored probes,
formatting, Clippy, workspace/example builds, 35 runtime checks, 33 native methods
with 54 fictional examples and trusted-function checks, 42 Python tests, six
presentation tests and document/contract/repository/commit validation. Both CI
workflows for the previous `5e0b848` commit were confirmed successful.

Physical phones, Android 16/17 socket permission enforcement, Flutter integration,
authenticated endpoint/model admission, actual LAN routes and real/off-LAN
appliance acceptance remain pending. This phase accessed no real account,
appliance or separate Flutter worktree. Its commit/push and remote CI are checked
after local acceptance; no package is published.

## Meiju login rejection classification

Both login endpoints now classify a bounded JSON object's provider code before
decoding successful data. A regression reproduced `code: 7400` with `data: {}`
being misreported as `MIDEA_RESPONSE_INVALID`; it now returns `AUTH_REQUIRED`.
Other nonzero codes retain `MIDEA_REQUEST_REJECTED` regardless of challenge data.
Successful payloads remain strictly validated, and malformed/oversized envelopes,
non-object roots and ambiguous codes are refused. No provider challenge is exposed.

Synthetic tests cover both endpoints, numeric/string codes, five failure payload
shapes, invalid successful data and raw envelope counterexamples. Native Session
tests preserve new/stored/pending account state, secrets and cards on rejection,
release leases, make no inventory request and allow a subsequent explicit login.
Native API 1.18.0, schemas, exports, storage formats and control authority are
unchanged. No real account, appliance, Flutter worktree or phone was accessed.

On October 16, 2026, verification passed: 451 Rust tests with two explicitly
ignored probes, formatting, Clippy, workspace/example builds, 35 runtime checks,
33 native methods/54 fictional examples and trusted-function checks, 42 Python
tests, six presentation tests and documentation/contract/repository/commit checks.
The first sandboxed suite could not bind loopback sockets; the complete rerun
outside that restriction passed. All three Android AAR variants passed 14-export
and 16 KiB alignment inspection. No emulator/APK acceptance was repeated for this
parser change. Both CI workflows for `b654721` were confirmed successful; this
phase's commit/push and CI are checked separately.

## Historical complete account inventory time

[ADR-0060](adr/0060-account-inventory-timestamp.md) adds nullable
`last_complete_inventory_at` to account summaries in native API 1.19.0. Complete
validated inventory, including an empty list, records the local UTC wall clock
after protected binding persistence. Partial/failed retrieval leaves it unchanged.
It does not establish current authorization, appliance freshness or successful
card publication. Unchanged credentials are not rewritten for this metadata.

SQLite schema 5 upgrades schemas 1/2/3/4 under the existing exclusive migration
lock and verified backup. Older accounts start with null evidence. Close older
clients before upgrade; they refuse schema 5. The 33 ordinary methods, C/JNI
exports, protected payload 3 and control authority remain unchanged. Local
list/details/rename and cache/history clearing preserve the time without secrets;
account removal deletes it. Storage failures preserve prior metadata and do not
undo completed protected persistence or independently mark credentials pending.

On October 16, 2026, verification passed: 458 Rust tests with two explicitly
ignored probes, formatting, Clippy, workspace/example builds, 35 runtime checks,
33 native methods/54 fictional examples and trusted-function checks, 42 Python
tests, six presentation tests and document/contract/repository/commit validation.
All three Android AAR variants passed 14-export and 16 KiB alignment inspection.
The isolated APK passed Java/C compilation, signature, ABI and ZIP checks. A fresh
AOSP Android 15 x86_64 emulator passed 19 network, 28 unsecured and 93 control
assertions. Eleven new unsecured assertions cover schema-4 backup/upgrade,
explicit null evidence, local metadata retention and reopen. The test package,
emulator and dedicated ADB server were cleaned up.

Both CI workflows for `9014fcd` were confirmed successful; this phase's push and
CI are checked separately. No real account, household appliance, physical phone
or separate Flutter worktree was accessed. Flutter rendering, real inventory
timing and physical-device acceptance remain unverified; no package is published.

## Trusted native connection-link bridge

[ADR-0061](adr/0061-native-connection-link-bridge.md) exposes three separate C
functions for the existing typed Core connection-link operation. Native API 1.20
retains immutable previews in a session-owned, eight-entry registry. Confirmation
consumes the ID once, including decoded admission failures; background/close
invalidates pending scope. The returned preview includes exact descriptor,
anchor, proposed connection and existing potential transport grants. No endpoint,
credential, ordinary JSON method or new grant is introduced. Mock and Meiju retain
their default refusal; adapter identity/model/route evidence remains mandatory.

Five new injected Session tests passed with real Core/SQLite and a fictional
opted-in adapter. Together with the eleven existing connection-link tests they
cover schema parity, exact scope/grants, metadata preservation, rediscovery and
reopen, foreign-session/consent-kind isolation, capacity/expiry, consuming failed
admission, metadata/evidence/account changes, concurrent confirmation and
cancellation/background/close/deadline/drop. Actual Linux C exports passed
closed-metadata and ordinary-dispatch refusal plus production Mock/Meiju refusal
without account credentials or networking. Successful linking is Session-level
synthetic evidence, not a production-adapter or physical-device claim.

On October 16, 2026, verification passed: formatting, Clippy, 463 Rust tests with
two explicitly ignored probes, workspace/example builds, 35 runtime checks,
33 native methods/54 fictional examples and all trusted-function checks,
42 Python tests, six presentation tests and document/contract/repository/commit
checks. The arm64, x86_64 and combined AARs passed architecture, 17-export and
16 KiB alignment inspection. No dependencies were added. No APK/emulator test
was repeated for this bridge; physical Android/iOS and Flutter remain unverified.

The 33 ordinary methods, prior C signatures, SQLite schema 5, protected payload 3
and appliance outcomes are unchanged. Negotiate API 1.20 before resolving the new
symbols. Both CI workflows for `fe10afd` were confirmed successful; this phase's
commit/push and exact remote CI are checked separately. No real account,
household appliance, separate Flutter worktree or physical phone was accessed.
No package is published.

## iOS Keychain credential backend

[ADR-0062](adr/0062-ios-keychain-credentials.md) adds an iOS implementation of the
existing native secret-store interface. Every operation includes a trusted
compile-time private access group, random profile service and credential reference.
Missing configuration fails closed. New items use passcode-set/device-only
accessibility with synchronization disabled. A bounded metadata lookup rejects
ambiguous/foreign/weakened entries before exact-reference read/update/delete.
Delete never reads credential data; no broad deletion or add/update retry exists.

Synchronous OS calls retain the caller's operation lifetime and account lease
until return. Pre-dispatch interruption keeps its normal error; unconfirmed
dispatched writes become `SECRET_STORE_WRITE_UNKNOWN`. OS error text and secret
bytes stay out of DTOs/logs. OS-reported lock/authentication failures are refused;
metadata lookup/deletion does not establish device-unlock state. The deliberately
deprecated fail-without-prompt option and its runtime gate are documented.

Local verification on Linux passed thirteen portable iOS tests, including a
barrier-controlled cancelled blocking call with retained caller lifetime, and
both arm64 device/simulator target checks. iOS device-target Clippy also compiled
the platform-specific Core Foundation tests with warnings denied. These tests
execute only on Apple platforms: this local run does not establish their result.
Final workspace verification passed 476 Rust tests with two explicitly ignored
probes, formatting, Clippy and workspace/example builds. The runtime suite passed
35 checks; native checks passed 33 methods/54 fictional examples and all trusted
bridges. Python tooling passed 42 tests, presentation passed six, and document,
contract, repository and existing commit-message validators passed. The new
caller-lifetime test was added after the first full run and the final workspace
test/lint run includes it. Android APK/emulator acceptance was not repeated.

The MVP workflow now contains a macOS job for those Core Foundation object tests
and complete iOS native-library linkage with a fictional group. It makes no
Keychain calls, installs no app and signs/publishes no package. Remote success
must be checked on the actual pushed commit. Signed entitlements, no-passcode and
lock behavior, persistence, framework packaging and Flutter/iPhone acceptance
remain pending; follow the [iOS guide](development/09-ios-native.md).

Native API 1.20, all C signatures, 33 ordinary methods, SQLite schema 5, protected
payload 3 and other platforms' storage behavior remain unchanged. The already
locked `core-foundation-sys` 0.8.7 becomes a direct iOS dependency and a macOS
test dependency; no dependency version changes. No real account, appliance,
separate Flutter worktree or physical phone was accessed.

## iOS development XCFramework packaging

The [iOS builder](development/09-ios-native.md#2-build-and-check-the-native-code)
packages release `cdylib` output into dynamic frameworks, then uses Xcode to create
an XCFramework archive. Device arm64 targets iOS 13.0; the universal arm64/x86_64
Simulator framework targets 14.0, matching the arm64 Simulator Rust minimum.
Only Simulator architectures are combined; device and Simulator remain separate.

Packaging copies the canonical C header/module map, sets the framework-relative
install name, checks Mach-O destination/minimum version, all declared exports and
public system dependencies, and creates local ad-hoc seals with no signing identity
or timestamp service. The final archive is extracted, reinspected and linked from
C module consumers for all three architectures before replacing a prior artifact.
No Flutter views, new API, credential access, native schema or protocol change is
introduced. Ad-hoc seals do not establish identity, provisioning or distribution
signing. There is no package upload or publication.

Eight portable package tests passed with fictional reports/files, bringing Python
tooling coverage to fifty tests. YAML parsing preserves the exact CI commands.
Local Xcode execution is blocked because this host is Linux without Apple SDKs;
the macOS CI job runs actual release packaging and consumer checks. Its result
must be read from the exact pushed commit before claiming success. Signed app
embedding, Flutter/Dart loading, Keychain system behavior and physical iPhone
acceptance remain open. The [previous storage phase](adr/0062-ios-keychain-credentials.md)
passed all sixteen macOS tests and two complete iOS library builds on `3742e67`;
that evidence does not by itself prove this new package format.

## Isolated iOS Simulator acceptance

The [Simulator harness](development/09-ios-native.md#4-run-isolated-simulator-acceptance)
embeds the production XCFramework in a test-only Objective-C app. A separate
Simulator-only Rust helper exercises the actual iOS secret store with fictional
bytes and a fresh UUID namespace; it is absent from the production framework and
public header. Native API 1.20, 16 C functions, 33 ordinary methods, SQLite schema 5
and protected payload 3 are unchanged.

The runner creates a new Simulator and targets only its returned UUID. Two fresh
reports tied to distinct launch PIDs are required. Checks cover Mock discovery,
power/state, persisted cards/cache/receipt, explicit Core lifecycle calls and
Keychain add/read/reopen/replace/isolation/pre-cancellation/delete. Failures stop
without replay, and cleanup never targets an existing user Simulator. Diagnostics
retain bounded test-app launch failures or fixed Boolean metadata flags; they
never request credential data for diagnostics or report raw attributes.

Runtime investigation identified four corrections:

- The test app embeds XML/DER entitlements in the Simulator executable and uses
  a plain ad-hoc host signature. Both sections and signature placement are checked.
  This resolved the launch refusal exposed by `4b1bc11`; `e7da257` launched and
  completed the first Core flow. Physical signing remains a separate mechanism.
- Security returns the synchronization flag as an integer. `3e3f099` accepts only
  Boolean or exact integer 0/1 representations, then still rejects synchronization.
  Missing flags, floats, other integers and wrong types remain refused.
- `2b7dbd3` confirmed an implicit access-control object alongside matching scope,
  passcode-only protection and a persistent reference. `b01f205` creates an explicit
  public zero-flag policy and compares returned objects against its complete value.
  Wrong types, weaker protection, extra user-presence constraints and differently
  represented existing objects remain refused. No automatic migration or recreation
  occurs. [ADR-0062](adr/0062-ios-keychain-credentials.md) records the pinned sources.
- `42ce428` confirmed that the complete policy matched but the exact lookup returned
  `errSecParam`. `d02b10a` separates admitted persistent references from attribute
  filters as required by Security. Metadata admission remains scoped to the exact
  private group; reads revalidate returned namespace/policy/reference, and mutations
  never fall back to a namespace or replay. Real-object regressions check the query
  shape and reject unadmitted references. ADR-0062 records the same-entitlement
  writer boundary of this OS API.

Local verification for the policy and reference-selector corrections passed 21
secret-store tests with two environment-dependent tests ignored, device-target and
Simulator all-targets Clippy, 62 Python tooling tests, formatting and documentation/contract/repository/
commit validation. The Python suite includes twelve Simulator-runner tests.
Linux cannot execute the real Security-object regression or the Simulator app.

On `d02b10a`, macOS job `116902983930` of run `38879635016` passed eighteen
Core Foundation/Security and orchestration tests, all three release framework
architectures, extracted-package inspection, C consumer linkage and the actual
isolated app on arm64 iOS 26.2 / iPhone SE (3rd generation) Simulator:

| Process phase | Core checks | Keychain checks |
|---|---|---|
| Initial write | 40 | 10 |
| Read after process restart | 41 | 4 |

Fresh reports and distinct launch PIDs established persistence across processes,
including replacement and final exact deletion. All three jobs in MVP workflow
`38879635016` and documentation workflow `38879635021` passed for that exact commit.
This is isolated native runtime
acceptance, not physical-phone or provider acceptance.

No real account, appliance, physical phone or separate Flutter worktree was
accessed. Simulator checks do not establish physical passcode/lock behavior,
no-prompt enforcement, signed entitlements, reinstall/backup behavior, Flutter
integration or provider/device/off-LAN acceptance. These remain open.

## Explicit dishwasher cycle options (API 1.21)

[ADR-0063](adr/0063-explicit-dishwasher-cycle-options.md) extends the reviewed
760Y0026 profile to pause/resume/cancel with a required explicit `additional`
choice. Startup grants bind exact arguments; omitted arguments remain exactly
empty and cannot authorize the new path. Its cycle-enabled capability is revision
4; the existing four profiles retain revision 3 and empty payload arguments.
No C signatures, ordinary methods, SQLite or protected-payload versions change.
Older native libraries reject the new grant field. This is synthetic evidence,
not a match to the owner's product or physical/off-LAN support. Product 7600016L
and refrigerator writes remain unadmitted.

The subsequent read-only extension gives that protected product capability
revision 5, including sessions without write grants, and adds optional raw
`vendor.midea.dishwasher.additional` observations. Known labels use existing
locales; unknown codes remain visible without becoming writable. Old descriptors
must be explicitly refreshed; malformed new values preserve the previous cache.
API 1.21.0 and storage/export formats remain unchanged. Other product profiles
neither gain this property nor inherit its code meanings.


## Saved cross-account conflict inspection (API 1.22)

[ADR-0064](adr/0064-saved-device-conflict-inspection.md) adds the 34th ordinary
native method, `inspect_device_conflicts`, with exact local account/device input.
It returns only local binding references and saved membership for possible
cross-account duplicates, including unpublished and historical records. Equal
saved provider IDs do not prove physical identity or compatible product metadata.
No cards, credentials, permissions or routes change. The bounded operation fails
without partial results when relevant protected records cannot be inspected.
C/JNI exports, JSON envelope, SQLite and protected-payload versions are unchanged.
Older libraries reject the method. Explicit reconciliation, account transfer,
Flutter interaction and real-account/physical acceptance remain pending.


## Flutter native API 1.14.1 baseline

Historical merge checkpoint: later Flutter consent and source-only work is recorded
in the [current verification summary](../apps/mobile/verification.md#current-checkpoint).

The frontend worktree merges committed Core `7db8e9b` and keeps the upstream
Android Bionic lock implementation plus its additional contention regression.
Existing Flutter account/cleanup/Mock pages remain compatible with the 31 ordinary
methods. Trusted approval UI and explicit Meiju grant configuration are not yet
wired; elevated controls stay unavailable and Meiju startup remains read-only.
Protected payload version 3 and SQLite schema 4 are retained without downgrade.

Local checks passed: 349 Rust tests (two opt-in tests ignored), formatting, Clippy,
workspace build, 35 runtime cases, native ABI/approval/dishwasher admission, 33
Python tests, six presentation tests, 68 Flutter tests and Flutter analysis.
The dual-ABI Android AAR passed eleven-export and 16 KiB ELF checks; the ordinary
APK passed alignment/install/cold-start and explicit Mock refresh. The complete
Android click journey also passed with simulated picker/input/lifecycle. See the
[Flutter verification report](../apps/mobile/verification.md) for evidence and
limits. Pixel equality remains failed at 0/82; this phase establishes no physical
account/appliance, iOS, remote CI or full mobile acceptance.


## Flutter trusted confirmation bridge

The frontend implements the existing API 1.13+ trusted prepare/execute/discard
functions without changing native methods, authority, schema or receipt semantics.
Deeply immutable, handle-owned previews require one explicit connection and a
separate confirmation click. Dismissal, navigation, background and expiry revoke
consent; a lost reply retains its original request ID for receipt lookup without
replay. High-risk actions remain unavailable and default Meiju remains read-only;
startup grant UI and physical acceptance are separate work.

All 72 Flutter regressions, static analysis and six presentation tests passed.
Four native approval scenarios also passed inside an Android 15 x86_64 app process,
including clicks, real expiry and simulated lost-reply recovery. The ordinary APK
rebuilt, passed alignment, installed and cold-started. The English catalog is
bundled locally and checked against its shared source, correcting an Android
asset-path failure found during integration. Strict prototype equality still
fails at 0/82. See the [verification report](../apps/mobile/verification.md).


## Integrated Flutter host (October 16, 2026)

The former UI branch is merged into main against native API 1.22.0. The app's
existing account, local organization, cleanup, Mock control and trusted-confirmation
flows call Core directly. Explicit saved-conflict, control-option and session-grant
inspection and historical inventory time are now available in the UI without
networking, new grants, route admission or identity merges. See the
[app verification record](../apps/mobile/verification.md) for current macOS,
Android packaging and emulator results and explicit failures/skips. Historical
Linux/Android records above retain their original source and evidence scope.

Meiju Flutter startup remains explicitly selected and read-only; real account,
physical-phone and appliance-control acceptance are separate. Native ABI signatures,
storage formats and reviewed model support are unchanged by the UI merge.


## Flutter session control permissions

The Meiju app starts read-only. Device Information -> Review control options now
renders exact model-scoped power, temperature, vendor fan and explicit power-on/mode
choices from API 1.16 inspection. A separate review and consent click constructs
only those existing startup grants. The trusted controller cancels work, closes
Core and reopens the same private profile. Grants remain in process memory and are
never imported from preferences or ordinary requests. Navigation/backgrounding
invalidates pending consent, including while reopening; failures remain closed
with an explicit Reopen Read-only recovery action.

Consent itself performs no discovery, provider request or appliance write. The
separate Retrieve Control Capabilities button explicitly refreshes the selected
account and descriptor. Core and the adapter still check every command. Revoke
Session Controls closes/reopens without grants; retained descriptors and receipts
are preserved. Process restart also starts read-only. Dishwasher and high-risk
permission setup are not added by this AC phase.

Five focused Flutter tests passed, covering actual native startup with fictional
exact grants, two-step UI consent, model-specific limits, unknown constraints,
separate authority, no implicit networking, revocation and lifecycle races. The
existing account/current-contract/trusted-approval regression selection also
passed (18 tests before the two new tests were added). This establishes host
integration, not real-provider or physical AC success. Native API 1.22, SQLite 5,
protected payload 3 and execution outcomes are unchanged.


## Flutter English and Simplified Chinese

The Flutter presentation layer now uses pinned slang/slang_flutter 4.19.0,
generated typed catalogs and Material localization delegates. Language selection
supports en-US, zh-CN and a nullable system default, persisted in the existing
version-1 presentation preferences. Unsupported system languages fall back to
English. Existing preferences without a locale remain valid.

No native API, SQLite schema, protected payload, action identifier, permission,
receipt or routing format changes. User/provider names and machine identifiers
remain literal. Known errors use localized catalog explanations; unknown errors
retain the code with a generic localized explanation, never raw provider text.
See the [verification record](../apps/mobile/verification.md) for actual checks and
remaining provider/physical acceptance limits.


## Android sign-in readiness

The trusted Android host exposes a local Flutter-only metadata check for native
credential-backend installation and secure/unlocked screen state, plus an explicit
system-security-settings action. It does not add Core methods, change native API
1.22, SQLite 5 or protected payload 3, or relax Android credential policy. Missing
or invalid channel replies and bounded timeouts fail closed. Foreground return
rechecks the flags; pending checks cannot dispatch login after navigation or
backgrounding. Non-Android storage semantics remain in Core.


## Default Flutter device controls

The owner-approved [ADR-0065](adr/0065-default-app-device-controls.md) supersedes
the Flutter session-permission UI described in the earlier historical phase.
Opening a supported device now prepares every existing reviewed low-risk AC
control value automatically, then refreshes that device's account to publish its
actions. The selection/review/revocation and separate capability-refresh controls
are removed. No appliance command is sent during setup; there is no startup or
background account refresh. Exact model/routing checks and existing outcome,
confirmation and replay semantics remain in Core. Native API 1.22, SQLite 5,
protected payload 3 and non-App host defaults are unchanged.


## Owner-observed Flutter power actuation

The default-control APK from `493b8f2` preserved the real account and Chinese
preference across an Android 15 emulator upgrade. The selected wall AC exposed
power, mode and fan controls immediately, without session-permission UI; its
temperature setter stayed unavailable. One owner-authorized power-off request
returned `unknown` after ten seconds, followed by an explicit off reading and
owner confirmation of physical shutdown. A separate restoring power-on request
returned `accepted` with `VERIFICATION_PENDING`; an independent read reported on.
Neither receipt is `verified`, and no uncertain write was replayed.

See the [exact runtime and artifact evidence](../apps/mobile/verification.md#default-control-apk-and-owner-observed-power-acceptance).
Other action/model, physical-phone and off-LAN acceptance remain open. Native API
1.22, SQLite 5, protected payload 3 and execution semantics are unchanged.


## In-place Flutter climate controls

Climate-and-power devices now show separate Mode, Air direction and Fan speed
buttons derived from exact descriptor parameters. Existing reviewed mode and fan
commands are reachable through bottom-sheet choices. Missing, ambiguous, coupled
or non-low-risk setters are not invented or promoted. The selected Meiju wall AC
still has no admitted direction setter; that button is explicitly unavailable.
Mode activation retains its disclosed power-on effect and existing native mapping.

Power and supported climate operations remain on the current control page; a single
foreground read follows a retained command result. Reads do not rewrite accepted,
failed or unknown receipts, trigger a retry, or continue after a lifecycle change.
Native API 1.22, SQLite 5, protected payload 3, grants, protocol support and Core
execution/verification semantics are unchanged. This UI revision adds no new
physical-device or model-support claim.

The owner-requested Meiju-style Flutter AC panel adds direct bounded temperature
interaction and descriptor-driven additional control tiles. It adds no adapter
functions or physical support. The selected AC's temperature admission defect
was resolved by the subsequent correction below; swing and comfort controls
remain tracked in the [AC parity review](development/10-meiju-ac-parity.md).
Native API 1.22.0, SQLite schema 5 and protected payload 3 are unchanged.

The optional-subtype correction in [ADR-0066](adr/0066-optional-wall-ac-subtype.md)
allows existing temperature mappings for the four exact reviewed wall models when
protected product and explicit wall-category evidence remain available. Known
subtype priority and excluded/ambiguous products are unchanged. The selected
wall AC was diagnosed using fixed missing/present flags only: its missing optional
subtype caused the earlier omission. Runtime command/read-back evidence is recorded
separately in the app verification record; model-wide physical support is not implied.

The final APK's selected-AC acceptance sent one decrease from 28 to 27.5 Celsius
and one restoration to 28 Celsius. Both receipts were `accepted` with matching
separate cloud observations, without automatic navigation or write replay.
These observations do not upgrade either receipt to `verified` or prove physical
temperature behavior. The in-place upgrade retained the saved account and Chinese
locale; native API 1.22.0, SQLite schema 5 and protected payload 3 remain unchanged.
