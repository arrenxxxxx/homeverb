# HomeVerb Flutter application

The native application embeds Core using native UI contract 1.22.0. The separate visual preview contains fictional Window Light fixtures and never opens Core. The default build opens an isolated Mock profile. Explicit Meiju builds provide protected account connection, local metadata, retrieval/recovery and scoped local removal; Android startup wiring and Mock process journeys are emulator-tested; actual Keystore and physical acceptance remain pending. Cache and history clearing are connected with separate confirmations. Nearby sharing and full-profile deletion remain unavailable. Their preview states are not live features.

## Device controls

Open a supported device after sign-in or retrieval to use its available controls.
There is no session-permission form, extra checkbox, review or revocation menu.
The trusted app prepares all reviewed low-risk control values for the opened
device using its exact saved model and cloud connection. It refreshes only that
account to publish the available actions. Startup lists local metadata only;
there is no periodic refresh, background setup or automatic appliance command.
See [ADR-0065](../../docs/adr/0065-default-app-device-controls.md).

The climate-and-power panel keeps power, mode, fan and temperature operations on
the device page. The shared Window Light materials surround a compact charcoal
temperature/power summary and expanded rows: mode, fan speed, horizontal airflow,
vertical airflow, then other settings. Only actual independently settable actions
with locally permitted values on the selected route appear. Session-grant inspection
on device opening is local; saved action metadata alone cannot expose a button.
Mode activation discloses its power-on effect before a direct selection.

Each airflow axis adapts independently: a known boolean swing setting uses a switch,
fixed positions use only the actual enum or numeric bounds, and unsupported choices
are absent. Missing boolean observations require explicit On/Off choices. Numeric
fan speed uses the described step; enum codes remain named presets without guessed
ordering or percentages. Sparse exact grants use explicit choices. Display light,
button sound and comfort settings follow the same descriptor and execution path.
Empty groups disappear. Coupled/unfamiliar setters keep supported explicit forms;
read-only observations remain distinct from actionable controls.

The climate page omits the reading-time footer and the trailing Refresh State,
Device information and Recent Operations buttons. Settings retains operation
history; command results still appear inline when available.

After a command, one bounded foreground read updates observations without rewriting
its receipt or retrying the write. Navigation, device/component changes and background
transitions prevent late reads. The outcome is shown inline with an optional View
Details action; quick card power stays on the home page with a local status message.

Account details show nullable historical complete-inventory time. Inspect Saved
Inventory exposes per-binding duplicate record inspection, including unpublished
bindings; it neither merges devices nor transfers authority. Query errors remain
errors, and navigation/cancellation or backgrounding fences late responses.

## Account integration

The default adapter remains `homeverb.mock`. An explicit Linux development build
can select the Meiju app host after reviewing the [native contract](../../docs/native-ui-contract.md):

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
unavailable system storage is an error, never a plaintext fallback. Account connection does not send appliance commands. Opening a supported device
automatically prepares its available controls.

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

The explicit Mock host permits fictional writes only. Native metadata and presentation preferences live in the OS application support directory. Opening a page does not discover or refresh devices. Local Data offers cache/history cleanup after reviewing each scope. History clearing also removes uncertain receipt details while retaining request-ID replay protection. Interrupted cleanup requires local reloading before an explicit retry; the UI never deletes database files. Use Add Device to discover the Mock appliance. Other control layouts retain Refresh State where available; the climate page omits that footer. Device menu > Select Devices offers explicit batch power actions; results remain per item. Unsupported or ambiguous capabilities fail admission before execution.

The isolated visual preview runs independently:

```sh
fvm flutter run -d chrome -t lib/main_preview.dart --dart-define=PREVIEW_SCREEN=C01
fvm flutter build web -t lib/main_preview.dart
```

For a stable local URL, run `fvm flutter run -d web-server --web-hostname
127.0.0.1 --web-port 5273 -t lib/main_preview.dart
--dart-define=PREVIEW_SCREEN=C01` and open `http://127.0.0.1:5273` in a browser.
Keep that development process running; `r` reloads source changes. A browser
refresh also resets the fictional interaction state.

