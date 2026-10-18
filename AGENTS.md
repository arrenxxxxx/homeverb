# AGENTS.md - Instructions for AI contributors

Scope: every file in this repository. Project: **HomeVerb**. Repository and CLI: `homeverb`. Implementation baseline: 0.2.0 Mock MVP, October 14, 2026.

## Read first

Read `README.md`, `docs/product-requirements.md`, `docs/architecture.md`, `docs/process-lifecycle.md`, `docs/domain-model.md`, `docs/capabilities.md`, `docs/adapter-contract.md`, and `docs/routing-and-execution.md`, in that order. Before starting a task, inspect `docs/task-breakdown.md` and `docs/compatibility.md`. Never present a planned feature as implemented.

## Non-negotiable product constraints

1. Do not require HA, a NAS, a project backend, a daemon, a startup service, background keep-alive, or periodic polling outside the owner-approved foreground App state-read exception in ADR-0067.
2. Core is an embeddable library. CLI, App, and MCP call Core directly. Do not make App or MCP shell out to the CLI and parse its output as the production architecture.
3. CLI commands release resources and exit. MCP supports only a host-managed local stdio process, ending with its host session. An idle session must not scan appliances or refresh every account.
4. The app performs foreground work only. ADR-0067 permits silent three-second state reads on visible device views; other work remains on demand. Leaving the foreground cancels networking and closes subscriptions. Unconfirmed writes are recorded as `unknown`. Do not promise a callback after a forced termination.
5. Do not implement remote HTTP MCP, public control APIs, device relays, cloud sync, or background automation. Changing this boundary requires an explicit owner decision and a new ADR. Do not reintroduce these features under an "optional" label.
6. Local SQLite and system secret stores do not require a self-hosted service. Temporary loopback authentication callbacks and short-lived protocol UDP sockets are allowed only within a bounded operation.
7. A platform requiring a confidential shared client secret, private server callback, or persistent relay is incompatible unless it has a viable native-client alternative. Never embed a backend secret in an app to claim support. The owner-approved ADR-0022 exception permits only the pinned public Meiju community app-compatibility material; it does not permit confidential developer secrets or real-device writes. The owner's project-wide open-source preparation includes this existing integration; do not require a duplicate Meiju-specific approval. Actual remote publication follows the delivery workflow.

## Layering rules

Upper layers use Device, Component, Capability, Property, Action, and Event. They must not branch on brand to choose business semantics. Platform/protocol adapters own encoding, pairing, login flows, capability mapping, and protocol evidence, not UI.

Unify only equivalent semantics. Use `vendor.<provider>.*` for vendor-specific features. Never hardcode universal temperature ranges, fan-speed enums, or brand-wide support. A device's LAN and cloud routes are connections to one physical device, not duplicate devices. Ambiguous identity matches require user confirmation.

## Write safety and outcomes

Validate identity, permissions, arguments, routing, and risk before sending anything. A network timeout does not prove that an operation was not executed. Read retries are bounded. Writes distinguish proven-not-sent, possibly-sent, accepted, and explicitly rejected phases. Never replay a non-idempotent uncertain write on another route.

Use `accepted`, `verified`, `failed`, and `unknown`; use per-item results and `partial` for batches. `verified` requires timestamped evidence and does not establish physical causality by itself. HTTP 200, stale cache entries, and optimistic state are not verification.

When changing execution semantics, update `docs/routing-and-execution.md`, `docs/errors.md`, the corresponding schemas, and positive/negative fixtures. Cancelling an already-sent request is not a physical rollback. Typed Rust elevated
approval follows ADR-0042: retain the immutable preview, require actual trusted UI
consent before consuming it, and invalidate/cancel on lifecycle changes. It is not
available in ordinary JSON, CLI or MCP. ADR-0043 exposes three separate trusted C
functions with a bounded session-owned preview registry; high-risk actions remain denied.
Consequential adapters must check Core's approval marker independently of public
risk metadata.

## Security

