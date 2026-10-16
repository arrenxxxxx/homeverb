# Implementation status and next tasks

The Mock MVP is runnable. Status below is evidence-scoped; no row implies physical-device support.

| ID | Task | Current status | Remaining acceptance |
|---|---|---|---|
| T00 | Documents and contracts | Implemented | Keep references synchronized |
| T01 | Toolchain and workspace | Implemented: Rust 1.96, Cargo workspace | Additional target builds |
| T02 | Domain DTOs and validation | Shared/component-scoped capability definitions, per-adapter descriptor admission, transactional rediscovery identity guards, and stored-metadata operational validation implemented | Expand tests with new capabilities and actual model constraints |
| T03 | Mock and controlled failures | Implemented | Add protocol-specific faults with adapters |
| T04 | Execution and routing | State-match and bounded correlated-completion verification, explicit independent-field decomposition, explicit device batches with whole-list preflight/shared deadlines, scoped authorization, and full-snapshot observation validation with per-property freshness limits and typed single-command elevated user approval and separate trusted native confirmation functions implemented | Flutter consent UI, reviewed consequential-action mappings, model-specific freshness/correlation evidence and real protocol guarantees |
| T05 | Storage and credentials | SQLite journal/cache, schema-1/2/3 backup/upgrade to schema 4, bounded detailed receipts, explicit cache/history clearing, retired-ID protection, and native account vault with persisted references, account leases, explicit Meiju pending-session recovery and same-account reauthorization, recoverable local removal without credential decryption, Linux Secret Service composition, and an Android Keystore/JNI backend with synthetic JVM and isolated AOSP Android 15 runtime verification implemented | Real provider identity/session acceptance, Android Keystore/phone acceptance, iOS storage and provider revocation |
| T06 | CLI | Mock queries/writes and explicit batches; experimental Meiju protected terminal connection, account list/details/rename/removal/refresh/recovery/inventory, local organization, Core reads, exact receipt lookup/history, redacted diagnostics and explicit owner-controlled power, fan, mode activation and profile-scoped temperature grant files implemented | Live CLI login/read/control acceptance and adapter-specific diagnostics evidence |
| T07 | Android embedding probe | Linux native ABI and foreground lifecycle tested; versioned [33-method UI contract 1.18.0](native-ui-contract.md), schemas and native examples implemented; API 28 arm64/x86_64 and combined AAR builds, exports and 16 KiB ELF alignment verified; isolated Android 15 test APK passes native Core/Keystore/lock/restart, grant revocation, trusted approval and uncertain-write/forced-stop recovery checks | Flutter startup wiring/APK integration, actual physical Android Keystore and phone; see the [Android guide](development/08-android-native.md) |
| T08 | Local MCP | Mock/Meiju reads and exact receipt lookup; explicit Meiju startup grants admit command planning/execution with unchanged Core policy; protocol, outcome and shutdown harnesses implemented | Target AI host validation and real Meiju read/control acceptance |
| T09 | First native authentication | Scoped community flow admitted under [ADR-0022](adr/0022-meiju-community-cloud-admission.md); native codec, restricted HTTPS and protected connection tested; one real mainland account login and independent saved-session refresh returned six appliances | Native account connection/list/details/rename/removal/refresh/recovery, protected saved-inventory inspection and read-only control-option inspection before grants and local session-grant inspection are callable; connection uses separate secret buffers and can reauthorize an explicitly selected pending account without resetting intact bindings. Verify real reauthorization identity, Flutter input/mobile storage and off-LAN behavior |
| T10 | First real adapter | Default read-only Meiju adapter admits six exact models and checks protected complete-inventory membership before live access and retains private product mapping evidence for reviewed profile-specific temperature contracts; real Core/native reads passed for six cards/seven components. Explicit experimental AC power, vendor fan, power-on/mode and profile-scoped temperature controls, scoped power probe, independent native AC grants, and profile-scoped dishwasher starts and independently granted pause/resume/cancel operations through trusted confirmation are synthetic-tested with conservative cloud outcomes; bounded LAN discovery/collection, V3/ordinary packet codecs, standard AC/E1/CA read candidates, one-operation TCP exchanges, account-bound v2 credential lookup, protected candidate reconciliation and composed family reads with message-version negotiation have fictional evidence only | Flutter account input/mobile integration, reauthorization identity, explicit cross-account reconciliation, physical AC power acceptance deferred by the owner; Flutter consent UI, physical AC temperature/preservation and dishwasher start acceptance, remaining dishwasher mappings and refrigerator controls, then LAN and model-specific physical/off-LAN acceptance |
| T11 | Flutter room/card UI, platform management, and dynamic device controls | UI bridge, Core room/card metadata, and locale resources implemented; complete Window Light visual specification available | Implement the [functional acceptance checklist](ui-functional-spec.md), [82-state inventory](ui-screen-inventory.md), and [one-skin extension contract](ui-window-light.md); actual Flutter/phone integration remains unverified. No HomeVerb login; rooms preserve device identity |
| T12 | Second protocol family | Planned after T10 | Equivalent capability calls on heterogeneous hardware |
| T13 | Security review | Mock component, argument, route, composite-policy, uncertainty, and diagnostic allowlist/marker tests implemented | Real credential, dependency, live-adapter log and diagnostic review |
| T14 | Release | Planned | License decision, packages, actual remote CI and device evidence |
| T15 | Foreground local-network authorization sharing in Settings | Planned | ADR-0008 contract/security review; peer verification and bilateral consent; secure import; provider eligibility evidence; honest scope and revocation; lifecycle cleanup and interruption tests |

