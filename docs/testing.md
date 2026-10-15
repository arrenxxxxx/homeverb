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
| Storage | WAL-aware schema-1/2/3 backup/upgrade to schema 4, rollback, newer-version refusal, exclusive migration access, detailed receipt pruning, active-write protection, and orphan recovery without replay |
| Local-data cleanup | Real SQLite/Core and C ABI cache/history clearing, preserved metadata, cache miss without reads, active composite journal refusal across store instances, rollback, cancellation/deadline/foreground checks, retired request IDs and reopen without replay |
| Native credentials | Opaque references, nonserializable/redacted secret buffers, unavailable/locked/duplicate error mapping, preflight cancellation/deadline refusal, and uncertain mutations; explicit Linux service probe below |
| Account vault | Persistent profile/account references, profile isolation, lease contention and latest-session rereading, pending/stored transitions, recovery without rewriting, unavailable storage, cancellation, injected SQLite update failures, and absence of session markers in SQLite/WAL; synthetic secrets only |
| CLI runtime | Actual binary output validated against JSON Schemas, exit codes, room/preferences and state persistence, batch rollback after an injected second-device update failure, filtered history, schema-1 upgrade, injected SQLite journal failure, dry run, SIGINT, and forced termination/restart |
| Android credential storage | Production Java compilation against official API stubs, 54 synthetic encrypted-entry assertions (round trips, randomized ciphertext, profile/reference binding, tampering, key loss, bounded data, locks, symlinks and deletion), and an explicitly run JVM/JNI test for array clearing, status mapping, cancellation/deadlines and uncertain writes; no Android Keystore or phone test |
| Android native library | API 28 arm64 AAR cross-build with NDK r28c, JDK 21 and API 35 stubs; production-only archive, eight native exports and 16 KiB ELF alignment checked; no APK or release-shrinking test |
| Meiju vendor fan control | Four candidate AC models and six codes, exact fan-only bodies, no implicit power/mode/temperature, descriptor/argument/protected-binding guards, independent exact-code startup policy, accepted/unknown/rejected receipts, no cache changes or replay, lifecycle cancellation and actual Linux ABI/schema startup checks; no physical control |
| Native ABI | Linux library loaded directly, Mock discovery/control, room/card requests, concurrent foreground cancellation and close; Meiju empty-account startup/discovery and invalid mode/grant/scenario refusal without vendor calls. Live read-only evidence is recorded in the native handoff |
| Native UI contract | All 31 published methods called through the built C ABI, offline method-specific response validation, saved examples, closed-field rejection, null success, admission errors, all execution outcomes, explicit cancellation and foreground/closed-session refusal |
| Meiju authentication/HTTPS | Fictional login/signing/session vectors, private input/session validation, exact request bodies and account binding, rejected challenges, no retries, cancellation/deadlines, redaction and local TLS certificate/redirect/body-limit/socket-closure checks; no real account test |
| Meiju inventory | Synthetic code/data envelopes, nested homes/appliances, candidate and unknown families, deduplication/conflicts, account isolation, malformed/partial/auth-expired responses, shared deadline/cancellation, no retries, and redacted native data; synthetic protocol evidence, not a device claim |
| Meiju state | Synthetic AC numeric/compound temperatures, fridge/freezer separation, vendor-specific dishwasher/AC codes, exact account/appliance/query routing, null/absent readings, malformed/oversized/rejected responses, unproven freshness, cancellation/deadlines, no retries, redaction and nonserializable source state; no live/model acceptance |
| Native account connection | Synthetic login-to-card publication without duplicate inventory, schema-validated summaries/results, same-subject reauthorization and preferences, partial/expired retrieval, saved-session refresh, uncertain binding storage, unavailable storage, wrong-subject/cross-account conflicts, shared operation IDs, cancellation/deadline/background/close and lease release; C ABI caller-buffer clearing and malformed/mode/closed refusal |
| Saved account inventory inspection | Actual synthetic host responses for listed/not-listed/unconfirmed and unpublished bindings, exact account/region isolation, no secret/network mutation, legacy preservation, redaction, unavailable/corrupt/pending/removing/busy errors, cancellation/deadline/background/close/drop and lease release; Linux ABI admission and closed request/response counterexamples |
| Local account metadata | Synthetic host and C ABI detail/rename calls, account/provider isolation, one ID per linked card, unsupported-card inclusion, unavailable secrets, pending-state preservation, lease contention, invalid names, injected SQLite rename failure and reopening; no live account edits |
| Pending Meiju reauthorization | Synthetic protected login with explicit account selection, expired saved-session recovery refusal, same-subject replacement, preserved bindings/preferences/grants, changed/missing/corrupt material, wrong identity, rejected login, uncertain persistence, SQLite confirmation failure, partial retrieval and lifecycle/lease cleanup; C ABI rejects forged identity/recovery fields and clears caller buffers |
| Meiju saved-session recovery | Synthetic host validation after uncertain initial/map writes, stable IDs/preferences, exact-byte confirmation without another login, partial/expired reads, corrupt/foreign/missing material, changed-byte races, SQLite failures, cancellation/deadline/background/close/drop and lease release; actual ABI admission without a keyring |
| Account removal | Exact scope and provider/route isolation, unreadable credential cleanup without secret reads, durable intent, unavailable/locked/conflicting/uncertain deletion, explicit retry without duplicate erasure, cancellation/background/close/dropped futures, schema-3 backup, rollback, late publication/cache refusal, retained receipts and rooms; ABI admission and completed-ledger replay without a keyring |
| Meiju connection | Protected session persistence followed by automatic inventory, same-subject reconnect across reopen, separate failed/partial retrieval, explicit refresh without login, changed-subject and foreign-region refusal, unavailable storage and unselected pending accounts before login, profile registration lock and cancellation release; synthetic account evidence only |
| Meiju inventory membership | Synthetic full/partial/failed retrieval, legacy payload migration, disappearance/reappearance, preserved IDs/cards/cache/receipts, stale descriptor publication, uncertain persistence and native read/control refusal before transport; no owner profile or appliance access |
| Meiju private bindings | Real SQLite with synthetic protected storage; stable device/connection/identity IDs across reopen and same-subject session replacement, preserved local preferences, partial/empty inventory retention, whole-update conflict refusal, account isolation, corrupt/missing payload refusal, no plaintext identifiers, uncertain-write recovery without regeneration, size limits and cancellation |
| Meiju bound Core reads | Test-only AC/E1/CA descriptors admitted by Core, exact protected connection/model/component preflight, stale cloud values and separate fridge/freezer caches, malformed/undeclared response refusal without cache replacement, cached reads without secret access, blocked writes, shared cancellation/deadline and account-lease release |
| Native power grants | Closed startup schema and C ABI admission/refusal before profile creation, exact canonical UUID/state/cloud grants, unchanged elevated-risk denial, synthetic host plan/execute/replay/receipt, read-only reopen, foreground cancellation/close and lease release; no real appliance operations |
| Meiju AC cloud control | Synthetic exact-model power descriptors, explicit admission and device/state/cloud grants, protected binding/model refusal, code-zero acceptance without verification, authorization rejection, ambiguous provider errors, lost replies, unsent/sent cancellation/deadlines, unchanged cache and replay across Core reopen; local TLS tests track send phases and socket closure without redirects/retries. No physical controls |
| Meiju model adapter | Exact six-model read admission through Core, unsupported visibility, protected model/family checks, partial/empty rediscovery and preferences, historical cross-account conflict refusal, cache preservation, local queries without secrets, cancellation/deadline and lease release; live read-only evidence recorded separately |
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

