# 8. Embed the Android Core and native credential store

This stage supplies native storage and an arm64 Android library build path for the
separate Flutter worktree. It does not change Flutter pages or enable physical
controls. Read the [native contract](../native-ui-contract.md),
[account recovery rules](../storage.md#native-credential-storage) and
[ADR-0032](../adr/0032-android-native-credential-storage.md) first.

## Build and verify on Linux

Requirements: the locked Rust toolchain and `aarch64-linux-android` target, a JDK
with `--release` support, official Android API 28+ stubs, and Android NDK r28c.
The current validation uses JDK 21 and Android API 35 stubs. SDK/NDK files belong
outside the repository and are not included in commits. Fetch Cargo dependencies
once before the offline commands.

```bash
rustup target add aarch64-linux-android
cargo fetch --target aarch64-linux-android
.venv/bin/python scripts/verify_android_store.py \
  --android-jar /absolute/android-sdk/platforms/android-35/android.jar \
  --java-home /absolute/jdk-21
.venv/bin/python scripts/build_android_native.py \
  --ndk /absolute/android-ndk-r28c \
  --android-jar /absolute/android-sdk/platforms/android-35/android.jar \
  --java-home /absolute/jdk-21
```

The verifier compiles production Java against real Android stubs, runs synthetic
JVM encryption assertions and explicitly runs the otherwise ignored JNI test.
It uses generated temporary paths and fictional keys, never Android or Linux
account credentials. The ordinary Rust suite does not start a JVM or open a keyring.

The builder compiles the complete Rust FFI library for API 28 arm64, checks eight
exports (seven C functions plus the Android JNI initializer), checks 16 KiB ELF
segment alignment and excludes desktop runtime dependencies. It creates
`target/android/homeverb-core-arm64.aar` with production Java, the native library,
manifest and JNI consumer rules. Test providers cannot enter that archive. This
is an arm64 library package, not an APK, emulator image or published release.

## Integrate trusted startup

Add the local AAR to the Android application dependencies, retain its consumer
rules, and set the application's minimum SDK to 28 or later. This artifact includes
only `arm64-v8a`; an x86 emulator needs a separately built and verified ABI.

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
- Ordinary account removal reads the protected entry before deleting it. Unreadable
  ciphertext or a permanently lost key therefore leaves removal pending and keeps
  the local metadata. There is no force-cleanup API; explicit cleanup of unreadable
  entries needs a separate reviewed contract. Never interpret unreadable as absent.

Android documents unlocked-device key bugs on Android 12-14, including key loss
after removing the secure screen lock and failures after some biometric unlocks.
These are fixed in Android 15. HomeVerb requires a secure screen lock and refuses
unusable keys; it never disables key protection to work around those failures.
See the [official Keystore API documentation](https://developer.android.com/reference/android/security/keystore/KeyGenParameterSpec.Builder#setUnlockedDeviceRequired(boolean)).

## Evidence and remaining acceptance

Java compilation and 50 encrypted-entry assertions passed on the Linux JVM. The
actual JNI test passed with synthetic storage, including copied-array clearing,
backend errors, pre-cancellation, deadline/cancellation after an OS-call stand-in,
uncertain mutations and explicit rereading. This does not exercise Android Keystore.
The complete arm64 AAR built with NDK r28c, JDK 21 and API 35 stubs targeting API 28.
Archive integrity, production-only Java classes, all eight native exports and
16 KiB ELF load alignment passed inspection. The Linux workspace tests, native
contract examples and MVP runtime checks also passed. No APK or phone test has passed.

Before enabling a phone release, verify actual Keystore round trips across process
restart, key invalidation, profile isolation, screen locking/unlocking, unavailable
storage, backup/restore and uninstall/reinstall behavior, release shrinking rules,
16 KiB devices, foreground/background/forced termination and measured cancellation
latency. Then test explicitly authorized Meiju login, identity-preserving
reauthorization, inventory and off-LAN reads. Real appliance writes require separate
owner authorization. iOS storage, x86 Android packaging and physical model/control
acceptance remain separate work.
