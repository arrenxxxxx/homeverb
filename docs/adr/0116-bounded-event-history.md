# ADR-0116: Bounded event occurrence history through Core

Status: Implemented through Core, protected adapter, native API 1.44, Flutter and CLI/MCP; real acceptance pending.
Date: October 30, 2026.

## Context

ADR-0115 supplies typed protocol reads for 78 exact event selectors. Numeric
history requires values, numeric property descriptors and distinct timestamps;
none represents event occurrences correctly. Several pinned events belong to
services without any admitted scalar reading. Inventing a numeric state property
or discarding those services would omit requested functionality.

## Decision

Introduce a separate event-history contract using local device/connection IDs and
a target of component, capability and event. The initial projection is explicitly
occurrence_times: only reported timestamps, never arbitrary payloads. Each option
contains the projection, maximum records and maximum window seconds. A descriptor
must declare the selected event; a history-only declaration need not claim live
subscription or a known payload schema. Adapter-owned exact event metadata may
add event-only components on explicit discovery/refresh, retaining identity and
all existing readable components. Never synthesize current readings for them.

Requests select one exact connection, inspected event target/projection and
canonical whole-second UTC start/end. A positive half-open window cannot end in
the future or exceed 32 days. An adapter may narrow the 31-record and 64-option
limits. No provider, remote ID, raw key, URL, UID, limit, cursor, write consent or
principal is caller selectable. Inspection must remain local and return no
credentials; the adapter independently derives support from protected bindings.

Core must validate descriptor/connection/event admission, duplicate options,
projection and limits before I/O, and apply the trusted host's read access.
The adapter must hold its account lease and validate exact protected account,
model/specification/root/membership and selected protocol for the entire request.
One device lock and one shared Operation bound the call. No automatic renewal,
fallback, paging, subscription or retry. Recheck metadata after I/O and reject
changed bindings or late replies. Unsupported adapters return no options and
refuse reads by default.

Results repeat the entire local selection/window, transport and local receipt
time, with sorted reported timestamps, rejected count, limit_reached and
completeness=unproven. Equal timestamps are valid distinct reported rows, without
physical uniqueness/count claims. Valid plus rejected counts must not exceed the
inspected bound; limit_reached must equal whether that bound was reached. Record
times must be within the window, and received_at within this operation. Empty
results do not prove event absence or delivery completeness. No value, event
payload, actor, media URL, provider event identifier or continuation token is
returned. Payload interpretation remains separate unfinished work, particularly
for the 25 argument-bearing pinned events.

Historical results do not populate current state, verify actions, create execution
receipts or persist event logs. Background/close/cancellation/deadline/drop ends
work and suppresses late output. App users explicitly choose a window and fetch;
there is no event polling. Alarm/lock/water reports do not establish current safety.

## Staged compatibility

The initial domain types, event-history schema and fictional fixtures add no
callable Core/adapter/native/App feature. Existing numeric history and public
capability schemas, API 1.43, SQLite 5 and protected payload 3 remain unchanged.
Default Adapter hooks and Core validation follow in a separate verified phase.
Mijia descriptor publication must use exact catalog identities and retain old
saved mappings; native/CLI/MCP/App adoption requires explicit integration and
versioned native negotiation. The 85fefd8 APK remains unchanged.

## Verification

Contract tests cover exact required fields and closed shapes; canonical UTC;
projection, bounds and completeness; empty/partial/full pages; equal timestamps;
and refusal of raw payloads, actors, authority, cursor and remote selector fields.
Core tests must additionally prove cross-field time/count/order/identity checks,
actual descriptor event admission, no cache/journal mutation and lifecycle cleanup.
Adapter tests must cover all exact protected source tuples, including event-only
services, with no networking during inspection and no protocol fallback. No schema
or fictional test establishes provider event delivery, retention, physical-device
or phone acceptance. Full event coverage remains incomplete until these later
stages and reviewed payload/paging/subscription work are resolved.


## Typed Core and protected Mijia implementation

Core now exposes inspect_event_history_options and read_event_history, with
additive default Adapter hooks. A selected event declaration must contain
history.projection=occurrence_times; arbitrary object metadata or a numeric
property does not establish support. Core applies the versioned schema plus
independent descriptor/selection/window/receipt/count/order checks, holds the
existing device lock, and rejects changed metadata and late results. Typed Core
retains the existing current-state read-access boundary; host session/foreground
admission is required when entry points adopt these methods.

MijiaAdapter::with_event_history explicitly enables typed event publication and
reads. Construction performs no I/O; existing native hosts do not opt in yet.
An explicit account refresh adds event capabilities derived from the independent
pinned catalog, including services with no readable properties. Namespace/service
capability IDs and event names retain exact source distinctions. Existing scalar
capabilities, component order, local identity and protected mapping bytes are not
rewritten merely to inspect or read events. Public event declarations do not
include raw payload schemas or claim that payload arguments were decoded.

Complete old and complete enriched component layouts remain valid for ordinary
reads; partial/mixed layouts refuse. Old layouts expose no event options until
explicit refresh. A host reopened without event support can still read existing
scalar fields on an enriched layout but cannot read events. Protected event reads
repeat exact account/device/model/spec/root/listed-membership and protocol checks,
retain the account lease and never fall back or renew credentials. Parent-bearing
logical entries do not acquire event reads. Native API 1.43, stored payloads and
the 85fefd8 APK remain unchanged. This phase still does not complete event payload,
pagination, subscription or physical acceptance requirements.
