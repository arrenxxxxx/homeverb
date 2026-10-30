# ADR-0105: Reported numeric property history

Status: Typed Rust protocol implemented; Core/native/App sample-history integration pending.
Date: October 30, 2026.

## Evidence

The pinned MIT ttys026/mit revision a5d32181d0251ee36dde6edb316e16879d5ac175
[src/mijia_api.rs](https://github.com/ttys026/mit/blob/a5d32181d0251ee36dde6edb316e16879d5ac175/src/mijia_api.rs)
provides /user/get_user_device_data at api.mijia.tech/app using the same per-field
modern community profile as ADR-0103. Its
[device history UI](https://github.com/ttys026/mit/blob/a5d32181d0251ee36dde6edb316e16879d5ac175/src/tui/device_history.rs)
constructs keys from exact SIID.PIID and reads response.result records with time
and value. Its [constants](https://github.com/ttys026/mit/blob/a5d32181d0251ee36dde6edb316e16879d5ac175/src/tui/mod.rs)
select type=prop. The history source blob is d15417049b3bb2d64f6abce4fc2f1fb70e156682.
The pinned Apache HA cloud client f78158257e19a21902adaf6bcc8440c3ba172a91
[xiaomi_cloud.py](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/xiaomi_cloud.py)
corroborates the legacy endpoint, parameter names and JSON-array value channel.
These sources establish an experimental generic property-log request, not complete
retention, physical observation, actor identity, causality or an event-history API.

## Decision

Implement one explicitly selected numeric property's reported history for the
existing pinned readable model/specification mappings. Re-derive the field from
exact model/specification and SIID/PIID before I/O; never accept a raw key, endpoint,
conversion or descriptor-supplied wire identity. Numeric enums remain raw numeric
codes. Boolean, opaque, private, health/media and unmapped fields are not coerced
into numeric samples. Existing scalar filtering remains the admission boundary.

Use legacy SID/origin for legacy bindings and the distinct modern community
SID/origin/per-field codec for modern bindings. No modern reference continuous
codec or cross-protocol fallback is attempted. Only the complete protected root
identity is sent, without source-client suffix stripping. Modern logical children
cannot become independent targets. No credentials are renewed or persisted.

The bounded foreground request sends did, key, type=prop, time_start, time_end
and limit=31. Use a positive half-open UTC interval of at most 32 days, ending no
later than admission. The 31-record bound retains the existing HomeVerb bounded
history budget; it does not claim the provider has only 31 records. No implicit
paging or omission-free continuation is invented from a timestamp. Empty pages
and short results cannot establish completeness. Raw/decoded response limits stay
1 MiB. Existing TLS, redirect, compression and cancellation safeguards apply.

Decode only one numeric channel in a JSON-array value string. Require finite
values satisfying the exact protected field's type, bounds, enum and step. Preserve
negative readings when admitted; do not apply daily-energy scaling. Reject numeric
underflow into fabricated zero. Retain only timestamped records within the query.
Supplied did/key/type correlation must match. Drop all records at an ambiguous
repeated second rather than choose a winner or invent ordering. Count rejected
records; never fill gaps, interpolate or compute daily aggregates. Provider actor
IDs, names and other raw metadata never leave the decoder.

## Core and entry-point delivery

The protocol phase is typed Rust only and does not expose a new public Host method
or history option. Native API 1.37, daily-energy schemas and App remain unchanged
until the dependent contract phase. Record its executed scope explicitly.

The subsequent contract must add reported_samples distinct from reported_daily,
permit finite signed values only for samples, and bind sample values/units to the
inspected readable numeric property. Missing units stay absent rather than becoming
kWh or a fabricated physical unit. Preserve daily-energy nonnegative validation and
unit conversion. Core must validate options/results before publication and retain
account/device leases, no state-cache or receipt writes, no background collection,
no automatic pagination and unchanged write authority. Version native/Flutter
integration before advertising it; old clients must reject/filter unknown series
without mislabeling samples as daily statistics. CLI/MCP reuse Core directly.
No public schema broadening is authorized without fixtures and a compatibility record.

## Required evidence

Fictional tests must assert both exact request profiles and independent per-field
signing, all pinned numeric mappings, exact enum/range/step/negative behavior,
unsupported/private/Boolean targets, time/count/body limits, foreign correlations,
malformed values, ambiguous timestamps, no raw identity leakage, pre-I/O refusal,
auth errors, pending deadline/cancellation/drop and post-reply lifecycle refusal.
Local TLS tests must cover both endpoint profiles. Later Core/native/App tests
must prove local inspection, protocol selection, descriptor revalidation, exact
option selection, lifecycle handling and no state/credential/receipt mutation.
No real account, provider query or appliance is required for these checks.

## Protocol phase evidence

The final protocol phase contains seven decoder/legacy tests, four modern tests,
and one added legacy TLS test. The existing modern TLS matrix now includes the
property-history endpoint and repeats certificate/cancellation/deadline/drop
checks for it. All 161 pinned numeric fields exercise their exact source domains;
Boolean/private/unmapped targets refuse before I/O. The initial failures exposed
missing range/enum constraints in existing read descriptors, so this implementation
derives temporary history domains directly from pinned source without changing
saved mappings. A successful-operation fixture's overly short 30 ms budget was
corrected; the dedicated pending-request deadline test remains bounded separately.

Executed on October 30: the complete Mijia suite passed 227 unit and 14 compile-fail
tests before the final cancellation case. The workspace run excluding the known
unrelated Meiju/macOS LAN-alias failures passed 629 tests with one ignored. The
subsequent legacy pending cancel/drop case passed separately. Strict workspace
all-target Clippy and workspace formatting passed. The rebuilt FFI and actual C
ABI checks passed all 36 ordinary methods/57 examples and dedicated boundaries,
including 76 modern-account cases. Documentation, source hygiene, 25 schemas with
69 positive/1196 negative fixtures, digest-based coverage regeneration and both
reference-projection tests passed. The direct jsonschema dependency reuses the
existing locked workspace version and disables external resolution.

No Dart source, native API version, ordinary schema, Core option, saved credential
or device descriptor changed. Flutter and Android packaging were not rerun for this
protocol-only phase. The delivered 9a6c004 package retains its recorded scope.
Real provider/sample availability, model behavior and phone acceptance are not
established by these fictional tests. The next dependent work is the versioned
reported_samples contract and end-to-end Core/native/App integration.
