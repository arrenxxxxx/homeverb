# ADR-0035: Gate Meiju requests on protected inventory membership

Status: accepted for offline implementation; physical acceptance remains deferred.

## Decision

Keep historical appliance bindings under ADR-0024, and record membership separately
in each protected binding. `listed` is true or false after a complete inventory;
null means no complete inventory has established membership. Complete retrieval
updates every binding, including an empty result. Partial retrieval may retain new
identity evidence, but never changes existing membership or admits new devices.
Invalid/conflicting inventories change nothing. Provider failure is not an empty list.

Before an account-bound state read or control, require `listed=true` under the same
account lease held through transmission. False returns `MIDEA_DEVICE_NOT_LISTED`;
null returns `MIDEA_INVENTORY_REQUIRED`. Both are non-retryable pre-send refusals.
The user can explicitly refresh the account. Do not log in, enumerate, retry or
remove anything automatically. Membership is evidence of the last complete list,
not a connectivity signal or proof of current provider authorization.

Retain cards, connection identity, preferences, caches, grants and receipts. Cached
reads and local descriptions remain available without credential access. Reappearance
with consistent family/model evidence reuses the same identities. Historical
cross-account conflicts still require explicit reconciliation. Identity-only
`resolve_device` remains available for diagnostics; it is not operational admission.

The protected map is the sole membership authority. Descriptors and prefetched
discovery results do not carry or update membership. A late descriptor publication
cannot revive a target excluded by newer complete retrieval. Static plans continue
to validate stored capabilities and trusted grants without reading credentials;
execution repeats the protected check and can return a failed/not-sent receipt.
This phase exposes actionable errors, not a new cached connection-status field.

## Migration and compatibility

Protected payload version 2 adds nullable `listed` to bindings. Version 1 can be
read without rewriting it; all its bindings start unconfirmed. Any subsequent
explicit protected update writes version 2, preserving subject/session and all IDs.
Only complete retrieval establishes membership; login or partial retrieval alone
does not. Existing recovery/reauthorization paths retain the same checks and pending
write rules. No live profile is migrated as part of development.

Older libraries reject version 2 and must not reset or regenerate its map. New
libraries reject version 1 carrying membership claims, malformed membership, and
unknown versions. Missing membership in version 2 fails closed as unconfirmed.
Native API 1.9.1 adds the refusal behavior and localized guidance; the 30 methods,
seven C functions, SQLite schema 4 and JSON response shapes stay unchanged.

## Verification

Use synthetic accounts and transports to check full/partial/failed inventories,
reappearance, reopening, legacy migration, uncertain storage, historical identity
conflicts, preserved cache/preferences/receipts, and stale descriptor publication.
Verify reads, power and fan controls refuse before transport, with no implicit
refresh or replay. This is an operational admission fix, not physical verification
or a claim that removal from a list proves provider revocation.
