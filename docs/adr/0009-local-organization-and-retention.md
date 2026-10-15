# ADR-0009: Keep local organization separate and retire receipts without replay

Status: accepted for the requested Core increment. Date: October 14, 2026.

## Local organization

Rooms, display aliases, favorites, and card order belong to one local profile. They do not change device identity, connections, capabilities, credentials, or grants. Deleting a room unassigns its devices. Discovery preserves user preferences. Room names are trimmed, limited to 80 characters, and unique after Unicode lowercasing; user-defined text is not translated. Reserved virtual filters such as All and Unassigned have no persisted room IDs.

Expose separate Room and DevicePreferences contracts instead of changing the existing Device envelope. Preference updates replace alias/room/favorite together; ordering uses an exact permutation of the current local IDs. Invalid, duplicate, missing, or foreign IDs fail before mutation. Only trusted local application hosts expose these mutations; MCP stays read-only.

Batch room assignment accepts 1-256 distinct exact device IDs and an explicit nullable destination room ID. It changes only room membership in one transaction, preserving each device's alias, favorite, position, identity, and grants. All references must exist before any update; a storage failure rolls back the whole batch. Results follow the requested ID order. Null means Unassigned; an omitted destination is invalid. This is a local metadata transaction, not a batch of physical appliance writes.

Device-specific recent operations filter the retained journal before applying the 100-row query limit. They do not create a separate retention policy or recover retired details. Parent and child receipts remain available as stored. Exact IDs are used in Core/native requests; CLI may resolve a history selector before querying. Unknown devices fail rather than returning a misleading empty history.

These add native methods and a RoomAssignment contract without changing existing method payloads or SQLite schema 2. Rust Store implementers must provide the new atomic assignment and filtered-history methods; this unreleased trait extension is coordinated with all in-repository implementations.

## Upgrade and retention

SQLite schema 2 adds organization tables, execution finalization, and a retired-request ledger. Upgrades require exclusive profile access, create a consistent SQLite backup, and apply changes in one transaction. Existing stores hold shared migration locks. Newer schemas are rejected. A failed upgrade leaves the previous schema and its backup available.

Keep at most 100 finalized detailed execution receipts, plus active writes, in first-insertion order. Parent and child receipts each occupy a row. When a receipt is retired, retain its request ID without arguments, observations, or result details. The compact ID ledger grows with distinct retired requests; this is not a fixed database-size guarantee. Reusing a retired ID returns `REQUEST_RETIRED` before transmission, even when its original outcome is no longer available. Never remove replay protection as a side effect of trimming history.

Active writes are not trimmed. Recovery may finalize an orphaned pending receipt only after acquiring its device lock, without sending anything. Composite parents remain active until their item receipts are collected. Every successful storage mutation uses a short transaction; no database transaction spans adapter I/O.

This adds local contracts and an error code without changing existing command fields. Account secure stores, real authentication, and authorization transfer remain separate work.
