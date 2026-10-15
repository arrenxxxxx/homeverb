# Local persistence and concurrency

## Data boundaries

| Area | Stores | Does not store |
|---|---|---|
| Configuration | Profiles, route defaults, UI preferences | Passwords or tokens |
| SQLite | Random profile/account/credential references, device descriptors, local rooms/preferences, latest snapshots, bounded detailed receipts, retired request IDs | Plaintext credentials, provider account identifiers, or continuous device history |
| System secret store | Account sessions, refresh tokens, pairing keys, private provider identity/device bindings | Ordinary diagnostic logs |
| Diagnostic logs | Redacted errors, request IDs, durations | Full vendor responses or Authorization headers |

Hosts inject paths following platform application-directory conventions. Do not hardcode a user's home directory. Desktop CLI/MCP may share storage for the same explicit user/profile. Mobile sandboxes remain independent; there is no automatic sync.

## Implemented schema 4

```text
devices(id, descriptor)
snapshots(device, component, snapshot)
rooms(id, name, name_key, position)
device_preferences(device_id, alias, room_id, favorite, position)
executions(request_id, command, result, finalized)
retired_requests(request_id)
profile_namespace(singleton, namespace)
accounts(id, provider, region, label, credential_ref, credential_state)
account_removals(id, record)
```

The SQLite `user_version` records the schema version. Each profile has its own directory. Device descriptors include components, connections, and capabilities; aliases overlay the descriptor at read time. Rediscovery preserves preferences. Deleting a room sets its assignments to null without deleting devices or credentials. Card reads join devices and preferences in one consistent query. See [ADR-0009](adr/0009-local-organization-and-retention.md).

`Store::save_device` compares an existing descriptor's device ID and complete connection bindings inside the same immediate transaction as the update. The domain check ignores connection order and non-identity metadata. A conflict rolls back that device update and leaves its preferences, cache, and receipts intact. Core preflights the entire adapter response, but concurrent conflicts or later storage failures can still leave earlier response members saved; there is no whole-discovery transaction. See [ADR-0013](adr/0013-preserve-device-connection-identity.md). No database migration is required.

Batch room assignments resolve every selected device and destination inside one immediate transaction, then update only `room_id`. Any failure rolls back all selected devices. Responses follow request order without changing stored card order. No additional table or schema upgrade is needed.

Account records contain only local references, provider/region, a local label, and `pending`, `stored` or `removing` credential state. The profile namespace and account/credential references are random UUIDs persisted before credential writes. Account ID, provider, region, and credential reference cannot be rebound; rename changes only the label. Meiju's internal protected payload now stores exact subject matching and appliance-to-local-ID bindings; model-scoped descriptor publication is implemented; real reauthorization identity acceptance remains pending. JSON columns are not permission to hide secrets in the database. Export using explicit allowlists. See [ADR-0021](adr/0021-persistent-account-vault.md).

## Native credential storage

[SecretStore](../crates/core/src/secrets.rs) defines asynchronous `get`, `put`, and `delete` with a shared Operation. `get` returns `Option<SecretBytes>`; `put` replaces an exact reference; deleting an absent reference succeeds. CredentialRef serializes a random UUID, while SecretBytes has redacted Debug, no serialization, and a zeroizing owned buffer. This does not erase copies made by callers, upstream libraries, or the operating system.

[SystemSecretStore](../crates/secret-store/src/lib.rs) accepts a trusted random profile UUID. Construction has no I/O. On Linux, each operation opens an encrypted DH Secret Service session on the user's local D-Bus, selects the default unlocked collection, and searches only the application/profile/credential attributes. Generic labels contain no account information. Locked or duplicate matches fail closed. There is no collection creation, fallback search, plaintext file, or automatic Unlock call. Other operating systems use the unavailable provider. See [ADR-0011](adr/0011-system-secret-store.md) and [dependency evidence R10](references.md).

Operations share the caller's deadline and cancellation. Create/delete may require a service-managed prompt during the explicit operation; HomeVerb does not start a separate unlock flow. An unconfirmed mutation returns `SECRET_STORE_WRITE_UNKNOWN`; it is never automatically repeated. The connection is closed after the operation, with at most 100 ms of additional cleanup waiting before handles are dropped. Tokio reclaims cancelled reader tasks asynchronously. Cleanup does not replace a confirmed mutation result or undo it.

The local synthetic test verifies create/read/reopen/replace, profile isolation, deletion/absence, and socket release within a bounded runtime cleanup window on the available GNOME Secret Service. Regular tests do not access a developer's keyring. This is not evidence of real-platform login, locked/prompted service integration, KWallet compatibility, or mobile support.