T15 does not establish real-platform sharing support. Its security and compatibility evidence is required before packaging any enabled sharing path.

## Next development order

Use the [progressive guides](index.md). UI development can proceed against the native room/card request contract and locale catalogs. Local organization, schema migration, detailed-receipt retention, and the native account vault with Linux secret-store composition are implemented. The owner selected Midea/Hualing air conditioners, dishwashers, and refrigerators. Follow the [pinned community-source integration plan](development/05-midea.md): secure account connection and automatic cloud device retrieval, cloud reads through Core, authorized cloud writes with physical evidence, then additional LAN connections. The complete mobile journey must work away from home without a gateway or manual IP/Token entry. A read-only stage or LAN-only control does not complete T10.

Research and offline contract/protocol implementation can proceed before exact model details arrive; account/region evidence gates real authentication and model/firmware evidence gates physical compatibility claims. The inspected refrigerator LAN setter is empty, while cloud mappings include compartment temperature settings; assess those routes separately. Android storage has synthetic/emulator evidence; physical-phone acceptance and iOS storage remain pending. Preserve specific blockers for live access and mobile tooling. Authorization sharing remains gated by ADR-0008.

The [LAN integration sequence and remaining gates](development/05-midea.md#implemented-native-lan-building-blocks)
separate implemented helpers from unfinished endpoint/model admission, trusted
network selection, Meiju connection-link integration and physical acceptance.
The [typed Core/Session linking foundation](adr/0052-trusted-connection-linking.md)
is synthetic-tested, with transactional storage and unchanged rediscovery guards.
Meiju has not opted in: protected route evidence, full-set rediscovery and a
trusted mobile consent flow are still required before publishing a LAN route.
The source's default AC decoder and the six cloud models do not establish a LAN
model allowlist. These gaps do not require reimplementing the completed codecs.

The [trusted foreground scan](adr/0054-trusted-meiju-lan-candidate-scan.md) now
composes selected-device discovery/reconciliation through typed Rust Session
preview and consuming confirmation. Synthetic tests cover lifecycle, identity,
lease and deadline boundaries. It does not fetch keys, read state or publish a
route. API 1.18 adds the [three trusted C scan functions](adr/0058-native-lan-scan-bridge.md)
with bounded, one-use previews and endpoint-free results. Android's
`NativeLanNetworks` now supplies foreground assigned-interface selections and
exact revalidation under [ADR-0059](adr/0059-android-lan-network-selection.md).
Flutter controller/permission integration, consent UI and the remaining
model/endpoint/linking gates are unfinished.

The selected refrigerator's [write evidence review](development/05-midea.md#refrigerator-write-evidence)
establishes model-specific manual bounds but leaves encoder preservation and
temperature step unproven. Keep its existing reads available while researching
that gate; Flutter can implement the read-only compartment view now.

Each completion record names the task, changed interfaces, commands actually run, results, skipped checks, compatibility effects, and external actions. All such records are English.

The owner requires one commit and remote push per completed, verified phase. The existing Mock MVP is committed as the reusable baseline. Independent Flutter work starts from the [native UI contract](native-ui-contract.md); frontend development must not wait for real-device acceptance to implement the available room/card/control flows. Subsequent account/cloud work adds reviewed contracts and implementation in separate verified phases.