## Meiju native protocol and HTTPS tests

`cargo test -p homeverb-midea --offline --locked` also starts generated local TLS
fixtures. A sandbox that prohibits loopback listeners cannot run those cases;
record the restriction and run with ordinary local socket access, rather than
skipping certificate, redirect, body-limit or cancellation tests. All credentials
in these tests are fictional. The known-answer fixture uses independent Python
hashlib/HMAC and OpenSSL calculations; no test contacts a vendor endpoint.

## Midea account and remote-control acceptance

These are required future checks, not results from the Mock suite. Follow the [integration guide](development/05-midea.md) and record model/firmware/app/region with sanitized evidence.

- Connect an existing Meiju account and automatically retrieve bound devices across accessible homes. Require neither re-pairing nor manual IP/Token entry. Distinguish unsupported, offline, empty, partial, and failed retrieval; never infer fresh state from device-list membership.
- Repeat retrieval and reauthorization without duplicate cards or loss of aliases, rooms, favorites, or established bindings. Test account/region isolation, ambiguous identity, partial-list preservation, credential expiry/refresh, and unavailable secure storage.
- Read and perform explicitly owner-authorized supported writes through cloud with the phone's Wi-Fi disabled, the appliances still connected to Midea, and no home computer/gateway. Repeat on an external network without LAN discovery or local-network permission. Verify each claimed capability independently.
- Exercise stale cloud state, rejection, lost acknowledgements, cancellation/backgrounding, and forced termination. Preserve accepted/unknown outcomes and do not replay uncertain writes, including across routes. After completion or cancellation, leave no polling or background networking.

## Unverified release gates

Physical-device authentication and control, real-provider account recovery and token rotation, actual Android Keystore, Android APK/iOS builds and phone lifecycle, a target AI host, dependency/security review, package installation, and licensing remain pending. The mainland read-only Meiju pilot and arm64 library build do not establish these acceptance gates. Real writes require explicit device-owner authorization. Two different protocol families must demonstrate equivalent capability calls before cross-platform support is claimed.
