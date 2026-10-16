# 10. Build and install the integrated Flutter Android app

The app in `apps/mobile` embeds native Core API 1.22.0 directly. It does not invoke
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
For the separately selected mainland Meiju read-only profile, run:

```sh
fvm flutter build apk --release --target-platform android-arm64,android-x64 \
  --dart-define=HOMEVERB_ADAPTER=midea.meiju
```

This second build exposes protected sign-in and explicit retrieval/reads; it
supplies no real-device write grants. Both APKs use the same Android package
`dev.homeverb.homeverb_mobile`, so installing one replaces the other. Their
Mock and Meiju data directories are separate. Neither is a store release:
release-mode compilation still uses a local development signing key.

For an isolated SDK override on Linux/macOS without legacy `~/.flutter_settings`,
create a temporary directory containing a `settings` JSON file with `android-sdk`
and `jdk-dir` paths, then set `XDG_CONFIG_HOME` to that directory only for the build
command. Set `ANDROID_HOME` and `ANDROID_SDK_ROOT` to the same SDK. This avoids
changing global Flutter settings. Verify the generated `android/local.properties`
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

The repository verifier additionally rejects incomplete extra ABIs and an embedded
integration-test plugin, and writes an artifact checksum/provenance report:

```sh
python3 scripts/verify_flutter_apk.py --apk /absolute/homeverb.apk \
  --sdk "$HOMEVERB_ANDROID_SDK" --java-home "$HOMEVERB_JAVA_HOME" \
  --adapter homeverb.mock --report /absolute/homeverb.json
```

Use `--adapter midea.meiju` for the explicit read-only build. The adapter value
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
