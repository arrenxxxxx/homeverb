# ADR-0061: Expose trusted native connection-link confirmation

Status: accepted for the existing additional-connection scope; adapters still
require independent protected route evidence before opting in.

## Decision

Native API 1.20 adds three separate C functions to prepare, confirm and discard
the typed connection-link operation from ADR-0052. They are trusted controller
entry points, not ordinary JSON, CLI or MCP methods. The preparation metadata
selects an exact device, anchor connection and proposed public Connection. No
endpoint, provider identifier, key, permission or confirmation boolean is accepted.
The adapter must validate its protected identity/model/route evidence locally;
caller metadata, discovery results and user consent cannot substitute for it.
Production Mock and Meiju retain their default refusal.

Preparation performs no networking or persistence. Retain the immutable Core
preview in a session-owned registry with at most eight entries, globally unique
process-local IDs and the existing 60-second monotonic expiry. Return only the
preview ID, descriptor, anchor, proposed connection, existing potential transport
grants and display expiry. Show this exact scope before actual trusted user
consent: adding a route may affect automatic reads and activate existing grants.
It neither creates a grant nor verifies an appliance or observation.

Confirmation consumes the session-owned ID before validating decoded request
metadata or registering work. Core rechecks scope, evidence and device/account
state, then appends the connection transactionally. The result is the stored
descriptor, not an appliance execution receipt. Failed or concurrent confirmation
cannot reuse the preview. Malformed JSON cannot identify a preview reliably;
the controller must discard its locally retained ID. Dismissal affects only an
unconsumed preview; active work uses operation cancellation. Interrupted callbacks
do not prove the local transaction failed: inspect the descriptor before preparing
a new attempt. Never automatically retry confirmation.

Background/close clears the registry and invalidates Core previews. Check the
session generation before inserting a preview so a preparation crossing a
background/resume cycle cannot leave usable consent. IDs from appliance approval,
LAN scanning or another session cannot confirm a link.

## Compatibility and verification

The 33 ordinary methods, existing function signatures, SQLite schema 5, protected
payload 3, cloud routing and appliance-write semantics remain unchanged. Check
API 1.20 before resolving the new symbols. Sixteen C functions plus Android
credential JNI initialization are required by AAR export inspection. No production
adapter is enabled and this bridge alone cannot publish a Meiju LAN route.

Use the existing fictional opted-in adapter with real Core/SQLite to test schema
parity, exact scope, grants, capacity, expiry, ID/session isolation, consuming
failure, metadata/evidence changes, concurrent confirmation and lifecycle races.
Test the actual Linux C exports for metadata, ordinary-dispatch and production
adapter refusal. Run native regressions and Android archive/export checks. No real
account, appliance, Flutter worktree or physical phone is accessed.
