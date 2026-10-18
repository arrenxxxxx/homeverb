# 10. Build and install the integrated Flutter Android app

The app in `apps/mobile` embeds native Core API 1.24.0 directly. It does not invoke
CLI, require a daemon or use a HomeVerb backend. Android 9 / API 28 is the minimum;
the development APK includes arm64-v8a and x86_64. Flutter 3.47.7 is pinned in
`apps/mobile/.fvmrc`. Build outputs are local and ignored by Git.

## Select an installed toolchain

Use Rust 1.96 or later, Python with `requirements-dev.txt`, FVM, JDK 21, Android
SDK platform 35 for native Java checks, platform 36 for Flutter, build tools
36.0.0, and NDK 28.2.13676358. Install and accept SDK components separately through
the Android SDK Manager. Do not copy another machine's absolute `local.properties`.

From the repository root, substitute your installed SDK/JDK paths:

```sh
export HOMEVERB_ANDROID_SDK=/absolute/android-sdk
export HOMEVERB_JAVA_HOME=/absolute/jdk-21
rustup target add aarch64-linux-android x86_64-linux-android
cargo fetch --locked --target aarch64-linux-android --target x86_64-linux-android
python3 scripts/build_android_native.py \
  --ndk "$HOMEVERB_ANDROID_SDK/ndk/28.2.13676358" \
  --android-jar "$HOMEVERB_ANDROID_SDK/platforms/android-35/android.jar" \
  --java-home "$HOMEVERB_JAVA_HOME" \
  --abi arm64-v8a --abi x86_64
```

The native builder supports Linux and macOS and inspects 17 exports per ABI,
production Java classes and 16 KiB ELF alignment. Its combined output is
`target/android/homeverb-core-universal.aar`; the app depends on exactly that file.
Build it from the same source checkout before assembling Flutter.

## Build a development APK

From `apps/mobile`, initialize FVM and configure Flutter to use the complete SDK.
`fvm flutter doctor -v` reports the selected location. An incomplete platform-tools
installation is not an Android SDK, even when it supplies a working `adb`.

```sh
fvm install
fvm flutter pub get
fvm flutter analyze
fvm flutter build apk --release --target-platform android-arm64,android-x64
```

The output is `apps/mobile/build/app/outputs/flutter-apk/app-release.apk` relative
to the repository root. Save the Mock artifact before the next build overwrites
that output. The default profile is fictional Mock and supports fictional writes.
For the separately selected mainland Meiju profile, run:

```sh
fvm flutter build apk --release --target-platform android-arm64,android-x64 \
  --dart-define=HOMEVERB_ADAPTER=midea.meiju
```

