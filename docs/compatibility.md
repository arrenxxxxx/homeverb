# Platform and device compatibility matrix

> The fictional `homeverb.mock` adapter is implemented and Mock-tested on Linux. The six listed Meiju models have experimental cloud reads; real-platform rows do not claim verified physical support. No physical-device verification records exist.

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
