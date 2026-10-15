# Test plan and quality gates

## Local verification

For an isolated Python environment, run `python3 -m venv .venv` and `source .venv/bin/activate` first. Install the declared requirements below before running Python tests.

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
cargo build --workspace --locked
python3 -m pip install -r requirements-dev.txt
python3 scripts/verify_mvp.py
python3 scripts/verify_native_contract.py
node packages/presentation/test/i18n.test.mjs
python3 scripts/validate_docs.py
python3 scripts/validate_contracts.py
python3 scripts/validate_repository.py
python3 scripts/validate_commits.py --all
python3 -m unittest discover -s tests -v
```

After dependencies are cached, add `--offline` to Cargo commands. Use a Python virtual environment when system package installation is restricted. `requirements-dev.txt` explicitly includes RFC 3339 validation; contract and runtime checks fail if `date-time` checking is unavailable rather than silently ignoring timestamp formats. The runtime acceptance script currently targets Linux (`libhomeverb_ffi.so`) and uses temporary fictional profiles. Its SIGINT case requires ordinary OS signal delivery. A sandboxed failure must be distinguished from an actual run outside the sandbox; never silently skip that case.

## Coverage

Flutter development uses FVM and the version pin in `apps/mobile/.fvmrc`. Build `homeverb-ffi` first, then run from `apps/mobile`:

```sh
fvm install
fvm flutter pub get
fvm flutter analyze
fvm flutter test --reporter expanded
fvm flutter test integration_test/native_smoke_test.dart -d linux
fvm flutter build linux --release
fvm flutter build web -t lib/main_preview.dart
```

The Linux integration can run under Xvfb on a headless host and requires Flutter's Linux development dependencies. Native library packaging is handled by the Linux CMake install step. From the repository root, `python3 apps/mobile/tool/compare_screens.py --require-exact` compares all 82 captures to the untouched prototype and fails on any pixel difference. See [Flutter verification](../apps/mobile/verification.md) for actual results and unverified phone/localization/accessibility gates. Simulated preview branch clicks do not establish real provider behavior.

| Layer | Evidence |
|---|---|
| Core integration | Real SQLite, invalid bounds/steps/enums, grants, hard routing, stale evidence, lost ACK, validation-inclusive and inherited host deadlines, cancellation, persisted/retired request IDs, locks, and composite results preserved across journal/cache failures |
| Descriptor admission | Complete per-adapter validation before saving, schema versions/shapes, unique IDs and references, locally compilable action schemas, foreign connections, ambiguous property names, preserved metadata, and CLI operational refusal followed by valid rediscovery |
| Component scopes | Shared legacy resolution, distinct compartment ranges/steps/freshness, independent grants/risk/routes, verified writes and cache isolation, null/type refusal, duplicate/missing/mixed scopes, invalid stored metadata, rediscovery preservation, and actual CLI scoped descriptors/cache |
| Action decomposition | Explicit independent-setter opt-in, full-argument delivery without it across idempotency categories, lost acknowledgement without field loss or replay, invalid combinations rejected before sends, legacy CLI metadata and rediscovery, and preserved composite policy/results |
| Rediscovery identity | Adapter takeover and each binding-field change rejected, whole-response preflight, metadata updates and reordered connections allowed, preferences/cache/history preserved, competing SQLite inserts, and actual CLI conflict output |
| Receipt lookup | No adapter or journal changes, lookup with invalid metadata/no write grants/device lock, active records outside recent history, exact and long child IDs, missing/retired/blank IDs, cancellation/deadline refusal, and CLI/native/MCP recovery queries |
| Property freshness | Deterministic per-property expiry and equality boundaries, legacy defaults, integer-valued floats and JSON-safe bounds, invalid descriptor limits, relaxed limits with retained stale flags, post-send evidence, and CLI mixed-limit cache reads without snapshot mutation |
| Correlated completion | Non-idempotent actions without observable state, exact identity/connection binding, absent/stale/future/reversed timestamps, accepted-only and method gating, rejection/uncertainty, cancellation/deadline after acceptance, journal failures, and CLI composite/restart evidence without fabricated snapshots |
| Device batches | Whole-list validation/grants/request conflicts, duplicate devices and composite child-ID collisions, four-device concurrency bound, ordered and mixed results, independent failures, shared deadlines/queued cancellation, individual replay, CLI exit/schema checks, and native foreground cancellation |
| Identifier boundaries | Maximum-length ASCII/UTF-8 caller IDs, composite planning and persisted child receipts, offline replay across CLI processes, oversize rejection, long-device writes, cross-instance lock contention, and fixed legacy/digest lock filenames |
| Observations | Full-snapshot version/identity/source/time and component property validation, type/enum/range/step failures, exact numeric target matching across integer/float representations and integer limits, unreadable/ambiguous metadata, read-only vendor values, old-source freshness, preserved cache/acceptance, empty state, clock rollback, cached cancellation, and actual CLI rejection of invalid cache records |
| Organization | Room/name validation, preference persistence, exact-ID permutations, atomic batch room assignment, preserved preferences, unassignment, alias ambiguity, device-history filtering before limits, and no permission escalation |
| Write policy | Component isolation, argument limits, complete-grant matching, granted route intersection/fallback, malformed schemas, composite step authorization, and unchanged high-risk denial |
| Storage | WAL-aware schema-1 backup/upgrade, rollback, newer-version refusal, exclusive migration access, detailed receipt pruning, active-write protection, and orphan recovery without replay |
| Native credentials | Opaque references, nonserializable/redacted secret buffers, unavailable/locked/duplicate error mapping, preflight cancellation/deadline refusal, and uncertain mutations; explicit Linux service probe below |
| CLI runtime | Actual binary output validated against JSON Schemas, exit codes, room/preferences and state persistence, batch rollback after an injected second-device update failure, filtered history, schema-1 upgrade, injected SQLite journal failure, dry run, SIGINT, and forced termination/restart |
| Native ABI | Linux library loaded directly, independent discovery/control, room/card requests, concurrent foreground cancellation, and explicit close |
| Native UI contract | All 22 published methods called through the built C ABI, offline method-specific response validation, saved examples, closed-field rejection, null success, admission errors, all execution outcomes, explicit cancellation and foreground/closed-session refusal |
| MCP runtime | Initialization, tools, structured results, default denial, forged fields, no Internet sockets while idle, and EOF exit |
| Presentation | Exact locale key/placeholder parity, locale fallback, missing keys, unknown observations, and uncertainty display |
| Diagnostics | Actual CLI/native closed-schema reports, omitted private markers from metadata/receipts/nested items, normalized unknown codes, fixed storage errors, bounded receipt/attempt output, no adapter calls, and no journal mutation |
| Repository | Markdown targets, fixtures, English policy with a narrow Chinese catalog exception, and commit conventions |

The CI workflow declares these build/test steps; a local run is not evidence that remote CI ran.

JSON Schema validation uses `jsonschema` 0.45.1 or a compatible patch, with default network-resolution features disabled and the resolved version in `Cargo.lock`. This version includes the [upstream correction for negative `multipleOf` values](https://github.com/Stranger6667/jsonschema/blob/master/CHANGELOG.md). Regression coverage exercises negative fractional temperatures through device constraints, trusted grants, writes, and live/cached observations.

## Linux Secret Service probe

This opt-in integration test needs an existing unlocked default Secret Service collection and permission to access the user's D-Bus. It creates only synthetic bytes under fresh HomeVerb namespaces, checks replacement and isolation across instances/profiles, deletes those entries, verifies absence, and checks bounded socket cleanup. It does not enumerate or read existing account secrets. Default `cargo test` ignores it.

```bash
cargo test -p homeverb-secrets --offline --locked --test system_store linux_secret_service_round_trip -- --ignored --exact
```

The probe passed locally on Linux with GNOME Secret Service. Locked collections, interactive service prompts, other implementations, and actual platform credentials were not integration-tested. Unit tests separately exercise sanitized locked/unavailable errors and deadline/cancellation outcomes without keyring access. The test cleans up on returned operation failures; forced process termination can still leave synthetic entries, just as it can interrupt an ordinary credential mutation.

## Midea account and remote-control acceptance

These are required future checks, not results from the Mock suite. Follow the [integration guide](development/05-midea.md) and record model/firmware/app/region with sanitized evidence.

- Connect an existing Meiju account and automatically retrieve bound devices across accessible homes. Require neither re-pairing nor manual IP/Token entry. Distinguish unsupported, offline, empty, partial, and failed retrieval; never infer fresh state from device-list membership.
- Repeat retrieval and reauthorization without duplicate cards or loss of aliases, rooms, favorites, or established bindings. Test account/region isolation, ambiguous identity, partial-list preservation, credential expiry/refresh, and unavailable secure storage.
- Read and perform explicitly owner-authorized supported writes through cloud with the phone's Wi-Fi disabled, the appliances still connected to Midea, and no home computer/gateway. Repeat on an external network without LAN discovery or local-network permission. Verify each claimed capability independently.
- Exercise stale cloud state, rejection, lost acknowledgements, cancellation/backgrounding, and forced termination. Preserve accepted/unknown outcomes and do not replay uncertain writes, including across routes. After completion or cancellation, leave no polling or background networking.

## Unverified release gates

Physical-device authentication and control, real-account credential integration, non-Linux secret stores, account refresh locking, Android/iOS builds and phone lifecycle, a target AI host, dependency/security review, package installation, and licensing remain pending. Real writes require explicit device-owner authorization. Two different protocol families must demonstrate equivalent capability calls before cross-platform support is claimed.
