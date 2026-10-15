# ADR-0023: Decode Meiju cloud state without inventing freshness or capabilities

Status: accepted for offline protocol implementation. Authentication admission remains subject to proposed ADR-0022.

## Decision

Extend the native Meiju read transport with the reviewed `/mjl/v1/device/status/lua/get` endpoint. Support an explicit default query and an air-conditioner indoor-temperature query, each as one bounded request with the exact account/appliance identity. There are no implicit retries, additional queries, controls or downloaded Lua programs.

Decode the code/data response using the pinned community client's AC, E1 and CA field mappings. Normalize explicit on/off values and numeric temperature strings inside the adapter. Air-conditioner target temperature requires both its whole and fractional fields; do not invent a missing fraction. Keep refrigerator and freezer readings on separate components of the same physical appliance. Dishwasher program, work status, door switch and warning codes retain vendor-specific semantics. Do not infer universal program enums, door polarity, ranges, writable controls or model support from these mappings. Unknown/null readings remain absent; malformed recognized fields reject the response without partial state publication.

The inspected state response does not establish a trustworthy observation timestamp or cache freshness. Record local receipt time, leave `observed_at` null, mark every projected observation stale, and include a fixed freshness warning. Ignore undocumented timestamp fields. Such a snapshot can display last returned values but cannot verify a write. Model-specific protocol evidence is required before relaxing this conservative rule.

Raw appliance identity stays in nonserializable native state with redacted Debug. Projection accepts a local device ID only after the future adapter has validated its encrypted identity binding; projection itself does not establish that binding. Core must still admit the corresponding component/property descriptors. Unknown family/query combinations and unsupported explicit temperature units fail before inventing semantics.

## Compatibility and verification

This extends the unreleased Rust ReadEndpoint enum; transport implementers must handle its new variant. Native UI API 1.0.0, C ABI, SQLite schema 3, registered adapters and Core execution semantics remain unchanged. No live transport, account method or physical compatibility is enabled.

Synthetic tests cover all three selected families, exact target routing, missing/null fields, numeric strings, malformed/oversized responses, conflicting response identity, vendor-specific values, conservative freshness, no retries, cancellation/deadlines and redaction. These are protocol tests, not physical-device acceptance or an approval of ADR-0022.
