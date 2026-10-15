# ADR-0032: Store Android credentials through a native Keystore bridge

Status: accepted for implementation; Android device acceptance remains required.

## Decision

Provide a Java/JNI credential backend alongside the Rust library, independent of
Flutter views. The trusted Android host installs one application-scoped backend
before opening Meiju. Initialization is process-local and cannot replace a live
backend. No credential bytes enter Dart JSON, SQLite, logs or command arguments.
Other platforms keep their existing backend or explicit unavailable result.

Use an Android Keystore AES-256-GCM key per random profile namespace. Keys are
nonexportable through this integration, require an unlocked device, and do not
request authentication prompts. Android API 28 is the minimum; OS/OEM behavior
and key invalidation require physical acceptance. Do not claim hardware backing.
Never recreate a missing key while saved encrypted entries remain.

Store only authenticated ciphertext under the application's no-backup directory.
Bind application, format version, profile and exact credential reference as GCM
associated data. Use a fresh provider-generated nonce for every encryption and
bounded, versioned files. Reject malformed data, symlinks and foreign references.
Replace via a synced temporary file and atomic rename in the same directory;
delete only the exact reference. There is no plaintext or portable-file fallback.
Local deletion does not revoke vendor credentials or guarantee forensic erasure.

JNI copies bounded bytes only for the current call and clears Java credential
arrays before releasing them. Backend errors cross as fixed status codes, never
exception text. Java/OS transient copies cannot be guaranteed erased. Existing
account leases and pending/write/stored transitions remain authoritative.

Check cancellation/deadline before and after synchronous JNI work. Already-entered
Android Keystore and filesystem calls cannot be preempted; retain the account
lease until the call returns, then refuse subsequent networking. Interrupted or
failed dispatched mutations are uncertain unless the backend proves no mutation.
Do not detach a worker, retry, prompt, or continue a later write in the background.
Phone tests must measure OS-call latency before claiming the cancellation target.

## Compatibility and verification

The seven C host functions, 30 JSON methods, native API 1.8, SQLite schema 4 and
Meiju protected payload remain unchanged. Android adds a platform-only JNI
initializer, not a public JSON credential API. The UI worktree supplies startup
wiring and mobile lifecycle integration; the shared package owns native storage.

Verify the portable encrypted-file engine with synthetic JVM tests, compile Java
against official Android API stubs, and cross-check/build the Rust Android target.
Keep these results separate from Android Keystore and APK/device tests. A Linux
round trip does not prove Android key protection, backup exclusion, OEM behavior,
uninstall/reinstall semantics, physical-device support or a phone release.
