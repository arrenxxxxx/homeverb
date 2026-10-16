# HomeVerb Flutter application

The native application embeds Core using native UI contract 1.14.1. The separate visual preview contains fictional Window Light fixtures and never opens Core. The default build opens an isolated Mock profile. Explicit Meiju builds provide protected account connection, local metadata, retrieval/recovery and scoped local removal; Android startup wiring and Mock process journeys are emulator-tested; actual Keystore and physical acceptance remain pending. Cache and history clearing are connected with separate confirmations. Nearby sharing and full-profile deletion remain unavailable. Their preview states are not live features.

## Account integration

The default adapter remains `homeverb.mock`. An explicit Linux development build
can select the read-only Meiju session after reviewing the [native contract](../../docs/native-ui-contract.md):

```sh
fvm flutter run -d linux --dart-define=HOMEVERB_ADAPTER=midea.meiju
```

This uses a separate app-private `native-meiju` directory, never the default Mock
`native` profile or an existing CLI profile. Opening the app lists local metadata
only. Platforms > Add Platform > Meiju previews the mainland account flow before
protected sign-in. The native connection automatically retrieves devices once;
partial/failed retrieval remains distinct from authentication and storage success.
Explicit Retrieve Devices uses the saved session without another login. Pending
accounts offer recovery or, with API 1.8+, sign-in to the original account.

Credential controllers clear on submission, backgrounding and disposal. Owned
managed byte arrays and native allocations clear on every wrapper return path.
Only nonsecret metadata enters JSON; credentials use the separate native function.
This does not guarantee erasure of keyboard, OS or managed-string copies. Locked or
unavailable system storage is an error, never a plaintext fallback. No real-device
write grants are supplied by this UI.

Account details offer explicit Inspect Saved Inventory when Core advertises the
method. It reads only the selected account's protected local bindings, distinguishes
listed/not-listed/unconfirmed membership, and joins a card only when both device
and connection IDs match. Historical and unpublished bindings remain visible;
inspection neither creates cards nor grants control. Empty results are not proof
of an empty cloud account, and query failures never fall back to cloud refresh.
Navigation, cancellation and backgrounding clear inspection results and discard
late replies. Use Retrieve Devices explicitly to refresh the account inventory.

Account removal displays the exact local card scope, including entries whose
metadata is unavailable, before confirmation. Pending cleanup and changed scope
require explicit review/retry. Other installations and provider authorization are
unaffected. Duplicate account labels include local identity suffixes. Unsupported
retrieved models remain visible without reads or fabricated controls.

Real account/secure-store and physical-device acceptance of these Flutter flows
remains pending; the account tests use isolated fixtures and empty native profiles.
Nearby sharing, provider revocation and full-profile deletion are unavailable.

## Development

Use FVM 4.3.1 and Flutter 3.47.7, pinned in `.fvmrc`.

```sh
fvm install
fvm flutter pub get
fvm flutter analyze
fvm flutter test --reporter expanded
```

Window Light button faces are four small local decorative assets; labels, focus,
semantics and interaction remain native Flutter controls. Their 2x corner artwork
preserves the prototype's inclusive pixel edges without stretching the corners.
Regenerate them from this directory with `python3 tool/generate_button_assets.py`
(Pillow required). Assets are committed, so normal Flutter builds do not require
Python. High contrast retains Flutter's existing solid outlined controls.

Build the library from the repository root before running native tests:

```sh
cargo build -p homeverb-ffi --offline --locked
```

Build and run a Linux native application after building the library:

```sh
fvm flutter build linux --release
build/linux/x64/release/bundle/homeverb_mobile
```

The Linux CMake install step packages `target/debug/libhomeverb_ffi.so` in the bundle's `lib` directory. Set CMake `HOMEVERB_NATIVE_LIBRARY` to use another build. Keep the whole bundle together. A development override is available through `--dart-define=HOMEVERB_LIBRARY=/absolute/path/to/libhomeverb_ffi.so`. Linux builds require Flutter's standard CMake, Ninja, Clang and GTK development dependencies.

