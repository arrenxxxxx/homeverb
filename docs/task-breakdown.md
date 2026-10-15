# Implementation status and next tasks

The Mock MVP is runnable. Status below is evidence-scoped; no row implies physical-device support.

| ID | Task | Current status | Remaining acceptance |
|---|---|---|---|
| T00 | Documents and contracts | Implemented | Keep references synchronized |
| T01 | Toolchain and workspace | Implemented: Rust 1.96, Cargo workspace | Additional target builds |
| T02 | Domain DTOs and validation | Shared/component-scoped capability definitions, per-adapter descriptor admission, transactional rediscovery identity guards, and stored-metadata operational validation implemented | Expand tests with new capabilities and actual model constraints |
| T03 | Mock and controlled failures | Implemented | Add protocol-specific faults with adapters |
| T04 | Execution and routing | State-match and bounded correlated-completion verification, explicit independent-field decomposition, explicit device batches with whole-list preflight/shared deadlines, scoped authorization, and full-snapshot observation validation with per-property freshness limits implemented | Model-specific freshness/correlation evidence and real protocol guarantees |
| T05 | Storage and credentials | SQLite journal/cache, schema-1/2 backup/upgrade to schema 3, bounded detailed receipts, retired-ID protection, and native account vault with persisted references, account leases, and Linux Secret Service composition implemented | Provider identity/session flows, account removal and mobile secure stores |
| T06 | CLI | Mock queries/writes and explicit batch files, local organization, exact receipt lookup, device-filtered history, and bounded redacted local diagnostic reports implemented | Real authentication and adapter-specific diagnostics evidence |
| T07 | Android embedding probe | Linux native ABI and foreground lifecycle tested; versioned [22-method UI contract](native-ui-contract.md), schemas and native examples implemented | Flutter SDK, Dart bindings, Android toolchain, physical phone |
| T08 | Local MCP | Read-only stdio with exact receipt lookup implemented and protocol harness tested | Target AI host validation and reviewed write grants |
| T09 | First native authentication | Official developer-secret flow and community shared-material flow reviewed; live admission unresolved under current rules | Owner decision on proposed [ADR-0022](adr/0022-meiju-community-cloud-admission.md), or evidence of a permitted independent native flow; then connect provider sessions to the vault and automatic retrieval, and verify account/region and identity preservation |
| T10 | First real adapter | Native Meiju inventory, protected account/device bindings and bound AC/E1/CA state reads tested through Core with fictional descriptors and synthetic responses; no live transport or adapter registration yet | Reviewed authentication/identity/signing/HTTPS, cross-account reconciliation and actual model capability admission, authorized controls, then LAN and model-specific physical acceptance |
| T11 | Flutter room/card UI, platform management, and dynamic device controls | UI bridge, Core room/card metadata, and locale resources implemented; complete Window Light visual specification available | Implement the [functional acceptance checklist](ui-functional-spec.md), [82-state inventory](ui-screen-inventory.md), and [one-skin extension contract](ui-window-light.md); actual Flutter/phone integration remains unverified. No HomeVerb login; rooms preserve device identity |
| T12 | Second protocol family | Planned after T10 | Equivalent capability calls on heterogeneous hardware |
| T13 | Security review | Mock component, argument, route, composite-policy, uncertainty, and diagnostic allowlist/marker tests implemented | Real credential, dependency, live-adapter log and diagnostic review |
| T14 | Release | Planned | License decision, packages, actual remote CI and device evidence |
| T15 | Foreground local-network authorization sharing in Settings | Planned | ADR-0008 contract/security review; peer verification and bilateral consent; secure import; provider eligibility evidence; honest scope and revocation; lifecycle cleanup and interruption tests |

T15 does not establish real-platform sharing support. Its security and compatibility evidence is required before packaging any enabled sharing path.

## Next development order

Use the [progressive guides](index.md). UI development can proceed against the native room/card request contract and locale catalogs. Local organization, schema migration, detailed-receipt retention, and the native account vault with Linux secret-store composition are implemented. The owner selected Midea/Hualing air conditioners, dishwashers, and refrigerators. Follow the [pinned community-source integration plan](development/05-midea.md): secure account connection and automatic cloud device retrieval, cloud reads through Core, authorized cloud writes with physical evidence, then additional LAN connections. The complete mobile journey must work away from home without a gateway or manual IP/Token entry. A read-only stage or LAN-only control does not complete T10.

Research and offline contract/protocol implementation can proceed before exact model details arrive; account/region evidence gates real authentication and model/firmware evidence gates physical compatibility claims. The inspected refrigerator LAN setter is empty, while cloud mappings include compartment temperature settings; assess those routes separately. Mobile needs its own secure-store implementation. Preserve specific blockers for live access and mobile tooling. Authorization sharing remains gated by ADR-0008.

Each completion record names the task, changed interfaces, commands actually run, results, skipped checks, compatibility effects, and external actions. All such records are English.

The owner requires one local commit per completed, verified phase. The existing Mock MVP is committed as the reusable baseline. Independent Flutter work starts from the [native UI contract](native-ui-contract.md); frontend development must not wait for real-device acceptance to implement the available room/card/control flows. Subsequent account/cloud work adds reviewed contracts and implementation in separate verified phases.
