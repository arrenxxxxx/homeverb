# ADR-0017: Execute explicit device batches within one operation

Status: accepted for the unreleased Core MVP.

## Decision

A BatchCommand contains schema version 1.0 and 1-32 complete Commands. Each device and caller request ID appears only once. The caller supplies exact device IDs; room membership never expands a batch or grants authority. Composite commands remain available within one device item.

Core preflights every command, its independent-field steps, permitted routes, and existing request records before starting a write. Caller and derived child IDs must not collide anywhere in the batch. Preflight failure returns a query error and sends nothing. This is a preflight guarantee, not a transaction: metadata, locks, or storage may change afterward. Every item still follows the existing execution validation, lock, journal, permission, and replay checks.

Execute groups of at most four commands concurrently using borrowed futures. No task detaches from the call. All groups inherit the original host Operation, including preflight and time spent waiting behind earlier groups. Each command's own budget may further limit its execution window. One device's failure does not cancel independent devices. Host cancellation or expiry prevents further sends; every remaining item still returns its own execution outcome. Known results survive cancellation, and physical changes are never compensated.

BatchExecution contains ordered per-device Execution items, start/end timestamps, and an aggregate status. Identical item statuses produce that status; any mixture produces `partial`. Nested composite receipts retain their own evidence. A batch has no fabricated device ID or shared verification evidence.

Aggregate timestamps describe this call. Replayed items keep their original timestamps and evidence; a new batch timestamp does not make an older observation fresh.

## Persistence and recovery

Existing per-command journals and request-ID protection remain authoritative. There is no batch ID, durable batch job, or aggregate receipt table. Inspect the supplied command IDs after interruption. Repeating an identical recorded command returns its saved receipt; an item that never reached execution before process termination may have no record and can execute if explicitly submitted later. A missing receipt does not authorize automatic resubmission. Do not automatically retry the batch or assign replacement IDs.

Preflight checks are repeated on an explicit batch invocation, so changed permissions, retired receipts, or invalid metadata can reject the whole invocation. Use exact receipt lookup to inspect prior outcomes independently of current write admission. Existing cross-process locking and request collision checks remain the final authority during execution.

## Hosts and compatibility

CLI adds `batch --file <path>` and supports `--dry-run`. The file supplies each command's route, wait condition, budget, and request ID; global route/wait/request-ID write overrides are rejected. The global timeout is the total batch budget. Native requests add `plan_batch` and `execute_batch` under the existing foreground/session lifetime. MCP remains read-only.

Existing single-command JSON, C ABI functions, and SQLite schema are unchanged. New BatchCommand and BatchExecution schemas describe the additive payloads. Older native hosts reject the new methods. CLI returns exit 13 for `partial`, the existing per-command code for a uniform failure, 11 for uniform uncertainty, and 12 when uniform acceptance does not satisfy every requested wait condition. Successful batch execution returns BatchExecution directly; admission errors use the existing query error envelope.

## Verification

Exercise whole-list admission, cross-item composite-ID collisions, per-device grants, bounded concurrency, input-order results, uniform/mixed outcomes, shared deadlines, cancellation, independent failures, request replay, and CLI/native behavior. Use fictional adapters and temporary profiles. Real-device and mobile support still require their separate admission gates.