The explicit Mock host permits fictional writes only. Native metadata and presentation preferences live in the OS application support directory. Opening a page does not discover or refresh devices. Local Data offers cache/history cleanup after reviewing each scope. History clearing also removes uncertain receipt details while retaining request-ID replay protection. Interrupted cleanup requires local reloading before an explicit retry; the UI never deletes database files. Use Add Device to discover the Mock appliance and Refresh State to read it. Device menu > Select Devices offers explicit batch power actions; results remain per item. Unsupported or ambiguous capabilities fail admission before execution.

The isolated visual preview runs independently:

```sh
fvm flutter run -d chrome -t lib/main_preview.dart
fvm flutter build web -t lib/main_preview.dart
```

`--dart-define=PREVIEW_SCREEN=H04` selects a fixture. The preview is labelled on screen. Screenshot tests use deterministic status-area fixtures; those are not phone screenshots. Compare `build/verification/screens/` to the approved atlas before claiming visual equivalence.

## Native boundary

- Blocking open/call/close functions execute in isolates. Cancellation and foreground gating use a separate binding on the UI isolate.
- Every call has a fresh operation ID; every write has a separate durable request ID. The UI plans before executing and never automatically replays a write.
- Execution envelopes retain errors, evidence and nested outcomes. Requested values never replace observations. Explicit refresh only reads.
- Core owns metadata. Presentation preferences contain only skin, root, sharing display name and route choices. Save failures retain previous preferences.
- Component selection uses exact IDs and each component's capability references. Observations are cached separately per device/component; a secondary compartment never replaces the primary card observation. Unsupported temperature actions remain read-only.
- Components with separate climate scopes share a compact overview. Opening reads local caches only; explicit Refresh State reads the components. Targets always carry an exact component ID, and partial reads retain independent observations. Cancelling or backgrounding stops the read sequence.
- Described scalar actions use typed parameter forms, exact bounds/steps/enums and explicit submission. Read-only, unsupported and trusted-approval actions remain unavailable. Isolated non-climate fixtures do not register additional native adapters.
- Single-family light/fan/media/sensor/cycle components use dedicated Window Light controls. Sliders and fan steppers use the described range and step; mixed or coupled actions retain explicit forms. Missing and stale observations remain labelled, and high-risk actions remain unavailable.
- Diagnostics show the loaded report in summary cards. Select the review note to inspect the complete report, including all retained receipts. Export saves that same snapshot; failed or cancelled reloads cannot expose an old or late report. Android uses the system Create Document picker restricted to local providers; cancellation writes nothing. The picker grants access to the selected document without broad storage permissions. Desktop builds use the file selector. iOS document export remains unverified.
- Modal focus stays above the inactive backdrop; Tab/Enter and Escape support keyboard navigation. Fields, sliders and switches expose their purpose. Busy activity becomes static when the system requests reduced motion. System high contrast uses solid surfaces, stronger outlines and larger semibold text; disabling it restores the original Window Light appearance.

## Android development build

The Android application includes the matching local Core AAR for `arm64-v8a` and
`x86_64`; 32-bit targets are excluded. Build the library from the same checkout
before assembling an APK. SDK/NDK paths are local environment settings, not
repository dependencies. The app requires Android API 28 or later.

From the repository root, with the two Rust Android targets installed:

```sh
python3 scripts/build_android_native.py \
  --ndk /absolute/android-ndk-r28c \
  --android-jar /absolute/android-sdk/platforms/android-36/android.jar \
  --java-home /absolute/jdk-21 \
  --abi arm64-v8a --abi x86_64
cd apps/mobile
fvm flutter build apk --debug --target-platform android-arm64,android-x64
fvm flutter build apk --release --target-platform android-arm64,android-x64
```

Run Flutter builds and integration tests sequentially in a checkout. When switching
between test/debug and release builds, keep dependency/tooling generation enabled:
`--no-pub` skips release plugin regeneration in this pinned Flutter version and can
leave the development-only integration-test plugin in the generated registrant.
Do not edit the generated registrant or add the test plugin to production dependencies.

