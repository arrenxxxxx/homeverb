# Add the iOS native credential store

This guide follows the [Android embedding guide](08-android-native.md). The iOS
Keychain backend is implemented under [ADR-0062](../adr/0062-ios-keychain-credentials.md).
Portable tests and target compilation are distinct from signed-device acceptance.
An iPhone app, packaged framework and Flutter integration are still required.

## 1. Supply the trusted build scope

Build `homeverb-secrets` (normally through `homeverb-ffi`) with
`HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP` set to the signed app's full, private
application identifier, including its App ID prefix. This value is nonsecret.
Resolve it from the actual signing configuration; do not assume the prefix always
equals a team identifier. Verify the signed entitlements, not just the Xcode UI.
Do not use a group shared with another app or extension. All Keychain operations
explicitly select this group and never fall back to another group.

The value is compiled into Rust with `option_env!`; changing it requires a rebuild
and changes which items can be accessed. Cargo tracks this build environment
dependency. Missing or malformed values return `SECRET_STORE_UNAVAILABLE` before
any Keychain call. A syntactically valid but unauthorized group is refused by the
OS. Runtime JSON, imports and Dart method arguments cannot change the group.

The random profile UUID and credential reference come from the existing native
account vault. Preserve its OS-private data directory across launches. Never use
an email address, provider ID, appliance ID or credential as the service/account
attribute. No new C initializer or JSON method is needed.

## 2. Build and check the native code

The following group is fictional and only for compiler/CI checks. It is not a
signing configuration and must not be shipped as a working account store.

```bash
rustup target add aarch64-apple-ios aarch64-apple-ios-sim
cargo test -p homeverb-secrets --offline --locked ios::
HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP=SYNTHETIC.org.homeverb.test \
  cargo clippy -p homeverb-secrets --target aarch64-apple-ios --offline --locked -- -D warnings
HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP=SYNTHETIC.org.homeverb.test \
  cargo check -p homeverb-secrets --target aarch64-apple-ios-sim --offline --locked
```

Rust target checking can run on Linux with cached dependencies; it does not link
Apple frameworks. On macOS with Xcode and the selected Apple SDKs, build
`homeverb-ffi` for both targets with `IPHONEOS_DEPLOYMENT_TARGET=13.0`. The
[MVP workflow](../../.github/workflows/mvp.yml) performs those builds and runs the
same tests on macOS, where three additional tests construct real Core Foundation
objects and check production query/result handling without Keychain access.
Inspect the exact commit's CI results before reporting them as passed.

The current `cdylib` build establishes native compilation/linkage only. Creating
and embedding a signed framework/XCFramework, checking exported symbols in that
package, and building an installable Flutter app remain separate work. Do not
publish an unsigned build as a supported iOS package.

## 3. Wire foreground account operations

Use the existing [native UI contract](../native-ui-contract.md), worker ownership,
protected connection buffers and foreground cancellation. Do not call Keychain
from the UI thread. System calls are synchronous and cannot be forcibly cancelled;
the current operation and account lease stay alive until the OS returns. After
return, a cancelled/deadline-expired read yields no credential to the caller.
A possibly completed mutation returns `SECRET_STORE_WRITE_UNKNOWN`; use explicit
account recovery, never automatic retry or delete/recreate.

New entries require a device passcode and use
`kSecAttrAccessibleWhenPasscodeSetThisDeviceOnly`, with synchronization disabled.
Existing entries must match that policy. The backend requests no authentication
UI; it deliberately uses Apple's deprecated fail-without-prompt option as recorded
in ADR-0062. An OS-reported lock/authentication refusal is sanitized. This is not
a general device-lock detector: metadata lookup or exact deletion may be permitted
without reading protected bytes. Do not infer unlock state from a successful
lookup/deletion or promise that deletion necessarily fails while locked.

Deletion reads metadata only and targets the admitted persistent reference.
Removing the passcode invalidates these credentials; a new passcode does not
restore them. Account metadata may remain and require explicit reconnection or
local removal. Keychain persistence across reinstall, backup/restore behavior,
prompt refusal and latency require the acceptance below. No local deletion proves
vendor revocation or forensic erasure.

## 4. Accept on a signed disposable app before real accounts

Use only fictional credentials and fresh random profile/reference namespaces.
Keep test identifiers, raw Keychain output and secret bytes out of reports.

1. Inspect signed entitlements and confirm the exact private group. A build with
   a missing or unentitled group must fail without a fallback or prompt.
2. With a passcode set and the device unlocked, add/read/replace/delete, reopen
   in another process, and verify profile/reference isolation. Missing reads
   return no value; deleting a confirmed absent reference succeeds.
3. Verify no-passcode refusal, locked-device read refusal, later unlock behavior
   and passcode-removal invalidation. Record metadata/deletion behavior separately;
   neither proves protected-data availability. Test actual iOS/OEM versions.
4. Inject malformed metadata, foreign scope, changed references and incompatible
   protection in isolated fixtures. Prove that no broad read, update or deletion
   follows rejection. A lost exact reference must never select another item.
5. Cancel/background before lookup, during the synchronous call and immediately
   after dispatch. Measure OS latency and verify account pending/recovery states,
   no detached work, no retries and lease retention until return. Force termination
   separately; there is no guaranteed completion callback.
6. Validate signed framework loading, Flutter protected input, lifecycle and
   restart paths, and the effect of uninstall/reinstall and backup/restore.
   Provider login, reauthorization and appliance writes require separate evidence
   and the existing owner authorization boundaries.

No result from portable tests, Core Foundation object tests or unsigned library
builds completes this device checklist.
