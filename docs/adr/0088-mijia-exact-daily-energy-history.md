# ADR-0088: Exact Mijia daily energy history

Status: accepted for implementation; no runtime completion claim.
Date: October 27, 2026.

## Evidence

The Apache-2.0 community integration at revision
`f78158257e19a21902adaf6bcc8440c3ba172a91` selects `v2/user/statistics` with
`data_type=stat_day_v3` and key `11.1` for the exact reference `cuco.plug.v3`.
Its model customization scales raw values by 0.01, and its matching attribute
unit is kWh. Device code sends `did`, `key`, `data_type`, epoch-second
`time_start`/`time_end` and `limit`. The decoder reads `time` and a JSON-string
`value` array. Its tests preserve missing/invalid values instead of manufacturing
zero. Sources and limits are recorded in the [integration guide](../development/14-mijia-reference-integration.md).

This is a distinct legacy statistics endpoint, not `user/get_user_device_data`,
modern `/miotspec/event/get`, instantaneous power or cumulative property 11.1.
The MIoT property itself has no unit; the community statistics conversion must
not change existing live scalar mapping or other models.

## Decision

Implement an explicit foreground native read in stages, starting with the typed
protocol. Require the exact model and pinned specification
`urn:miot-spec-v2:device:outlet:0000A002:cuco-v3:2` and a mainland legacy session.
The endpoint/key/type/limit are internal constants, not generic RPC parameters.
Use existing protected identity, signing, encrypted response checking and the
operation's cancellation/deadline. No saved token crosses service origins.

A caller selects a positive epoch-second interval of at most 32 days ending no
later than request admission. Send at most one request with limit 31, without
retry, relogin, pagination, polling or background work. Reject invalid windows
before transport. Keep every admitted finite, nonnegative numeric record and its
provider-reported UTC timestamp; convert only this exact profile by 0.01 to kWh.
Reject malformed envelopes and excess records. Count malformed/out-of-window
records, retain valid neighbors, and remove all conflicting or duplicate timestamp
records rather than guessing which is authoritative. Optional returned device/key
correlation fields must match if supplied. Never export provider identity or raw
record content. Empty/malformed records never become zero readings.

Return sanitized records, requested interval, receipt time, rejected-record count
and whether the provider reached the fixed limit. Completeness is always unproven,
even below the limit. Do not infer the provider's daily timezone, missing dates,
retention, a complete month total or an equivalent current observation. A later
explicit read may show a corrected lower value; do not silently retain a higher
historical value as though it were new evidence.

## Layering and compatibility

The typed protocol alone does not add a Core capability or an App feature. A
subsequent Core history contract must use local device/component/property IDs,
policy and exact protected adapter evidence. Native and App adoption must preserve
this distinct historical meaning, partialness, lifecycle and timezone limits.
Do not overload a current-state snapshot, execute history as an action or create a
synthetic vendor device. Ordinary JSON/CLI/MCP, native API 1.32, persisted formats,
grants and current-state mappings remain unchanged in the protocol phase.

## Acceptance

Use fictional encrypted transport replies and a local TLS fixture. Cover exact
model/specification/region admission, immutable wire parameters, both numeric
zero and scaled nonzero values, number/string timestamps, malformed and ambiguous
records, duplicates, limits, provider auth/errors, cancellation/deadline/drop and
one-send behavior. No provider account or appliance is needed. Real statistics
and calendar semantics require separate acceptance; no such acceptance is claimed.
