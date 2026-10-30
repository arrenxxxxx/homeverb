# ADR-0094: Modern Mijia reference-session reads

Status: accepted for implementation; protected account and App integration pending.
Date: October 30, 2026.

## Evidence and scope

The digest-pinned October 22 reference in the
[integration guide](../development/14-mijia-reference-integration.md) records
successful read-only calls using SID=mijia at `https://core.api.mijia.tech/app`.
Its `contract.json` and `scripts/readonly_client.py` establish three read paths:

| Operation | Route | Evidence |
|---|---|---|
| Directory | `/v2/home/device_list_page` | Fixed inclusion flags, logical split children, `list`, `has_more`, `next_start_did` |
| Homes | `/v2/homeroom/gethome_merged` | App version 12, bounded home list, private home/owner identities |
| Properties | `/miotspec/prop/get` | One to 90 exact DID/SIID/PIID queries; correlate successful items with present values |

Request signing uses SHA-256(ssecurity || nonce), SHA-1 signatures and RC4-drop1024.
One RC4 stream processes the ordered `data` and `rc4_hash__` fields. This is
incompatible with the legacy/community per-field reset even though the primitive
and field names match. Responses use a new RC4 stream; `MIOT-CONTENT-ENCODING:
GZIP` identifies compression inside the encrypted envelope. Unencrypted JSON is
error evidence only. The archive reused an App session; it does not establish
that a newly issued ADR-0093 session passes these routes.

The separate MIT community implementation pinned in ADR-0093 corroborates SID
binding, signing primitives and encrypted gzip responses, but uses
`api.mijia.tech`, per-field RC4 and different inventory paths. Its endpoint,
codec or automatic refresh behavior is not a fallback for this reference profile.
No GPL implementation is copied. HomeVerb implements the protocol facts in Rust.

## Decision

Extend the independent modern Rust session with explicit directory, homes and
reviewed scalar reads at the exact reference origin. No legacy session, arbitrary
origin, route, write, action or RPC is admitted. Startup and login perform no
implicit inventory or state request. Each operation inherits one cancellation and
deadline; no retry, automatic renewal, alternate codec or origin fallback occurs.
PassToken remains exclusive to authentication, and device reads carry only the
reviewed encrypted subject, service-token cookies and country code.

Directory traversal retains opaque logical entries, including split children,
without publishing them as additional physical Devices. Entries are bound to the
protected modern subject; another account cannot query them. Duplicate IDs,
model/spec conflicts, malformed pages, repeated cursors and ambiguous continuation
fail closed. At most 16 pages and 4,096 entries are collected. Later ordinary page
failures may return positive partial entries and sanitized issues; authentication,
cancellation and deadline failures abort. An empty or exhausted directory is never
proof that every physical/shared device is represented, and never deletes cards.
Home reads retain opaque home/owner identities, optional names and conservative
completeness. No room or account authority is inferred from these values.

Property reads take only a directory entry and one to 90 distinct queries from
its pinned exact reference mapping. Unknown models remain visible entries with
no readable properties. Model/spec mismatch, service-1 identity properties,
credentials, media and opaque properties remain excluded. Responses correlate
DID/SIID/PIID; missing or failed values stay absent, not false/zero. No observation
time or physical freshness is invented from a cloud response.

Raw and decompressed envelopes each have an 8 MiB ceiling. Gzip processing is
bounded, rejects corrupt, concatenated or trailing payloads, and uses the pure
Rust `flate2` backend for native portability. This adds a locked dependency, not
a service. HTTP Content-Encoding is not silently decompressed. Header mismatch,
malformed data and plaintext code-zero envelopes fail closed.

No C/JSON API, protected payload, SQLite schema or App changes occur in this
protocol phase. Native account adoption, reconciliation of logical children,
protected persistence and foreground App wiring remain subsequent work. Real
account/provider/device acceptance is still required; synthetic success does not
establish full modern reference parity.

## Validation

Independent synthetic known-answer continuous-stream vectors must distinguish
legacy resets. Fictional encrypted directory/home/property peers cover paging,
partial failures, conflicts, account isolation, all pinned scalar mappings,
negative property admission and explicit expired-session handling. Generated local
TLS peers verify the fixed URL, exact headers/cookies, bounded bodies, no redirect
or retry, and cancellation/deadline/drop socket release. Gzip fixtures cover
compression bombs, CRC/truncation/trailing data and encoding mismatches. Preserve
legacy protocol tests and current contract/schema compatibility.
