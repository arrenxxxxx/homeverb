# ADR-0103: Modern Mijia daily energy history

Status: Implemented with fictional protocol/Core/Host evidence; provider and mobile package acceptance pending.
Date: October 30, 2026.

## Evidence

MIT `ttys026/mit` revision `a5d32181d0251ee36dde6edb316e16879d5ac175`
[src/mijia_api.rs](https://github.com/ttys026/mit/blob/a5d32181d0251ee36dde6edb316e16879d5ac175/src/mijia_api.rs)
implements SID=mijia statistics at `https://api.mijia.tech/app/v2/user/statistics`.
It sends did, key, data_type, time_start, time_end and limit, with per-field
RC4-drop1024 reset and modern session cookies. Its storage identity helper prefers
the numeric userId. Plain JSON and encrypted JSON/gzip replies are accepted by
that source. This differs from ADR-0094's continuous-stream reference reads at
core.api.mijia.tech; neither profile is a fallback for the other.

ADR-0088's Apache community evidence supplies the exact cuco.plug.v3 key 11.1,
stat_day_v3 type, single numeric channel and 0.01 kWh conversion. Reusing that
profile on the modern statistics route is source-based experimental integration,
not evidence of provider retention, timezone or physical compatibility. The
community source's device-ID suffix stripping is not adopted: only the exact
protected root identity is sent. No new credential or private source is needed.

## Decision

Extend the fixed modern read transport with one statistics endpoint. Its internal
request builder selects the community origin, codec, persisted login agent and
cookies before I/O. Public history requests cannot select origin, codec, key,
provider ID, model or credential slot. PassToken is never sent to a data endpoint.
All other modern reads retain their existing continuous codec and cookies.

Admit only a listed protected root with the exact ADR-0088 model/specification
and modern-selected binding. The typed protocol independently checks subject,
model/specification, window and mapped target. Core/adapter repeat identity,
permission, exact target and route checks. A modern-only account needs no legacy
slot. Legacy bindings retain the legacy statistics path, even when modern login
material exists. Missing transport/session or any request failure never selects
another slot, renews, logs in or retries.

One foreground request inherits account/device leases and the caller's deadline.
The interval is half-open UTC, at most 32 days, ending no later than admission;
limit is 31 records. Raw and decoded statistics bodies have a 1 MiB ceiling.
Gzip decoding rejects corrupt, trailing, concatenated and oversized output.
Plain code-zero JSON is accepted only by this reviewed statistics profile; the
reference profile still refuses it. Provider authentication errors remain explicit.

Reuse ADR-0088/0089 record validation, duplicate removal and conversion. Optional
provider identity/key correlation must match, missing days are not zero, daily
timezone and completeness remain unknown. No current-state cache, write receipt,
control permission, device mapping or credential persistence changes. Recheck
lifecycle after decoding and before publication.

The existing Core/native/CLI/MCP/App history contract carries these results. Native
API 1.37, C exports, JSON schemas, SQLite 5 and protected payloads are unchanged.
No broader history/event route or new control action is admitted. The delivered
API 1.37 APK predates this addition until explicitly rebuilt.

## Required evidence

Fictional transport checks must distinguish per-field reset from continuous RC4,
assert exact origin/cookies/parameters and root identity, test response forms,
bounds, malformed/auth responses, subject/model refusal and late cancellation.
Local TLS checks cover headers, redirects, TLS validation and socket cancellation.
Core tests cover modern-only/mixed accounts, preserved legacy selection, restart,
no fallback/secret/cache/receipt mutation, exact target refusal and account/device
lease release. Run native ABI and generic App history regressions; no real account,
phone or appliance acceptance may be inferred from them.

## Executed evidence

The Rust workspace run excluding the previously documented unrelated Meiju/macOS
LAN-alias failures passed 604 tests with one ignored. Nine new tests cover the
statistics protocol, protected Core queries and the native Host history journey;
the existing TLS matrix now includes the new exact origin. The Mijia-only run
passed 204 unit tests and 14 compile-fail documentation tests. After extending
negative cases for concatenated gzip, conflicting encoding flags and encrypted
authentication failures, all four protocol tests passed again.

Strict workspace/all-target Clippy and the rebuilt FFI passed. The native checker
passed 36 methods/57 fictional examples, all dedicated bridge suites and 76 modern
account boundary cases. The 65 affected Flutter history and modern-device flow
tests passed; no App implementation change was needed. This combines actual Host
history behavior with generic App interface tests, not a real phone/QR journey.
Documentation, repository hygiene, all 25 schemas (69 positive/1196 negative
fixtures), reference regeneration and both projection tests passed.

Commands executed: cargo +1.96.0 test --workspace --exclude homeverb-midea
--offline --locked; cargo +1.96.0 clippy --workspace --all-targets --offline
--locked -- -D warnings; cargo +1.96.0 build -p homeverb-ffi --offline --locked;
python scripts/verify_native_contract.py; fvm flutter test
test/numeric_history_test.dart test/modern_device_flow_test.dart --reporter expanded.
All use fictional data. No new Android/iOS package, installation, provider account,
appliance write, remote CI, push or publication was performed. Broader modern
controls/history/events and real data/phone/model acceptance remain incomplete.