This second build exposes protected sign-in and device retrieval. Opening a
supported device automatically prepares its reviewed controls without a separate
permission form; see the [control guide](../../apps/mobile/README.md#device-controls). Both APKs use the same Android package
`dev.homeverb.homeverb_mobile`, so installing one replaces the other. Their
Mock and Meiju data directories are separate. Neither is a store release:
release-mode compilation still uses a local development signing key.

For an isolated SDK override on Linux/macOS without legacy `~/.flutter_settings`,
create a temporary directory containing a `settings` JSON file with `android-sdk`
and `jdk-dir` paths, then set `XDG_CONFIG_HOME` to that directory only for the build
command. Set `ANDROID_HOME` and `ANDROID_SDK_ROOT` to the same SDK. This avoids
changing global Flutter settings. Also set `ANDROID_USER_HOME` to your existing,
persistent Android user directory (normally `$HOME/.android`). Otherwise Android
Gradle may create a new development signing key under the temporary XDG directory;
a later APK cannot update the installed app if that key is lost. Keep signing keys
outside the repository and do not delete the persistent Android directory. Verify
the generated `android/local.properties`
contains that SDK. Do not accept unrelated SDK licenses to work around a wrong
location. Remove only the temporary configuration created for this build.

## Inspect and install

Use the selected SDK's build tools against each saved artifact:

```sh
"$HOMEVERB_ANDROID_SDK/build-tools/36.0.0/apksigner" verify --verbose /absolute/homeverb.apk
"$HOMEVERB_ANDROID_SDK/build-tools/36.0.0/zipalign" -c -P 16 4 /absolute/homeverb.apk
"$HOMEVERB_ANDROID_SDK/build-tools/36.0.0/aapt" dump badging /absolute/homeverb.apk
shasum -a 256 /absolute/homeverb.apk
```

The repository verifier also compares each packaged Core library with the current
universal AAR, accepting only exact bytes or the same NDK
`llvm-strip --strip-unneeded` output used by Android packaging. A same-named stale
library fails even if it has the expected exports.

The repository verifier additionally rejects incomplete extra ABIs and an embedded
integration-test plugin, and writes an artifact checksum/provenance report:

```sh
python3 scripts/verify_flutter_apk.py --apk /absolute/homeverb.apk \
  --sdk "$HOMEVERB_ANDROID_SDK" --java-home "$HOMEVERB_JAVA_HOME" \
  --adapter homeverb.mock --report /absolute/homeverb.json
```

Use `--adapter midea.meiju` for the explicit Meiju app build. The adapter value
is a build declaration, not runtime proof. Check both native ABIs and the Core
exports, minimum SDK, package identity and backup exclusions. Preserve the APK checksum, source revision, native AAR checksum
and toolchain versions together. Do not use an older successful artifact as proof
that the latest build succeeded.

Install only on an explicitly selected device/emulator:

```sh
adb -s DEVICE_SERIAL install -r /absolute/homeverb.apk
```

No credentials or provider device identifiers belong in commands or logs. The
Meiju account input belongs only in the app's protected sign-in form. Do not run
physical appliance writes as part of packaging verification.

## Runtime verification and remaining acceptance

On an isolated emulator, run these sequentially from `apps/mobile`:

```sh
fvm flutter test integration_test/native_smoke_test.dart -d EMULATOR_SERIAL
fvm flutter test integration_test/approval_smoke_test.dart -d EMULATOR_SERIAL
fvm flutter test integration_test/account_smoke_test.dart -d EMULATOR_SERIAL
```

These use temporary fictional or empty profiles, exercise the real native bridge
and make no vendor request. They do not prove real sign-in, physical Keystore
policy, phone/OEM lifecycle or appliance support. Rebuild the ordinary release APK
after integration tests; never distribute the test APK as the app artifact.

Actual commands, outcomes, known pixel failures, host-specific test limitations
and delivery hashes are recorded in [app verification](../../apps/mobile/verification.md).

## Diagnose scrolling performance

Check the emulator's renderer before interpreting app frame times:

```sh
adb -s EMULATOR_SERIAL shell dumpsys SurfaceFlinger | rg '^GLES:'
```

SwiftShader and Lavapipe are software renderers. They are useful for isolated
compatibility checks but can make the same scrolling workload much slower than
host GPU rendering. For interactive review, close the selected emulator and
restart its existing AVD with `emulator -avd AVD_NAME -gpu host -no-snapshot
-no-boot-anim`. Do not use `-wipe-data`. Confirm the actual renderer after startup;
the host flag alone does not establish that acceleration succeeded. Preserve any
existing Android SDK/AVD directory selection and application signing key.

The opt-in scrolling benchmark uses the production screen, background and device
grid with twelve fictional AC cards, decoded images, one warm-up sweep and four
measured flings. It does not open Core or an account. From `apps/mobile`, run:

```sh
HOMEVERB_PERFORMANCE_REPORT=/absolute/scroll-profile.json \
  fvm flutter drive --profile --no-dds --keep-app-running -d EMULATOR_SERIAL \
  --driver=test_driver/performance_driver.dart \
  --target=integration_test/scroll_performance_test.dart \
  --dart-define=HOMEVERB_PERFORMANCE=true
```

The pinned Flutter version uses `flutter drive` for profile-mode integration
tests; `flutter test` has no `--profile` option. `--no-dds` permits the timing
collector's device-local VM connection. The host driver saves UI/raster frame
timings and sample counts to the explicit report path. The test is skipped unless
opted in and refuses debug timing. Run comparisons with the same renderer,
viewport, workload and idle host, without screenshots or competing builds during
measurement. Report rendering and emulator changes separately; average raster
duration is not a displayed-FPS guarantee. Representative physical-phone profile
measurements remain necessary.

Use an empty AVD for profiling. `--keep-app-running` is mandatory when retaining
an installation: this Flutter version otherwise stops and uninstalls the package
after `drive`, deleting its local account data and Android Keystore entries. A
temporary fixture inside the test does not protect the ordinary installation
from that tooling behavior. After profiling, stop the test app and install the
ordinary APK with `adb install -r`; never uninstall it to switch entry points.


## Select the app language

Settings > Language offers System default, English and Simplified Chinese. The
selection applies immediately and is stored only in local presentation preferences;
process restart preserves it. It does not change Core authority or provider state.
After changing shared catalogs, regenerate using the
[internationalization guide](../../apps/mobile/README.md#internationalization)
before building. Verify both language directions and a cold start on the target
Android environment; host widget tests alone do not prove packaged font rendering.


## Prepare protected account storage

Before real sign-in, configure a PIN, pattern or password in the Android system
security settings and leave the device unlocked. The production credential backend
refuses an unsecured device with SECRET_STORE_UNAVAILABLE. The Flutter sign-in
page now distinguishes this prerequisite from unavailable startup registration
and a locked device, with a direct system-settings action. Credentials and the
screen-lock secret must be entered only in their respective device UIs.

The local preflight has a three-second deadline and runs on page entry, foreground
return and explicit submission. It returns only fixed readiness labels; no key,
credential, identifier or exception text crosses the channel. It is not Keystore
read/write acceptance or provider authentication evidence. Continue to use Core's
actual error and retained account outcome if the following operation fails.
