# Android native platform package

This package supplies the Java side of the Rust Android secret-store backend
and explicit foreground network-selection metadata.
It owns no Flutter views, account form, network service or background worker.
The application still calls the 13 C functions in
[homeverb.h](../../crates/ffi/include/homeverb.h) directly.

Use the [Android integration guide](../../docs/development/08-android-native.md)
for build, startup, lifecycle and physical acceptance. Android API 28 or later,
a configured secure screen lock, and an unlocked device are required. Android
12-14 have documented Keystore limitations; Android 15 or later is the preferred
first physical acceptance target. No Android version is physically verified yet.

`src/main/java` is production code. `src/test/java` contains synthetic exportable
JVM keys and must never enter an application artifact. The AAR builder includes
only production classes and the selected arm64 and/or x86_64 Rust libraries.
The default remains arm64; use `--abi x86_64` or repeat `--abi` for both. Integrate
one matching AAR, as separate AARs duplicate the Java classes. Keep `consumer-rules.pro` when
integrating sources manually: JNI requires the exact class, method and field names.

The separate `acceptance/` package builds a network-disabled test APK with the
production AAR. It owns no Flutter pages and must never enter a production AAR.
Its [four-phase emulator procedure](../../docs/development/08-android-native.md#run-the-isolated-native-acceptance-apk)
checks native Core, Android Keystore, process restart and local account cleanup
using fictional material. The additional `networks` phase reads only emulator
connectivity metadata with `ACCESS_NETWORK_STATE`; the APK still lacks `INTERNET`.
The runner refuses physical devices.

`NativeLanNetworks` supplies instance-owned selections from Android-assigned
Wi-Fi/Ethernet link properties. It starts backgrounded and reads metadata only on
explicit `snapshot` / `requireCurrent` calls. Mirror foreground/close events,
revalidate before preparing and confirming the native scan, and keep selections
out of persistence/logs. It adds no socket, callback, worker or permission prompt.
See the [network-selection handoff](../../docs/development/08-android-native.md#select-an-assigned-lan-interface).

The implementation uses Android Keystore keys and authenticated ciphertext in the
application no-backup directory. Missing keys, damaged ciphertext, unavailable
storage and lock-screen failures have no plaintext or key-regeneration fallback.
There is no public credential JSON method. Core's account leases and pending-state
recovery remain required; this backend alone does not establish provider identity.

The project license remains undecided. Local build artifacts are for development;
this work does not publish a package or establish release readiness.
