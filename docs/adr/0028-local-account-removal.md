# ADR-0028: Remove local account authorization with durable cleanup intent

Status: accepted for native implementation; real account deletion is not exercised.

## Decision

Native API 1.5 adds `remove_account(account_id, expected_device_ids)`. The caller
first obtains local details and confirms removal on this installation. The exact
device-ID set is checked again in one SQLite transaction before changing anything.
The operation deletes the referenced local secret, account, exclusively linked
device cards, preferences and snapshots. Rooms, execution receipts and retired
request IDs remain. Provider revocation and copies on other installations are not
affected. Shared devices with routes outside this account/provider are refused
before mutation; safe route detachment remains a separate identity decision.

SQLite schema 4 adds `removing` credential state and an account-removal ledger.
The ledger retains only local account/device IDs, provider/region and completion.
Account registration and account leases coordinate the operation with Meiju
authentication, discovery and actual network use. Persist intent and `removing`
before touching the secret store. Ordinary leases refuse this state. Do not undo
intent or reactivate credentials automatically after a failure.

SQLite and the secret service cannot form one transaction. After intent, inspect
the exact credential reference, delete it once only when present, then atomically
remove local metadata and mark the ledger complete. Any interruption/error before
finalization leaves a visible pending cleanup, with its specific error when the
process can return one. An explicit retry first inspects the exact reference; an
absent secret is not deleted again. Startup performs no cleanup. Forced termination
does not promise a callback. Repeating completed removal with the same scope returns
the persisted completion without secret access. Changed scope is refused.

Both descriptor and snapshot saves consult the removal ledger inside their write
transaction. Snapshot saves additionally require the device to exist. Thus an old
discovery/read finishing after intent cannot restore a removed card or cache.
Completed ledger entries are retained for this admission guard. Already recorded
or still-finalizing execution outcomes remain independent of device metadata;
removal neither replays nor rolls back a command. Meiju holds an account lease
through each network operation, so concurrent removal returns `RESOURCE_BUSY`.
Locally buffered UI results may still arrive; discard obsolete presentation callbacks.

The result uses the query envelope with data containing `account_id`, `device_ids`,
`status: removed | pending`, `remote_revocation: not_attempted` and an inner `error`.
`pending` always carries an error and must not be displayed as successful removal.
Failures before durable intent use the outer query error. Account lists/details
expose pending cleanup as `credential_state: removing`, without credentials.
Reconnect creates new local IDs only after cleanup completes; old grants do not
transfer to those IDs. Pending cleanup may block cross-account identity validation
until resolved; no account is silently skipped to bypass ambiguity checks.

## Compatibility and verification

Schema 1/2/3 upgrades use the existing exclusive migration lock and verified backup.
Schema-3 clients reject schema 4. Close older clients first and update native/UI
code together: strict old account readers do not accept `removing`. Existing
commands, execution outcomes and seven C symbols retain their meaning. The
unreleased Rust AccountStore trait gains local cleanup methods.

Use synthetic credentials and SQLite to verify scope changes, shared-route refusal,
account/provider isolation, unavailable storage, uncertain deletion, cancellation,
restart recovery, transaction rollback, late cache/discovery rejection and retained
receipts. The ordinary ABI must refuse invalid/forged requests, gate background
work and report pending cleanup without a keyring. Do not delete the owner's live
account or operate appliances during implementation.
