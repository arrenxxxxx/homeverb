# 8. Embed the Android Core and native credential store

This stage supplies native storage and arm64/x86_64 Android library build paths for the
separate Flutter worktree. It does not change Flutter pages or enable physical
controls. Read the [native contract](../native-ui-contract.md),
[account recovery rules](../storage.md#native-credential-storage) and
[ADR-0032](../adr/0032-android-native-credential-storage.md) first.

## Build and verify on Linux

Requirements: the project Rust toolchain and the selected Android targets, a JDK
with `--release` support, official Android API 28+ stubs, and Android NDK r28c.
The current validation uses JDK 21 and Android API 35 stubs. SDK/NDK files belong
outside the repository and are not included in commits. Fetch Cargo dependencies
once before the offline commands.

```bash
rustup target add aarch64-linux-android x86_64-linux-android
cargo fetch --target aarch64-linux-android --target x86_64-linux-android
.venv/bin/python scripts/verify_android_store.py \
  --android-jar /absolute/android-sdk/platforms/android-35/android.jar \
  --java-home /absolute/jdk-21
.venv/bin/python scripts/build_android_native.py \
  --ndk /absolute/android-ndk-r28c \
  --android-jar /absolute/android-sdk/platforms/android-35/android.jar \
  --java-home /absolute/jdk-21
```

Omitting `--abi` preserves the arm64 build and output path. Add `--abi x86_64`
for a typical Linux-host Android emulator, or repeat the option to package both:

```bash
.venv/bin/python scripts/build_android_native.py \
  --ndk /absolute/android-ndk-r28c \
  --android-jar /absolute/android-sdk/platforms/android-35/android.jar \
  --java-home /absolute/jdk-21 \
  --abi arm64-v8a --abi x86_64
```

| ABI selection | Output under `target/android/` | Native entries |
|---|---|---|
| Omitted or `--abi arm64-v8a` | `homeverb-core-arm64.aar` | `jni/arm64-v8a/libhomeverb_ffi.so` |
| `--abi x86_64` | `homeverb-core-x86_64.aar` | `jni/x86_64/libhomeverb_ffi.so` |
| Both | `homeverb-core-universal.aar` | Both entries above |

The combined artifact covers these two 64-bit ABIs only. No 32-bit ARM or x86
support is included. Single-ABI builds require only their selected Rust target.

The verifier compiles production Java against real Android stubs, runs synthetic
JVM encryption assertions and explicitly runs the otherwise ignored JNI test.
It uses generated temporary paths and fictional keys, never Android or Linux
account credentials. The ordinary Rust suite does not start a JVM or open a keyring.

The builder compiles the complete Rust FFI library for each selected ABI at API 28,
checks its ELF architecture, eleven exports (ten C functions plus the Android JNI
initializer), 16 KiB ELF segment alignment and absence of desktop runtime
dependencies. It packages production Java, the selected native libraries, manifest
and JNI consumer rules. Test providers cannot enter that archive. All checks must
pass before replacing the output; a failed build may leave an older artifact and
must not be treated as success. This is a library package, not an APK, emulator
image or published release.

## Integrate trusted startup

Add the local AAR to the Android application dependencies, retain its consumer
rules, and set the application's minimum SDK to 28 or later. Select the artifact
matching the device/emulator ABI, or the combined AAR. Add only one HomeVerb AAR
to the application: combining separate AAR dependencies duplicates Java classes.
Cross-compilation alone does not establish library loading or Keystore behavior.
The isolated acceptance APK below supplies AOSP Android 15 x86_64 evidence;
Flutter assembly, physical devices and release shrinking still require their own
mobile acceptance.

Before Dart opens a Meiju host, invoke these calls from trusted Android startup:

```java
boolean available;
try {
    System.loadLibrary("homeverb_ffi");
    available = org.homeverb.NativeCredentials.install(getApplicationContext());
} catch (LinkageError unavailable) {
    available = false;
}
```

Initialize in the Application startup or before creating the Flutter engine.
Treat a false result or load failure as unavailable native credential support.
Installation registers the application-scoped backend once; it does not read keys,
authenticate, discover or control devices. It cannot replace an installed backend.
Do not expose the initializer, context, storage paths or profile selection as model
tool parameters. The OS application context selects the private no-backup directory;
Rust supplies the persisted random profile and credential references.

Dart loads `libhomeverb_ffi.so` and uses the existing worker-thread C ABI. Supply an
OS-private application directory for SQLite. Use `midea.meiju` explicitly, collect
account input through the protected connection function, and preserve foreground
cancellation behavior. An uninstalled backend returns `SECRET_STORE_UNAVAILABLE`.
Do not infer current key access from successful installation or API availability:
the device may subsequently lock or invalidate keys.

## Persistence, failure and lifecycle

- One nonexportable Android Keystore AES-256-GCM key belongs to each random profile.
  AAD includes application, format, profile and credential reference. Ciphertext
  uses fresh provider-generated nonces, bounded files, synced temporary writes and
  atomic replacement. A profile file lock refuses contention instead of waiting.
- Device locking returns `SECRET_STORE_LOCKED`; missing secure screen lock or
  unavailable/invalid keys returns `SECRET_STORE_UNAVAILABLE`. A missing key with
  remaining entries is never regenerated. A corrupt entry is never an empty account.
- Java and JNI clear valid bounded credential arrays. JVM/OS copies and secure
  erasure of deleted filesystem data cannot be guaranteed. No raw provider errors,
  identifiers, token strings or Java exception messages enter ordinary diagnostics.
- JNI storage calls are synchronous on the native worker. They retain the account
  lease until the OS call returns, with cancellation/deadline checks before and
  after. Already-entered OS calls cannot be preempted. Cancelled dispatched writes
  report `SECRET_STORE_WRITE_UNKNOWN`; no detached worker can write later after the
  call returns. Measure device latency before claiming the cancellation target.
- Account pending/stored/removing semantics and exact-byte recovery remain those
  of Core. Lock loss, missing keys or corrupt data cannot trigger automatic login,
  credential reset or device-ID regeneration. Removing local data does not revoke
  vendor authorization. Neither keys nor entries are an authorization-sharing format.
- API 1.8.1 account removal can delete the exact encrypted file without decrypting
  it, after the user confirms the linked device set. Corrupt contents or a missing
  key do not prevent this cleanup while the native store can access that file.
  A secure, unlocked device is still required. Refused or uncertain deletion keeps
  cleanup pending; there is no automatic erasure or metadata-only force flag. See
  [ADR-0033](../adr/0033-delete-unreadable-account-credentials.md).

Android documents unlocked-device key bugs on Android 12-14, including key loss
after removing the secure screen lock and failures after some biometric unlocks.
These are fixed in Android 15. HomeVerb requires a secure screen lock and refuses
unusable keys; it never disables key protection to work around those failures.
See the [official Keystore API documentation](https://developer.android.com/reference/android/security/keystore/KeyGenParameterSpec.Builder#setUnlockedDeviceRequired(boolean)).

## Evidence and remaining acceptance

Java compilation and 54 encrypted-entry assertions passed on the Linux JVM. The
actual JNI test passed with synthetic storage, including copied-array clearing,
backend errors, pre-cancellation, deadline/cancellation after an OS-call stand-in,
uncertain mutations and explicit rereading. This does not exercise Android Keystore.
The complete arm64, x86_64 and combined AARs built with Rust 1.96.0, NDK r28c,
JDK 21 and API 35 stubs targeting API 28. Archive integrity, production-only Java
classes, each architecture's eleven native exports and 16 KiB ELF load alignment
passed inspection. Six package tests cover architecture, undefined/missing exports,
alignment, desktop dependencies, ABI directory placement, test-class exclusion and
preserving a previous archive on failure. All 29 Python tests and the document,
contract, repository and commit checks passed. Prior Linux workspace, native
contract and MVP runtime evidence remains separate; no Rust application behavior
changed in that packaging phase. The later isolated APK evidence is below;
physical phone acceptance remains outstanding.

## Run the isolated native acceptance APK

Use a newly created disposable AOSP emulator with no accounts, an explicit serial
and a separate ADB server port. Never run lock-screen setup or cleanup against a
personal device or a shared development emulator. `verify_android_acceptance.py`
refuses non-emulators, but cannot prove that an emulator is disposable.

Build a fresh matching AAR first using the commands above, then:

```bash
.venv/bin/python scripts/build_android_acceptance.py \
  --aar target/android/homeverb-core-x86_64.aar --abi x86_64 \
  --ndk /absolute/android-ndk-r28c \
  --android-jar /absolute/android-sdk/platforms/android-35/android.jar \
  --build-tools /absolute/android-sdk/build-tools/35.0.0 \
  --java-home /absolute/jdk-21
adb -P 5047 -s emulator-5588 emu avd name
adb -P 5047 -s emulator-5588 install -t target/android/homeverb-native-acceptance-x86_64.apk
.venv/bin/python scripts/verify_android_acceptance.py \
  --adb-port 5047 --serial emulator-5588 --phase unsecured
```

Replace the example port/serial with the dedicated emulator's values after checking
its AVD name. The builder uses an ephemeral development signing key; rebuilding
requires uninstalling the old test package and restarting all phases. The APK
requests no permissions, including Internet, disables backup, has no application
UI and includes only production credential classes plus the test harness. The
builder verifies native exports, signature, exact ABI libraries and 16 KiB ZIP
alignment before replacing its output. Test classes and JNI wrappers never enter
the production AAR.

Run these phases in order, waiting for each OS lock transition to complete:

| Phase | Emulator setup | Required result |
|---|---|---|
| `unsecured` | Fresh emulator without a secure screen lock | Store refusal; native Mock discovery/write, shared migration leases, device-lock contention/release, foreground/background/close and JNI pre-login refusal |
| `write` | Configure a test PIN on this disposable emulator, then leave it unlocked | Android Keystore round trip, nonexportable key, ciphertext, profile isolation, input clearing and protected fictional account fixture |
| `locked` | Lock the emulator and wait until Android reports it locked | Java store and native JNI connection return locked errors before provider work; native Mock operations remain local |
| `read` | Unlock with the test PIN and wait for the unlocked state | A different process reads prior ciphertext; nonce replacement, tamper refusal, deliberately deleted-key refusal without regeneration, native JNI inventory reading, account-lock contention/release and account deletion |

Repeat the verifier command with the selected `--phase`. `am instrument` can exit
zero on assertion failure; the verifier requires matching phase, positive assertion
count, explicit PASS and Android success code. Failures report only fixed status
and assertion counts. If a lock-state precondition fails, complete the OS transition
before rerunning that phase. A failed destructive `read` phase requires a fresh
test installation; do not treat a partially consumed fixture as a new baseline.

The fixture uses the native-created schema-4 database and encrypted version-3
account payload, with no real credentials or provider requests. Successful secure
writes use the production Java backend; native inventory reads and removal also
exercise the production Rust/JNI backend. This does not test successful provider
login or JNI session replacement. On completion, uninstall only
`org.homeverb.nativeacceptance` and stop only the emulator/ADB server created for
this run. Temporary tools and images stay outside the repository.

On October 16, 2026, AOSP Android 15 x86_64 image revision 2 and Emulator 37.2.12
passed all four phases: 17, 17, 16 and 27 assertions respectively. The initial APK
failed before Core opened its database; a separate native probe showed standard
Rust locks returning `Unsupported` while Bionic `flock` succeeded. The Android
[file-lock fix](../../crates/storage/src/file_lock.rs) made the full procedure pass.
Both lock/unlock tests initially ran before the OS transition completed; rerunning
after the required state was visible passed. This is emulator evidence, not a
physical phone, hardware-backed key, Flutter journey or 16 KiB-page runtime claim.

Before enabling a phone release, repeat Keystore round trips across process
restart, OS/OEM key invalidation, profile isolation, screen locking/unlocking, unavailable
storage, backup/restore and uninstall/reinstall behavior, release shrinking rules,
16 KiB devices, foreground/background/forced termination and measured cancellation
latency. Then test explicitly authorized Meiju login, identity-preserving
reauthorization, inventory and off-LAN reads. Real appliance writes require separate
owner authorization. iOS storage, physical Android runtime and model/control
acceptance remain separate work.
