# Platform and device compatibility matrix

> The fictional `homeverb.mock` adapter is implemented and Mock-tested on Linux. Every real-platform row below remains a candidate, not an implemented support claim. No physical-device verification records exist.

| Candidate | Potential route | Evidence still required | Project status |
|---|---|---|---|
| Midea air conditioners, dishwashers, and refrigerators | Required Meiju cloud enumeration/read/control; additional AC/E1/CA LAN routes | Native account authentication, Rust implementation, exact model/app/region/version, physical and off-LAN tests | protocol-researched / models unverified |
| Hualing devices in the selected appliance scope | Meiju cloud and LAN paths only when actual account/protocol evidence matches | Confirm account visibility, family/version and supported cloud/LAN capabilities per device; do not infer from brand | protocol-researched / models unverified |
| Samsung televisions | Candidate LAN protocol | Pairing, firmware, wake behavior, capabilities, authentication | planned / unverified |
| SmartThings | Vendor cloud API | Backend-free native authorization, region, API permissions and cost | planned / auth gate pending |
| Hisense devices | Candidate account platform/API | Regional platform, app, account, protocol, native authorization | planned / unverified |

Brand names express user interest, not protocol boundaries. `midea-local` and SmartThings CLI provide protocol/API references, but do not prove native authentication, mobile embedding, or model support in this project. [R5 and R6](references.md)

The owner selected Midea/Hualing air conditioners, dishwashers, and refrigerators on October 15, 2026, with automatic retrieval of existing Meiju-bound devices after authorization and daily cloud control at home or away. The [integration guide](development/05-midea.md) pins separate cloud and LAN sources. Source research is sufficient to start offline work; it does not establish HomeVerb support. Refrigerator writes are absent from the inspected LAN implementation, while cloud mappings declare compartment settings. Neither a mapping nor a dishwasher message class proves a verified model-specific action. No new real adapter is registered yet.

`homeverb-midea` now implements the Meiju home/appliance inventory protocol behind an injected read transport, verified with synthetic responses. This is protocol implementation evidence only: live authentication, HTTPS, capability admission, actual provider identity evidence and physical tests remain pending. Existing native API version 1.0.0 and Mock behavior are unchanged. See [ADR-0020](adr/0020-midea-cloud-inventory.md).

The library also decodes selected AC/E1/CA cloud state fields under [ADR-0023](adr/0023-meiju-cloud-state-evidence.md). Returned cloud observation time is unproven; projected readings are stale and cannot verify writes. Vendor-specific mode, fan and dishwasher codes retain their meaning without invented common controls. The unreleased Rust endpoint enum gains an appliance-state variant; native API 1.0.0, SQLite schema 3 and registered adapters are unchanged. Synthetic decoding does not establish support for a physical model.

The Mock climate capability is revision 2 and explicitly permits independent-field decomposition. Rediscovery updates existing profiles. Older descriptors without the declaration keep complete argument objects together; request-ID replay remains unchanged. See [ADR-0012](adr/0012-explicit-action-decomposition.md). No real-platform compatibility follows from this change.

Rediscovery preserves established connection bindings and refuses identity changes with `DEVICE_IDENTITY_CONFLICT`. Stable Mock identities require no migration. Adding/removing routes and changing accounts need a future explicit confirmation flow; initial real-device identity and deduplication still require adapter evidence. See [ADR-0013](adr/0013-preserve-device-connection-identity.md).

Core now accepts bounded adapter-correlated completion evidence under [ADR-0016](adr/0016-correlated-completion.md). The `correlated-completion` Mock scenario changes declared verification methods on discovery and returns fictional evidence. Normal Mock behavior, JSON result fields, and SQLite schema remain compatible. The unreleased Rust `AdapterOutcome` adds `completion`; existing adapters set it to `None`. No real-platform correlation or device support follows from this change.

Explicit device batches are additive Core/CLI/native functionality under [ADR-0017](adr/0017-bounded-device-batches.md). Existing single-command payloads, SQLite records, and C ABI functions are unchanged. Older hosts reject `plan_batch` and `execute_batch`. The built-in Mock host still exposes one fictional appliance; multi-device concurrency is exercised by a dedicated integration adapter, not a real-platform support claim.

Component-scoped capability definitions are supported under [ADR-0018](adr/0018-component-capability-scopes.md). Existing shared definitions retain their JSON representation; old strict readers reject the new scope field. Update consumers before loading scoped descriptors, and never discard scope to claim compatibility. SQLite and C ABI functions are unchanged. The fridge/freezer fixtures demonstrate Core semantics only; no real refrigerator model is verified.

The [native UI contract](native-ui-contract.md) adds `get_api_info` and publishes API version 1.0.0 under [ADR-0019](adr/0019-native-ui-contract.md). Existing valid JSON requests, C symbols and stored data are unchanged. No-argument methods now reject previously ignored extra fields, enforcing the existing closed-request policy. Older hosts reject the new query; new UIs must require an upgraded library before assuming method availability. The callable catalog describes interfaces, not real-platform support or write permission.

The native Meiju account layer under [ADR-0024](adr/0024-meiju-private-device-bindings.md) adds protected subject/session/device-map persistence through the existing vault. Synthetic evidence covers stable local bindings across reopening, session replacement, partial inventories and descriptor rediscovery. It neither registers a real adapter nor establishes provider identity, cross-account deduplication or physical support. SQLite schema 3, native API 1.0.0 and its 22 methods remain unchanged.

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

The October 15 Meiju review found a developer-secret requirement in the official v3 API and shared app material in the pinned community login path. Neither is currently admitted as HomeVerb live authentication. Proposed [ADR-0022](adr/0022-meiju-community-cloud-admission.md) requires an owner decision before adopting the community material; no official approval or physical support follows from that proposal. Device models are not the missing evidence for this gate.

Do not skip authentication feasibility merely because another feature works. Label cloud-only operation `cloud-required`. Separate cloud-dependent onboarding from daily LAN operation. Platform support does not imply every product sold under that brand is supported.
