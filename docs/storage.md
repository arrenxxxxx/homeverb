# Local persistence and concurrency

## Data boundaries

| Area | Stores | Does not store |
|---|---|---|
| Configuration | Profiles, route defaults, UI preferences | Passwords or tokens |
| SQLite | Device/component/connection references, capability cache, snapshots, execution phases | Plaintext credentials or unbounded history |
| System secret store | Account sessions, refresh tokens, pairing keys | Ordinary diagnostic logs |
| Diagnostic logs | Redacted errors, request IDs, durations | Full vendor responses or Authorization headers |

Hosts inject paths following platform application-directory conventions. Do not hardcode a user's home directory. Desktop CLI/MCP may share storage for the same explicit user/profile. Mobile sandboxes remain independent; there is no automatic sync.

## Proposed tables

```text
profiles(id, settings_json)
accounts(id, provider, region, credential_ref)
devices(id, alias, brand, model)
connections(id, device_id, adapter_id, transport, account_id, identity_ref)
capability_snapshots(device_id, component_id, revision, document_json, captured_at)
observations(device_id, component_id, payload_json, received_at, source)
executions(request_id, device_id, phase, status, started_at, completed_at, redacted_result_json)
schema_migrations(version, applied_at)
```

JSON columns are not permission to hide secrets in the database. Export using explicit allowlists. Real device identities are sensitive and do not belong in default error messages.

## Concurrency

Use short local transactions and write coordination. Never hold a database write transaction during network I/O. Refresh locks work across local processes, with a reread of the newest session after acquiring the lock. Device write locks use bounded waits. They cannot coordinate another phone or the vendor app.

A `request_id` provides local deduplication and correlation, not device-side idempotency. After a crash, unresolved transmitted writes are shown as unknown, never automatically resent by a recovery queue.

## Retention and migration

The proposed initial default is the most recent 100 redacted operations and the latest device snapshot, not continuous history. This is a product choice, not measured capacity. Users can clear it. Migrations take a local backup and run transactionally; failures preserve the old state and report an error. Backups exclude credentials by default.

Deleting a profile removes its local records and secret-store entries. Do not promise reliable erasure of OS backups or vendor-cloud data.
