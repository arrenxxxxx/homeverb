# ADR-0031: Reauthorize an explicitly selected pending Meiju account

Status: accepted for native implementation with synthetic account evidence.

## Decision

Extend the protected account connection flow to accept an explicitly selected
pending mainland Meiju account when its protected identity and device bindings
remain intact. An expired saved session cannot pass inventory-based recovery,
but a fresh password login can establish the same provider subject. This operation
replaces credentials rather than confirming the old session as usable.

Before login, retain the profile registration and account leases, inspect the exact
protected reference, and validate its bounded payload and encoded session/subject
association. Retain the original protected bytes throughout login. After successful
login, require an exact subject match and reread the reference. Replace only if its
bytes are unchanged. Preserve the local account, all protected device bindings,
label, organization and existing grants. Never confirm the old credentials merely
to allow replacement. The normal pending/write/stored sequence governs the new
session; uncertain persistence leaves pending and returns its existing error.

This path requires an explicit existing account ID and fresh protected input.
Connection without an account ID still refuses pending accounts before login;
it cannot resolve, skip or overwrite them automatically. Missing, malformed,
foreign or internally inconsistent protected material remains unrecoverable through
login because it cannot establish the original identity and bindings. A changed
subject or bytes during login refuses replacement. Removing accounts remain blocked.

Successful persistence triggers ordinary bounded inventory and publication. Failed
or partial inventory is distinct from authentication failure and preserves existing
cards. No device controls, automatic login, retry, credential deletion, identity
merge or new permission follows. Inventory-based `recover_account` remains an
explicit alternative that requires no fresh password and never initiates login.

## Compatibility and verification

API 1.8.0 extends the existing `homeverb_connect_account` behavior. Its protected
buffers, request/response shapes, seven C symbols, 30 ordinary methods, SQLite
schema 4 and protected payload version 1 remain unchanged. Flutter checks the API
version before offering reauthorization for a pending account; older libraries
refuse it. There is no general JSON credential method.

Synthetic host tests cover expired pending sessions, same-subject replacement,
stable IDs/preferences/grants, wrong identity, rejected login, missing/corrupt
material, concurrent protected-byte changes, uncertain writes, metadata failure,
partial retrieval and cancellation/deadline/background/close/dropped futures.
Actual C ABI tests cover protected-input admission and clearing, not live login.
Real reauthorization identity stability, mobile secure stores and physical behavior
remain unverified. No owner account is used or changed by this phase.