`homeverb_host::open_accounts` composes the persisted profile namespace, SQLite account records and system secret store without accessing the keyring. Core's [AccountVault](../crates/core/src/accounts.rs) creates pending references or leases existing accounts; neither authenticates a provider. Attribute namespaces prevent accidental mixing; they do not isolate malicious processes running as the same OS user. No credential bytes are exposed through CLI, MCP, or native JSON requests.

Hold the account lease across session read, any future provider refresh, and replacement. `store_session` persists `pending` before calling the secure store, then records `stored` only after confirmed success. An interruption before finalization leaves pending and ordinary reads return `ACCOUNT_SESSION_UNCONFIRMED`. If the initial metadata update fails, no secret write starts. `stored` proves local persistence only; the adapter must still check expiry, provider identity and validity. A missing referenced secret returns `AUTH_REQUIRED`.

Trusted recovery uses `inspect_session`, validates those exact bytes with the provider while retaining the lease, then calls `confirm_session`. Confirmation rereads and compares the bytes before marking stored, without repeating the secret mutation. A mismatch or absent entry remains unconfirmed. There is no automatic recovery, refresh or credential deletion. Explicit removal is described below. Native account methods include protected connection, explicit refresh, local list/details and rename. Rename uses the account lease and changes only the label, without secret access or a credential-state transition. Native Meiju `recover_account` implements explicit pending-session validation and exact-byte confirmation under [ADR-0030](adr/0030-explicit-meiju-account-recovery.md), followed by ordinary binding/publication. Complete inventory is required before confirming pending material; no original secret mutation is repeated. Explicit same-account reauthorization under [ADR-0031](adr/0031-pending-account-reauthorization.md) instead replaces a pending session after fresh login, exact subject matching and unchanged-byte checks, preserving intact bindings without confirming the old session. Missing/corrupt protected identity remains refused. Actual provider rotation and mobile secure-store acceptance remain pending.

The [Meiju private account layer](../crates/midea/src/account.rs) stores its versioned session and identity map together under the leased credential reference. A complete payload is bound to the immutable local account/provider/region/reference. New appliance IDs receive random local device, cloud-connection and identity-reference IDs before descriptor publication. Reauthorization requires the same provider-validated subject; no subject or appliance identifier is written to SQLite. Every changed map uses the same pending/stored transition as session replacement. Uncertain saves return no generated bindings and require explicit recovery; unchanged maps do not rewrite credentials. Missing or malformed protected state cannot be reset through an existing lease. See [ADR-0024](adr/0024-meiju-private-device-bindings.md) for account scope and remaining cross-account identity work.

## Explicit account cleanup

[ADR-0028](adr/0028-local-account-removal.md) coordinates removal under the profile
registration lock and account lease. An immediate transaction checks the reviewed
ID set, refuses cards with other account/provider routes, saves a removal ledger
entry and sets `removing`. Only then may the secret store be inspected/deleted.
Ordinary leases refuse `removing`; it cannot be reactivated by changing credential
state. A later transaction removes the account, cards/preferences/snapshots and
marks the ledger complete. Rooms, detailed receipts and retired request IDs remain,
including active receipts still finalizing. No transaction spans secret-store I/O.

The ledger retains only local account/device IDs, provider/region and completion.
It rejects late descriptor saves referencing that account and answers repeated
removal without secret access. Snapshot saves check device existence and the same
ledger in their write transaction, preventing late reads from recreating cache.
Ledger entries grow with explicit removals and are not erased automatically.

