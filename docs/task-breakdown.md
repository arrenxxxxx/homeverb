# Implementation status and next tasks

The Mock MVP is runnable. Status below is evidence-scoped; no row implies physical-device support.

## Paused handoff: October 16, 2026

The owner requested a development pause and a complete repository handoff to
another computer. Resume only after a new owner instruction. The overall usable
MVP remains incomplete; this checkpoint is not a release or physical-support claim.

Current interface: native API **1.22.0**, 34 ordinary methods, 16 C functions
plus the Android-only JNI initializer, SQLite schema 5 and protected payload 3.
Start with [the native contract](native-ui-contract.md), the status table below,
and [the Meiju guide](development/05-midea.md).

Completed foundations include embedded Core, CLI/local stdio MCP, room/card
organization, secure account storage, cloud login/inventory/reads, reviewed
experimental AC and dishwasher controls, mobile native bridges and synthetic
LAN building blocks. One real mainland account previously returned four ACs,
one dishwasher and one refrigerator. All six have experimental cloud reads;
**zero of six have physical control acceptance**. Android emulator and iOS
Simulator evidence does not establish physical-phone or Flutter acceptance.

The final development phase adds saved cross-account conflict inspection to
Session, the native JSON interface and CLI. It reports local candidate IDs only.
It does not select an account, merge devices, transfer grants or change routing.
Local evidence: 485 Rust tests passed, two environment-dependent tests ignored;
35 CLI/MCP runtime tests; 34 native methods and 55 fictional examples; 62 Python
tooling tests; seven presentation tests; build, Clippy, formatting and repository,
documentation and contract validation. See the Meiju guide for the phase record.

Remaining work, in dependency order:

1. Implement an explicit, reviewed account-choice/reconciliation flow using the
   saved conflict candidates. Preserve stable device IDs, rooms and history;
   revalidate protected identity/model evidence and never transfer authority
   implicitly. Design recovery before adding cross-store writes.
2. Resolve exact-model control gaps. The selected refrigerator remains read-only:
   temperature step and encoder preservation are unproven. Reuse the recorded
   source research rather than repeating the same searches. Complete outstanding
   dishwasher mappings only when exact product evidence supports them.
3. Integrate the existing native contract with the separately developed Flutter
   app: protected account input, foreground lifecycle, trusted confirmation,
   outcome display and account recovery. This checkout does not establish that
   separate worktree's implementation status.
4. With separate owner authorization, verify real reauthorization, physical
   phone credential storage, model-specific controls and the away-from-home
   Internet journey. The owner has deferred appliance operations; do not perform
   them or read the local pilot credential directory during resumption by default.
5. Finish protected LAN endpoint/model admission, route publication and full-set
   rediscovery before claiming LAN support. Bluetooth, authorization sharing,
   another protocol family and release/license gates remain separate unfinished
   scope; they are not prerequisites for implementing the cloud UI.

Remote CI last fully passed at `406b212`. Runs for `f6f6e28` failed before executing
steps because GitHub reported failed account payments or an insufficient spending
limit (MVP run `38884135175`, docs run `38884135233`). This is an external admission
failure, not test evidence. Do not change billing or repeatedly rerun unchanged
jobs. Inspect CI for the pushed handoff revision before claiming remote success.

### Continue on another computer

Clone the repository, or pull `main` with `git pull --ff-only` after checking local
changes. Read `AGENTS.md` and the progressive [development index](index.md), then
use [the test guide](testing.md). The commands below are for a Linux checkout;
dependencies need network access on the first machine setup.

```bash
git clone https://github.com/arrenxxxxx/homeverb.git
cd homeverb
python3 -m venv .venv
.venv/bin/python -m pip install -r requirements-dev.txt
cargo build --workspace --locked
cargo test --workspace --locked
.venv/bin/python scripts/verify_mvp.py
.venv/bin/python scripts/verify_native_contract.py
node packages/presentation/test/i18n.test.mjs
```

Install Rust with rustfmt/Clippy, Python 3 and Node.js first; Cargo uses the checked-in
toolchain and lockfile. Add `--offline` only after dependencies are cached.
Mobile SDK setup is documented in the Android and iOS progressive guides.
Build outputs and Python environments are regenerated, not transferred through Git.
Secrets and local account databases are not in the repository: use an explicitly
authorized secure sign-in on the new computer, never a plaintext export. The
separate Flutter worktree is outside this handoff; its uncommitted work is not
included or inspected. UI design sources under `output/imagegen/` and the approved
prototype under `output/prototypes/` are repository artifacts, not working Flutter
screens. No credentials or physical device operations are needed for the fictional
local checks above.

