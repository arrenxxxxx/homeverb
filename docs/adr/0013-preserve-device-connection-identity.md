# ADR-0013: Preserve established connection identities during rediscovery

Status: accepted for the unreleased Core MVP.

## Context

Write grants, preferences, cached observations, and execution history refer to stable local device IDs. Discovery previously overwrote a stored descriptor when an adapter returned the same ID, even if its account, adapter, or identity reference changed. Schema validity alone cannot establish that the result still represents the same appliance.

## Decision

An existing device ID retains its complete set of connection bindings: connection ID, adapter, transport, account ID, and identity reference. Rediscovery may reorder connections and update labels, capabilities, and other non-identity metadata. Adding, removing, renaming, or replacing bindings returns `DEVICE_IDENTITY_CONFLICT`; ordinary discovery never transfers existing grants to a new binding.

Core checks the entire adapter response against stored identities before saving any member. SQLite repeats each identity comparison inside its immediate write transaction, preventing two store instances from overwriting a binding established by the other. `Store::save_device` implementations must provide this transactional guarantee. The shared domain check is `Device::validate_rediscovery`.

This does not make an entire discovery response transactional. A concurrent conflict or storage failure detected during saving can leave earlier validated members saved. No transaction spans adapter I/O, and earlier adapters are not rolled back.

## Compatibility and limits

JSON fields, SQLite schema, and the native ABI remain unchanged. The stricter update behavior is intentional for this unreleased API. Existing Mock identities continue to match; corrupt identity metadata is not silently repaired by rediscovery. Labels and capabilities can still be repaired while the established bindings match.

Identity references must be stable local references, not changing IP addresses or access tokens. Temporary unreachability does not remove a binding. Account reauthentication retains the local account reference; replacing an account or adding a route needs a separately reviewed confirmation flow, which is not implemented. Do not work around a conflict by copying grants to another device ID.

Matching references do not prove physical identity. Adapters still need model-specific evidence for initial binding, deduplication, and private reference resolution. The guard is not protection against direct database tampering or malicious in-process code.

Regression tests cover changed bindings, adapter takeover, whole-response preflight, allowed metadata updates, cross-instance storage races, preserved preferences/cache/history, and actual CLI refusal after an injected stored-identity conflict.
