# ADR-0029: Clear cached state and retire history without resetting safeguards

Status: accepted for the requested local-data UI increment.

## Decision

Native API 1.6 adds two explicit foreground methods for the selected local profile:
`clear_cached_state` and `clear_execution_history`. The UI confirms each scope
separately. Neither method accepts device IDs, account IDs, filesystem paths,
permissions, or a combined full-reset flag. MCP remains read-only.

Cache clearing deletes the snapshots present in its SQLite transaction and returns
`removed_snapshots`, counting device/component snapshots rather than individual
properties. Device cards, rooms, preferences, credentials, receipts and grants stay
intact. Subsequent cached reads return `CACHE_MISS` until an explicit live read
saves a snapshot. An already-running read or verification may also repopulate cache;
clearing is not cancellation or a promise that cache remains empty.

History clearing checks for any unfinalized execution in the same immediate
transaction. If one exists, return `RESOURCE_BUSY` without clearing any history.
This also protects completed children of an active composite operation. Otherwise,
copy every retained request ID to `retired_requests`, delete detailed receipts,
and return `removed_receipts`. Parent and child rows count separately. An atomic
rollback preserves both history and the existing ID ledger on any storage error.
Unknown or accepted receipts may be finalized; clearing their details does not
resolve their outcomes. The confirmation must explain that uncertainty is lost
from the detailed history while replay prevention remains.

No cleanup method invokes an adapter, loads credentials, cancels physical work,
or changes an appliance. Cache and history clearing are separate transactions.
Startup never repeats them. Cancellation/deadline checks precede local mutation;
SQLite transactions are synchronous and cannot be interrupted midway. A caller
terminated before receiving the result must reload local data without assuming
rollback. New work can add cache/history after a successful clear.

## Compatibility and verification

Seven C symbols and SQLite schema 4 stay unchanged. The unreleased Rust Store
trait gains two methods; older native clients must check the method catalog.
Full-profile deletion, provider revocation and secure erasure remain unavailable.
Backups, SQLite pages, account-removal ledgers and retired request IDs are retained.

Use fictional profiles to verify cache misses without network fallback, preserved
organization/accounts, composite and uncertain receipt retirement, active-operation
refusal across store instances, rollback after injected deletion errors, and
reopening without replay. Exercise actual C ABI responses, closed-field rejection,
foreground/close/cancellation behavior, and saved examples. No real profile or
appliance is changed for this phase.
