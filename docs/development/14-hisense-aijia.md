# Hisense Aijia washer integration

Status: offline profile foundation implemented; callable platform integration
remains **incomplete**. Date: October 23, 2026.

## Source and implemented scope

The owner requested a new branch from main for domestic Aijia washer integration.
The [supplied reference package](../adapter-references/README.md) pins Aijia
7.0.8.12, two internal product identities and original-App cache observations.
It contains no independent login, renewal or signed HTTP implementation. Its
model codes are not independently confirmed retail labels, and its virtual
family card is not a third physical appliance.

`homeverb-hisense` is an offline Rust library. Exact product/category/partner
selection supplies component-scoped read-only capability definitions, program
metadata and a bounded ordinary-property cache decoder:

| Product | Required category | Components | Program records |
|---|---|---|---|
| WV20W-1 | connector.device.type.washer.multiple | main, left, right | 27 left + 27 right |
| WH120E5S-1 | connector.device.type.washer | main | 37 |

Both pinned profiles require partner `1001`. This metadata match establishes
neither physical identity nor protected account membership. Do not create a
binding from a display name, virtual group or guessed device ID.

`power.state` retains explicit observed on/off semantics. Other reviewed readings
use `vendor.hisense.washer`: online hint, child lock, remote permission, raw
vendor stage/program/pause tokens, remaining minutes and model-specific door
fields. Each field belongs to its actual component. Definitions have no actions
or events. Online hints and remote permission are observations, never grants.

Programs retain original codes and values: `Baby_care_wash.left/right` is `12`
on WV20W-1, while `Baby_care_wash` is `19` on WH120E5S-1. Even stage and pause
codes differ: stage `15` is `WaterDetection` on the upper product and `AICheck`
on the lower product. No shared numeric interpretation or control range is
inferred. Program references to 22/15 absent general-function definitions remain
listed; complete per-program temperature/spin/rinse constraints are blocked.

The decoder accepts a flat JSON map of original-App ordinary state properties,
not a gateway response or full private cache dump. It bounds input to 64 KiB,
omits missing/null fields and ignores unreviewed fields. Present known fields
must have their reviewed string representation, enum or numeric domain; malformed
known values reject the entire projection with `HISENSE_INVALID_CACHE`. Errors
contain no raw values. Missing values never use configuration defaults.

`CachedState` always reports stale. `captured_at` records cache collection only;
it is not provider receipt time or physical sampling time. The result deliberately
does not implement a Core Snapshot or create a cloud Connection. Power off with
43 remaining minutes stays representable without inferring that a cycle is
running. No import to the user's profile, credentials or cache is performed.

## Community authentication research

Public source was inspected on October 23, 2026 without account login or appliance
requests. The initial archive-only authentication blocker now has concrete
community implementation leads; HomeVerb has not implemented or validated them.

