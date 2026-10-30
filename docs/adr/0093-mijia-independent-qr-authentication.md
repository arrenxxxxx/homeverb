# ADR-0093: Independent Mijia QR authentication

Status: typed Rust protocol implemented with synthetic tests; protected account, native host, App and authenticated acceptance pending.
Date: October 30, 2026.

## Evidence

The community implementation in MIT-licensed `ttys026/mit`, pinned at
`a5d32181d0251ee36dde6edb316e16879d5ac175`, provides a concrete `sid=mijia`
service-login, QR initialization, long-poll completion and passToken renewal
sequence in [mijia_api.rs](https://github.com/ttys026/mit/blob/a5d32181d0251ee36dde6edb316e16879d5ac175/src/mijia_api.rs).
Its Rust client remains a protocol reference, not an executable dependency.
Preserve the [source notice](../../third-party/mit-mijia/NOTICE.md) and license.

GPL-3.0 `Do1e/mijia-api` at `2cc61f2e4ae21d9e3736e450c968bbe53c87ebd1`
corroborates the sequence. Its `mijiaAPI/miutils.py` resets RC4 for each field;
this differs from the continuous stream in the October 22 official-App archive.
No GPL implementation is copied or transplanted. The separately pinned
[MiServer enum](https://github.com/DarkDance/yqfw-common-third-xiaomi/blob/a268ff8ba316c28a256b7916649bb6361ccc7352/src/main/java/cn/jzyunqi/common/third/xiaomi/account/enums/MiServer.java)
corroborates `https://sts.api.mijia.tech/mijia/sts`; its source license has not
been established and no code is copied.

On October 30, two anonymous, bounded HTTPS GETs independently checked the
service-login and QR-initialization response structures. No account, password,
existing cookie, phone number or device was used. No QR was displayed or scanned;
no completion poll, STS exchange, renewal, message or appliance request ran.
Only field names and URL scheme/host/path were retained, never ticket values:

| Stage | Exact observed destination or response |
|---|---|
| Bootstrap | `https://account.xiaomi.com/pass/serviceLogin`, `sid=mijia`, code 70016 |
| Login location | `https://account.xiaomi.com/fe/service/login` |
| STS callback | `https://sts.api.mijia.tech/mijia/sts` |
| QR initialization | `https://account.xiaomi.com/longPolling/loginUrl`, code 0 |
| Scan payload | `https://c3.account.xiaomi.com/longPolling/login`, query includes `sid=mijia` |
| Completion poll | `https://c3.lp.account.xiaomi.com/lp/s`, query key `k` |
| QR image | `https://account.xiaomi.com/pass/qr/login`; not fetched |

These observations establish anonymous endpoint shapes only. Other shards,
regions, successful account replies, expiry codes and renewal remain unverified.
The community device origin is `api.mijia.tech/app`; the archive uses
`core.api.mijia.tech/app`. Neither is enabled by this authentication phase.

## Decision

Add an independent typed Rust authentication module. Its session cannot be
converted into the legacy `MijiaSession`, and the existing account payload and
adapter keep using their existing protocol. Bootstrap and refresh send cookies
only to the exact account endpoint. QR initialization and polling do not carry
account tokens; STS uses its dedicated exact host/path with no account cookies.
No automatic redirects, proxy use, retries, periodic checks or device requests
are admitted. No wildcard host or arbitrary URL constructor is public.

A single-use protected challenge owns its original cancellation and monotonic
expiry, capped at 180 seconds and the initial operation deadline. It exposes only
the protected scan payload to a trusted native renderer; remote QR images are
not fetched. An explicit finish operation makes one long-poll request capped at
120 seconds and remaining lifetime, then one validated STS exchange. Leaving the
foreground must cancel the original operation; drop releases pending requests.
Timeout consumes the challenge and does not silently regenerate or poll again.

A successful result retains protected numeric subject, encrypted subject,
ssecurity, service token, passToken and generated client identity. Renewal is an
explicit bounded operation on that exact session, replaces it only on success,
requires a fresh signing key and token, and rejects subject changes before STS.
No guessed 30-day expiry is used. Rejected renewal returns `AUTH_REQUIRED`; the
caller must retain old metadata and request new user authorization. No write is
retried or enabled as a consequence of authentication.

This phase adds no native C/JSON method, App flow, persisted payload, inventory
route or public control. Host storage, same-account adoption, native challenge
registry, QR rendering and foreground App continuation require subsequent
verified phases. Modern official-App parity remains incomplete.

## Required validation

Synthetic exact request sequencing and cookie isolation; malformed, duplicate,
wrong-SID and foreign/shard URL refusal before network; single-use ownership,
initial and continuation cancellation/deadline/drop; wrong-subject renewal,
missing tokens/keys and sanitized rejection; local TLS response bounds, redirect
and duplicate-token refusal with no retry; redacted debug and nonserializable
sessions/challenges. Preserve all existing legacy protocol regressions.


Executed evidence: 140 Mijia unit and nine documentation tests passed; all 13
modern protocol/TLS cases passed again after the final adjustment. Workspace
regression passed 499 tests with one ignored, excluding the previously recorded
macOS `homeverb-midea` LAN-alias failures. Strict Clippy, formatting, contracts,
documentation and reference-projection checks passed. This is fictional/local
transport evidence, not successful provider login or native/App acceptance.
