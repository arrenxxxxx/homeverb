# ADR-0062: Store iOS credentials in an explicitly scoped Keychain

Status: accepted for implementation; signed iOS runtime acceptance remains required.

## Decision

Implement the iOS `SystemSecretStore` with Apple's generic-password Keychain APIs.
The trusted native build supplies `HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP`, equal to
the signed application's private application identifier. Every query and mutation
includes that exact group, a versioned service containing the random profile UUID,
and the opaque credential reference as account. Missing or malformed configuration
fails before Keychain access. This is nonsecret build configuration, not an ordinary
request parameter. Packaging must verify the entitlement and exclude shared groups;
syntax validation alone cannot establish app isolation. Do not use the implicit
first group or search across all entitled groups.

New items use `kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly` through an explicit
`SecAccessControlCreateWithFlags` object with zero additional flags, and explicitly
disable synchronization. Add supplies `kSecAttrAccessControl`, not a conflicting
second `kSecAttrAccessible` representation. This requires a device passcode and unlocked access;
removing the passcode deletes these items, and they do not migrate to another
device. Do not claim hardware backing, forensic erasure or vendor revocation.
There is no plaintext fallback, background task or credential export.

First request at most two metadata/persistent-reference matches, without password
data, including synchronizable matches so a conflicting protection policy cannot
be silently ignored. Reject ambiguity, foreign attributes, unreviewed access-control
objects, or protection weaker/different than the required policy. A returned
access-control object must have the Security framework's type and compare equal
to a newly created complete policy with the exact protection and zero flags.
Do not parse descriptions, inspect private APIs, or admit objects based only on
their accessibility attribute. Then read, replace or
delete the exact persistent reference with the namespace and protection attributes
still present. On iOS the selector is `kSecValuePersistentRef`, not macOS's
`kSecMatchItemList`. Return-data queries use one match only. Delete does not read
credential data. Add only after confirmed absence; never delete/recreate, broaden
an exact query, or retry after a race or failure.

Metadata lookup or deletion may be permitted without decrypting the credential.
Surface OS-reported lock errors, but do not treat these operations as device-lock
probes or promise that deletion necessarily fails while locked. Protected reads
and passcode invalidation need separate signed-device acceptance.

For this C-only backend, explicitly set `kSecUseAuthenticationUIFail` on every
operation. Apple deprecates this value in favor of an `LAContext` with
`interactionNotAllowed`; the documented fail-without-prompt behavior is retained
deliberately to avoid introducing an unverified Objective-C runtime bridge.
Signed-device acceptance must verify prompt refusal. A future replacement must
preserve that behavior and does not authorize interactive authentication here.

Keychain calls block the current native worker. Check the original Operation
before and after each call, retain the caller's account lease until return, and
do not detach work. A failed or interrupted dispatched mutation returns
`SECRET_STORE_WRITE_UNKNOWN`; pre-dispatch cancellation/deadline retains its usual
error. OS calls cannot be preempted, so phone latency must be measured separately.
Bound copied credential bytes to 1 MiB and transient persistent references to
4 KiB. Never serialize/log OS text, references or credential bytes. Release owned
Core Foundation results; Rust secret buffers zeroize, while OS copies are outside
that guarantee.

## Compatibility and verification

Native API 1.20, the 16 C functions, 33 ordinary methods, SQLite schema 5 and
protected payload 3 remain unchanged. iOS gains a platform implementation with
explicit build configuration; other platforms retain their existing behavior.
The initial accessibility-only add path returned an implicit access-control object
that the decoder rejected in isolated Simulator testing. Explicit policy creation
and comparison correct this mismatch. Differently represented existing objects
remain refused; no automatic migration, deletion or recreation is introduced.
Use the existing locked `core-foundation-sys` 0.8.7 as a direct iOS dependency
(MIT OR Apache-2.0). Security framework declarations follow Apple's public C API.

Verify portable orchestration with injected storage failures, identity/protection
rejection, exact replacement/deletion, and cancellation around dispatch. Check
both iOS Rust targets separately. Neither synthetic tests nor cross-target
`cargo check` proves framework linkage, signed entitlements, lock/passcode behavior,
Keychain persistence or a Flutter/iPhone journey. Those remain release gates.

## Evidence

- [Apple access-group rules](https://developer.apple.com/documentation/security/ksecattraccessgroup).
- [Passcode-set device-only accessibility](https://developer.apple.com/documentation/security/ksecattraccessiblewhenpasscodesetthisdeviceonly).
- [Keychain query results](https://developer.apple.com/documentation/security/secitemcopymatching(_:_:)) and [match limits](https://developer.apple.com/documentation/security/ksecmatchlimit).
- [Apple's public SecItem header](https://github.com/apple-oss-distributions/Security/blob/main/keychain/headers/SecItem.h), inspected October 16, 2026; it explicitly distinguishes iOS persistent-reference selectors from macOS selectors.
- [Noninteractive authentication and deprecation](https://developer.apple.com/documentation/security/ksecuseauthenticationuifail).
- [SecItem dictionary semantics](https://developer.apple.com/forums/thread/724023).
- [Public access-control API](https://github.com/apple-oss-distributions/Security/blob/db15acbe6a7f257a859ad9a3bb86097bfe0679d9/keychain/headers/SecAccessControl.h) and [complete-object equality implementation](https://github.com/apple-oss-distributions/Security/blob/db15acbe6a7f257a859ad9a3bb86097bfe0679d9/OSX/sec/Security/SecAccessControl.m), inspected October 16, 2026.
- [Returned access-control conversion](https://github.com/apple-oss-distributions/Security/blob/db15acbe6a7f257a859ad9a3bb86097bfe0679d9/OSX/sec/Security/SecItem.m) and [implicit protection construction](https://github.com/apple-oss-distributions/Security/blob/db15acbe6a7f257a859ad9a3bb86097bfe0679d9/keychain/securityd/SecDbQuery.c).
