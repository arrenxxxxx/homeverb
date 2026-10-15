# ADR-0033: Remove confirmed local accounts without decrypting credentials

Status: accepted for implementation; no owner account is removed by this work.

## Problem

Core reads a secret before deleting it under ADR-0028. Android key loss or corrupt
ciphertext can prevent that read even when the native backend can safely delete
the exact file. The account then remains in pending cleanup indefinitely. Reading
private bytes is unnecessary to authorize an already confirmed local removal.

## Decision

Keep the existing explicit `remove_account` request and exact reviewed device set.
Retain profile registration and account leases, provider/region validation, shared
route refusal and durable removal intent. After intent, call `SecretStore.delete`
directly with the account's persisted reference. Never obtain or parse credential
bytes for removal. This replaces the Core pre-read in ADR-0028.

The backend must identify the exact profile/reference without decrypting its
contents, reject locked/unavailable/ambiguous storage, and treat a confirmed absent
entry as successful deletion. Android must still require a secure, unlocked device
and refuse symlinks or foreign paths. Key loss or corrupt content alone does not
prevent removing an exact regular encrypted file. Deletion never regenerates keys.
An unavailable store is not proof of absence; no metadata-only bypass is added.

Only confirmed deletion or confirmed absence permits metadata finalization.
Uncertain deletion, cancellation, backend refusal or finalization failure preserves
the removal ledger and metadata. Startup does nothing. An explicit retry delegates
exact-entry inspection to the backend; an absent entry causes no second mutation.
Completed removal remains a local ledger lookup with no secret access. These rules
do not authorize automatic credential erasure, provider revocation, appliance writes,
or recovery/reauthorization using unreadable identity. Reconnection after completed
removal still creates new local IDs and does not inherit old grants.

## Compatibility and verification

Native API 1.8.1 corrects existing removal behavior without adding a force flag,
method, outcome or error. The 30 methods, seven C symbols, schemas, SQLite schema 4
and protected Meiju payload are unchanged. Older clients can call the same method;
older libraries can leave unreadable credentials pending. The UI may use the patch
version to explain that limitation, but cannot assume deletion will succeed.

Use synthetic host tests to prove deletion without reads, stored/pending account
cleanup, exact-account isolation, no cloud work, unavailable/locked/conflicting
backends, uncertain deletion, cancellation, reopen and metadata rollback. Verify
Android encrypted-file deletion with missing keys, failing key access and damaged
ciphertext on the JVM. These tests do not establish Android Keystore, phone or
real-provider acceptance. Preserve positive/negative response contract coverage.
