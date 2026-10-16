# ADR-0058: Expose a trusted native Meiju scan preview

Status: accepted for synthetic implementation within the additional-LAN scope.

## Decision

API 1.18.0 adds `homeverb_prepare_lan_scan`, `homeverb_confirm_lan_scan` and
`homeverb_discard_lan_scan` as separate trusted C functions. The 33 ordinary JSON
methods, CLI and MCP remain unchanged. Reuse ADR-0054's selected-device scan;
do not add another collector, endpoint authentication or connection-link flow.

The trusted mobile controller obtains an assigned IPv4 address and prefix from
the platform, handles network permission, and passes them as scalar C arguments
to preparation. The unsigned address is the numeric value of its four octets
(192.168.42.2 is `0xC0A82A02`), independent of the caller's memory byte order.
Metadata contains only schema version, operation ID, timeout and exact local
device/connection IDs. Private address validation does not establish interface
ownership. Platform selection and permission are still required before enabling
these functions in an app; do not accept network values from ordinary JSON tools.

Preparation performs protected local preflight and returns an immutable preview
with local device/connection IDs, the effective alias, selected address/prefix,
remaining lifetime and a process-local scan ID. Show that scope to the user.
Store at most eight previews per Session, prune expired entries before inserting,
and use ADR-0054's 60-second monotonic expiry. No scan occurs during preparation.

After actual user consent, the controller rechecks that the selected interface
and permission still apply and calls confirmation with only version, operation
ID, timeout and scan ID. No replacement device, network or confirmation boolean
is accepted. A decoded confirmation consumes a matching ID before admission,
including refusal. Malformed input that cannot be decoded does not consume it.
IDs belong to one handle and must not be persisted or logged. Restart discards
all previews. Scan and appliance-approval IDs share a process-wide allocator
while their registries remain separate, preventing consent-kind collisions. These
functions are a trusted application boundary, not an in-process sandbox.

Confirmation reuses the typed scan and its leases, descriptor/binding checks,
deadline and cancellation. Return only local IDs and one of `not_observed`,
`incomplete`, `ambiguous` or `unique_candidate`, with `authenticated: false` and
`route_added: false`. Drop all candidate endpoints and provider identifiers before
serialization. A unique unauthenticated hint does not authorize a key lookup,
TCP connection, state read, connection publication or physical-device claim.
The native endpoint is not retained for a later operation.

Dismissal removes a pending preview without affecting an active scan. Existing
operation cancellation handles active work. Background/close clears all previews
and cancels active operations; generation checks prevent a prepare/background/
foreground race from publishing stale consent. Missing, foreign, consumed or
pruned IDs return `LAN_SCAN_INVALID`; retained expired IDs return
`LAN_SCAN_EXPIRED`. Neither failure automatically retries or falls back to cloud.

## Compatibility and verification

Require API 1.18 before resolving the three additional symbols. There are 13 C
functions plus Android credential JNI initialization. Storage, grants, cloud
behavior and command outcomes are unchanged. Publish closed metadata/result
schemas and fictional examples separately from ordinary native requests.

Use injected fictional discovery to test Session success, scope, privacy,
one-use/cross-handle IDs, capacity, expiry, changed metadata, admission failures,
cancellation and lifecycle cleanup. Exercise C parsing/admission on isolated
profiles and inspect all Android AAR exports. Existing collector loopback tests
cover bounded UDP behavior. No real account, LAN scan or appliance access is
needed. Mobile interface/permission selection, Flutter consent, authenticated
endpoint/model admission, linked routes and physical acceptance remain pending.