Never put passwords, tokens, pairing keys, or live device identifiers into the repository, ordinary configuration, plaintext SQLite fields, command-line arguments, stdout, or logs. Use credential references and system secret stores. Fail closed when secure storage is unavailable, or use a separately reviewed interactive encrypted store; never silently fall back to plaintext.

MCP principals come from trusted host configuration. Tool parameters must not select a more privileged identity. High-risk operations require trusted user-side approval; `confirmed=true` is not approval evidence. In-process plugins are not sandboxed. Version one uses reviewed, compile-time adapter registration rather than downloading executable plugins.

## Language and commit policy

Use the owner-approved name **HomeVerb** in prose and app branding, `homeverb` for the repository and executable, and `homeverb-` for proposed module names. Follow `docs/adr/0006-project-name.md`. Do not mark the name as provisional.

Write all documentation, code comments, diagram labels, example messages, issue/PR text, commit subjects, and commit bodies in English. Keep public identifiers stable and English. Do not add translated documentation. The owner-authorized exception is the `packages/presentation/locales/zh-CN.json` UI translation catalog; all public identifiers and other source text remain English.

Use `<type>(<optional-scope>): <English imperative summary>`. Allowed types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `ci`, `build`, `chore`, `style`, and `revert`. Use `feat`, not `feature` or `fateture`. Documentation-only work is `docs`, not a claim that a product feature now works. See `docs/commit-conventions.md`.

## Implementation status and test honesty

Experimental Mijia Rust integration is implemented under
[ADR-0072](docs/adr/0072-mijia-protected-core-reads.md): mainland native login,
version-1 protected account/device/spec bindings, partial inventory cards and
reviewed MIoT fan/doorbell/lock/battery reads through Core. Read
[the Mijia guide](docs/development/11-mijia.md) before changing it. Native API 1.24 adds Mijia-only/combined account routing and partial positive
card results under [ADR-0073](docs/adr/0073-mijia-native-account-routing.md).
Flutter now exposes registered Mijia/Meiju platform choices, protected sign-in,
partial named cards and capability-driven read-only detail views. Fan controls and
owner-run real-account/device acceptance remain pending. No lock write or complete-inventory claim is admitted. Existing Meiju
payload 3, Mijia payload 1 and SQLite 5 remain unchanged.

The Rust Mock MVP, Linux Secret Service library, and native account vault are implemented. Read the progressive guides in `docs/development/` before changing them. Native Meiju login, protected connection and automatic inventory are synthetic-tested with a local read-only probe. One real mainland account has passed login/inventory and saved-session refresh; six exact models have experimental read-only Core/native mappings. Native API 1.22.0 provides local account details/rename/removal and explicit cache/history clearing, selects Mock by default or explicit Meiju reads, lists/refreshes/recovers accounts and accepts protected connection buffers outside general JSON, including explicit pending-account reauthorization with intact protected identity. Read-only control-option inspection exposes reviewed model-specific choices before grants without networking, descriptor mutation or authority changes. Local session-grant inspection separates current authority from saved capabilities without credentials or networking. API 1.22 adds saved cross-account device-conflict inspection with local references only; it does not merge devices or transfer authority. API 1.19 account summaries expose nullable historical complete-inventory time from SQLite schema 5; this is never authorization or online evidence, and does not rewrite unchanged credentials. API 1.18 additionally exposes three separate trusted LAN-scan preview/confirm/discard functions; platform-assigned interface/permission and actual UI consent remain required, and scan results never expose endpoints or add routes. Trusted startup can independently grant exact-device AC power states, reviewed vendor fan codes, explicit power-on/mode activation or protected-profile temperature targets over cloud; ordinary requests cannot edit authority. Exact dishwasher program/argument startup grants additionally require a reviewed protected product mapping and separate trusted user confirmation; ordinary plan/execute remain denied for this elevated non-idempotent action. Independent profile-scoped pause/resume/cancel grants use the same trusted confirmation boundary; product 760Y0026 requires an explicit additional option in both the exact grant and command, while other unreviewed centralized profiles remain unavailable. An explicitly opted-in AC power probe is synthetic-tested; default native Meiju sessions remain read-only and real writes require separate owner authorization. An Android Keystore/JNI backend has synthetic JVM evidence, and its API 28 arm64 AAR cross-build passed archive/export/alignment checks. Real reauthorization and Flutter account acceptance, physical Android Keystore/phone acceptance, physical iOS Keychain/Flutter acceptance, and physical-device support remain pending; see `docs/task-breakdown.md`. A real adapter is `verified` only after model-specific physical-device testing.