## Task status

| ID | Task | Current status | Remaining acceptance |
|---|---|---|---|
| T00 | Documents and contracts | Implemented | Keep references synchronized |
| T01 | Toolchain and workspace | Implemented: Rust 1.96, Cargo workspace | Additional target builds |
| T02 | Domain DTOs and validation | Shared/component-scoped capability definitions, per-adapter descriptor admission, transactional rediscovery identity guards, and stored-metadata operational validation implemented | Expand tests with new capabilities and actual model constraints |
| T03 | Mock and controlled failures | Implemented | Add protocol-specific faults with adapters |
| T04 | Execution and routing | State-match and bounded correlated-completion verification, explicit independent-field decomposition, explicit device batches with whole-list preflight/shared deadlines, scoped authorization, and full-snapshot observation validation with per-property freshness limits and typed single-command elevated user approval and separate trusted native confirmation functions implemented | Flutter consent UI, reviewed consequential-action mappings, model-specific freshness/correlation evidence and real protocol guarantees |
| T05 | Storage and credentials | SQLite journal/cache, schema-1/2/3/4 backup/upgrade to schema 5, bounded detailed receipts, explicit cache/history clearing, retired-ID protection, and native account vault with persisted references, account leases, explicit Meiju pending-session recovery and same-account reauthorization, recoverable local removal without credential decryption, Linux Secret Service composition, and an Android Keystore/JNI backend with synthetic JVM and isolated AOSP Android 15 runtime verification implemented; iOS Keychain backend has portable/target checks and isolated iOS 26.2 Simulator round-trip/restart evidence | Real provider identity/session acceptance, Android Keystore/phone acceptance, physical iOS Keychain/entitlement policy and provider revocation |
| T06 | CLI | Mock queries/writes and explicit batches; experimental Meiju protected terminal connection, account list/details/rename/removal/refresh/recovery/inventory, local organization, Core reads, exact receipt lookup/history, redacted diagnostics and explicit owner-controlled power, fan, mode activation and profile-scoped temperature grant files implemented | Live CLI login/read/control acceptance and adapter-specific diagnostics evidence |
| T07 | Android embedding probe | Linux native ABI and foreground lifecycle tested; versioned [34-method UI contract 1.22.0](native-ui-contract.md), schemas and native examples implemented; API 28 arm64/x86_64 and combined AAR builds, exports and 16 KiB ELF alignment verified; isolated Android 15 test APK passes native Core/Keystore/lock/restart, grant revocation, trusted approval and uncertain-write/forced-stop recovery checks | Flutter startup wiring/APK integration, actual physical Android Keystore and phone; see the [Android guide](development/08-android-native.md) |
| T08 | Local MCP | Mock/Meiju reads and exact receipt lookup; explicit Meiju startup grants admit command planning/execution with unchanged Core policy; protocol, outcome and shutdown harnesses implemented | Target AI host validation and real Meiju read/control acceptance |
| T09 | First native authentication | Scoped community flow admitted under [ADR-0022](adr/0022-meiju-community-cloud-admission.md); native codec, restricted HTTPS and protected connection tested; one real mainland account login and independent saved-session refresh returned six appliances | Native account connection/list/details/rename/removal/refresh/recovery, protected saved-inventory inspection and read-only control-option inspection before grants and local session-grant inspection are callable; connection uses separate secret buffers and can reauthorize an explicitly selected pending account without resetting intact bindings. Verify real reauthorization identity, Flutter input/mobile storage and off-LAN behavior |
| T10 | First real adapter | Default read-only Meiju adapter admits six exact models and checks protected complete-inventory membership before live access and retains private product mapping evidence for reviewed profile-specific temperature contracts; real Core/native reads passed for six cards/seven components; protected product 760Y0026 additionally has synthetic-tested optional read-only additional-option observations. Explicit experimental AC power, vendor fan, power-on/mode and profile-scoped temperature controls, scoped power probe, independent native AC grants, and profile-scoped dishwasher starts and independently granted pause/resume/cancel operations (including explicit additional options for product 760Y0026) through trusted confirmation are synthetic-tested with conservative cloud outcomes; bounded LAN discovery/collection, V3/ordinary packet codecs, standard AC/E1/CA read candidates, one-operation TCP exchanges, account-bound v2 credential lookup, protected candidate reconciliation and composed family reads with message-version negotiation have fictional evidence only | Flutter account input/mobile integration, reauthorization identity, explicit cross-account reconciliation (saved candidate inspection is implemented), physical AC power acceptance deferred by the owner; Flutter consent UI, physical AC temperature/preservation and dishwasher start acceptance, remaining unreviewed dishwasher mappings and refrigerator controls, then LAN and model-specific physical/off-LAN acceptance |
| T11 | Flutter room/card UI, platform management, and dynamic device controls | UI bridge, Core room/card metadata, and locale resources implemented; complete Window Light visual specification available | Implement the [functional acceptance checklist](ui-functional-spec.md), [82-state inventory](ui-screen-inventory.md), and [one-skin extension contract](ui-window-light.md); actual Flutter/phone integration remains unverified. No HomeVerb login; rooms preserve device identity |
| T12 | Second protocol family | Planned after T10 | Equivalent capability calls on heterogeneous hardware |
| T13 | Security review | Mock component, argument, route, composite-policy, uncertainty, and diagnostic allowlist/marker tests implemented | Real credential, dependency, live-adapter log and diagnostic review |
| T14 | Release | Planned | License decision, packages, actual remote CI and device evidence |
| T15 | Foreground local-network authorization sharing in Settings | Planned | ADR-0008 contract/security review; peer verification and bilateral consent; secure import; provider eligibility evidence; honest scope and revocation; lifecycle cleanup and interruption tests |