Configure the Android SDK through the local `android/local.properties` or standard
SDK environment. Application startup loads Core and installs the Android credential
backend before Flutter starts. Installation does not read credentials or contact a
provider; Core checks actual storage availability on each protected operation.
Loading/installation failure never enables a plaintext fallback. Application data
is excluded from cloud backup and device transfer, because credentials require the
original Keystore. Internet permission supports explicit foreground operations.
The default build still uses Mock; the separate opt-in Meiju profile and write
restrictions described below are unchanged.

Run the process tests on a connected Android device or emulator:

```sh
fvm flutter test integration_test/native_journeys_test.dart -d DEVICE_ID --no-uninstall
fvm flutter test integration_test/native_smoke_test.dart -d DEVICE_ID
fvm flutter test integration_test/account_smoke_test.dart -d DEVICE_ID
```

These tests create temporary isolated profiles. The full journey captures its
Flutter surfaces under Dart's temporary directory (`code_cache` on the tested
Android emulator) at `homeverb-native-captures`, using
the device dimensions. It simulates text input and the file picker and injects
lifecycle transitions; it does not verify the OS keyboard, picker or task switcher.
The account test uses empty local
metadata and fictional input without submitting valid credentials or calling a
vendor. Passing these checks does not establish Android Keystore round trips or
physical-device support. Release signing still uses the development debug key;
these builds are development artifacts, not distributable store releases.

## Verification boundary

See [verification.md](verification.md) for executed checks and remaining acceptance gates. All 82 native-widget preview states render and all 198 simulated atlas event branches are clicked separately from the real Core tests. The actual Linux Flutter process also discovers, reads and writes the fictional Mock appliance through FFI. Screenshot equality has **not passed**; do not describe the application as a pixel-identical or phone-verified delivery.

Generate the local comparison report from the repository root after screenshot tests:

```sh
python3 apps/mobile/tool/compare_screens.py --require-exact
```

The command generates `output/flutter-verification/index.html`, reference/actual/difference images and JSON metrics. `--require-exact` exits nonzero while any pixel differs. The generated image collection is ignored by Git; the reference remains unchanged.

The native widget click test also captures its actual Mock-backed views using the bundled fonts, simulated phone insets and strict pointer hit checks. Build a separate gallery after that test passes:

```sh
python3 apps/mobile/tool/native_gallery.py
```

The generated `output/flutter-verification/native-gallery.html` labels clicks, drag gestures and direct test navigation separately. Its reference column uses different sample data; this gallery does not replace strict pixel comparison or physical-phone acceptance.

Android APK packaging and x86_64 emulator Mock journeys are verified separately from actual Keystore, physical-phone lifecycle, iOS packaging, accessibility and performance. The default build uses Mock; Meiju requires an explicit opt-in. Project release licensing remains undecided. Bundled Inter fonts use the adjacent SIL Open Font License.


## Trusted single-command confirmation

Native API 1.13 or later enables the separate prepare/execute/discard bridge for
an elevated action already admitted by Core. The described control form requires
an explicit connection, then displays Core's immutable device/component, exact
arguments, selected connection, action effects and expiry. Only Confirm Command
consumes that preview; ordinary execution still refuses elevated actions. Cancel,
navigation, timeout and leaving the foreground revoke pending consent. High-risk
actions remain unavailable. Interrupted replies retain the original request ID
for an explicit receipt lookup, without automatic replay.

The `approval-required` Mock scenario provides the fictional development journey.
Run `fvm flutter test test/approval_test.dart` or the device process suite
`fvm flutter test integration_test/approval_smoke_test.dart -d DEVICE_ID`.
This does not enable Meiju startup grants or operate a physical appliance. The
production default remains normal Mock, or explicitly selected read-only Meiju.

Confirmation labels use the shared English presentation catalog. After changing
that catalog, copy `../../packages/presentation/locales/en-US.json` to
`assets/en-US.json` from this directory; the approval suite checks exact equality.
Bundling a local asset keeps Android paths valid. Full app locale wiring is still
pending; adding translated catalog entries alone is not localization acceptance.
