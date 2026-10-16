# HomeVerb

**One language for your appliances. No daemon required.**

HomeVerb is the owner-approved project name; `homeverb` is the executable and repository name.

> **Status: 0.2.0 Mock MVP.** The Rust Core, persistent fictional appliance, CLI, local MCP with default read-only policy, and native UI bridge are implemented. An experimental Meiju adapter has read six actual appliances through Core and the native bridge; physical verification and real control acceptance remain pending. Android/iOS packaging and visual UI are separate work.

## Run the MVP

Requirements: Rust 1.96 or later. The first build downloads dependencies; `Cargo.lock` records resolved versions. Commands use a private local profile and never contact an appliance.

```bash
cargo build --workspace --locked
./target/debug/homeverb --mock device discover
./target/debug/homeverb --mock device list
./target/debug/homeverb --mock device describe mock-ac
./target/debug/homeverb --mock power set mock-ac --on
./target/debug/homeverb --mock climate set mock-ac --temperature 25
./target/debug/homeverb --mock device state mock-ac --fresh
```

All operation output is JSON. Mock state survives process exit. Use `--data-dir /absolute/private/path` for an isolated installation and `--profile demo` for an isolated profile. An `unknown` result is intentional when acknowledgement is lost:

```bash
./target/debug/homeverb --mock --scenario lost-ack power set mock-ac --off
```

This exits with code 11. Refresh state to inspect the result; do not blindly resend. See the [runnable walkthrough](docs/development/01-run-mock.md).

## What is implemented

- Capability-driven validation, device/component/argument/route-scoped low-risk grants, shared deadlines, cancellation, and timestamped verification.
- SQLite metadata/cache/execution journal, cross-process device locks, and request-ID replay protection.
- A [trusted single-command approval bridge](docs/adr/0043-native-trusted-approval-bridge.md) for elevated actions: immutable preview, matching grants, 60-second expiry, one-use execution and lifecycle invalidation. Three separate C functions and a fictional Mock scenario support Flutter integration; actual consent UI and physical acceptance remain pending. Profile-scoped experimental Meiju dishwasher starts now require the separate confirmation path and exact startup grants.
- A bounded Linux Secret Service library and native account vault with persisted profile/account references, leased session updates, redacted secret buffers, and no plaintext fallback. A native Meiju login/read client is synthetic-tested; protected connection and automatic inventory are available through a [local read-only probe](docs/development/06-meiju-probe.md). One real mainland account has passed login/inventory and saved-session refresh; six exact model mappings now support read-only Core/native access. Native API 1.15.0 supports protected connection and same-account pending reauthorization, local account details/rename/removal, account list/refresh/recovery and optional independent exact-device power, fan, explicit power-on/mode and profile-scoped temperature grants; an [Android native store/build package](docs/development/08-android-native.md) supplies synthetic JVM/JNI evidence, while Flutter account input, Android Keystore/phone acceptance and iOS storage remain pending.
- Protected Meiju inventory membership gates live access while retaining historical cards and identity; legacy accounts require an explicit complete refresh. Native `inspect_account_inventory` exposes saved membership without cloud work or credential changes. See the [native migration guidance](docs/native-ui-contract.md#missing-devices-and-legacy-account-inventory).
- Local rooms, aliases, favorites, and card ordering through Core, CLI, and the native UI bridge; rediscovery preserves preferences.
- Explicit cached-state clearing and history retirement, preserving active journals and request-ID replay protection.
- Schema-1/2/3 migration to schema 4 with a consistent backup, bounded detailed receipts, and retained request IDs that prevent replay after history is retired.
- Explicit local diagnostic previews with allowlisted fields and bounded receipt summaries, available through CLI and the native UI bridge.
- Persistent Mock power/climate operations and fault scenarios, including non-atomic partial outcomes.
- Experimental Meiju AC power controls through an explicit [developer preview/probe](docs/development/07-meiju-control.md), with scoped grants and synthetic uncertainty tests. Default native sessions remain read-only; [scoped startup grants](docs/native-ui-contract.md#scoped-experimental-ac-power-startup) can admit experimental AC power controls; API 1.9 adds [independent vendor fan permissions](docs/native-ui-contract.md#scoped-experimental-ac-fan-startup). No physical control is verified.
- Bounded correlated-completion verification with exact request/device/connection and timestamp checks; fictional evidence does not establish real protocol support.
- Explicit device batches through Core, CLI files, and native requests, with whole-list preflight, four-device concurrency, shared deadlines, and per-device receipts.
- Component-scoped capabilities for different constraints on one physical appliance, verified with fictional fridge/freezer fixtures.
- One-shot CLI, host-owned [MCP stdio](docs/mcp.md) with Mock/Meiju reads and explicitly granted Meiju command planning/execution, and a C ABI that loads Core directly in the UI process. MCP defaults to read-only and never connects accounts automatically.
- Experimental [Meiju CLI account and read commands](docs/cli.md#meiju-account-and-read-commands): protected Linux terminal login, automatic card retrieval, saved-session refresh/recovery, local account management and on-demand Core reads. Default startup stays read-only; an explicit [owner-controlled grant file](docs/cli.md#scoped-meiju-controls) can admit exact-device power states, vendor fan codes, explicit power-on/mode activation and profile-scoped temperature targets. Live CLI/phone acceptance is separate.
- Framework-independent `en-US` and `zh-CN` catalogs, locale normalization, and presentation helpers. Visual UI design remains separate.
- An isolated [Android native acceptance APK](docs/development/08-android-native.md#run-the-isolated-native-acceptance-apk) has passed Core initialization/Mock execution, Android Keystore persistence across processes, lock-screen refusal, file-lock contention and local account cleanup on AOSP Android 15 x86_64. Its fictional fixtures and disabled networking do not establish Flutter, physical phone or appliance acceptance.

## Boundaries

Each host owns its Core instance. No HA, NAS, project backend, daemon, relay, background polling, cloud sync, or remote HTTP MCP is required or implemented. The experimental Meiju adapter contacts the vendor cloud directly through the owner-admitted native flow. Mock success does not establish device compatibility or mobile support.

## Develop progressively

1. [Run and verify the Mock flow](docs/development/01-run-mock.md).
2. [Understand and extend the Core](docs/development/02-core.md).
3. [Connect a UI and localization](docs/development/03-ui-integration.md).
4. [Admit a real adapter](docs/development/04-add-adapter.md).

[Documentation index](docs/index.md) | [Current task status](docs/task-breakdown.md) | [Compatibility](docs/compatibility.md) | [Testing](docs/testing.md) | [AI contributor rules](AGENTS.md)

Repository documentation, code, diagnostics, and commit messages are English. Chinese is permitted only in the requested `zh-CN` translation catalog. Use English [Conventional Commits](docs/commit-conventions.md). The [project license](LICENSE-DECISION.md) remains undecided; nothing has been published by this implementation work.
