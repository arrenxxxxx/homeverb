# ADR-0060: Retain the last complete account inventory time

Status: accepted for the platform-details requirement; physical acceptance remains separate.

## Decision

Native API 1.19 adds nullable `last_complete_inventory_at` to account summaries.
It is the local UTC wall-clock time recorded after a complete provider inventory
has passed identity validation and its protected bindings have been saved. An
empty complete inventory counts. Login alone, partial/failed retrieval, local
inspection, state reads and LAN scans do not update it. Older accounts start with
null; never infer a historical time from file modification times or migration.

This is historical retrieval metadata, not device observation time, online status,
current authorization or proof that Core published every card. Later descriptor
admission/publication can still fail; callers must inspect the retrieval result.
Keep `authorization: "not_checked"`. Device clock changes can move the timestamp
backward or put an older record in the future; display it as device-clock evidence
without deriving freshness, validity, permission or automatic refresh behavior.

SQLite schema 5 adds one nullable account column. Store this nonsecret metadata
under the account lease after protected persistence, without rewriting unchanged
credentials. Recording it requires `stored` state and cannot recreate a removed
account. A metadata failure returns the existing storage error; it does not roll
back a completed protected save or turn stored credentials into pending ones.
Cancellation before the metadata operation leaves the old timestamp; termination
after its synchronous commit may leave a new timestamp without a caller response.

Account list/details remain local and require no secret-store access. Rename,
reauthorization and cache/history clearing preserve the historical timestamp.
Successful explicit recovery may replace it after complete retrieval. Removing
the account deletes it; reconnecting creates new local identity and null history
until complete retrieval. No authority or appliance execution rule changes.

## Compatibility and verification

Upgrade SQLite 1/2/3/4 through the existing exclusive lock and verified backup.
Older binaries reject schema 5; close them before migration. Protected payload 3,
session encoding, the 33 methods and C/JNI exports stay unchanged. New clients
negotiate API 1.19 before requiring the field; null means no recorded evidence.

Test migration/backup and rollback, malformed stored timestamps, complete/empty/
partial/failed retrieval, unchanged secret writes, pending recovery, rejection,
reopening, lease/lifecycle boundaries, local reads without secrets and removal.
Validate native positive/negative examples and both locale catalogs. Use fictional
accounts only; neither this implementation nor its timestamp certifies real support.
