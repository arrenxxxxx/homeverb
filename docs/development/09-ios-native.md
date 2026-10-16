# Add the iOS native credential store

This guide follows the [Android embedding guide](08-android-native.md). The iOS
Keychain backend is implemented under [ADR-0062](../adr/0062-ios-keychain-credentials.md).
Portable tests and target compilation are distinct from signed-device acceptance.
The [development package](../../packages/ios/README.md) adds XCFramework packaging
and consumer-link checks. A separate disposable Simulator app exercises runtime
behavior; Flutter integration and physical-phone acceptance are still required.

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
rustup target add aarch64-apple-ios aarch64-apple-ios-sim x86_64-apple-ios
cargo test -p homeverb-secrets --offline --locked ios::
HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP=SYNTHETIC.org.homeverb.test \
  cargo clippy -p homeverb-secrets --target aarch64-apple-ios --offline --locked -- -D warnings
HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP=SYNTHETIC.org.homeverb.test \
  cargo check -p homeverb-secrets --target aarch64-apple-ios-sim --offline --locked
```

Rust target checking can run on Linux with cached dependencies; it does not link
Apple frameworks. The [MVP workflow](../../.github/workflows/mvp.yml) runs the same
tests on macOS, where three additional tests construct real Core Foundation
objects and check production query/result handling without Keychain access.
On commit `3742e67`, all sixteen iOS tests and complete native-library builds for
arm64 device/simulator passed. Commit `d4525b1` subsequently passed the actual
three-architecture XCFramework build, extracted-package inspection and consumer
linkage in macOS CI. Neither phase ran Keychain or an iOS app.

On macOS with Xcode, Python 3.11+ and cached dependencies, create the release
development package with the trusted group from step 1:

```bash
python3 scripts/build_ios_native.py
ditto -x -k target/ios/homeverb-core.xcframework.zip target/ios/integration
```

The builder requires `HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP` in its environment. It
creates separate dynamic frameworks for arm64 iOS 13.0+ and arm64/x86_64 Simulator
14.0+, with a canonical C header/module map and
`@rpath/HomeVerb.framework/HomeVerb` install name. The higher Simulator floor
matches the Rust arm64 Simulator target; it is not a physical support claim.
The generated XCFramework is inspected, archived, extracted and inspected again.
A C module consumer links every native function on all three architectures before
the archive replaces a previous deliverable. The builder does not run the consumer.

Local ad-hoc signatures use `codesign --sign - --timestamp=none`: they provide
integrity without using an identity, signing key or timestamp service. They are
not distribution signatures or provisioning. The CI package uses a fictional
group and must not be shipped as a working account store. Do not publish the
development artifact or treat package checks as signed-app/Keychain acceptance.

In the Flutter iOS project, add the extracted `HomeVerb.xcframework` to the app's
framework dependencies and select **Embed & Sign**. Let Xcode choose the matching
slice; never copy a simulator binary into a device app. Ensure the app's framework
runpath includes `@executable_path/Frameworks`. Resolve the embedded
`HomeVerb.framework/HomeVerb` path from the app bundle and load it through the
existing Dart FFI worker wrapper. Check
`get_api_info` before optional calls. The package adds no API methods and changes
no ownership/cancellation rules. Actual Dart loading, app signing and runtime
validation remain the Flutter worktree's integration/acceptance work.

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

## 4. Run isolated Simulator acceptance

On macOS with Xcode and an installed iOS 14+ Simulator runtime, build the fictional
package and run the isolated app:

```bash
export HOMEVERB_IOS_KEYCHAIN_ACCESS_GROUP=SYNTHETIC.org.homeverb.test
python3 scripts/build_ios_native.py
python3 scripts/verify_ios_acceptance.py
```

The runner selects an installed compatible iPhone runtime/type, creates a new
Simulator, and targets only its returned UUID. It never uses `booted`, `all`, a
physical phone or an existing user Simulator. All app builds and operations are
bounded. The new Simulator is shut down and deleted even after a returned failure;
an externally killed runner may require manual cleanup of its named test device.
There is no app/write retry after missing or failed evidence.

The test app embeds the inspected production framework and calls the public C ABI
from a worker. It verifies Mock discovery, power execution and reading, persisted
cards/cache/receipt after a separate process launch, explicit Core foreground
refusal/restoration and closed-handle refusal. It does not implement a Flutter
view or establish UIKit background-delivery/forced-termination behavior.

A Simulator-only Rust static-library example calls the actual `SystemSecretStore`
with hardcoded fictional bytes in fresh UUID namespaces. It verifies missing and
isolated references, add/read/reopen/replace, pre-cancelled read/write/delete
refusal, persistence across different PIDs, exact deletion and absent deletion.
Its test export is absent from the production framework and public header.
The runner reads back the ad-hoc signed app's exact fictional private group.
This verifies the test bundle's scope configuration, not physical entitlement
enforcement or distribution signing.

Reports contain only run/process identity, fixed stages, counts and numeric store
codes: 0 success, 1 test/runtime failure, 2 locked, 3 unavailable, 4 uncertain
mutation, 5 conflict, 6 cancelled, 7 deadline exceeded. No secret, account,
native response or OS error text is exported. A new run ID, exact launch PID,
different reader PID and complete successful reports are all required. Portable
runner tests are not evidence that this app ran; inspect the exact macOS CI job.

The app makes only Mock and fictional Keychain calls. iOS has no Android-style
Internet manifest permission, so this is not a claim of OS-enforced network denial.
Simulator round trips do not establish passcode/lock protection, no-prompt policy,
reinstall/backup behavior or physical iPhone compatibility. Never weaken production
Keychain accessibility or access-group checks to make Simulator acceptance pass.

## 5. Accept on a signed disposable app before real accounts

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

No result from portable tests, Core Foundation object tests, native-library builds
or locally sealed development packages completes this device checklist.
