# Local persistence and concurrency

## Data boundaries

| Area | Stores | Does not store |
|---|---|---|
| Configuration | Profiles, route defaults, UI preferences | Passwords or tokens |
| SQLite | Device descriptors, local rooms/preferences, latest snapshots, bounded detailed receipts, retired request IDs | Plaintext credentials or continuous device history |
| System secret store | Account sessions, refresh tokens, pairing keys | Ordinary diagnostic logs |
| Diagnostic logs | Redacted errors, request IDs, durations | Full vendor responses or Authorization headers |

Hosts inject paths following platform application-directory conventions. Do not hardcode a user's home directory. Desktop CLI/MCP may share storage for the same explicit user/profile. Mobile sandboxes remain independent; there is no automatic sync.

## Implemented schema 2

```text
devices(id, descriptor)
snapshots(device, component, snapshot)
rooms(id, name, name_key, position)
device_preferences(device_id, alias, room_id, favorite, position)
executions(request_id, command, result, finalized)
retired_requests(request_id)
```

The SQLite `user_version` records the schema version. Each profile has its own directory. Device descriptors include components, connections, and capabilities; aliases overlay the descriptor at read time. Rediscovery preserves preferences. Deleting a room sets its assignments to null without deleting devices or credentials. Card reads join devices and preferences in one consistent query. See [ADR-0009](adr/0009-local-organization-and-retention.md).

`Store::save_device` compares an existing descriptor's device ID and complete connection bindings inside the same immediate transaction as the update. The domain check ignores connection order and non-identity metadata. A conflict rolls back that device update and leaves its preferences, cache, and receipts intact. Core preflights the entire adapter response, but concurrent conflicts or later storage failures can still leave earlier response members saved; there is no whole-discovery transaction. See [ADR-0013](adr/0013-preserve-device-connection-identity.md). No database migration is required.

Batch room assignments resolve every selected device and destination inside one immediate transaction, then update only `room_id`. Any failure rolls back all selected devices. Responses follow request order without changing stored card order. No additional table or schema upgrade is needed.

Account records and persisted credential references remain pending. The Linux system-store library is implemented separately below. JSON columns are not permission to hide secrets in the database. Export using explicit allowlists. Real device identities are sensitive and do not belong in default error messages.

## Native credential storage

[SecretStore](../crates/core/src/secrets.rs) defines asynchronous `get`, `put`, and `delete` with a shared Operation. `get` returns `Option<SecretBytes>`; `put` replaces an exact reference; deleting an absent reference succeeds. CredentialRef serializes a random UUID, while SecretBytes has redacted Debug, no serialization, and a zeroizing owned buffer. This does not erase copies made by callers, upstream libraries, or the operating system.

[SystemSecretStore](../crates/secret-store/src/lib.rs) accepts a trusted random profile UUID. Construction has no I/O. On Linux, each operation opens an encrypted DH Secret Service session on the user's local D-Bus, selects the default unlocked collection, and searches only the application/profile/credential attributes. Generic labels contain no account information. Locked or duplicate matches fail closed. There is no collection creation, fallback search, plaintext file, or automatic Unlock call. Other operating systems use the unavailable provider. See [ADR-0011](adr/0011-system-secret-store.md) and [dependency evidence R10](references.md).

Operations share the caller's deadline and cancellation. Create/delete may require a service-managed prompt during the explicit operation; HomeVerb does not start a separate unlock flow. An unconfirmed mutation returns `SECRET_STORE_WRITE_UNKNOWN`; it is never automatically repeated. The connection is closed after the operation, with at most 100 ms of additional cleanup waiting before handles are dropped. Tokio reclaims cancelled reader tasks asynchronously. Cleanup does not replace a confirmed mutation result or undo it.

The local synthetic test verifies create/read/reopen/replace, profile isolation, deletion/absence, and socket release within a bounded runtime cleanup window on the available GNOME Secret Service. Regular tests do not access a developer's keyring. This is not evidence of real-platform login, locked/prompted service integration, KWallet compatibility, or mobile support.

The future account host must persist its nonsecret profile namespace and credential references before relying on them for recovery. Recreating namespaces on each launch loses access to existing entries. Attribute namespaces prevent accidental mixing; they do not isolate malicious processes running as the same OS user. Account refresh locking and cross-process credential rotation remain unimplemented. No credential bytes are exposed through CLI, MCP, or native JSON requests.

## Concurrency

Writes use short transactions; no database write transaction spans adapter I/O. Device file locks are nonblocking and return `RESOURCE_BUSY` on contention. SQLite lock waits are limited to 250 ms per call. These locks cannot coordinate another phone or the vendor app. Account-refresh locks and rereading the newest session after acquiring one remain real-authentication requirements.

Lock filenames retain the existing lowercase UTF-8 hex encoding for device IDs up to 127 bytes. Longer IDs use `sha256-` followed by the lowercase SHA-256 digest of the complete UTF-8 ID, keeping filenames below 255 bytes without truncation. The prefix cannot collide with legacy hex names. The mapping is deterministic across processes and preserves existing short-ID lock coordination; no SQLite migration is needed. Older binaries could not create the oversized hex filenames for these longer IDs on the verified Linux filesystem.

A `request_id` provides local deduplication and correlation, not device-side idempotency. After a crash, unresolved transmitted writes are shown as unknown, never automatically resent by a recovery queue.

Device batches reuse these individual command journals and device locks. No aggregate batch record or durable queue is created. A returned batch preserves every item's result, but forced termination may leave later items unstarted and unrecorded. Inspect the original command IDs through receipt lookup. Never treat a missing item as automatic permission to rerun the batch; see [ADR-0017](adr/0017-bounded-device-batches.md).

## Retention and migration

The store retains at most 100 finalized detailed receipts plus active records. Retention and the recent-record query use first-insertion order, not completion time. Composite parents and children each occupy a receipt row. Retired IDs remain in `retired_requests` without arguments or outcomes. Reuse returns `REQUEST_RETIRED` before transmission; it does not establish whether the original action succeeded. The ID ledger grows with distinct requests, and SQLite may retain allocated pages. This is a detailed-history limit, not a fixed disk-size guarantee.

Exact receipt lookup reads a retained row by request ID, including active rows outside the recent-history window. It does not acquire a device lock, finalize records, update timestamps, or extend retention. Retired and absent IDs are distinguished without reconstructing old outcomes. Ordinary store startup still performs the existing orphan recovery before a host handles queries.

Device-specific history filters the command's exact device ID in SQLite before limiting the result to 100 rows. The finalized-receipt retention limit remains shared across the profile; a device filter cannot restore already retired details. Active records on other devices cannot hide a matching retained receipt merely by filling the global query's first 100 rows.

Startup recovery finalizes orphaned pending records only after acquiring their device locks. Records owned by a live writer are left active. Recovery preserves their last known outcomes and never sends commands. A failed final journal update can leave an older conservative receipt even when the caller received stronger evidence.

New empty profiles initialize schema 2. Schema 1 upgrades take a consistent, WAL-aware SQLite backup under `backups/schema-1-<uuid>.sqlite3`, check its integrity, and sync it before a transactional migration. Failure preserves the old schema and completed backup. Newer versions and unversioned foreign databases are refused. Existing stores hold shared migration leases; an upgrade requires exclusive access and returns `RESOURCE_BUSY` on conflict. Close older clients before upgrading, including versions that predate migration locks. Backups are retained locally and may contain older detailed history; they contain no credentials in the current schema.

Profile deletion is not an implemented command. A future deletion flow must remove its local records and secret-store entries. Do not promise reliable erasure of OS backups or vendor-cloud data.