T15 does not establish real-platform sharing support. Its security and compatibility evidence is required before packaging any enabled sharing path.

## Next development order

Use the [progressive guides](index.md). UI development can proceed against the native room/card request contract and locale catalogs. Local organization, schema migration, detailed-receipt retention, and the native account vault with Linux secret-store composition are implemented. The owner selected Midea/Hualing air conditioners, dishwashers, and refrigerators. Follow the [pinned community-source integration plan](development/05-midea.md): secure account connection and automatic cloud device retrieval, cloud reads through Core, authorized cloud writes with physical evidence, then additional LAN connections. The complete mobile journey must work away from home without a gateway or manual IP/Token entry. A read-only stage or LAN-only control does not complete T10.

Research and offline contract/protocol implementation can proceed before exact model details arrive; account/region evidence gates real authentication and model/firmware evidence gates physical compatibility claims. The inspected refrigerator LAN setter is empty, while cloud mappings include compartment temperature settings; assess those routes separately. Android storage has synthetic/emulator evidence; physical-phone acceptance remains pending. The iOS Keychain backend and production framework have isolated Simulator runtime evidence; signed physical-app and phone acceptance remain pending; see the [iOS guide](development/09-ios-native.md). Preserve specific blockers for live access and mobile tooling. Authorization sharing remains gated by ADR-0008.

The [LAN integration sequence and remaining gates](development/05-midea.md#implemented-native-lan-building-blocks)
separate implemented helpers from unfinished endpoint/model admission, trusted
network selection, Meiju connection-link integration and physical acceptance.
The [typed Core/Session linking foundation](adr/0052-trusted-connection-linking.md)
is synthetic-tested, with transactional storage and unchanged rediscovery guards.
Meiju has not opted in: protected route evidence, full-set rediscovery and a
trusted mobile consent flow are still required before publishing a LAN route.
API 1.20 exposes three separate trusted C linking functions under
[ADR-0061](adr/0061-native-connection-link-bridge.md), backed by the existing Core
checks and a bounded session preview registry. This does not satisfy Meiju's
protected endpoint/model evidence or implement the Flutter confirmation screen.
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

The [iOS package builder](development/09-ios-native.md#2-build-and-check-the-native-code)
passed actual macOS CI on `d4525b1`, including separate device/Simulator slices,
canonical C module metadata, local ad-hoc seals and consumer-link checks. The
[isolated Simulator runner](development/09-ios-native.md#4-run-isolated-simulator-acceptance)
passed Mock/Core and fictional Keychain round trips across separate processes on
`d02b10a` in macOS job `116902983930`. Physical app signing,
Flutter embedding/runtime and Keychain/phone policy acceptance remain open.
