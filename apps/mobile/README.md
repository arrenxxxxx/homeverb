# HomeVerb Flutter application

The native application embeds Core using native UI contract 1.22.0. The separate visual preview contains fictional Window Light fixtures and never opens Core. The default build opens an isolated Mock profile. Explicit Meiju builds provide protected account connection, local metadata, retrieval/recovery and scoped local removal; Android startup wiring and Mock process journeys are emulator-tested; actual Keystore and physical acceptance remain pending. Cache and history clearing are connected with separate confirmations. Nearby sharing and full-profile deletion remain unavailable. Their preview states are not live features.

## Current Core inspection

Device Information provides explicit connection-scoped control-option and session
permission inspection. Options never enable controls, and current local grants do
not establish provider authorization. Account details show nullable historical
complete-inventory time. Inspect Saved Inventory exposes per-binding duplicate
record inspection, including unpublished bindings; it neither merges devices nor
transfers authority. Query errors remain errors, and navigation/cancellation or
backgrounding clears results and fences late responses. These operations do not
refresh cloud accounts or open appliance connections.

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
unavailable system storage is an error, never a plaintext fallback. Account connection supplies no write grants; exact session controls require the
separate trusted permission review described below.

On Android, the sign-in page first checks trusted startup registration and the
system's secure-screen-lock/unlocked flags through a bounded local method channel.
A missing PIN/pattern/password shows Open Android Security Settings before any
credential fields. Returning to the foreground rechecks; submission checks again
before calling Core. Backgrounding clears fields and invalidates late checks.
These checks do not read credentials, open the Keystore, contact a provider or
prove that later storage will succeed. Core still validates every operation.
A missing backend, malformed reply or three-second timeout fails closed; there
is no insecure storage fallback. Other platforms retain their Core storage checks.

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

The UI is merged into `main`. See [verification.md](verification.md) for current
and historical evidence. The preserved surface experiment still has no whole-screen
pixel-equality acceptance; prototype references are unchanged.

See the [integrated Android build/install guide](../../docs/development/10-flutter-android.md)
for local APK packaging, profile selection and verification.

## Development

Use FVM and Flutter 3.47.7, pinned in `.fvmrc`. On a Linux or macOS host,
build the native library from the repository root before running the Flutter
tests. Omit `--offline` on the first build so Cargo can obtain dependencies.

```sh
cargo build -p homeverb-ffi --locked
cd apps/mobile
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
Python. Run `python3 tool/generate_text_metrics.py` to regenerate baseline-relative
ink bounds for the bundled Inter Regular and SemiBold control styles. Buttons,
row labels, tabs and codes use those bounds with the native painted baseline to
center visible ascenders and descenders. Text, wrapping and semantics stay native.
Unknown glyphs and system bold/spacing overrides retain ordinary native layout.
High contrast retains Flutter's existing solid outlined controls.

The shared navigation and control glyphs use a small tintable atlas exported from
the approved prototype's drawing methods. Regenerate it from this directory with
`python3 tool/generate_glyph_assets.py`. The generator loads only those methods;
it does not render or update reference pages. The masks contain no labels or
whole controls. Flutter retains native hit targets, focus, selection and semantics,
and tints the same masks for high contrast. Other sizes keep the vector fallback.

Build and run a Linux native application from `apps/mobile` after the steps above:

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
- Described scalar actions use typed parameter forms, exact bounds/steps/enums and explicit submission. Read-only and unsupported actions remain unavailable. Admitted elevated actions use the separate trusted confirmation flow below; high-risk actions remain unavailable. Isolated non-climate fixtures do not register additional native adapters.
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

See [verification.md](verification.md) for executed checks and remaining acceptance gates. All 82 native-widget preview states render and all 201 simulated atlas event branches are clicked separately from the real Core tests. The actual Linux Flutter process also discovers, reads and writes the fictional Mock appliance through FFI. Screenshot equality has **not passed**; do not describe the application as a pixel-identical or phone-verified delivery.

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

## Internationalization

The app follows LocalSend's slang pattern: generated typed translations,
`TranslationProvider`, `LocaleSettings`, Material localization delegates and a
persisted nullable locale. Settings > Language offers System default, English
(`en-US`) and Simplified Chinese (`zh-CN`). A manual choice applies immediately
and survives process restart; System default follows OS locale changes, with
English as the fallback for unsupported languages.

The canonical catalogs are `packages/presentation/locales/en-US.json` and
`packages/presentation/locales/zh-CN.json`. Only the latter contains authored
Chinese text. From the repository root, after `fvm flutter pub get` in this app:

```sh
python3 scripts/generate_flutter_i18n.py --generate
```

The script checks key/placeholder parity, stages slang input under `.dart_tool`,
generates typed Dart translations and the legacy presentation-token bridge, and
synchronizes the existing English approval asset. Generated Dart uses Unicode
escapes; edit the canonical catalogs rather than generated files. New widgets
can use `translationsOf(context)` typed getters. Data-driven labels use `Copy`
and explicit `UiMessage` parameters. Wrap user/provider names in `LiteralText`:
localization must never change a device/account/room name, public identifier,
command argument, error code, receipt or action callback token. Known errors use
catalog explanations; unknown errors show their code without raw provider text.
Evidence timestamps retain their original representation.

Run `fvm flutter test test/localization_test.dart` for language switching,
persistence, system fallback, narrow enlarged-text layouts and receipt/error
semantics. Source and Android runtime results are recorded in
[verification.md](verification.md). Localization does not establish successful
provider login, secure-store availability or physical appliance support.


## Allow exact real-device controls

Meiju starts read-only on each process launch. After protected sign-in and device
retrieval, open Device Information and Review control options for its cloud
connection. Select exact supported values, review them, then choose Allow Selected
Controls. Mode activation explicitly includes power-on. Consent changes only
session startup authority; it does not operate the appliance.

Choose Retrieve Control Capabilities explicitly to refresh the account and
publish admitted actions, then return to the device controls. Revoke Session
Controls removes all session write permissions without erasing retained metadata
or receipts. A cancelled/failed reopen offers Reopen Read-only. Permissions are
not saved across process restarts. Physical effects and provider acceptance must
be checked separately; accepted and unknown results must never trigger a replay.