API 1.20 adds three trusted C connection-link functions with session-owned,
bounded previews. Actual UI consent and independent adapter identity/model/route
evidence are required. Production Mock and Meiju keep their default refusal;
ordinary JSON, CLI and MCP cannot link. SQLite schema 5 and protected payload 3
are unchanged. See ADR-0061 before implementing an adapter opt-in.

An isolated test APK has passed native Core and Android Keystore checks on AOSP
Android 15 x86_64, including process restart and lock-screen refusal. Its separate control phase verifies
local grant revocation, trusted approval consumption/lifecycle and uncertain
write receipt recovery without replay on Android. A separate force-stop phase
verifies a live pending write, actual process disappearance and new-process
recovery of the unchanged unknown receipt with replay/retirement guards. It uses
fictional material and no Internet permission. Android storage must use the
reviewed Bionic file-lock path; standard Rust file locking is unsupported there.
See the [Android acceptance guide](docs/development/08-android-native.md).

The [iOS Keychain backend](docs/development/09-ios-native.md) requires an explicit
trusted compile-time private access group. Portable tests and iOS target checking
do not prove signed entitlements, Keychain runtime, framework packaging or phone
acceptance. It deliberately uses the documented deprecated no-prompt option;
see ADR-0062. Metadata lookup/deletion is not a device-lock probe. No real account
or appliance is needed for its synthetic or Core Foundation object tests. The
isolated iOS 26.2 Simulator app has passed Mock/Core and fictional Keychain
round trips across distinct processes; physical policy and Flutter acceptance
remain open. See the iOS guide for the exact evidence scope.

The Android AAR also supplies `NativeLanNetworks` for explicit foreground
assigned-interface snapshots and exact revalidation. Selections expire and are
invalidated by background/close; they never grant socket permission or scan
consent. Its Android 15 metadata acceptance APK has `ACCESS_NETWORK_STATE` only,
without `INTERNET`; no discovery or appliance traffic is used. Flutter controller
and Android 16/17 runtime-permission integration remain pending.

Current checks:

```bash
python3 scripts/validate_docs.py
python3 -m pip install -r requirements-dev.txt
python3 scripts/validate_contracts.py
python3 scripts/validate_repository.py
python3 scripts/validate_commits.py --all
```

Run the Rust and runtime commands in `docs/testing.md` for application changes. The integrated app pins Flutter through FVM in `apps/mobile/.fvmrc`; executed Linux/Android emulator checks and remaining acceptance are recorded in `apps/mobile/verification.md`. Recheck local tool availability on each workstation. Never report unexecuted `cargo test`, mobile builds, physical-device tests, or remote CI as successful. Record a specific `blocked` reason when hardware, accounts, authentication evidence, or licensing is missing.

## Delivery workflow

Choose a small task with acceptance criteria. Update contracts and tests, then Core/Adapter logic, then entry points. Keep documentation, fixtures, errors, and compatibility records synchronized. Write an ADR before a breaking public API change.

Commit each completed and verified development phase separately, as requested by the owner. Keep unrelated work out of the commit. A phase commit is local; pushing or publishing remains a separate external action. Flutter development uses the versioned contract in `docs/native-ui-contract.md` and may proceed in a separate owner-created worktree.

A completion report identifies changed files, commands actually run, results, unverified work, compatibility effects, and external actions. Creating or pushing repositories, publishing packages, performing real-device writes, and uploading diagnostics require explicit user authorization. Never claim a remote action succeeded without checking the resulting remote state.