| Source | Pinned revision | Findings and limits |
|---|---|---|
| [HisenseHA main](https://github.com/manymuch/HisenseHA) | `3d02d9859b7663412ded3e5e51608db4ebb34c20` | MIT; portal phone/password login and legacy gateway token refresh; primarily AC/fridge support |
| [HisenseHA v1.7.7-beta](https://github.com/manymuch/HisenseHA/tree/43c7f17ca7a2eb92ff064f7ac40bc18f04ad56f6) | `43c7f17ca7a2eb92ff064f7ac40bc18f04ad56f6` | Modern AIHome home/list/detail requests and a named-state washer reader; beta, not proof of these exact products |
| [AIHome fridge PR 31](https://github.com/manymuch/HisenseHA/pull/31) | `896a58680b4cbdc80321390e3dec19ea12c40022` | Unmerged model-aware fridge work on the modern gateway; no exact multi-tub acceptance |
| [Washer issue 16](https://github.com/manymuch/HisenseHA/issues/16) | inspected current comments | Maintainer lacks a washer; reports include status parsing failures and subsequent beta suggestions |
| [ConnectLife HA](https://github.com/oyvindwe/connectlife-ha#login) | inspected current README | Explicitly excludes Chinese-account login; not a domestic Aijia authentication substitute |

The pinned Python client uses `portal-account.hismarttv.com/mobile/se/signon`,
AES-CBC/PKCS7/Base64 field encoding and a Base64-MD5 body signature in
`X-Sign-For`. The beta posts to `iot-aihome.hismarttv.com/4.0` with a distinct
read envelope (`sourceType=3`, UUID v4 and integer app type); the official-App
archive records a different pre-native logical envelope (`sourceType=20`, UUID
v1 and string app type). Keep these protocol profiles separate. A Python helper
named `refresh` in the beta actually signs in again with saved username/password;
it does not establish an independent modern refresh-token exchange.

The source hardcodes portal app material. Its classification and independently
public native-client provenance still need review before any HomeVerb admission;
public GitHub availability alone does not prove that a backend secret is safe to
embed. No literal key, secret, upstream code or credential was imported. The
source's logging, automatic retries, optimistic state and HA persistence are not
HomeVerb behavior. Its generic washer interpretations are not reused: for these
profiles, stage `1` is reservation waiting, not proof of running.

Pinned `pyhisenseapi.py` SHA-256:

- Main: `6be35406795f414faa259a6b75cbfbf6d3a9f61fb93479cf397423863a0a12f7`.
- Beta: `84845b5ce7724ef12f06e9c735bece06114e5d9057213740c098f6437aa3ecf9`.
- PR: `9a5eb2cf648e9d3902f53fcdf66535b60a6a702df9facaeb459d2db2661b0099`.

## Remaining phases and acceptance

1. Review admissible native authentication material and independently test exact
   request bytes with fictional credentials. Then verify owner-side login,
   challenges, stable customer identity and modern session renewal or explicit
   reauthorization. Do not ask for credentials in chat or copy the official App's
   private session as an implicit authentication shortcut.
2. Implement allowlisted bounded HTTPS with inherited deadlines/cancellation,
   no redirects or implicit retries. Verify business errors and response shapes;
   reconcile complete account/home inventories, including multi-tub/shared
   entries, with protected membership and stable random local bindings.
3. Register the adapter through host, add protected native account connection and
   synchronize native/App contracts. Keep unsupported products visible without
   fabricating capabilities. The current App/CLI/MCP do not offer Aijia login.
4. Verify model-specific live state and source freshness read-only. Keep control,
   scheduling, diagnostics, settings, binding and OTA unavailable until separate
   reviewed contracts, exact grants and trusted consequential approval exist.
   Physical writes require separate owner authorization.

The existing Meiju admission does not admit Hisense material automatically.
No backend, HA installation, relay, background task or new public API is added.
Native API 1.26, SQLite schema 5, protected payload 3 and execution semantics are
unchanged. Real login, live reads, mobile packaging and physical acceptance are
unverified.

## Reproduce the offline phase

```sh
python3 scripts/build_hisense_profiles.py --check
cargo test -p homeverb-hisense --offline --locked
cargo clippy -p homeverb-hisense --all-targets --offline --locked -- -D warnings
```

The generator verifies the original ZIP checksum and model-manifest checksum,
then projects only reviewed state metadata and non-private program identifiers.
It executes no archive script and imports no historic account readings. Use
`python3 scripts/build_hisense_profiles.py` after a reviewed source/projection
change. Eight fictional tests cover exact selection, program/component isolation,
model-specific code differences, malformed inputs, missing values, stale cache
semantics and capability schema/property-domain conformance.

Executed on this branch:

| Check | Result |
|---|---|
| `python3 scripts/build_hisense_profiles.py --check` | Passed |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --offline --locked -- -D warnings` | Passed |
| `cargo test --workspace --offline --locked` | 565 passed, 0 failed, 2 ignored opt-in tests; fictional loopback peers only |
| `cargo build --workspace --offline --locked` | Passed |
| `python3 scripts/validate_docs.py` | Passed |
| `.venv/bin/python scripts/validate_contracts.py` | 21 schemas, 52 positive and 341 negative fixtures passed |
| `python3 scripts/validate_repository.py` | Passed |
| `python3 scripts/validate_commits.py --all` | Passed before the phase commit |

No remote CI, mobile package, account login, live cloud read or physical-device
test is implied. Upstream source was read only; no upstream implementation was
executed or bundled.
