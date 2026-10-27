# ADR-0089: Bounded numeric history through Core

Status: contract accepted; Core, adapter and entry-point implementation pending.
Date: October 27, 2026.

## Context

ADR-0088 supplies an exact native daily-energy protocol. Current snapshots cannot
represent provider daily buckets, rejected records, requested windows or unproven
completeness. Treating statistics as current state would manufacture observation
freshness and could wrongly verify device writes. Upper layers must not dispatch
history by brand or provider-specific endpoint/key.

## Decision

Add separate typed Core history inspection and reading. A history target contains
an existing component ID, capability ID and property ID. Only reviewed numeric
historical aggregates are admitted initially, with `aggregation=reported_daily`.
Adapters default to no options and unsupported reads. A local inspection returns
options derived from protected identity/model evidence, never from vendor strings
in the UI. Each option includes target, reported unit, maximum window seconds,
maximum records and an explicitly unknown bucket timezone (`null`). This unit and
aggregate describe the historical series, not the property's current wire value.

Requests use an exact local device and connection ID, the inspected target and
aggregation, and whole-second UTC `start`/`end` timestamps. No URL, key, remote
device ID, provider, user identity, limit or conversion is caller selectable.
Core resolves and validates the complete descriptor and connection, checks the
option against an existing readable number/integer property, and rejects missing,
duplicate, foreign or excessive options before sending a request. This initial
contract allows at most 64 options, a 32-day window and 31 returned records. An
option may narrow those bounds. The interval is half-open `[start, end)`, positive
and no later than operation admission. No implicit route fallback or pagination.

Reading requires an explicit foreground operation and the same trusted session
read access as current-state reads. No new write grant, identity or privilege is
accepted. This admission does not extend to health, media, free-form events or
sensitive identities. Core locks the local device for a read and the adapter
independently checks protected account/model/connection evidence and holds its
existing account lease during the bounded provider request. Changed or removed
bindings refuse. There is no credential refresh, account enumeration or discovery.

Results carry the local target, connection, source transport, inspected unit and
aggregation, original query interval, receipt time, sorted distinct timestamped
nonnegative finite numbers, rejected count and limit status. All returned dates
must lie within the requested interval; receipt time must belong to this operation.
Core validates every field before returning it. Valid plus rejected counts cannot
exceed the inspected bound; `limit_reached` must agree with their sum. Completeness
remains `unproven`; no missing-day zero or complete calendar sum is allowed. A
provider timestamp is a reported bucket label, not a trusted physical observation
or proof of cloud timezone. The supported bucket timezone is explicitly unknown.

History results are not snapshots, never populate the state cache, cannot verify
writes, and do not create execution receipts. They are not persisted in this
phase. Background, close, cancellation or deadline ends the operation and drops
late results. Subsequent user reads may show provider corrections, including
lower values; there is no hidden monotonic-value restoration.

## Compatibility and staged delivery

The domain schema and fixtures are introduced first. Existing capability/device
schemas, 34 ordinary methods, native API 1.32, C exports, SQLite 5 and protected
payloads remain unchanged at the contract phase. Core default adapter methods
will be additive. Host/native/CLI/MCP/App entry points require their own versioned
integration, fixtures and tests before being declared available. Existing adapters
without opt-in keep their current behavior. The exact Mijia energy option must
use the pinned property mapping and ADR-0088 protocol independently of public
metadata. No brand conditional belongs in the App.

## Verification

Schema fixtures cover options, requests and partial results, rejecting unknown
fields, authority/remote identity fields, bad numeric types, values, timestamps,
limits, complete claims and duplicate options. Core runtime tests additionally
must cover cross-field targets, descriptor admission, time/count/order consistency,
changed bindings, cancellation and no snapshot/receipt writes. Mijia adapter tests
must prove protected profile and account admission, lack of networking during
inspection and exact encrypted history retrieval. Entry-point/UI acceptance remains
pending until implemented; protocol tests alone do not satisfy this contract.