Cleanup errors leave visible intent; explicit retry reads the exact credential
reference before any deletion. An absent secret needs no second delete. Startup
never resumes cleanup. Removal does not revoke vendor grants, affect copies elsewhere
or erase backups/retained receipts. Reconnecting receives new local IDs. See the
[native removal contract](native-ui-contract.md#remove-an-account-from-this-installation).

## Concurrency

Writes use short transactions; no database write transaction spans adapter I/O. Device and account file locks are nonblocking and return `RESOURCE_BUSY` on contention. Account locks use `account-<local-uuid>` and remain held by the lease until drop. The vault rereads metadata only after acquiring that lock, then reads the current secret. SQLite lock waits are limited to 250 ms per call. These locks cannot coordinate another phone or the vendor app. Synchronous storage work is not preemptible; subsequent secure-store work rechecks the original operation budget.

Lock filenames retain the existing lowercase UTF-8 hex encoding for device IDs up to 127 bytes. Longer IDs use `sha256-` followed by the lowercase SHA-256 digest of the complete UTF-8 ID, keeping filenames below 255 bytes without truncation. The prefix cannot collide with legacy hex names. The mapping is deterministic across processes and preserves existing short-ID lock coordination; no SQLite migration is needed. Older binaries could not create the oversized hex filenames for these longer IDs on the verified Linux filesystem.

AccountLock requires Send + Sync for borrowed account-session reads inside the existing Send adapter futures. This changes the unreleased Rust trait bound for custom stores, not process-lock behavior or storage layout. The built-in file guard already meets it. The [bound Meiju read](adr/0025-bound-meiju-core-reads.md) tests verify lease contention during an in-flight request and release after cancellation or expiry.

A `request_id` provides local deduplication and correlation, not device-side idempotency. After a crash, unresolved transmitted writes are shown as unknown, never automatically resent by a recovery queue.

Device batches reuse these individual command journals and device locks. No aggregate batch record or durable queue is created. A returned batch preserves every item's result, but forced termination may leave later items unstarted and unrecorded. Inspect the original command IDs through receipt lookup. Never treat a missing item as automatic permission to rerun the batch; see [ADR-0017](adr/0017-bounded-device-batches.md).

## Explicit cache and history clearing

Under [ADR-0029](adr/0029-local-cache-and-history-clearing.md), cache clearing deletes
snapshots in one immediate transaction. It does not cancel in-flight reads, which
may repopulate cache. History clearing refuses any `finalized=0` row, protecting
active composite parents and their child receipts across store instances. Otherwise,
it inserts all request IDs into `retired_requests` and deletes detailed receipts in
one transaction. Any error rolls back that cleanup; success returns the removed row
count. No cleanup needs a device/account lease, provider request or credential access.
New writes admitted after the transaction can create new history normally.

Cache and history have separate explicit methods; clearing one preserves the other.
Both preserve organization, account records/removal ledgers, credentials and grants.
Retired IDs and backups remain; no disk compaction or secure-erasure claim is made.

## Android credential backend

[ADR-0032](adr/0032-android-native-credential-storage.md) adds a trusted JNI backend using Android Keystore keys and authenticated ciphertext in the app no-backup directory. Account pending-state and exact-byte recovery rules are unchanged. Java calls stay on the operation worker and retain its lease until returning; cancellation after dispatched mutation remains uncertain. OS calls cannot be preempted. No backend is installed through ordinary JSON, and no plaintext fallback exists. See the [integration and acceptance guide](development/08-android-native.md); synthetic JVM success does not verify Android Keystore.

Ordinary account removal reads the credential before deleting it. A missing key or corrupt ciphertext returns a store error, leaving removal pending and retaining metadata. The backend's exact-file deletion capability does not expose a public force-cleanup path. Unreadable protected material must never be treated as absent or erased automatically.

## Retention and migration

The store retains at most 100 finalized detailed receipts plus active records. Retention and the recent-record query use first-insertion order, not completion time. Composite parents and children each occupy a receipt row. Retired IDs remain in `retired_requests` without arguments or outcomes. Reuse returns `REQUEST_RETIRED` before transmission; it does not establish whether the original action succeeded. The ID ledger grows with distinct requests, and SQLite may retain allocated pages. This is a detailed-history limit, not a fixed disk-size guarantee.

Exact receipt lookup reads a retained row by request ID, including active rows outside the recent-history window. It does not acquire a device lock, finalize records, update timestamps, or extend retention. Retired and absent IDs are distinguished without reconstructing old outcomes. Ordinary store startup still performs the existing orphan recovery before a host handles queries.

Device-specific history filters the command's exact device ID in SQLite before limiting the result to 100 rows. The finalized-receipt retention limit remains shared across the profile; a device filter cannot restore already retired details. Active records on other devices cannot hide a matching retained receipt merely by filling the global query's first 100 rows.

Startup recovery finalizes orphaned pending records only after acquiring their device locks. Records owned by a live writer are left active. Recovery preserves their last known outcomes and never sends commands. A failed final journal update can leave an older conservative receipt even when the caller received stronger evidence.

New empty profiles initialize schema 4. Schema 1, 2 and 3 upgrades take a consistent, WAL-aware SQLite backup under `backups/schema-<previous-version>-<uuid>.sqlite3`, check its integrity, and sync it before a transactional migration. Failure preserves the old schema and completed backup. Newer versions and unversioned foreign databases are refused. Existing stores hold shared migration leases; an upgrade requires exclusive access and returns `RESOURCE_BUSY` on conflict. Close older clients before upgrading, including versions that predate migration locks. Schema-3 binaries reject schema 4; do not alternate older and newer binaries on an upgraded profile. Backups are retained locally and may contain older detailed history; they contain no credentials in the current schema.

Profile deletion is not an implemented command. A future deletion flow must remove its local records and secret-store entries. Do not promise reliable erasure of OS backups or vendor-cloud data.