`C01` now reuses the production `ClimateControls` widget with fictional descriptors
and memory-only preferences. Its preview selector switches between fixed-position
airflow, swing-only airflow and no airflow controls. Temperature, mode, fan and
other settings edit only that fixture; no Core, credentials, provider requests or
execution receipts are involved. These scenarios do not add adapter support.
The preview defaults to Simplified Chinese; `--dart-define=PREVIEW_LOCALE=en-US`
selects English, and the climate fixture also offers a language switch.

`--dart-define=PREVIEW_SCREEN=H04` selects another catalog fixture. The preview is
labelled on screen. Screenshot tests use deterministic status-area fixtures;
those are not phone screenshots. Compare `build/verification/screens/` to the
approved atlas before claiming visual equivalence.

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
fvm flutter test integration_test/native_smoke_test.dart -d DEVICE_ID --no-uninstall
fvm flutter test integration_test/account_smoke_test.dart -d DEVICE_ID --no-uninstall
```

These tests create temporary isolated profiles. The full journey captures its
Flutter surfaces under Dart's temporary directory (`code_cache` on the tested
Android emulator) at `homeverb-native-captures`, using
the device dimensions. It simulates text input and the file picker and injects
lifecycle transitions; it does not verify the OS keyboard, picker or task switcher.
The native smoke test selects an inline climate mode, increases the temperature,
checks verified Mock receipts and hides unsupported fan, airflow and settings rows.
It also renders the detail page in Simplified Chinese; the optional
`HOMEVERB_CLIMATE_CAPTURE` Dart define saves that Flutter surface to an explicit
device-local path. This is native Mock execution and platform-font evidence,
not a provider login or physical-appliance acceptance test. Reinstall the ordinary
APK after process tests; `--no-uninstall` preserves the existing application data.
The account test uses empty local
metadata and fictional input without submitting valid credentials or calling a
vendor. Passing these checks does not establish Android Keystore round trips or
physical-device support. Release signing still uses the development debug key;
these builds are development artifacts, not distributable store releases.

For scrolling investigations, the opt-in profile benchmark records UI/raster
timings from twelve fictional cards without opening Core. See the
[Android performance procedure](../../docs/development/10-flutter-android.md#diagnose-scrolling-performance)
for its command, renderer checks and measurement limits. The shared background
and device tiles have separate paint boundaries; native masks, gradients,
shadows, text, interactions and control authority are retained.
Use an empty AVD and keep `--keep-app-running` on the profiling command: Flutter
drive otherwise uninstalls the package and deletes its local data after the test.

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
This Mock confirmation fixture does not operate a physical appliance. The
production default remains Mock; explicitly selected Meiju uses the default
device-control policy described above.

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


## Default controls and compatibility

The owner has chosen default access to supported device controls. Core still
checks exact device identity, reviewed model constraints, arguments and routing
for each command. The app uses existing trusted startup fields internally and
never imports authority from ordinary preferences or descriptors. Every supported
AC power state, reviewed temperature target, fan code and mode is prepared without
an extra user permission step. No new model support or elevated/high-risk action
is created by this UI policy.

Preparation happens on explicit foreground device opening, including after a
process restart. It is scoped to that device so native grant-size bounds do not
limit the number of saved devices. Switching devices prepares their own values.
A failed or interrupted setup remains an error; opening the device again retries
setup without replaying any appliance command. Accepted/unknown receipts retain
their existing meanings and uncertainty protections.

The live AC panel uses the owner-approved expanded Window Light composition and
direct descriptor-bounded actions. The current Meiju adapter admits power, mode,
fan and eligible temperature actions; swing, fixed positions, display and sound
remain hidden because their native mappings are not yet admitted. This is not
complete Meiju feature parity. See the [function comparison](../../docs/development/10-meiju-ac-parity.md)
for unfinished model-specific controls and evidence requirements.
