# Flutter implementation verification

## Light device cards: October 17, 2026

Device cards use white surfaces for observed power on and slightly gray-white
(`#f0f1ee`) for off or unknown power. Text is dark, with a subtle outline; high
contrast keeps white surfaces and black outlines. The shared Surface accepts an
explicit fill without its baked face or gradient overriding it. Other surfaces,
including the climate summary, retain their existing materials. Pending operations
retain the previous observed surface color until a read supplies new evidence.

Executed from `apps/mobile` using pinned Flutter 3.47.7:

- `fvm flutter analyze`: passed with no issues.
- `fvm dart format --output=none --set-exit-if-changed lib test`: 61 files,
  unchanged.
- `fvm flutter test test/device_cards_test.dart test/accessibility_test.dart
  test/contrast_test.dart test/climate_rows_test.dart
  --dart-define=HOMEVERB_CARD_CAPTURE=/private/tmp/homeverb-light-device-cards.png
  --reporter expanded`: 24 passed. Existing card assertions now check actual fills,
  readable text contrast, unknown state and unchanged color throughout a pending
  command/read. Narrow/enlarged-text and full-atlas high-contrast checks passed.
  The fictional on/off capture was inspected with decoded appliance artwork at
  its display size; refrigerator and dishwasher silhouettes remain distinct on
  light surfaces. This does not establish a physical-device state transition.
- `fvm flutter build apk --release --target-platform android-arm64,android-x64
  --dart-define=HOMEVERB_ADAPTER=midea.meiju`: passed in 34.1 seconds after tests.

Artifact: `output/flutter-verification/android/homeverb-0.2.0-meiju-light-cards.apk`,
70,573,038 bytes, SHA-256
`1f98d9cec89816b81fed67a88e6226d9ace4ffe612a8a06245e85ba84d2befb3`.
The adjacent verifier report records base source `8bc1901` with this phase's
uncommitted changes, unchanged AAR, two complete ABIs, 17 exports per ABI,
development signature and 16 KiB ZIP/Core ELF alignment. Archive checks passed.

In-place `adb -s emulator-5586 install -r` and cold launch succeeded (1694 ms)
on the existing Android 15 arm64 AVD. Local screenshots show white observed-on
cards, gray-white off cards, dark readable text and distinct refrigerator/dishwasher
artwork. The real account, six cards and Chinese preference survived. Inspection
only scrolled the home screen; no control command, uninstall, data reset or test
APK was used. Existing automatic foreground reads remain enabled. The H5 server
remains stopped.

Documentation, contracts (20 schemas, 45 positive/250 negative fixtures), repository
checks and seven shared presentation tests passed. The full Flutter suite, Rust
workspace, physical-phone rendering and physical appliance transitions were not
rerun. Native API 1.22, SQLite 5, protected payload 3, authority, model support and
execution outcomes are unchanged. No push, publication or diagnostic upload occurred.

## Silent foreground state refresh: October 17, 2026

The owner explicitly requested automatic three-second device-state reads while
the App is open and no reminder on failure. [ADR-0067](../../docs/adr/0067-foreground-state-refresh.md)
records this foreground-only exception to the former polling prohibition; the
repository instructions and lifecycle/architecture/product documents now agree.

The production native entry point enables an App-owned timer. Home/search/room
views read supported components in the current filter; controls read the selected
device's components. Initial metadata/cache loading precedes the first three-second
interval. Other views, backgrounding and disposal stop the schedule. Read cycles
never overlap or queue missed ticks, and user operations cancel/drain automatic
reads before using the native session. Lifecycle changes during that drain prevent
a delayed command from starting after resume.

Success updates the normal card/component caches while preserving timestamps,
source and stale markers. Failures preserve earlier observations without loading
UI, banners, snackbars or modal reminders. They do not overwrite manual errors or
execution receipts. The loop never discovers devices, refreshes account inventory,
prepares grants, reauthorizes accounts or replays a write. Every request retains
the existing bounded Core read and route policy.

Executed using pinned Flutter 3.47.7 from `apps/mobile`:

- `fvm flutter analyze`: passed with no issues.
- `fvm dart format --output=none --set-exit-if-changed lib test`: 61 files,
  unchanged.
- `fvm flutter test test/state_refresh_test.dart test/device_cards_test.dart
  test/climate_controls_test.dart test/climate_rows_test.dart
  test/session_permissions_test.dart test/accessibility_test.dart
  test/localization_test.dart test/receipts_test.dart test/account_inventory_test.dart
  test/current_contract_test.dart test/cleanup_test.dart
  test/compartment_overview_test.dart test/organization_test.dart
  test/native_test.dart --reporter expanded`: 80 passed in 21 seconds. Eight new
  virtual-time/controller/widget tests cover cadence, filtered component targets,
  independent silent failures, old snapshot/receipt retention, slow cycles,
  background/resume, late replies, cancellation/drain before a fictional user
  command, cancellation while waiting, non-device navigation and quiet UI. Existing
  native Mock/control/account/cleanup/accessibility regressions also passed.
- `fvm flutter build apk --release --target-platform android-arm64,android-x64
  --dart-define=HOMEVERB_ADAPTER=midea.meiju`: passed in 34.5 seconds, after the
  Flutter tests completed.

The verified artifact is
`output/flutter-verification/android/homeverb-0.2.0-meiju-auto-refresh.apk`,
70,573,038 bytes, SHA-256
`2cb9a4207ede5f8fe9ba1db44b5ed647998cae4f5a436cdd8e424584a0f57267`.
Its adjacent report records base source `b7b87c3` plus these uncommitted changes
(`source_dirty: true`), unchanged AAR
`d43145c239cb13c993198dc31dbc28bedc75f884bc6270f06e682138f3c2fd6b`,
two complete ABIs, 17 exports per ABI, backup exclusion, development signature,
no integration-test plugin and 16 KiB ZIP/Core ELF alignment. Archive checks passed.

`adb -s emulator-5586 install -r` succeeded on the existing host-GPU Android 15
arm64 AVD, preserving the real account, six cards and Chinese preference. Cold
start returned `Status: ok` in 1367 ms. Without pressing Refresh, a local scrolled
screenshot showed the refrigerator card change from Not read/dash to a saved
-18 Celsius observation. The AC cards retained their observations and the
dishwasher still showed Not read at that checkpoint. No automatic loading/error
UI appeared. This proves an unattended native read updated a card, not success
for every appliance, exact provider timing or physical freshness. No control
command, inventory-refresh button, uninstall, data reset or test APK was used.
The H5 server remains stopped.

Documentation (141 Markdown files), contract validation (20 schemas,
45 positive/250 negative fixtures), repository/commit checks and seven shared
presentation tests passed. The full Flutter visual suite, Rust workspace, native
Android smoke, physical-phone timing and provider-rate acceptance were not rerun.
Native API 1.22.0, SQLite 5, protected payload 3, control authority, adapter support
and execution outcomes are unchanged. No push, upload or publication occurred.

## Home power feedback and appliance artwork: October 17, 2026

Native home/search/room cards now use observed power colors and separate icons:
orange on, gray off, and neutral disabled unknown state. The affected shortcut
shows the existing three-stroke pulse throughout device preparation, execution and
the one foreground follow-up read. Duplicate submissions are blocked, observations
remain unchanged until a read, and routine accepted/verified home snackbars are
removed. Uncertain/failed outcome messages and original history remain available.

Card loading locally inspects reviewed control options or exact native-session
grants, respects the selected transport and reads cached observations only. It
does not prepare authority, refresh accounts, contact appliances or fall through
to live reads. The actual tap prepares/rechecks the selected session before the
existing Core plan/execute path. Read-only/missing/ambiguous whole-device power
actions have no shortcut; refrigerator compartment power is not an appliance
switch. Foreign replies and lifecycle changes cannot install late choices.

The owner-invoked imagegen workflow generated separate transparent refrigerator
and dishwasher product images, each followed by an extraction refinement. Both
selected RGBA PNGs are 1254-square, preserved under `assets/` and decoded at bounded
card dimensions. Decorative selection uses component/capability metadata, never
aliases or brands; unknown types have a generic icon. The [artwork record](artwork.md)
contains the built-in generation mode, exact generation prompts and refinement
scope. Fictional rendered cards and the actual emulator home were inspected at
their display sizes; full-source matte-edge remnants are documented there.

Executed from `apps/mobile` using pinned Flutter 3.47.7:

- `fvm flutter analyze`: passed with no issues.
- `fvm dart format --output=none --set-exit-if-changed lib test`: 60 files,
  unchanged.
- `fvm flutter test test/device_cards_test.dart test/climate_controls_test.dart
  test/climate_rows_test.dart test/session_permissions_test.dart
  test/accessibility_test.dart test/localization_test.dart test/receipts_test.dart
  test/account_inventory_test.dart test/current_contract_test.dart
  test/cleanup_test.dart test/compartment_overview_test.dart
  test/organization_test.dart --reporter expanded`: 60 passed. This run included
  eight initial card tests plus existing control, lifecycle, local account,
  uncertainty, cleanup, compartment and accessibility regressions.
- `fvm flutter test test/device_cards_test.dart test/native_test.dart
  --reporter expanded`: 20 passed after adding the late/foreign inspection case
  and a real native Mock quick-power grant assertion. Nine card tests and eleven
  native tests use fictional material and isolated profiles, without providers.
- A card-test run with
  `--dart-define=HOMEVERB_CARD_CAPTURE=/private/tmp/homeverb-device-cards.png`
  produced a locally inspected image after actual font loading and image decoding.
  The card tests cover 390-wide normal text and 320-wide 2x text/high contrast,
  absent unsupported shortcuts, unchanged pending state and one command/read.
- `fvm flutter build apk --release --target-platform android-arm64,android-x64
  --dart-define=HOMEVERB_ADAPTER=midea.meiju`: passed in 18.9 seconds. An initial
  overlapping test/build used test-generated plugin registration and failed;
  the successful build ran after tests completed. Archive inspection excludes
  the integration-test plugin from the delivery APK.

Artifact:
`output/flutter-verification/android/homeverb-0.2.0-meiju-home-cards.apk`,
70,573,038 bytes, SHA-256
`a251cd5e11cf5abc4896ad81fccf241f1718b3bfaf200e1f731a71f7e0ae515f`.
Its adjacent verifier report records base source `fd42ec3` plus these uncommitted
changes (`source_dirty: true`), unchanged native AAR
`d43145c239cb13c993198dc31dbc28bedc75f884bc6270f06e682138f3c2fd6b`,
two complete ABIs, 17 native exports per ABI, backup exclusion, development
signature and 16 KiB ZIP/Core ELF alignment. All archive checks passed.

`adb -s emulator-5586 install -r` succeeded on the existing host-GPU Android 15
arm64 AVD. Cold start returned `Status: ok` in 1236 ms. Six saved cards and Chinese
locale survived. Local screenshots show four AC shortcuts in their cached off
state, distinct refrigerator/dishwasher images and no power shortcut on those two
appliances. The home screen was scrolled only; no appliance command, provider
refresh, uninstall, data reset or integration-test APK was used. Cached state is
not a fresh or physical-state verification. The H5 server remains stopped.

Seven shared presentation tests, docs (140 Markdown files), contracts (20 schemas,
45 positive/250 negative fixtures), repository and existing commit checks passed;
the contract check used the temporary environment with declared dependencies.
The full Flutter visual suite, Rust workspace, native Android smoke and physical
phone/appliance acceptance were not rerun for this Dart/assets-only phase.
Native API 1.22.0, SQLite 5, protected payload 3, authority, routing, adapter
support and outcome semantics are unchanged. No push, upload or publication
occurred.

## Power state and activity feedback: October 17, 2026

The climate power control now distinguishes observed on/off state using orange
and gray, separate icons and Powered on/Powered off labels. Its tooltip names the
opposite action. Commands retain their prior observation until the existing
follow-up read returns. Accepted/verified receipts no longer create a bottom
card; failed, unknown, partial and unrecognized results keep their explanation
and details, and Settings history retains the original receipts.

Indeterminate operations use a small three-stroke pulse instead of a page-edge
bar. Climate commands place it in the power button, temperature heading or
affected row and disable duplicate submission across the command and follow-up
read. Other pages, sheets and dialogs use the same pulse beside an activity
caption within the content. Its isolated painter does not rebuild the page per
frame. Reduced motion is static; backgrounding and inactive subtrees stop the
animation. No percentage, optimistic state or completion claim is inferred.

Executed from `apps/mobile` with pinned Flutter 3.47.7:

- `fvm flutter analyze`: passed with no issues after using the pinned ticker API.
- `fvm dart format --output=none --set-exit-if-changed lib test`: 58 files,
  unchanged.
- `fvm flutter test test/climate_controls_test.dart test/climate_rows_test.dart
  test/preview_climate_test.dart test/accessibility_test.dart
  test/localization_test.dart test/receipts_test.dart --reporter expanded`:
  30 tests passed. Gated command/read fixtures verify local activity, unchanged
  observations and selections, duplicate prevention, colors after read-back and
  accepted-card removal; existing uncertainty, capability, lifecycle and history
  regressions also pass.
- `fvm flutter test test/accessibility_test.dart --reporter expanded`: six passed
  after adding actual background/resume/disposal animation checks. Reduced-motion
  semantics retain an activity label and no percentage.
- `fvm flutter build apk --release --target-platform android-arm64,android-x64
  --dart-define=HOMEVERB_ADAPTER=midea.meiju`: passed in 33.5 seconds.

The current artifact is
`output/flutter-verification/android/homeverb-0.2.0-meiju-feedback.apk`,
68,267,876 bytes, SHA-256
`94ce91019d43672d8562f6459d9ed94d55da758f472d7a10f623d594401780f8`.
Its adjacent verifier report records base source `be6a864` with these uncommitted
Dart/catalog/docs changes (`source_dirty: true`) and unchanged native AAR
`d43145c239cb13c993198dc31dbc28bedc75f884bc6270f06e682138f3c2fd6b`.
Signature, exactly two complete ABIs, 17 exports per ABI, backup exclusion and
16 KiB ZIP/Core ELF alignment passed. Signing remains local development only.

`adb -s emulator-5586 install -r` succeeded on the existing host-GPU Android 15
arm64 AVD. Cold start returned `Status: ok` in 1512 ms. The reconnected account's
cards and Chinese locale survived the upgrade. Reopening the same wall AC showed
its saved 27 Celsius target and gray Powered off control, with supported mode/fan
rows and no routine result card. The existing device-opening flow prepares exact
controls through its bounded account refresh and reads cached observations;
this is not a fresh appliance-state or actuation verification. No appliance
command, uninstall, data reset or profiling/integration-test APK was used.
The H5 server remains stopped.

Seven shared presentation tests and documentation, contract (20 schemas,
45 positive/250 negative fixtures), repository and commit checks passed. The
contract check used the declared requirements in a temporary Python environment.
The full Flutter visual suite, Rust workspace, native Android smoke, physical
phone and new real-control journeys were not rerun for this Dart-only phase.
Native API 1.22.0, SQLite 5, protected payload 3, authority, routing, adapter
support and outcome semantics are unchanged. No push, upload or publication
occurred.

## Native runtime handoff and language-button removal: October 17, 2026

The owner requested removal of the climate fixture's language button and actual
account/device operation instead of fictional preview data. The extra button is
removed; the native app retains language selection in Settings. Analysis, pinned
format checking and both preview widget tests passed. The running H5 entry point
still has no native Core or real account/device implementation. No HTTP control
bridge, credential copy, browser secret store or new adapter path was added.

The current ordinary Meiju release APK built successfully in 33.1 seconds with
both Android ABIs and production plugin generation. The inspected artifact is
`output/flutter-verification/android/homeverb-0.2.0-meiju-live.apk`, 68,202,196
bytes, SHA-256
`5870bfce29ce8c61b59bf9c0bfb43e14d0ead1a66d2dbb3e5be3c33763d3ebb3`.
The adjacent verifier report records base source `d99ccc8` with the uncommitted
preview-button/README changes (`source_dirty: true`), unchanged native AAR,
17 exports per ABI, development signature and 16 KiB alignment. Archive checks
passed; they are not runtime or physical-device acceptance.

The existing Android 15 arm64 AVD started with its same data partition and
`-gpu host -no-snapshot -no-boot-anim`; SurfaceFlinger confirmed the Apple M1 Pro
host renderer. `adb install -r` succeeded without uninstalling or clearing data.
At this handoff Android user 0 was `RUNNING_LOCKED` after reboot. Activity startup was refused
while credential-encrypted user storage was locked; actual native startup and
account/device acceptance remain blocked until the owner unlocks the emulator.
The screen was woken for the owner, without supplying or changing a lock secret.
No provider credential was accessed or provider read/appliance command sent,
and no upload, push or publication occurred. The owner subsequently unlocked the
emulator; the App launched successfully and initially had no connected accounts.
The later reconnection and in-place update are recorded in the power-state phase
above. The H5 fixture does not provide real account/device operation.

## Climate footer removal: October 17, 2026

Following the owner's browser annotation, the shared climate view removes the
reading-time/no-reading footer and its trailing Refresh State, Device information
and Recent Operations buttons. This applies to the native widget and H5 fixture.
Capability rows, additional settings and command-result handling are unchanged.
The native smoke test now seeds its first read through the controller because
the removed refresh button is no longer part of this screen's journey.

`fvm flutter analyze`, the pinned format check and sixteen focused climate/preview
tests passed. The H5 development server reloaded the shared widget; Chrome's
scrolled view visually confirmed that the three buttons and reading time are gone.
The revised
native smoke test was analyzed but not rerun on Android; no APK was rebuilt or
installed in this phase. Native API, grants, storage and adapter support are
unchanged. No provider request, real-device command or external publication was
performed.

## H5 development preview: October 17, 2026

The owner requested a browser view for interactive UI iteration. The pinned
Flutter Web development server now runs the isolated `main_preview.dart` entry
point at `http://127.0.0.1:5273`, initially showing `C01` in Simplified Chinese.
The [preview procedure](README.md#development) documents the exact command.
The climate fixture reuses production `ClimateControls`, including descriptor
filtering and the shared Window Light surfaces. Its selector supplies fictional
fixed-position, swing-only or absent airflow actions. Controls update memory-only
fixture observations; no native session, account, secret store, provider request,
appliance command or execution receipt is created. The HTTP listener only serves
development UI resources and does not expose a device-control API.

The initial Web compile exposed an unintended `dart:ffi` dependency through
localization. Client interfaces/errors now live in portable `client_contract.dart`
and remain re-exported by `native_client.dart`. Production session inspection uses
the native-session interface, preserving grant/route filtering. Trusted native
previews remain private, deeply immutable and bound to their original handle's
one-use registry; external implementations are explicitly rejected. File-backed
preferences retain their required File constructor and atomic save behavior;
only the explicit preview constructor uses memory.

Static analysis and a release Web build passed. The build command was
`fvm flutter build web -t lib/main_preview.dart --dart-define=PREVIEW_SCREEN=C01
--no-web-resources-cdn`; the SDK reported a missing Cupertino font-family warning.
Forty focused Flutter tests passed across native calls, current contract, climate
controls/rows, localization/preferences, approvals and the new preview tests.
The four approval cases passed again after adding the external-preview rejection
assertion. The preview tests exercise a fictional temperature/power edit and
capability-dependent hiding of positions and axes. Chrome rendered the Chinese
screen at 430 by 932; its keyboard temperature control changed the fixture from
24 to 24.5 Celsius. A development hot restart reported a disposed-engine-view
assertion; a full browser refresh recovered. This is not a full visual-suite,
Android/iOS rebuild, performance or real-provider acceptance claim. Native API
1.22.0, SQLite 5, protected payload 3 and reviewed model support are unchanged.
No publication or diagnostics upload occurred.

## Scrolling investigation: October 17, 2026

The owner reported severe scrolling stutter on the current emulator. The selected
Android 15 / API 35 arm64 AVD was launched with `-gpu swiftshader`. SurfaceFlinger
confirmed ANGLE/SwiftShader, and Flutter used Impeller OpenGLES. These are actual
runtime observations, not an inference from debug-build speed. General Android
`gfxinfo` exposed only one wrapper-view frame and was not used as Flutter FPS.

The new opt-in `scroll_performance_test.dart` exercises the production screen,
background and device grid with twelve fictional AC cards. It opens no Core or
account and verifies that scrolling invokes no device action. After decoding and
warming the images, four alternating 600-logical-pixel flings at 1800 logical
pixels/second are measured with Flutter's frame-timing collector. The host driver
saves reports. Executed commands used `fvm flutter drive --profile --no-dds`,
`test_driver/performance_driver.dart`, this test target and
`HOMEVERB_PERFORMANCE=true`; the viewport remained 1080 by 2400 at 420 dpi.

| Configuration | Frames sampled | Mean UI / p90 (ms) | Mean raster / p90 (ms) | Raster p99 (ms) |
|---|---:|---:|---:|---:|
| Original painting, forced SwiftShader | 72 | 3.506 / 6.877 | 166.484 / 216.977 | 265.086 |
| Separate background/tile paint boundaries, SwiftShader | 72 | 1.284 / 3.508 | 155.291 / 195.323 | 248.276 |
| Same paint boundaries, host GPU | 162 | 0.633 / 1.206 | 13.966 / 17.829 | 27.001 |

All three recorded profile runs passed. Separating painting alone did not resolve
the software raster bottleneck. Restarting the same data partition with `-gpu
host -no-snapshot -no-boot-anim` selected the Apple M1 Pro host renderer; the AVD's
local GPU-enabled/mode settings now also select host graphics. The restart used
no `-wipe-data` or snapshot restore; the profiling-tool uninstall described below
later removed the original package data. The renderer change accounts for most of the
improvement and must not be attributed solely to the Dart change. The remaining
tail exceeds a 60-Hz frame budget; no locked-60-FPS or physical-phone claim follows
from average raster time. No layer/picture raster-cache hits were reported.

Production changes only isolate the fixed decorative background and each device
tile with `RepaintBoundary`. Masks, gradient shaders, shadows, layout and native
semantics are retained. There is no new polling, network work, control grant or
execution behavior. Native API 1.22.0, SQLite 5 and protected payload 3 are unchanged.
The profile command and GPU selection are documented in the Android app guide.
Local reports are `output/flutter-verification/android/scroll-before.json`,
`scroll-boundaries.json` and `scroll-host-gpu.json`. These are ignored artifacts.
`fvm flutter analyze`, the pinned Dart format check, documentation/repository
validators and `git diff --check` passed. The complete `fvm flutter test --reporter
expanded` passed 115 tests and failed the same four known macOS glyph-pixel cases:
button labels, editable-field alignment and two control-label checks. References
and tolerances are unchanged; this is not an entirely passing visual suite.

**Data-loss incident:** the profiling commands above initially omitted
`--keep-app-running`. Inspection of the pinned Flutter drive service confirmed
that its default completion path stops and uninstalls the package, clearing the
emulator's prior local account/device data. In-place installation and temporary
test fixtures did not prevent that post-test uninstall. The final ordinary APK
opens with empty account/device state; preservation must not be claimed. No
credential was read, provider command sent or appliance changed during profiling.
The owner was informed of the mistake and the need to reconnect the account.
The Android procedure now requires `--keep-app-running` and recommends an empty
AVD. The earlier successful preservation record for `flutter test --no-uninstall`
does not cover `flutter drive`.

### Ordinary APK delivery and retained-driver recheck

The ordinary dual-ABI Meiju release APK was built from clean source
`e2c9e7bdf92f66f0b187bb30661ae443c184e905` with production plugin regeneration:

- Artifact: `output/flutter-verification/android/homeverb-0.2.0-meiju-scroll.apk`.
- Bytes: 68,202,196.
- SHA-256: `59902e69bd60ff7f6c41ac5769f59fe46268a02a6f1d549876687fa10684430c`.
- The APK verifier passed signature, two complete ABIs, 17 Core exports per ABI,
  backup exclusions and 16 KiB ZIP/Core ELF alignment. The adjacent JSON report
  records unchanged native/AAR hashes and the clean source revision. Signing is
  for local development, not store distribution.

The corrected host-GPU profile command was rerun with `--keep-app-running`.
It passed again, reported mean raster 12.884 ms, p90 17.397 ms and p99 20.329 ms,
and explicitly left the application running. A subsequent package lookup
confirmed it remained installed. The additional local report is
`output/flutter-verification/android/scroll-host-gpu-retained.json`.
The ordinary APK was then restored with `adb install -r` and launched. Android
SurfaceFlinger confirms the Apple M1 Pro host renderer. The app opens in Chinese
with empty account/device state after the incident above; account reconnection
and an actual saved-device scrolling review remain blocked. The fictional-grid
profile evidence does not replace those checks. No provider credentials or
physical appliance were used. No push, publication or diagnostic upload occurred.

## Expanded capability-dependent climate rows: October 17, 2026

The owner approved the Window Light row proposal and required every control to
adapt to the selected device's actual operations. The live page restores the
shared background, porcelain groups, charcoal summary and terracotta selection.
Mode, fan, horizontal airflow, vertical airflow and other settings expand in that
order. Empty groups and unsupported controls disappear. A swing-only axis uses a
switch; fixed-position options require an independent action and its exact values.
Numeric fan settings use the described step; enum fan codes remain presets.
Known booleans use compact switches; missing observations use explicit choices.

Production NativeClient sessions inspect current local grants on device opening.
Quick controls intersect complete grants, exact component/action values and the
selected route. Saved descriptors from a prior writable session do not expose
buttons after read-only reopening. Sparse grants never widen into a numeric range.
Unknown grant-schema constraints hide quick controls conservatively; Core remains
the complete planning/execution authority. No permission is read from preferences
or ordinary request arguments. Coupled supported actions retain explicit forms.

Every selection uses the existing in-process plan/execute path. Power, mode, fan
and eligible temperature actions remain on the page and retain their receipt.
One bounded foreground read remains separate from that outcome. There is no
optimistic observation, automatic replay, implicit retry or background work. Old
callbacks and slider releases are invalidated by navigation, component selection
and background/resume. Mode activation discloses its existing power-on effect.

Executed source checks:

- `cargo build -p homeverb-ffi --offline --locked`: passed on this macOS host.
- `python3 scripts/generate_flutter_i18n.py --generate` and `fvm flutter analyze`:
  passed with the pinned Flutter 3.47.7 SDK; English/Chinese catalogs are synchronized.
- Focused climate/row/outcome/session-permission tests: 24 passed before final
  composition refinements. The final complete `fvm flutter test --reporter expanded`
  passed 115 tests and failed only the four existing macOS glyph-pixel tests:
  button-label ink, two control-label cases and editable-field alignment. No
  reference image or tolerance changed. This is not a fully passing visual suite.
- Six new row tests cover axis-specific fallbacks and exact command fields,
  numeric/enum fan differences, missing booleans, plan denial without execution,
  read-only/coupled/ambiguous/conditional refusal, real native read-only reopening,
  row ordering, narrow 200% Chinese text and high contrast. Existing native Mock
  click journeys cover actual FFI writes, receipts, one follow-up read and no replay.
- Seven shared presentation tests passed. Documentation, contract (20 schemas,
  45 positive and 250 negative fixtures), repository and existing commit checks
  passed. `git diff --check` passed.
- The Android AAR was rebuilt from this checkout for arm64-v8a and x86_64; archive
  inspection passed 17 exports per ABI and 16 KiB ELF alignment.

Generated widget captures are under `apps/mobile/build/verification/climate-rows/`:
`main-controls.png`, `settings.png`, `zh-large-text.png` and `high-contrast.png`.
English captures were visually inspected with the actual bundled Inter and
Material Icons fonts. Host Chinese captures assert translated strings and layout,
but the test renderer lacks the platform CJK fallback; they are not packaged
Chinese glyph acceptance. Android runtime acceptance is recorded separately.

Compatibility: native API 1.22.0, SQLite 5, protected payload 3, existing startup
grant formats, adapter admission and Core execution semantics are unchanged. The
current Meiju adapter still admits only power, mode, fan and eligible temperature
controls. Swing, positions, display, sound and comfort operations remain blocked
for Meiju by missing reviewed native mappings, grants and exact-model acceptance;
their widgets have fictional fixture evidence only. This source phase accessed no
real account or appliance and sent no physical-device write. No push, publication
or diagnostic upload was performed.

### Android delivery and authentication block

The release-mode Meiju APK was built from clean application source at
`b03d113f6d748873f6a206f3c5ca8c6dbc86a6f7` with the pinned Flutter SDK and the
existing Android SDK/NDK and JDK 21. A temporary Flutter configuration selected
these tools without changing global settings. The build targeted android-arm64
and android-x64 with `HOMEVERB_ADAPTER=midea.meiju`; no test entry point was used.

- Artifact: `output/flutter-verification/android/homeverb-0.2.0-meiju-climate-rows.apk`.
- Bytes: 68,202,196.
- SHA-256: `f719e5ca69d9bf2a286241c7b68bf6bf223e7eb3309c4fd116981038c7067220`.
- `scripts/verify_flutter_apk.py` passed both complete ABIs, 17 Core exports per
  ABI, signature, backup exclusion and 16 KiB ZIP/Core ELF alignment. This is a
  local development signature, not store-distribution acceptance. The adjacent
  `homeverb-0.2.0-meiju-climate-rows.json` records the source and native hashes.
- `adb install -r` succeeded on the existing Android 15 / API 35 arm64 emulator.
  The app launched with its saved cards and Chinese preference intact; visible
  Chinese navigation and status glyphs rendered correctly.

Runtime detail acceptance is **blocked**: opening an existing AC card returned
`AUTH_REQUIRED` and requested account reconnection before entering its detail
page. No credentials were entered, no reauthorization was attempted, and no
appliance control was selected or sent. This does not establish runtime detail
layout, current session grants, a successful provider read or physical control.
Chinese detail glyphs and exact-model controls still require a valid session.
The source/widget and native Mock evidence above remains separate from this block.

Local captures are
`output/flutter-verification/android/climate-rows-home-zh-CN.png` and
`output/flutter-verification/android/climate-rows-auth-required-zh-CN.png`.
The APK, report and captures are local ignored artifacts; no upload, push or
publication occurred. Native API, schemas and adapter admission are unchanged.

### Emulator recheck and isolated Native Mock acceptance

The owner requested installation and emulator verification again. In-place
installation of the same verified APK succeeded on the Android 15 / API 35 arm64
emulator. Opening a saved AC card still returned `AUTH_REQUIRED`; the review did
not enter credentials, reauthorize an account or select a physical-device control.

Two separate process tests then ran with temporary Native Mock profiles and
`--no-uninstall`, preserving the real profile and presentation preferences:

- `fvm flutter test integration_test/native_journeys_test.dart -d EMULATOR_SERIAL
  --no-uninstall --reporter expanded`: one test passed in 3 minutes 18 seconds.
  It exercised direct Heat selection and its verified native receipt, temperature
  forms, quick power, local organization/preferences, scoped cleanup and lifecycle
  refusal through the embedded Core. Text input, file picker and lifecycle events
  remain simulated as documented above; no provider or appliance was used.
- `fvm flutter test integration_test/native_smoke_test.dart -d EMULATOR_SERIAL
  --no-uninstall --reporter expanded`, with a device-local
  `HOMEVERB_CLIMATE_CAPTURE` Dart define: one test passed in 19 seconds. The smoke
  test now uses inline Heat selection instead of the removed mode dialog. It
  checks observed Heat and a verified receipt, increases the described target
  from 26 to 26.5 degrees Celsius through the new plus button, checks the observed
  target and its verified receipt, and confirms unsupported fan, airflow and
  display/sound rows are absent. It then renders the same page in Simplified
  Chinese before restoring English and checking scoped cache/history cleanup.

The Android Chinese climate capture was visually inspected: labels and mode
choices render with platform CJK fallback, without the host widget renderer's
missing glyphs. This establishes native Mock rendering and execution, not a Meiju
read, extra adapter mapping, physical-appliance acceptance or pixel equality.
`fvm flutter analyze` and the pinned Dart format check passed after the smoke-test
update. No production Dart, Rust, ABI, schema or startup-grant code changed.

Local evidence under `output/flutter-verification/android/`:
`climate-rows-native-mode-en-US.png`, `climate-rows-native-zh-CN.png`,
`climate-rows-native-journeys.log`, `climate-rows-native-smoke.log`,
`climate-rows-recheck-auth-required-zh-CN.png` and
`climate-rows-restored-home-zh-CN.png`. These are ignored local artifacts.
After both tests, the ordinary verified Meiju APK was reinstalled successfully
and launched with its original saved cards and Chinese preference intact. The
emulator is left on that ordinary build's home screen. Real Meiju detail/control
acceptance remains **blocked** by the account's authentication requirement.
No physical-device write, upload, push or publication occurred.

## Main integration: October 16, 2026

Phase 1 merges `origin/codex/flutter-ui` at `834739b` into main at `cb76cbb`.
Conflicts retain native API 1.22, SQLite schema 5, the newer native safety gates
and both sets of presentation keys. The bundled English catalog matches the
shared catalog. Source merge checks passed: documentation links, contract fixtures,
repository hygiene, existing commit conventions, seven presentation tests and
`git diff --check`. This phase does not claim a rebuilt Flutter or Android artifact;
subsequent integration/build checks must use the merged Core. Earlier entries
are historical evidence from the separate UI branch.


### Phase 2: current Core integration on macOS

Flutter negotiates API 1.22 and exposes explicit saved-conflict, control-option
and session-grant queries using exact local account/device/connection IDs. Account
details show nullable historical complete-inventory time. Cancellation, navigation
and foreground changes clear inspection output and discard late replies. The
new fixture/native tests cover unpublished bindings, query failure versus empty
success, scope refusal, lifecycle races and retained read-only authority.

The native package builder now selects the installed Linux/macOS NDK toolchain;
Flutter and Python FFI tests load the host-specific library suffix. CLI Linux
policy is unchanged; macOS-only lint warnings are scoped without enabling grants.
The local TLS fixture now flushes its final record before closing. Obsolete paused
handoff instructions were removed; the app README and task status are the entry
points. API signatures, SQLite schema 5 and protected payload 3 are unchanged.

Executed checks on this Mac:

- Flutter 3.47.7 analysis passed. Full `fvm flutter test --reporter expanded`:
  84 passed, four existing strict glyph-pixel assertions failed by one pixel on
  macOS (`button_labels_test`, two `control_labels_test` cases and
  `field_alignment_test`). Skin/reference files and tolerances were not changed.
  This is functional/native integration evidence, not pixel-equality acceptance.
- `cargo build --workspace --locked` passed with Rust 1.99.1. The full Rust suite
  initially failed 14 LAN loopback fixtures because macOS does not assign
  `127.0.0.2` and `127.0.1.1` by default. Temporary alias setup was blocked by
  the administrator-password requirement; no interface was changed. With only
  those 14 cases explicitly filtered, 466 tests passed and one environment probe
  remained ignored. A separate TLS fixture truncation exposed during this run
  was repaired by flushing, then passed in the final portable suite.
- `scripts/verify_native_contract.py` passed all 34 methods, 55 fictional examples
  and trusted approval/scan/link checks on the actual macOS library.
- Python tooling: 63 tests passed. Shared presentation: seven tests passed.
- The arm64-v8a/x86_64 AAR build passed all 17 native exports per ABI and 16 KiB
  ELF alignment checks using NDK 28.2.13676358, API 35 stubs and JDK 21.

`cargo +1.96.0 clippy --workspace --all-targets --locked --target-dir target/lint-rust-1.96 -- -D warnings` passed. Rust 1.99 reports newly introduced chunk-iteration
lints in unchanged protocol code. The portable CLI/MCP suite passed 28 tests with
seven explicit Linux-only skips; its initial unadapted run failed those platform
assumptions and the macOS terminal-input expectation. APK build
and emulator results belong to the following delivery phase. No provider account,
physical appliance, physical phone, diagnostic upload or remote push was used.


### Phase 3: Android runtime and packaging guard

An isolated Android 15 / API 35 arm64 emulator passed the actual Flutter process
checks: native discovery/control/cache/history cleanup and API 1.22 negotiation
(one test), trusted confirmation scope/consumption, expiry, background cancellation
and lost-reply receipt recovery (four tests), and empty Meiju account navigation
with protected-input boundaries and no vendor request (one test). Commands were
`fvm flutter test integration_test/native_smoke_test.dart`,
`integration_test/approval_smoke_test.dart` and `integration_test/account_smoke_test.dart`,
each with the isolated emulator selected explicitly. No physical device or account
was used. The confirmation case waited for actual native expiry on Android.

The first release-mode APK built, installed and cold-started successfully. Archive
inspection found a transitive `armeabi-v7a/libdartjni.so` without the rest of the
32-bit runtime, despite the requested two ABIs. Packaging now explicitly excludes
unsupported 32-bit directories; the rebuilt Android test APK contains exactly
arm64-v8a and x86_64. Final release artifacts must be rebuilt with this correction.
The new `scripts/verify_flutter_apk.py` refuses incomplete ABI coverage, test-plugin
contamination, wrong package/minimum SDK, debug mode, enabled backup, invalid
signatures, missing Core exports and invalid 16 KiB alignment. Three negative
archive tests passed; the complete Python suite now passes 66 tests.

The initial SDK selection failed before compilation because an incomplete
platform-tools directory was chosen. An isolated temporary Flutter configuration
selected the existing complete SDK/NDK and JDK without changing global settings
or accepting new licenses. Source hygiene, documentation/contract checks and
`git diff --check` passed. The [Android app guide](../../docs/development/10-flutter-android.md)
now covers this setup, profile selection, checksums and installation. The final
local artifact delivery is recorded separately below.


### Phase 4: local APK delivery

Both final release-mode APKs were built from clean application source at
`c5f0a0758c342fb00df5ea3bd7d7d13a3b5710aa`, after Android integration tests,
using Flutter 3.47.7, Rust 1.99.1, NDK 28.2.13676358 and JDK 21. They use local
development signing, minimum API 28 and target API 36. Both have exactly two
complete ABIs, arm64-v8a and x86_64, and no integration-test plugin. APK signature,
ZIP alignment, all eight native libraries' 16 KiB ELF alignment, 17 Core exports
per ABI, package identity and disabled backup were checked. The API version was
also established by the Android runtime integration test, not archive inspection.

Local outputs are under `output/flutter-verification/android/`:

| Artifact | Bytes | SHA-256 |
|---|---|---|
| `homeverb-0.2.0-meiju-readonly.apk` | 66210688 | `ff8f39ec25431b6f8dc27a4e9409c99b9ec3721c9a78df4f0e5e24f120cc7699` |
| `homeverb-0.2.0-mock.apk` | 66210688 | `d23430e3ab66c1ce668d26595aeccb2eeeb5a16c8275224bb070becd889605bf` |

The adjacent JSON reports record source revision, clean-tree state, native hashes
and AAR hash. `SHA256SUMS`, `runtime-evidence.json`, release screenshots and logs
remain local ignored artifacts. A clone must rebuild them; nothing was uploaded.

The final ordinary Mock APK installed and cold-started on the isolated emulator.
Actual accessibility-driven Android taps selected Add Device, Discover Mock Device
and Refresh State; the saved card displayed a 26.0 Celsius observation. The final
Meiju APK installed and cold-started independently, showed an empty separate
profile and its explicit read-only notice through Add Platform. No credentials
were entered, and no provider request or physical write was made. The two APKs
share a package ID and replace each other on installation; their profile directories
remain separate. Physical Android/Keystore/provider acceptance, full localization,
accessibility and pixel equality remain open, including the four macOS pixel-test
failures recorded above.

The temporary emulator was shut down after acceptance; its generated AVD, isolated
Flutter configuration and dedicated ADB server were removed/stopped. The obsolete UI branch is
fully contained in local main. Remote main push and remote branch removal are a
separate owner-authorized step under AGENTS.md; no remote publication is implied
by these local artifacts or commits.


Date: October 16, 2026. Baseline: native UI contract 1.14.1 (pinned Core `7db8e9b`) and the approved Window Light atlas. This is an implementation checkpoint, not completion of the owner's pixel-identical acceptance requirement.

## Historical UI branch checkpoint

Source checkpoint: `2d4209c` on `codex/flutter-ui`, October 16, 2026. Native API
1.14.1 and SQLite schema 4 remain pinned; this is not main's newer Core baseline.
All 84 Flutter tests, static analysis and the native-library build passed for
this source. The last Android-tested application source is `d3d3395`; the later
surface experiment has not been rebuilt or exercised on Android.

The preview covers 82 states and 201 simulated branches, not 201 native provider
operations. Last recorded experimental comparison: 0/82 exact, mean channel error
0.5268/255, with the S11 regression and endpoint-rounding issue still open.
Pixel work remains paused. Real Flutter accounts/Keystore, physical phones, iOS
app integration, complete localization/accessibility and performance remain
unaccepted; sharing, identity merge and full-profile deletion remain unavailable.

Use [README.md](README.md) for portable recovery and remaining work, and the final
[source-only checkpoint entry](#source-only-paused-checkpoint--october-16-2026)
for its exact commands. Earlier counts, API versions and APK hashes below are
historical phase evidence, not the latest build state. Logs, captures, reports
and APK paths are local artifacts excluded from Git; a fresh clone must regenerate
them. Their absence does not mean source or required app assets are missing.

## Implemented scope

The application pins Flutter 3.47.7 through FVM 4.3.1. It loads the native library in process, negotiates API availability, runs blocking work in isolates, gates background calls, cancels outstanding operations and preserves uncertain outcomes. No production HTTP bridge, CLI parsing, periodic refresh or account service was added.

The native Mock path provides first launch, three roots, local search, room filters, room create/rename/reorder/delete, atomic assignment, alias/pin/order, explicit discovery and reads, component-described climate controls, explicit selected-device power batches, exact receipts/history, local route/name/skin preferences and allowlisted diagnostics preview/export. Loading and failed local reads do not display fictional preview cards. Modal backgrounds use the selected native device state.

All 82 atlas states have native-widget implementations in an isolated, visibly labelled preview. The 201 event buttons in that preview simulate atlas branches. They are **not 201 successful native operations**. Flutter account metadata, protected input, automatic-retrieval results, pending-session recovery/reauthorization and scoped removal are implemented. Their successful provider flows are tested with isolated contract fixtures, not real credentials. Nearby sharing, identity merge and full-profile deletion remain unavailable. Native cache/history cleanup is implemented with separate scope confirmations. Non-climate native forms consume described scalar properties/actions. A selected component with one light, fan, media, sensor or cycle family now uses a dedicated Window Light layout; mixed families retain the descriptor list. These isolated fixtures are not registered native adapters or physical-device support. The multi-compartment overview now groups exact component readings and target controls. Strict layout equality remains pending.

## Earlier baseline verification

The following table preserves earlier phase results; use the current checkpoint
above for the latest test count and source. Commands below ran in the isolated worktree; Flutter commands used the pinned FVM SDK. Temporary profiles contained only fictional data.

| Check | Evidence / result |
|---|---|
| `fvm flutter analyze` | No issues found |
| `fvm flutter test --reporter expanded` | Full 73-test suite passed, including the fractional geometry regression, four trusted confirmation scenarios and two saved-inventory checks, two native account and five synthetic account checks, three cleanup regressions and all 11 prior native/controller checks and the inline-stepper interaction check. Other coverage: one extended native click scenario, three native outcome scenarios, two component checks, four compartment-overview checks, five accessibility checks, three system-contrast checks, two receipt checks, two descriptor-control checks, six dedicated appliance-layout checks, three organization checks, nine screen/interaction/reflow checks, two Android export-channel checks and three diagnostics-card checks |
| `fvm flutter test integration_test/native_smoke_test.dart -d linux` under Xvfb | Actual Linux process loads the bundled Core library without a path override, discovers Mock, reads state, sets mode and verifies the returned receipt; process screenshot captured; the latest run also cancels and confirms cache cleanup and confirms history cleanup |
| `fvm flutter build linux --release` | Linux bundle includes `libhomeverb_ffi.so`; the bundled Rust library is the checked debug build |
| `fvm flutter build web -t lib/main_preview.dart` | Isolated preview compiles; web preview does not call native Core |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --offline --locked -- -D warnings` | Passed |
| `cargo test --workspace --offline --locked` | Passed |
| `cargo build --workspace --offline --locked` | Passed |
| `python3 scripts/verify_native_contract.py` | 31 native methods and 51 live fictional examples passed |
| `python3 scripts/verify_mvp.py` | 24 runtime checks passed |
| `python3 scripts/validate_contracts.py` | 16 schemas, 29 positive and 90 negative fixtures passed |
| `node packages/presentation/test/i18n.test.mjs` | 5 shared-catalog tests passed; this does not establish Flutter localization |
| `python3 -m unittest discover -s tests -v` | 23 repository tests passed |

The native click scenario exercises discovery, read, mode and target writes, quick power before/after reading, card long press, invalid target rejection, blank selection rejection, batch execution, blank/overlong/control-character/duplicate room rejection with retained input, room rename/drag/menu reorder/delete, exact-room assignment, All/custom/Unassigned filters, alias/pin/unpin, search/clear/exit, unchanged route persistence, sharing-name persistence, receipt refresh with no current device selection, history, diagnostics full-report review and export cancel/save/failure/retry, unavailable gates and foreground lifecycle. Room-menu assignment preselects its exact room and rejects empty device selection; device-menu assignment visibly selects the current device. Corrected input clears outdated validation messages before the next submission. The file picker is replaced by a controlled platform implementation; saved file bytes must equal the exact preview. The actual Linux integration separately exercises the real Flutter process, not the system save chooser.

The native widget click test now loads the bundled Inter/Material fonts and fails missed pointer targets. Its latest successful run saves 92 viewport captures across 27 screen IDs, including errors and intermediate states. Generate `output/flutter-verification/native-gallery.html` with `python3 apps/mobile/tool/native_gallery.py`. Entries identify clicks, drags and direct test navigation; all records are fictional native Mock data. Phone insets are simulated and OS status icons are absent. This is separate evidence from the 82-state isolated atlas and from the actual Linux-process screenshot. The native room page now uses the prototype's grouped rows and exposes assignment through its row menu; its drag handle and menu ordering are both exercised.

Three additional click scenarios use the native `lost-ack`, `accept-only`, and `reject` fixtures. They click discovery, read, power, receipt refresh and history; assert the displayed `unknown`, `accepted`, or `failed` outcome; retain observations until an explicit read; and verify the original request ID and unchanged history count after refresh. These are fictional native operations, not physical-device evidence.

Two isolated contract-fixture tests load the repository's refrigerator and compartment capability examples. They verify independent cached observations, exact component references, absence of unrelated shared capabilities, blocked selection changes while busy, distinct cold-storage ranges and steps, plan-before-execute, and disabled read-only temperature controls. The fixture exists only in tests; it is not a registered native adapter. The real native Mock host still exposes one climate appliance. Multi-selection preview rows also retain every selected item and support independent selection/deselection.

History and batch results now use grouped rows with readable outcomes and UTC timestamps. Receipt details separate returned values, recorded attempts, errors, warnings and timestamped verification evidence; they do not invent original request arguments or claim that returned values are current observations. Exact request IDs select retained history, and returned nested items open directly without requiring a separately retrievable child receipt. Back restores the parent receipt or batch. Two isolated receipt-fixture checks exercise duplicate device aliases, exact item selection, correlated completion, missing evidence, storage warnings, skipped steps, parent navigation and read-only refresh without execution. These pages also render at 320 logical pixels with 200% text. The three native outcome checks and extended native screenshot scenario were rerun after the final receipt copy change.

Descriptor-driven action forms now expose selected-component scalar fields, typed enum/boolean choices, numeric bounds and steps, units, readable properties, stale markers and received timestamps. Optional values are omitted until explicitly chosen; empty/unknown observations never become zero or false. A bounded numeric draft gains a Window Light slider. Core planning remains authoritative for full schemas and permissions. Unsupported field shapes, read-only setters and actions requiring trusted approval remain unavailable. Common power/mode rows also disable unavailable writes. The form uses the selected device behind its sheet rather than the fictional washer background.

Two isolated descriptor-control checks cover brightness/temperature limits, enum fan speed, oscillation, media input source/mute/volume/play/pause, missing sensor readings, denied plans with retained drafts, plan-before-execute, accepted outcomes without optimistic observations, disabled duplicate submit, and cancellation with a late result. Final checks additionally cover 320-logical-pixel forms with 200% text and a simulated 280-logical-pixel keyboard. The test writes 14 labelled captures under `build/verification/controls/`; these are fixture screenshots, not additional native devices or reference-equality evidence. The form remains the fallback for unsupported direct-control shapes and coupled actions; dedicated single-family layouts share the prototype slider, stepper and row components. Visual refinement against C05-C09/C13 remains necessary.

Device ordering now uses grouped Window Light rows with drag handles and accessible up/down menus. The full exact permutation merges the visible order into the unchanged hidden slots. Invalid visible selections fail before mutation; a native permutation conflict reloads local lists and asks the user to review the new order without automatically retrying. Done returns to the recorded caller and preserves the selected device/filter. Two isolated multi-device fixture checks cover duplicate aliases, menu bounds, actual drag gestures, hidden positions, metadata preservation, stale device and room lists, and no mutation replay. A separate actual native test adds a room between snapshot and reorder, verifies conflict reload, then explicitly retries the reviewed order; it also checks single-device ordering. The fixture screenshot at `build/verification/organization/D10.png` uses decoded assets and simulated phone insets; it does not establish physical multi-device behavior.

Six additional appliance-layout checks exercise light brightness/color-temperature/power, numeric and enumerated fan speeds, oscillation, media source/mute/volume/play/pause, read-only sensor values, and unavailable elevated-risk cycle actions. Numeric controls use declared bounds and steps; only known absolute setters receive direct controls. One-shot actions with similarly named parameters retain explicit forms. Coupled required parameters also retain complete forms. Failed plans preserve numeric drafts and original observations; accepted receipts never replace readings. Missing slider values announce Unknown to accessibility services, and a real zero sensor value remains zero. Device-information and exact-device history links are clicked. The checks capture 13 initial/error/large-text viewports under `build/verification/appliances/`; the local comparison gallery is `output/flutter-verification/appliances/index.html`. The artwork column has different sample data and does not constitute pixel equality or physical-device evidence.

Native room assignment and batch selection now reuse the Window Light choice cards, including multi-selection semantics and disabled device selection while a request is pending. An isolated duplicate-name fixture clicks exact device IDs, verifies deselection, preserves device/destination selection after a rejected move, and retries only on another click. The real native library rejects a mixed valid/missing-device assignment and a deleted destination without partially changing room membership, alias or favorite. A subsequent explicit valid assignment and unassignment pass. These checks establish local Mock storage behavior, not physical multi-device control. The rejected-choice fixture capture is `build/verification/organization/R03-rejected.png`.

Four additional compartment-overview checks cover cached-only opening, exact-component target commands without changing the current selection, partial reads with independent retained observations, explicit retry, missing and read-only targets, and navigation cancellation or rapid background/resume before the next component read. The target row opens the selected component input; coupled temperature/mode requirements open the full typed form over the overview. Extra controls appear only when the descriptor includes additional actions or parameters. Primary card observations remain bound to the primary component. Seven fixture viewports under `build/verification/compartments/` cover initial, refreshed, missing, partial-read, read-only and large-text states. Large-text steppers expand to keep temperature values readable. These fixtures do not register a refrigerator adapter or establish physical compartment behavior.

Five accessibility checks verify modal Tab focus containment, Escape return, native-root keyboard navigation with persisted preferences, named text fields/sliders/switches, and labelled tap targets across all 82 atlas states. Disabling focus isolation reproduces the backdrop-focus failure; restoring it passes. Page headings expose route semantics, navigation exposes selection, and errors/status cards use live regions. Switch rows have one keyboard control rather than duplicate row/switch stops. Reduced-motion busy activity uses a static marker without announcing a percentage or completion. The actual Linux FFI integration and release build were rerun after these changes. These automated checks do not establish TalkBack/VoiceOver behavior, complete focus order, high contrast, or full animation accessibility.

Three system-contrast checks verify the system preference, live restoration of the default skin, and all 82 states at 320 logical pixels with 200% text. Flutter's automated text-contrast guideline passes on all 82 initial viewports with decoded assets and the bundled fonts. High contrast uses solid surfaces, stronger boundaries and semibold text of at least 12 logical pixels; the enlarged bottom navigation grows to fit wrapped labels. Seven screenshots are saved under `build/verification/accessibility/`. These checks do not cover every offscreen paragraph, native system dialog, real phone, or physical screen reader, and do not constitute a complete WCAG audit. The Linux release bundle was rebuilt; the last actual-process FFI integration ran after the preceding focus/semantics phase. Default-mode strict comparison remains 0/82 exact with unchanged mean error.

The phone header now places the Search glyph at (300, 65) with size 22 and Add Device at (345, 64) with size 23 in the 390-by-844 reference viewport. The wide room-settings glyph is (974, 84), size 20. Measured widget geometry matches those prototype coordinates; button padding no longer compresses the glyph height. The platform root heading gap is aligned at standard text size. The temporary geometry probe was removed after saving its local JSON evidence. Shared typography and card-spacing experiments were not retained because they enlarged differences on other screens. Full 49-test regression passed before restoring the previous card spacing; the final 10 screen/reflow/contrast checks were rerun afterward. The final comparison remains 0/82 exact; mean error decreased from 1.6967 to 1.6798 / 255. P01 decreased from 2.9590 to 2.1770, while S07 retained its prior 2.5924. No native API or execution behavior changed.

Controller/FFI tests additionally cover persisted reopen, accepted/verified/failed/unknown results, nested partial batch decoding, no write replay on read, numeric bounds/steps/component scopes, preference corruption and rapid background/resume races. The Mock host exposes only one appliance. Multi-device mixed outcomes use native fixtures; no physical multi-device batch was performed.

## Visual gate: failed

The screenshot test captures every ID at the reference size: 780 x 1688, or 2048 x 1536 for W01/W02. It loads the licensed Inter fonts and approved product/background assets. All text, controls and semantics are widgets; complete screen images are never used as the app interface. Test status/navigation chrome is deterministic artwork, not a captured phone OS.

The latest complete comparison has **0 of 82 exact matches**, with mean absolute channel error **1.2943 / 255**, down from 6.1482 before rendering refinements. Native switches now use the prototype's accent/gray tracks and light thumbs. A lower average alone does not establish correct states or layouts. These metrics measure differences, not an acceptance tolerance. Text rasterization, icons, surfaces, controls and layout still differ. P08's base result now follows the combined account/retrieval contract in both the independently rendered atlas and the Flutter catalog; its remaining pixel differences are still failures.

The latest shared-widget refinement aligns the non-root header, raises button labels to the reference position, and measures actual label/value font widths before wrapping rows. Fan and refrigerator preview steppers now expose separate increase/decrease controls with explicit preview-only feedback. The added interaction check clicks all six controls without changing page or claiming a native write. All 29 Flutter tests, static analysis and the Linux release build passed after this refinement.

This refinement aligns the surface gradient, text spacing, root navigation, form heights, sheet corners, sliders, climate panel, large cards and skin miniature. The bottom sheet no longer applies the keyboard inset a second time after Scaffold resizing. All 82 states also render at 844 x 390 logical pixels with 150% text; all phone states render at 320 logical width with 200% text. Every atlas form accepts edited text while its input remains above a simulated 280-logical-pixel keyboard. These widget checks do not establish physical keyboard or screen-reader behavior.

Generate the full reference / actual / amplified-difference report with:

```sh
python3 apps/mobile/tool/compare_screens.py --require-exact
```

The command verifies the complete 82-file set, writes `output/flutter-verification/index.html` and `metrics.json`, and exits 1 while differences remain. The reference is never overwritten and generated artifacts are ignored by Git. Passing screenshot rendering is not passing visual equality.

## Acceptance status

| Requirement | Current status and remaining evidence |
|---|---|
| UI-01 | 82 states and 201 simulated branches covered; native available routes clicked. System-owned authorization/permission transitions unavailable. |
| UI-02 | Native first launch, three roots, search/clear, separate card/quick-control clicks, long press and room filters tested with actual fonts and strict pointer hits. Root keyboard navigation, modal focus containment and labelled tap targets tested. Full keyboard/screen-reader audit pending. |
| UI-03 | Native room/card CRUD, room drag/menu sorting, exact-room assignment and preservation tested. Empty device selection and blank, overlong, control-character and duplicate room names are click-tested with retained input. Isolated multi-device fixtures additionally verify drag/menu sorting, unchanged hidden positions and metadata, invalid/stale permutations, and reload without automatic retry. Real native missing-device and deleted-room assignment conflicts preserve metadata atomically; duplicate-name selection, failed-choice retention and explicit retry are fixture-tested. |
| UI-04 | Exact-component selection, capability references, cache isolation, climate bounds/enums and unknown/read-only values implemented. Isolated fridge/freezer fixture clicks verify distinct ranges and steps; Scalar non-climate action forms, stale/read-only properties, denied plans and cancellation are fixture-tested. Dedicated single-family layouts, numeric/enum fan controls, one-shot-action exclusion, unknown-value semantics and coupled-action fallback are fixture-tested. The combined overview, exact targets, partial-read recovery and compound target forms are fixture-tested. Pixel parity and physical multi-component acceptance remain pending. |
| UI-05 | Native outcomes preserved, including nested partial results with per-item evidence navigation. Explicit batches plan before execution; no aggregate rollback/retry. Physical evidence unavailable. |
| UI-06 | Explicit bounded Mock discovery implemented. Real permission, identity-merge and nearby-pairing paths unavailable. |
| UI-07 | Protected Dart FFI, exact-account selection, local rename, automatic/partial/failed/empty retrieval, explicit recovery/reauthorization, cancellation and field/buffer clearing are implemented. Fixture clicks and native rejection/empty-profile checks pass; successful real login, secure-store and mobile acceptance remain pending. |
| UI-08 | Flutter confirms exact linked-card scope, distinguishes pending cleanup, refuses stale scope until review, and retains unrelated accounts/rooms. Synthetic clicks pass. Real account removal and provider revocation remain unverified/unavailable respectively. |
| UI-09 | No current native sharing methods; no peer networking or credential transfer. |
| UI-10 | Dart/native foreground, cancellation and rapid-resume checks passed. Component read sequences discard late results and stop before the next read after cancellation/backgrounding. Physical Android/iOS background and termination acceptance pending. |
| UI-11 | One registered skin, local persistence/fallback and preview implemented and tested. |
| UI-12 | Exact diagnostics preview/export and retained receipt navigation tested. Confirmed native cache/history cleanup, cancel, busy-journal refusal, local reload and request retirement are click/native-tested. Android system-document cancel/save and exact-byte read-back passed on the isolated emulator. Full-profile deletion, physical phones and other OS save choosers remain pending. |
| UI-13 | Reference phone/wide captures, 320-width/200% text, all-state landscape/150% text and all-form simulated keyboard checks passed. Field/control semantics, all-state tap labels and reduced-motion busy indicators tested. System high contrast passes the 82 initial-viewport text-contrast checks and all-state 320-width/200% layouts. Full zh-CN wiring, offscreen contrast, remaining animation behavior, screen-reader and physical-keyboard acceptance pending. |
| UI-14 | Linux process/bundle and Android debug/development-release APKs verified. Android 15 x86_64 journeys, cold launch, real IME and Home/resume checks passed. Physical phones, iOS, profiling and pixel equality remain unverified. |
| UI-15 | Meiju Core and explicit read-only Flutter startup/account integration are present; real account and off-LAN acceptance remain pending. P08 base artwork now matches the combined retrieval result; separate runtime-variant artwork and real off-LAN acceptance remain pending. |

## Earlier compatibility and environment

This paragraph records the earlier cleanup phase, superseded by the API 1.14.1
checkpoint above. The frontend merged Core API 1.9 and SQLite schema 4 from pinned commit `00d44b7`, including the protected seventh C symbol. This cleanup UI uses the existing native methods without further schema changes. Presentation preferences are app-private JSON containing no credentials. No real-platform compatibility row is promoted. No real-device write, diagnostic upload or package publication was performed. The owner-authorized frontend implementation has been pushed to `origin/codex/flutter-ui`; the branch is retained. This is an incomplete acceptance checkpoint, not a merge into main or a claim that the pixel gate passed.

Android startup embeds the matching dual-ABI AAR, and debug/development-release
APKs run on an isolated Android 15 x86_64 emulator. The task-local SDK/NDK and JDK
21 support the recorded build. Actual Keystore and physical-phone acceptance
remain pending; Xcode/Apple validation is unavailable on this Linux host.
No device choice is required to continue shared Flutter UI work. Linux GTK/Clang development packages were extracted into a temporary local sysroot for this run; no system package installation was performed. Standard installed Linux dependencies are sufficient for normal development. FVM owns the SDK; no project-local SDK copy is committed.

## Core 1.9 integration checkpoint

The frontend branch merges pinned Core commit `00d44b7` without rewriting frontend
history. Native API 1.9.0, the seventh protected C symbol, SQLite schema 4, account
methods, scoped grants, cache/history clearing and the Android library package are
now present. Earlier sections describe the prior 1.0 UI baseline; account and cleanup
UI wiring remains separate work at this merge checkpoint. No real profile is opened.

Executed checks: offline locked workspace build; the Rust native-contract test;
`verify_native_contract.py` (30 methods, 50 actual fictional ABI examples); docs,
contract and repository validators; and all 49 existing Flutter tests using FVM.
All passed. This verifies compatibility of the existing Mock UI with the new Core,
not account UI completion, physical support or exact screenshot equality.

## Native cleanup UI checkpoint

Settings > Local Data now negotiates cache/history methods and opens a separate
scope confirmation for each. Cancel performs no mutation. Cache cleanup invalidates
component/card snapshots; history cleanup drops visible detail/aggregate receipts
and diagnostic previews. Both reload local metadata and retained history; any
selected component uses cached reads only, with missing data shown as unknown.
A lost reply invalidates affected UI data and requires local reloading before
explicit reconfirmation. No cleanup retry or network refresh runs automatically.

Three cleanup regressions cover actual ABI cache misses, preservation of room/card
organization and observations, unknown-receipt retirement, failed replay with zero
attempts, zero-item cleanup, interrupted results, and a fixture busy-journal
confirmation/reload/reconfirmation sequence. The extended native widget scenario
clicks cancel and confirm for both methods and records screenshots. Full-profile
deletion remains explicitly unavailable. No existing owner profile is modified.

The merged Core also passed offline workspace formatting, Clippy with warnings
denied, all default workspace tests and all 24 MVP runtime tests. Physical account
or appliance tests were not run. Strict screenshot equality remains a failing gate.

The final 52-test Flutter suite, analyzer, Linux release build and actual Linux
Flutter process cleanup scenario passed. The process screenshot is retained in
`output/flutter-verification/cleanup-native-linux.png`. The strict reference run
still reports **0/82 exact matches**, mean channel error **1.6798/255**; exit status
1 is retained as a failed acceptance gate. This phase changes native data-backed
views, not the fictional prototype raster references. The 88 native widget captures
cover 27 screen IDs, including cleanup scope/cancel/completion states.

## Protected account UI checkpoint

The default app remains Mock. `HOMEVERB_ADAPTER=midea.meiju` explicitly selects a
separate private read-only profile with no appliance grants. Native version/method
checks precede protected-symbol use; pending reauthorization requires API 1.8.
The separate worker closure receives native addresses and nonsecret metadata, not
managed credential arrays. The caller retains native buffer ownership until the
worker completes, then clears/frees both allocations on success or failure.

Platforms now loads local summaries and details, displays actual credential states
without inventing login identifiers/scopes/check times, renames by local ID, and
shows saved linked cards. Duplicate labels have visible identity suffixes. Retrieval
and recovery expose inner partial/failed results; empty inventory preserves saved
cards. Sign-in uses the exact selected UUID for reauthorization and clears fields
before awaiting completion, on backgrounding and disposal. Late callbacks do not
navigate after cancellation. Unsupported models stay visible with a limitation
and do not trigger cached or bulk live reads. Wide nested account/settings pages
select their correct navigation section.

Local removal displays the reviewed ID set, including missing-metadata entries,
and sends that unchanged set after confirmation. Scope changes require another
review. Pending cleanup is displayed with its error and explicit retry; successful
removal reloads local cards/accounts without deleting rooms or receipts. No
provider revocation or full-profile deletion is fabricated.

Two actual native tests cover protected-buffer clearing on Mock refusal, invalid
input, background, operation capacity, old version and queued pause/resume; plus
empty read-only Meiju startup/local queries. Five account tests cover exact duplicate
identities, rename validation, unsupported cards, partial/expired/empty retrieval,
new-account automatic retrieval without a duplicate refresh, pending recovery and
reauthorization at 320 logical width/200% text, secure-input lifecycle, late results,
unknown outcomes, local removal cancel/scope-change/pending/success and preservation.
The main account click scenario captures 26 states across eight screen IDs; the local gallery is `output/flutter-verification/accounts/index.html`.
All account responses are fictional; no real provider or secret store is contacted.

Final account-phase verification: all **59 Flutter tests** and static analysis passed.
The actual Linux Flutter process opened an empty isolated Meiju profile, navigated
the account catalog into protected input, refused empty submission, cleared fields
through valid background/foreground transitions, and cancelled back to the setup
page. No valid credentials were submitted. The existing actual-process Mock
discovery/control/cache-history cleanup regression also passed. The default-Mock
Linux release bundle built successfully. Actual account-process capture:
`output/flutter-verification/accounts/linux-process.png`.

Documentation, schema and repository validators passed. The complete strict
comparison remains **0/82 exact**, mean channel error **1.6798/255**. No reference
image was replaced and no tolerance was accepted. New protected-login and amended
account outcome states have functional screenshot evidence but do not establish
authoritative prototype parity. Android packaging/Keystore, iOS, physical account
acceptance, localization and full accessibility remain unfinished.

## Android embedding and emulator checkpoint

The application now packages the matching arm64-v8a/x86_64 Core AAR, requires API
28, installs the JNI credential backend from Application startup, declares Internet
permission, and excludes private app data from cloud backup and device transfer.
FVM remains pinned to Flutter 3.47.7. The development build used JDK 21, SDK 36,
NDK r28c and the checksum-pinned Gradle 9.3.1 wrapper. No toolchain files are committed.

An actual first launch exposed `Unsupported` from Rust 1.96 standard-library file
locks on Android, previously reported as `RESOURCE_BUSY`. An isolated Android
executable reproduced it. The Android-only Bionic `flock` implementation preserves
nonblocking shared/exclusive leases and release on close. All 13 storage tests
passed on Linux and Android 15 x86_64, including migrations, contention, rollback
and replay retention. Workspace formatting, Clippy, Rust tests, 24 MVP runtime tests
and all 31 native methods/51 fictional C ABI examples passed on Linux.

The corrected dual-ABI debug and development-signed release APKs built, installed and cold-started on an isolated
Android 15 x86_64 Pixel 7 emulator with KVM and SwiftShader at 1080 x 2400. Both
packaged Core libraries passed eight-export and 16 KiB ELF alignment checks; the
APK passed 16 KiB ZIP alignment. No physical phone or real appliance was used.

The comprehensive Mock journey passed in actual Linux and Android processes. The
Android run retained 88 captures over 27 screen IDs, including native discovery,
read/control receipts, room/card organization, settings, diagnostics and separate
cache/history cleanup. It uses simulated text input/file-picker responses and
injected lifecycle transitions; these do not prove OS keyboard, picker or task
switcher behavior. Direct test navigation is labeled separately from clicks.
The gallery is `output/flutter-verification/android/index.html`. Use
`--no-uninstall` to retain device captures; Dart's temporary directory is
`code_cache` on this emulator. The local manifest records OS and actual viewport.

The Android account-process test also passed with an empty isolated Meiju profile:
local catalog navigation, empty-submission refusal, confirmed fictional field
values, lifecycle clearing, local account reread and cancellation. No credentials
were submitted to a provider, and no real secret store round trip was performed.
Its capture is `output/flutter-verification/android/account-process.png`.

All 59 Flutter tests passed. The strict unchanged-reference comparison remains
**0/82 exact**, with mean channel error **1.6798/255**. Actual Keystore, physical-phone
and iOS acceptance, full localization/accessibility, provider/device acceptance
and strict prototype equality remain unfinished. The default app remains Mock;
explicit Meiju builds retain separate profiles and no appliance write grants.

The ordinary `lib/main.dart` release APK was rebuilt with tooling regeneration
enabled, excluding the dev-only integration plugin. The final package passed both
Core ABI/eight-export and 16 KiB ELF/ZIP checks. Its compiled manifest retains API
28, Internet permission, HomeVerbApplication, disabled backup and extraction rules.
R8 retains the JNI credential class names. Actual Android input opened the system
IME and entered fictional search text; Home/resume preserved that nonsecret query,
and force-stop/restart returned to the local home page. Screenshots and UI dumps
are saved as `output/flutter-verification/android/release-*`. These limited real OS
checks complement, rather than replace, the injected lifecycle journey tests.

The installable artifact is
`output/flutter-verification/android/homeverb-0.2.0-android-development-release.apk`;
`release-artifact.json` records its SHA-256 and acceptance boundaries. It uses the
development debug signing key and is not a store release. The earlier startup
failure artifact and integration-test APK are not delivery artifacts.

## Selected-account saved inventory

Account details now expose the API 1.10 `inspect_account_inventory` action only
when advertised and the selected account has stored credentials. Inspection reads
protected local bindings explicitly; app startup and ordinary account navigation
never trigger it. Three saved membership states remain distinct. A card alias is
used only for an exact device/connection pair; unmatched historical or unpublished
bindings remain visible without creating cards, changing grants or inventing online
state, physical support or retrieval timestamps. Explicit Retrieve Devices is the
only offered cloud-refresh path. Empty results and query failures remain distinct.

Two new tests use the versioned fictional native-contract inventory example. They
click inspection, check the exact selected account, all three memberships, matching
and mismatched connections, unchanged cards, empty results, locked/busy/corrupt
query errors without cloud fallback, large text, pending/removing guards and late
success/error after cancellation/background. Three decoded widget captures are
saved under `apps/mobile/build/verification/inventory/`. The native empty-profile
test also verifies `ACCOUNT_NOT_FOUND` and hides/refuses inspection when the method
is absent. These tests do not read a real account's protected inventory or establish
prototype equality for the added contract state.

Final inventory-phase analysis and all 61 Flutter tests passed. The unchanged-reference
strict comparison still fails at 0/82 exact, mean channel error 1.6798/255.

The updated ordinary Android release APK also rebuilt, passed both ABI/export and
16 KiB ELF/ZIP checks, installed and cold-started on the isolated Android emulator.
`inventory-release-home.png` records that launch. The full OS keyboard/Home/resume
checks remain recorded against the preceding Android checkpoint `68fdaf6`.

## Climate glyph alignment checkpoint

Measured reference/Flutter ink bounds exposed a 2.5-logical-pixel vertical drift
in the large observed-target value, and 10/5-pixel drift in the climate minus/plus
glyphs. Their corrected ink bounds match the reference at the 780 x 1688 test
viewport; this does not claim matching antialiasing. Inline steppers also align
their glyphs without shrinking hit targets or changing action semantics.
`output/flutter-verification/climate-ink-bounds.json` records the measured boxes.

Only those glyph/control adjustments are retained. Shared divider and titled-row
experiments were discarded because some individual screens regressed. The final
82-state comparison improves C01, C02, C03, C04, C06, C08, C14 and W02; all other
screens retain their previous metrics. Mean channel error drops from 1.6798 to
1.6514/255. The strict gate remains **0/82 exact**, and references are unchanged.

Static analysis and the full 61-test suite passed during the refinement. After
restoring the original rows, all 10 screen/reflow/high-contrast checks passed,
including 198 simulated atlas branches. All 82 captures were regenerated after
the final half-pixel horizontal glyph adjustment. The Android actual-process Mock
discovery/control/receipt/cache/history smoke test passed before the row rollback;
no real provider or appliance was contacted. These tests supplement the prior
native click coverage and do not establish physical-device or pixel acceptance.

The final ordinary Android release APK rebuilt, passed 16 KiB ZIP alignment,
installed and cold-started on the isolated emulator. The delivery APK and SHA-256
manifest were updated; `visual-alignment-release-home.png` records the launch.

## P08 contract and reference review

The original P08 reference still presents manual Find Devices after successful
authorization. The current contract instead combines connection and automatic
retrieval and explicitly provides no staged-progress callback. The isolated P08
preview no longer claims completed authorization or a fabricated 40% completion.
It displays a single connecting/retrieving state with Cancel. The native account
flow already waits for the actual returned report; its behavior is unchanged.

All 10 screenshot/branch/reflow/high-contrast checks passed, including an assertion
against staged completion/determinate progress in P08. Static analysis and the
isolated web-preview build passed. The unchanged 82-reference comparison still
fails: 0 exact, mean channel error 1.6529/255. This small increase follows the
required semantic correction; it is not a new tolerance or acceptance claim.

Seven separate fictional P08 candidate images cover connecting, complete, empty,
partial, failed, reauthorization and interrupted results. They reuse a temporary
copy of the existing prototype renderer, have checked dimensions/content bounds
and SHA-256 values, and are available locally under
`output/flutter-verification/p08-proposal/index.html`. The proposal is awaiting
owner review. It has not replaced any approved reference or changed the strict
comparison. Approval of its layout would still require Flutter implementation,
click verification and independent screenshot comparison for the revised states.

The ordinary Android release APK was rebuilt with the corrected catalog, passed
16 KiB ZIP alignment, installed and cold-started on the isolated emulator. The
local delivery APK/hash manifest and `p08-catalog-release-home.png` were refreshed.


## Icon outlines and wide-card geometry

Rounded icon outlines and power/refresh arcs now stay inside the bounds supplied
by the prototype renderer, using its individual stroke widths and arc angles.
The wide home grid uses the reference's 235-pixel card height and 18-pixel row gap;
its value, subtitle and power control follow the corrected card bottom. Phone
cards retain their 233-pixel height, and large-text cards continue to reflow.

All 82 native-widget captures were regenerated. Against the preceding checkpoint,
21 screens improve and 61 retain their previous metrics; none regresses. Mean
channel error drops from 1.6529 to 1.6261/255. The original references remain
unchanged, and the strict gate still fails at **0/82 exact**. W01 reference and
actual card crops were also inspected side by side. These geometry corrections
do not establish identical rasterization or resolve the pending P08 review.

Static analysis and all 10 screen/reflow/high-contrast checks passed, including
198 explicitly simulated atlas branches. Documentation, contract and repository
validators passed. The change does not add native methods, alter operation
outcomes or promote real-device compatibility.

The ordinary Android release APK rebuilt, passed 16 KiB ZIP alignment, installed
and cold-started on the isolated emulator. The delivery APK/hash manifest was
updated; `icon-geometry-release-home.png` records the inspected launch. Earlier
real OS keyboard and lifecycle evidence remains tied to its recorded checkpoint.


## Device-image pixel alignment

Device illustrations now preserve their fitted aspect ratio while aligning image
bounds to physical pixels. The original renderer rounds half-pixel ties to even;
using ordinary Dart rounding moved the home AC and TV images by half a logical
pixel. The retained implementation follows the original rounding rule and uses
the four source assets' dimensions. It keeps Flutter widgets and runtime image
scaling; no page screenshot or reference image replaces interactive content.
Unbounded layouts retain the normal image-fitting fallback.

A temporary layout probe measured the actual image positions and was removed
after diagnosis. The 82-state comparison improves 15 screens and leaves 67
unchanged, with no regression against the preceding checkpoint. Mean channel
error drops from 1.6261 to 1.6208/255. Reference/actual home-image crops were
inspected. The unchanged-reference strict gate still fails at **0/82 exact**;
remaining resampling, glyph rasterization and other layout differences are not
accepted by this result, and P08 remains pending owner review.

Static analysis, the full 61-test Flutter suite, documentation, contract and
repository checks passed. The suite includes the actual native Mock widget-click
journey and 198 explicitly simulated atlas branches, plus narrow/large-text and
high-contrast layouts. No native contract, Core storage, real-provider admission
or device-operation semantics changed.

The ordinary Android release APK rebuilt, passed 16 KiB ZIP alignment and installed
on the isolated emulator. Actual UI taps opened Add device, discovered the fictional
Mock appliance and opened its card. Android screenshots record the device image
and detail screen. Force-stop/cold restart restored the saved Mock card, confirmed
by the accessibility tree and `image-alignment-release-home.png`. The local delivery
APK/hash manifest was updated. No real network appliance or owner account was used.


## Surface-gradient raster alignment

The prototype evaluates its 35% horizontal / 65% vertical gradient at pixel
origins and truncates each RGB channel. A normal Flutter gradient rounds channel
values, leaving roughly half of sampled interior pixels one level too bright.
Window Light now loads a bundled fragment program before the screen catalog and
computes that formula directly for each physical pixel. Responsive bounds,
opacity and solid high-contrast surfaces remain supported. Standalone widgets
retain the ordinary gradient until application skin assets have loaded.

All 82 captures were compared with the unchanged references. Seventy-four screens
improve and eight remain unchanged; none regresses. Mean channel error drops from
1.6208 to 1.5145/255. D01 changed pixels drop from 39.408% to 13.078%; S07 drops
from 45.149% to 10.734%. Two interior regions (12,375 dark-card pixels and 11,250
light-card pixels) match all RGB channels exactly. Their coordinates and image
hashes are recorded in `output/flutter-verification/gradient-pixel-samples.json`.
This regional evidence does not establish whole-page equality: the strict gate
still fails at **0/82 exact**, and the P08 baseline remains pending owner review.

The final 61-test Flutter suite passed, including native Mock widget clicks,
198 explicitly simulated atlas branches, narrow/large-text layouts and high
contrast. No native method or appliance-operation semantics changed.

An initial multi-stop gradient approach produced black dots in the actual Android
renderer despite passing desktop capture checks. That approach was discarded.
The direct fragment program renders correctly in the ordinary Android release
APK. The same 16,425-pixel background region changed from 172 anomalous black
pixels to zero; the dark card and light Settings cards were visually inspected.
Cold launch and root navigation passed on the isolated emulator. These checks do
not establish other GPU, physical-phone or iOS compatibility.

Final static analysis and the isolated web-preview build passed; browser runtime
rendering was not rechecked in this phase. Documentation, contract and repository
validators passed. The ordinary Android APK passed 16 KiB ZIP alignment; the local
delivery APK/hash manifest was updated. GPU comparison evidence is stored as
`output/flutter-verification/android/gradient-gpu-check.json`, and
`gradient-alignment-release-home.png` records the final cold launch.


## Surface-shadow parameters

The approved renderer uses a Gaussian standard deviation of 10 physical pixels
at 2x scale, equivalent to 5 logical pixels. Flutter's `blurRadius: 10` converts
to a larger standard deviation. Window Light now uses the inverse Flutter
conversion and matches the reference's 26/255 shadow opacity. The RGB color and
three-pixel vertical offset remain unchanged.

All 82 screenshots were regenerated. Seventy-four screens improve and eight
remain unchanged; no page regresses. Mean channel error drops from 1.5145 to
1.4999/255. Card/reference crops were visually inspected, and both previously
measured interior gradient regions remain pixel-identical. The strict whole-page
gate still fails at **0/82 exact**, with unchanged reference artwork.

Static analysis and all 10 screenshot/interaction/reflow/high-contrast checks
passed, including 198 explicitly simulated atlas branches. Documentation,
contract and repository validators passed. No native operation, widget geometry
or interaction target changed.

The ordinary Android release APK rebuilt, passed 16 KiB ZIP alignment, installed
and cold-started on the isolated emulator. The final card/shadow screenshot was
inspected, and the sampled gradient region remained free of anomalous black
pixels. The local delivery APK/hash manifest and
`shadow-alignment-release-home.png` were updated.


## Long device names and room labels

Fixed-height device cards now truncate long names, room labels, readings and
captions within their existing text areas. The information-page hero also limits
its fixed-height name and room lines. Large-text layouts continue to flow with
full text. Accessibility semantics retain the complete name and room, and the
information rows continue to display the full saved alias.

A 73-character fictional alias exposed overlapping text in a compact card; actual
Android input additionally exposed overlap in the information hero. Two regression
checks cover card widths of 350 and 300 logical pixels, 100% and 200% text, full
semantics and exact card-click dispatch, plus the information heading at 300
logical pixels with both text scales. The full 63-test suite and static analysis
passed. All 82 screenshot metrics are unchanged from the preceding checkpoint:
mean channel error 1.4999/255 and **0/82 exact**. Original references are unchanged.

The ordinary Android release rebuilt, passed 16 KiB ZIP alignment and installed
on the isolated Android 15 emulator. Actual keyboard input saved the fictional
alias; force-stop and cold restart retained it. Final-build taps opened the exact
renamed card and its information page. Inspected screenshots show separated name
and room lines, and the information row/accessibility tree retain the full alias.
The final hero name ends at physical y=806 and the room begins at y=841. Captures
are `long-label-release-home.png`, `long-label-release-detail.png` and
`long-label-release-info.png` under `output/flutter-verification/android/`.
The local delivery APK and hash manifest were refreshed. No real account or
appliance was used, and native operation semantics are unchanged.


## Row-divider and title geometry

The row separator previously straddled the row boundary and antialiased an extra
physical pixel. It now paints a physical-pixel-aligned rectangular line beginning
at that boundary, including the source renderer's final endpoint. Titled groups
use a one-pixel bottom spacer: the measured 20-pixel title and 13-pixel top spacer
now total the reference's 34 pixels instead of accumulating 0.4 pixels per group.
Large-text content continues to flow, and row click regions are unchanged.

All divider coordinates in A05, C08, P03 and S07 match the original references at
780 x 1688. Image hashes and sampled scanlines are recorded in
`output/flutter-verification/row-divider-samples.json`. This verifies coordinates,
not complete pixel equality. The strict 82-page comparison remains **0/82 exact**.
Mean channel error changes from 1.4999 to 1.4822/255: 41 pages improve, 34 stay
unchanged, and seven regress (A01, A02, C06, C10, H07, S05 and S07). The regressions
are retained as unresolved visual work; they are not waived by the lower mean.
Chip hit-area spacing, dialog positioning and text rasterization still differ.
The prototype images and strict comparison criteria are unchanged.

Static analysis and the full 63-test Flutter suite passed, including the native
Mock click journey and 198 explicitly simulated preview branches. Temporary
layout probes were removed after measuring the offsets. No Core API, routing,
permissions or appliance-operation semantics changed.

The ordinary Android release APK rebuilt and passed 16 KiB ZIP alignment. A
separate actual-process export check on the preceding `eb72306` APK reproduced
`EXPORT_FAILED`: `file_selector_android` does not implement `getSaveLocation`.
The earlier simulated-picker tests did not cover that platform limitation. Android
system-document export is a confirmed remaining functional defect, not a passed
acceptance item.


## Android system-document export

Actual clicks in the ordinary APK reproduced an Android export failure before
this change: the desktop-oriented save-location API is not implemented by the
Android file-selector plugin. Android now uses ACTION_CREATE_DOCUMENT with JSON
MIME type and local-only providers, then writes the exact captured preview through
the granted document URI. It requests no broad storage permissions. A bounded
worker performs the write; success is returned only after the stream closes.
Cancellation, picker failure, write failure and activity destruction are distinct
bridge outcomes. Repeated clicks cannot open another export while one is pending.
No report is regenerated during the picker round trip.

Two channel tests verify exact preview transfer, cancel/success results and
failure/missing-confirmation rejection. The complete 65-test Flutter suite passed;
a subsequent native Mock click regression additionally verified that a second
click during a pending export makes no new picker call. Its Android integration
variant retains an explicitly simulated picker through the new channel; that
full on-device simulated journey was not rerun in this phase. Static analysis,
ordinary dual-ABI Android release compilation and 16 KiB ZIP alignment passed.

The final ordinary APK was installed on the isolated Android 15 x86_64 emulator.
Actual taps opened Settings, Diagnostics and the system Downloads picker. Back
cancelled without error or a created file. A second open/save returned to the
unchanged preview and reenabled export. Reading the resulting local file through
adb proved all 308 bytes equal the pre-export accessibility preview, including
the generated timestamp. Evidence and hashes are recorded in
`output/flutter-verification/android/export-system-picker-check.json`; screenshots
record the original failure, real system picker and successful return. The
Settings screenshot also records the preceding row-divider correction in the
actual Android renderer. No real account, appliance or diagnostic upload was used.

The delivery APK/hash manifest was refreshed. All 82 atlas metrics remain at the
row-geometry checkpoint: 0/82 exact, mean channel error 1.4822/255, with its seven
recorded visual regressions unresolved. iOS document export, physical devices and
full prototype parity remain unverified.


## Runtime diagnostics cards and complete report review

The native S07 page now uses the prototype's six-row report summary and four-row
latest-receipt card rather than displaying the entire JSON as the page body.
Values come directly from the loaded allowlisted report. Generated time is shown
in UTC; zero counts, uncertain outcomes and empty history are preserved. The last
retained attempt is marked as retained when attempts were omitted, rather than
being presented as complete evidence. Summary rows are read-only.

The original review note now opens a selectable, scrollable full-report dialog.
That view includes the unmodified report and all retained receipt summaries, not
only the compact card. Export continues to serialize the same loaded snapshot;
no new report is fetched when the dialog opens or the file picker returns. The
Window Light surface color is used for the report dialog. Explicit export retry
clears the previous export error, and loading/missing reports cannot be exported.
Failed reloads clear the old report. Late success or failure after navigation
cancellation/backgrounding cannot repopulate the report or publish a stale error.

Three added checks use the versioned fictional diagnostics example to verify
summary semantics, empty/truncated evidence, read-only rows, review clicks,
320-pixel/200% layouts, and stale/failing load behavior. Two decoded fixture
captures are in `apps/mobile/build/verification/diagnostics/`; their live data
values differ from the atlas and they do not establish pixel equality. The full
68-test Flutter suite and static analysis passed. The extended native Mock click
journey opens/closes the full report, compares its exact text, and verifies
cancel/save/failure/retry with the explicitly simulated picker. It now records
92 captures across the same 27 screen IDs. The isolated 82-state atlas comparison
remains 0/82 exact with mean channel error 1.4822/255; prior visual regressions and
other unverified acceptance items remain unresolved.

The final ordinary Android APK rebuilt, passed 16 KiB ZIP alignment, installed
and cold-started on the isolated Android 15 emulator. Actual taps opened the
summary cards, opened/closed full-report review, and saved through the local
system document picker. The resulting 308-byte file exactly matches the dialog's
captured text; summary values and the generated timestamp remain unchanged after
return. The existing earlier export was retained under its original name.
`output/flutter-verification/android/diagnostic-cards-check.json` records hashes,
actions and screenshots. The delivery APK/hash manifest was refreshed.

The owner's main-worktree native contract has advanced to API 1.14.1, while this
verified frontend build remains pinned to API 1.10.0. Integration of that later
contract is outstanding; this checkpoint does not claim current-main parity.


## Native API 1.14.1 integration checkpoint

The frontend branch merges committed Core baseline `7db8e9b`, preserving its
Flutter pages and earlier runtime evidence. Uncommitted main-worktree LAN changes
are excluded. Both independently developed Android file-lock fixes implement the
same Bionic behavior; the merge retains the upstream implementation and the
frontend's additional contention regression. SQLite schema 4 and protected account
payload 3 follow the current native contract. Existing protected profiles must not
be downgraded to a library that cannot read their payload version.

The 31 ordinary JSON methods remain compatible. This checkpoint does not yet
wire the three separate trusted approval exports or the new startup grant UI.
Elevated actions remain explicitly unavailable in Flutter, and Meiju startup stays
read-only. No owner account or appliance is used by the integration checks.

Executed native checks passed: formatting, warnings-denied workspace Clippy,
offline locked workspace build and tests (349 passed, two explicitly ignored),
35 CLI/runtime cases, the 31-method/51-example native ABI suite including trusted
approval and dishwasher admission, 33 Python tooling tests, six presentation
tests, and document/contract/repository validators. The system Python lacked the
required date-time checker; rerunning with the existing dependency-complete
virtual environment passed. The dual-ABI Android AAR rebuilt with eleven exports
per ABI and 16 KiB ELF alignment checks.

All 68 Flutter tests and static analysis passed against the rebuilt Linux library.
The complete Android native click journey passed in the actual Android 15 x86_64
application process (92 captures across 27 screen IDs; injected text input,
picker and lifecycle remain explicitly simulated). The ordinary dual-ABI APK
then rebuilt, passed 16 KiB ZIP alignment, installed and cold-started. Actual taps
opened the retained fictional device and refreshed its unknown state to the Mock
observation (26 degrees, cool, power off). Screenshots are retained as
`api-114-release-home.png`, `api-114-release-device.png` and
`api-114-release-read.png` under `output/flutter-verification/android/`.

Strict comparison was rerun after the full Flutter suite: **0/82 exact**, mean
channel error **1.4822/255**. The seven prior row-geometry regressions remain.
No visual acceptance, real-account/physical-device support, iOS validation or
trusted Flutter approval flow is claimed by that integration checkpoint.


## Trusted confirmation implementation

The three separate trusted C exports are now lazily resolved only after API 1.13
negotiation. Prepare and execute run in workers and free each owned response once;
discard consumes its Boolean result without treating it as an owned string.
Native previews are deeply immutable, handle-owned Dart objects. Controller/UI
code retains the exact returned scope and original request ID, requires explicit
route selection and a distinct Confirm Command click, and consumes consent before
awaiting execution. Background/close/navigation and conservative monotonic expiry
invalidate previews; late preparation cannot restore a cancelled confirmation.
There is no automatic preparation, consent, retry or route fallback.

The existing described-action sheet now renders elevated confirmation using
Window Light surfaces and shared action-effect labels. High-risk controls and
older libraries remain unavailable. An interrupted confirmation reply opens an
explicit original-request lookup; returned accepted/unknown/failed receipts retain
the existing outcome UI. Meiju remains read-only because startup grant UI is not
part of this phase. Only fictional Mock profiles were exercised.

Four added regression scenarios cover immutable nested arguments and transports,
ordinary-policy refusal with zero attempts, handle ownership, single consumption,
discard/navigation/background and a late preparation, older-version admission,
read-only refusal, lost-reply receipt recovery without replay, and actual review,
cancel, expiry, confirmation and receipt-lookup clicks. Captures with real fonts
include the review, accepted outcome and lost-reply states; 320-logical-pixel
review at 200% text is also checked. These are native-backed dynamic UI evidence,
not new authoritative prototype references or pixel equality.


Final checks for this phase passed: all 72 Flutter tests, static analysis, six
presentation tests and document/contract/repository checks. The Android process
suite passed all four approval scenarios, including a real 61-second expiry wait;
foreground changes remain injected and lost replies are an explicit fixture.
Android review/accepted/lost-reply captures are under
`output/flutter-verification/android/approval/`. The ordinary dual-ABI APK then
rebuilt, passed 16 KiB ZIP alignment, installed and cold-started with its saved
Devices page and packaged labels. The earlier Android asset failure was fixed by
bundling a local catalog copy whose exact source equality is tested; repository
traversal asset paths are not used.

The final strict 82-state run remains **0/82 exact**, mean channel error
**1.4822/255**. No reference was changed or blessed. Meiju startup grant UI,
real accounts/appliances, full localization, physical phones and iOS acceptance
remain outstanding. This is a verified frontend phase, not goal completion.


## Owner-requested vertical centering — October 16, 2026

The owner explicitly revised the prototype: vertically center text throughout
control rows, not only C01–C10. The shared raster renderer now measures text
bounds for labels, values, multiline choices, buttons, fields, chips, section
bands, slider headings, pairing codes and appearance labels. All 82 reference
screens and 21 boards were regenerated as a design revision. This is not an
automatic reference blessing from Flutter captures. P08's amended application
copy and all manual native-contract inventory amendments remain intact.

Flutter uses ordinary symmetric line layout for control copy, removes the
previous upward button/stepper translations, centers chip and code content, and
explicitly centers field text. Home room-filter labels are centered above their unchanged underline positions.
Native enum controls and appearance labels share
the correction. Stepper symbols retain their labeled interactive widgets and
use centered vector marks. Titles and paragraphs keep their existing layout.
The prototype now gives wrapped chip runs the same 48-pixel pitch as Flutter's
interaction regions. Progress caption spacing also matches the existing
38-pixel-plus-caption block height, avoiding downstream card displacement.

Executed checks:

- `fvm flutter test --no-pub --reporter expanded`: all 72 tests passed, including
  all 82 screenshot states, 198 explicitly simulated atlas branches, native Mock
  click journeys and large-text/high-contrast reflow.
- `fvm flutter analyze --no-pub`: no issues.
- Prototype render/build/validate: 82 screens, 21 boards, 138 image hashes,
  547 local links, 14 journeys, 198 branches, zero layout overflow.
- `python3 apps/mobile/tool/compare_screens.py --require-exact`: **failed**, with
  **0/82 exact**, mean absolute channel error **1.5424/255** against the revised
  references. These metrics are not comparable to old-reference alignment metrics
  as an acceptance trend. Remaining raster/layout differences and the explicit
  P08 contract mismatch stay visible in `output/flutter-verification/index.html`.

The earlier experimental one-pixel trailing-value translation was discarded in
favor of this shared centering rule. No Core API, operation authority, data schema
or supported-platform claim changes in this phase.

The complete `integration_test/native_journeys_test.dart` also passed in the
Android 15 API 35 x86_64 application process using the embedded Mock Core
(2m20s). This suite injects keyboard input, lifecycle changes and save-picker
results; it does not establish physical-device or real-provider acceptance.
Document, contract and repository validators and `git diff --check` passed.

The final ordinary dual-ABI release APK was rebuilt after the room-filter change,
passed ZIP/ELF 16 KiB alignment and exact 11-export checks for both Core ABIs,
installed and cold-started on the same emulator. A saved fictional Mock card
survived the upgrade/restart; the centered room filter and card opened normally.
Explicit Refresh State returned target 26, cool mode and power off. Settings
opened with its expected actions. The final ordinary APK captures are
`output/flutter-verification/android/centering-release-{home,device,settings}.png`.
No real appliance was used. The installable development-signed APK and its hash
manifest remain local under `output/flutter-verification/android/`.


## Fractional control geometry — October 16, 2026

A font-metric probe confirmed that Flutter rounds the 13-pixel control line from
18.2 to 18 pixels and the 14-pixel title line from 19.6 to 20 pixels. Consequently,
repeated choice rows were 0.2 pixels too short and sectioned-card title bands were
0.4 pixels too tall. The title band now measures 34 pixels. Choice rows measure
the actual wrapped/scaled text and retain the reference's minimum height of
34 plus 18.2 pixels per unscaled line. Text and selection marks remain centered;
large text may grow the row and is never clipped to a fixed screenshot height.

The added geometry regression checks three repeated sectioned groups, successive
single-line choices and a two-line choice with centered content. All 73 Flutter
tests and static analysis passed, including screenshot, accessibility and native
Mock click checks. The authoritative centered references were unchanged.

The strict comparison remains **0/82 exact**, mean channel error **1.4886/255**
(previously 1.5424). Thirteen states improve, 68 are unchanged and S03 regresses
by 0.0038; its remaining raster difference is not hidden by a tolerance. C02
improves from 2.1464 to 1.3482, R03 from 2.7279 to 1.9918 and P01 from 2.9174 to
2.3522. Per-screen deltas are in
`output/flutter-verification/fractional-geometry-comparison.json`.
The unchanged P08 reference still depicts a superseded manual-discovery journey;
that contract/design mismatch is separate from this geometry correction.

The ordinary dual-ABI release APK rebuilt successfully and passed 16 KiB ZIP
alignment, install and cold-start checks on the Android 15 API 35 emulator.
No native API, permission, data or prototype-content changes were made.

The installed release also passed native pointer clicks on Connections: select
Local Network, restore Auto, and Save Preference returned to Settings with Auto
shown. The selected row capture is
`output/flutter-verification/android/geometry-release-connections.png`.
Both packaged Core libraries are byte-identical to the previously validated
centering APK, retaining its exact exports and ELF alignment evidence.

## P08 combined-result alignment — October 16, 2026

The owner selected implementation under the current native contract. P08 now
shows the completed combined account connection/retrieval result, replacing the
obsolete manual Find Devices step. The raster reference is independently composed
by the existing Python renderer; no Flutter capture was used as a baseline.
The Flutter catalog and full atlas data are identical and the validator checks
that equality. Five result actions replace the two obsolete branches, increasing
the atlas from 198 to 201 simulated branches. Production connection still waits
for the single native report; its behavior and Core ABI are unchanged. Android
clicks also exposed fictional titles in the native connection path: P03 now
shows Connect Meiju and P13 uses Platform connection. The isolated fictional
prototype retains its own example platform names.

The new regression test connects an isolated fictional account once, compares
production P08 content with the reference definition, and clicks View Devices.
It confirms there is no follow-up refresh or discovery request after connecting.
Existing account checks retain complete-empty, partial, AUTH_REQUIRED, pending
recovery, cancellation, late-result and exact local removal scope coverage.
These are fixture checks, not real provider acceptance. Separate authoritative
artwork for the busy/error/interrupted variants is still required.

Executed verification:

- All **74 Flutter tests** passed, including 82 regenerated captures, all **201**
  simulated branch clicks, native Mock interactions and account fixture checks.
  After correcting the native titles, all seven account checks and static
  analysis passed again, including the added unavailable-connection click case.
- FVM static analysis, documentation/contracts/repository validation and
  `git diff --check` passed.
- The prototype validator passed: 138 images, 547 links, 21 boards, 82 states,
  14 journeys, 201 branches and zero content overflow. Existing P10/P11/S04
  inventory amendments were preserved when rebuilding the generated atlas.
- Strict screenshot comparison still fails: **0/82 exact**, mean channel error
  **1.3958/255**. P08 improves from 10.6171 to 3.0031 after removing its semantic
  mismatch; text rasterization and fractional layout differences remain. The
  obsolete P08 intentional-mismatch annotation was removed from the report.
- The dual-ABI development release APK rebuilt with FVM and passed 16 KiB ZIP
  alignment, emulator installation and cold launch. Both Core libraries are
  byte-identical to the preceding validated APK; bundled prototype bytes match
  source. The ordinary Android Mock app opens Platforms and Add Platform with
  the account-connection limitation visible. Choosing Meiju opens the unavailable
  state with Platform connection as its heading; Back to Accounts returns to
  the platform root. Screenshot: `output/flutter-verification/android/p08-release-account-gate.png`.

Logs: `/tmp/homeverb-p08-tests.log`, `/tmp/homeverb-p08-analyze.log`,
`/tmp/homeverb-p08-pixels.log` and `/tmp/homeverb-p08-release.log`.
No real account, appliance, physical phone or iOS test was performed. This phase
does not complete strict visual acceptance or the full original goal.

## Fractional paragraph heights — October 16, 2026

A regression test reproduced a three-line 13-point paragraph occupying 54 logical
pixels instead of the specified 54.6. The paragraph layout now measures its actual
wrapped lines using the inherited font metrics and system text scale, then retains
the fractional 1.4 line-height multiple. Only explanatory note blocks opt in;
existing titles and vertically centered controls keep their established layout.
Text remains a native Text widget with its semantics and wrapping intact.

The regression covers explicit lines, natural wrapping, successive block origins,
13- and 14-point sizes, and 100%/200% text. The full **76-test** suite and static
analysis passed after narrowing the change to paragraph blocks. This includes
all 82 screenshot states, 201 simulated branches, native Mock fixture interactions,
account results and narrow/large-text/high-contrast checks.

Strict comparison still fails: **0/82 exact**, mean channel error **1.3467/255**
(previously 1.3958). Twenty-nine screens improve, 43 are unchanged and ten regress.
P08 improves from 3.0031 to 1.6800; its primary button's top pixel at x=100 now
matches reference y=708 instead of the previous y=706. The remaining one-pixel
bottom-edge difference is still visible. S09 improves from 1.8609 to 1.4386.
The largest remaining regressions are C14 (+0.2463) and D03 (+0.1356); their
reference/before/after images were inspected and preserved. Fractional logical
geometry does not establish identical glyph rasterization or pixel-aligned paint.
No prototype images, screenshot tolerances or native contracts changed.

Evidence: `output/flutter-verification/paragraph-comparison.json` and
`paragraph-P08.png`, `paragraph-C14.png`, `paragraph-D03.png` in the same directory.
Logs: `/tmp/homeverb-paragraph-before.log` (reproduced failure),
`/tmp/homeverb-paragraph-tests-final.log`, `/tmp/homeverb-paragraph-analyze.log`
and `/tmp/homeverb-paragraph-pixels-final.log`.

The full Android native Mock click journey passed on API 35 x86_64 in 2m18s.
Its text input, file picker and lifecycle transitions remain injected test inputs,
not physical keyboard or operating-system picker acceptance. Integration teardown
removed the test package before its on-device screenshot folder could be copied;
no complete Android screenshot archive is claimed for that run. The retained log
is `/tmp/homeverb-paragraph-android.log`.

The ordinary dual-ABI development release rebuilt successfully, passed 16 KiB
ZIP alignment, and was installed and cold-launched after test teardown. Both Core
libraries are byte-identical to the preceding validated APK. Native pointer
clicks opened Settings > Connections; its wrapped explanation was inspected,
and Save Preference returned to Settings with Auto retained. The capture is
`output/flutter-verification/android/paragraph-release-connections.png`.
There was no real account, physical appliance, physical phone or iOS execution.

## Inclusive button backgrounds — October 16, 2026

Both bundled Inter fonts are byte-identical to the prototype renderer's fonts.
The remaining differences are not caused by a different font file. One measured
paint difference was the prototype's inclusive rounded-rectangle boundary: at
2x, a 350-by-48 logical face includes 701-by-97 physical pixels, while the prior
native face excluded the terminal right and bottom pixels.

Four locally generated 61-by-61 decorative corner assets now paint button faces
with nine-slice stretching and the inclusive terminal pixel. They contain no
text or page images. Labels, hit targets, focus, semantics, disabled behavior and
state feedback remain native FilledButton behavior. High contrast retains the
existing solid outlined controls. The independent prototype is unchanged.
`python3 apps/mobile/tool/generate_button_assets.py` reproduces these assets.

All **77 Flutter tests** and static analysis passed. The new regression verifies
both terminal edges, the secondary border, the unpainted pixel immediately beyond
the edge, unchanged 350-by-48 native control bounds, enabled tap semantics and
working/disabled callbacks. Existing narrow/large-text/high-contrast checks,
201 simulated branch clicks and native Mock/account interactions also passed.
The final-suite captures are byte-identical to the compared capture set.

Strict comparison remains failed: **0/82 exact**, mean channel error
**1.2943/255** (previously 1.3467). Sixty-five screens improve, 12 are unchanged
and five regress. The largest regression is S10 (+0.0473); P08 improves from
1.6800 to 1.5488. Four P08 button-background edge regions, excluding the label,
now match exactly. Its label still starts at y=744 instead of reference y=745;
partial-region equality is not whole-button or screen equality. Glyph placement,
other primitives and remaining layout differences still require correction.

Evidence: `output/flutter-verification/button-comparison.json` and
`button-pixel-audit.json`. Logs: `/tmp/homeverb-button-tests.log`,
`/tmp/homeverb-button-analyze.log` and `/tmp/homeverb-button-pixels.log`.

The complete Android native Mock journey passed in 2m20s on API 35 x86_64.
The ordinary dual-ABI development release rebuilt, passed 16 KiB ZIP alignment,
and was installed and cold-launched. Both packaged Core libraries are unchanged
from commit `55467c2`, and all four packaged decorative assets match source.
The generated assets also reproduce byte-for-byte when the script is rerun.

Ordinary native pointer clicks opened Add Platform and Back returned to the
original Devices page. The new button backgrounds were inspected in
`output/flutter-verification/android/button-release-home.png` and
`button-release-platforms.png`; `button-release-clicks.json` records the result.
The Android journey still injects text input, file picker and lifecycle events.
No physical phone, real account, appliance or iOS execution was performed.

## Native button ink centering — October 16, 2026

Button labels now center the visible glyph group using baseline-relative bounds
from the bundled Inter SemiBold font. Native Text still paints every glyph;
the metadata contains only per-character vertical bounds and the font hash.
Flutter's actual alphabetic baseline is used because its rounded line metrics
can differ by 0.4 logical pixels. Pixel rounding occurs in screen coordinates,
so fractional paragraph and sheet origins do not add another one-pixel offset.
Paint, semantic transforms and hit testing share the same translation. Unknown
glyphs and system bold/spacing overrides retain ordinary native text layout.

The progress caption now preserves its specified 16.8 logical pixel line height
instead of rounding it to 17. This corrects the downstream button origin; it
also exposes remaining glyph-placement differences in the following paragraph.
No native contract, approved reference image or screenshot tolerance changed.

All **80 Flutter tests** and static analysis passed. New raster coverage checks
eight labels, including ascenders and descenders, at four fractional screen
origins against independent font-derived bounds. Additional cases cover natural
and explicit wrapping, 100%/200% text, unknown-glyph fallback, native semantics
and taps. Progress-caption geometry is checked at both text sizes. Existing
82-state captures, 201 simulated branches, native Mock interactions, account
fixtures, high contrast and narrow layouts also pass their functional checks.

Strict comparison still fails: **0/82 exact**, mean channel error **1.2137/255**
(previously 1.2943). Sixty-four states improve, 16 are unchanged and two regress:
C03 (+0.1206) and A03 (+0.0090). Their reference/before/after images were inspected;
the fractional progress-caption correction is retained rather than restoring an
incorrect line box to compensate for other glyph rasterization differences.
This is not whole-screen visual acceptance. The remaining body/row glyph
placement, antialiasing and layout differences still require correction.

Evidence: `output/flutter-verification/label-comparison.json` and
`label-C03.png`, `label-A03.png`, `label-H09.png`, `label-S09.png`, `label-P08.png`.
Logs: `/tmp/homeverb-label-tests-final.log`,
`/tmp/homeverb-label-analyze-final.log` and `/tmp/homeverb-label-pixels-final.log`.

The final button renderer passed the full Android native Mock journey in 2m29s
on API 35 x86_64. Text input, file picker and lifecycle remain injected inputs.
A partial archive of 59 native journey captures was copied before teardown;
it is not a complete screenshot archive. The subsequent progress-caption-only
change is covered by the final widget suite and included in the ordinary APK.
The native journey log is `/tmp/homeverb-label-android-final.log`.

The ordinary dual-ABI APK rebuilt, passed 16 KiB ZIP alignment, and was installed
and cold-launched. Both Core libraries are byte-identical to commit `21a35eb`;
the four decorative faces and font metrics in the APK match source. Native
pointer clicks opened Add Platform and Back restored the original Devices page.
Captures: `output/flutter-verification/android/label-release-home.png` and
`label-release-platforms.png`; click evidence: `label-release-clicks.json`.
No real account, appliance, physical phone or iOS execution was performed.

## Shared control-text ink centering — October 16, 2026

The font-derived centering now covers native control text as well as buttons:
row labels/values, grouped headings, choices, tabs, freshness captions, skin
labels and verification codes. The committed `text-ink-metrics.json` records
independent bundled-font bounds for the eight used Regular/SemiBold variants
from 10 to 32 points. `python3 apps/mobile/tool/generate_text_metrics.py`
reproduces it; the four decorative button faces remain unchanged.

The native paragraph snaps its painted alphabetic baseline in logical pixels
before canvas scaling. Accounting for that step fixes the 10-point caption's
one-pixel discrepancy without adding a per-font or per-screen pixel offset.
The shared render wrapper measures after native layout and forwards intrinsic
and dry sizing, preserving ChoiceChip and ActionChip behavior. Paint, semantics
and hit testing use the same translation. Unknown glyphs, clipped text and
system bold/spacing overrides retain native fallback behavior. No Core contract,
reference image, screenshot tolerance or native operation semantics changed.

All **81 Flutter tests** and static analysis passed. New raster checks compare
ten labels across every bundled control style with independent visible bounds.
The existing eight-button/four-origin pixel checks also pass. The full suite
covers all 82 captures, 201 simulated branches, native Mock and account fixture
interactions, narrow/large-text/high-contrast layouts and keyboard avoidance.
An existing assertion now checks the unchanged native line-box layout separately
from the new visible-glyph paint assertions.

Strict comparison remains failed: **0/82 exact**, mean channel error
**0.9358/255** (previously 1.2137). Sixty-nine states improve, 13 are unchanged,
and none regress. Nine of C01-C10 improve; C03 remains unchanged. This does not
resolve the earlier C03 body-text regression or establish complete screen parity.
Remaining text rasterization, body-text origins and primitive/layout differences
still require correction.

Evidence: `output/flutter-verification/control-ink-comparison.json` and
`control-ink-C01.png`, `control-ink-C04.png`, `control-ink-C05.png`,
`control-ink-R01.png`, `control-ink-S10.png`, `control-ink-W02.png`.
Logs: `/tmp/homeverb-control-targeted.log`, `/tmp/homeverb-control-tests-final.log`,
`/tmp/homeverb-control-analyze-final.log` and `/tmp/homeverb-control-pixels.log`.

The complete Android native Mock journey passed on API 35 x86_64 in 2m25s;
text input, file picker and lifecycle transitions remain simulated. The ordinary
APK rebuilt and passed 16 KiB ZIP alignment. Both Core binaries are unchanged
from `f5b35c8`; packaged font metrics and button faces match source, and the
obsolete button-only metric asset is absent.

Native pointer clicks discovered Mock, opened its card, read target 26/cool/power
off and opened Choose Mode. The device rows and selected/unselected labels were
visually inspected in `output/flutter-verification/android/control-ink-release-device.png`
and `control-ink-release-mode.png`. This inspection also exposed an existing
sheet safe-area defect: part of Cancel overlaps the Android navigation area.
That defect remains pending a separate layout correction; the successful test
journey does not establish unobscured system-inset behavior. No complete Android
capture archive was retained for this run. No real account, appliance, physical
phone or iOS execution was performed.

## Native sheet bottom safe area — October 16, 2026

The Android mode-sheet inspection exposed Cancel ending at physical y=2316,
42 pixels below the navigation area's top at y=2274. Native bottom sheets now
include the current bottom safe inset inside their scrollable content padding.
Deterministic prototype chrome and centered dialogs retain their existing layout.
Scaffold continues to handle keyboard resizing; the inset follows MediaQuery's
current padding rather than adding keyboard height a second time.

A regression using the emulator's 1080-by-2400 size, 2.625 pixel ratio and
48-logical-pixel bottom inset failed before the change: the button ended at
882.2857 instead of at or above the safe boundary of 866.2857 logical pixels.
It now passes and taps near the bottom of the full-height action successfully.
All **82 Flutter tests** and static analysis pass. All 82 prototype-sized captures
are byte-identical to the preceding control-centering phase; strict comparison
remains **0/82 exact**, mean channel error **0.9358/255**.

The final ordinary dual-ABI APK rebuilt, passed 16 KiB ZIP alignment, and retained
the same two Core library hashes and packaged assets. It was installed and
cold-launched with the retained Mock card. Native pointer clicks opened the
card, refreshed its fictional observation and opened Mode. Cancel now occupies
physical bounds `[53,2064][1028,2190]`, clearing system navigation by 84 pixels.
A tap four pixels above its bottom edge dismissed the sheet and preserved cool
mode. The screenshots were inspected. No real account, appliance, physical
phone or iOS execution was performed.

Evidence: `output/flutter-verification/android/safe-area-release-mode.png`,
`safe-area-release-device.png` and `safe-area-release-clicks.json`.
Logs: `/tmp/homeverb-safe-area-before.log`, `/tmp/homeverb-safe-area-after.log`,
`/tmp/homeverb-safe-area-tests.log`, `/tmp/homeverb-safe-area-analyze.log` and
`/tmp/homeverb-safe-area-release.log`. The full Android native journey described
above ran before this padding-only correction; the final APK's system-inset
behavior was verified by the direct native pointer sequence recorded here.

## Native body and heading ink alignment — October 16, 2026

Body copy and headings now use the bundled font's visible glyph bounds instead
of an estimated upward offset proportional to font size. The shared native-text
wrapper supports both centered controls and top-aligned copy. Each wrapped line
is translated to its own rounded ink-top coordinate at the specified 1.4 line
advance. Painting clips between native ink bands while retaining one native
Text paragraph, its full label and intrinsic layout. Unsupported glyphs,
truncation and system spacing overrides retain the previous native fallback.
The generated metrics cover the 25 Regular/SemiBold styles used by Copy.

The climate target now uses the prototype's y=52 coordinate; the previous y=49.5
compensated for the old font offset. Header branding retains its y=62 ink origin.
No reference artwork, comparison threshold, Core contract or operation semantics
changed. Controls remain native widgets; no text is replaced with raster assets.

All **83 Flutter tests** and static analysis pass. The added raster check verifies
eight single-line styles and a wrapped paragraph at four fractional origins,
including the independently rounded second-line top. The complete regression
also captures all 82 states, clicks all 201 explicitly simulated branches, and
checks native Mock/fixture interactions, high contrast, enlarged text, keyboard
avoidance and safe-area behavior.

Strict comparison still fails: **0/82 exact**, mean channel error **0.7289/255**,
compared with 0.9358 before this phase. All 82 states improve; none regress. C03
improves from 1.3023 to 0.7404. Remaining rasterization, horizontal placement,
primitive and layout differences still prevent exact-screen acceptance.

Evidence: `output/flutter-verification/body-ink-comparison.json`, the live
`index.html` report and `body-ink-{C03,C04,A01,S10,D01,W02}.png` review panels.
Logs: `/tmp/homeverb-body-tests.log`, `/tmp/homeverb-body-analyze.log` and
`/tmp/homeverb-body-labels.log`.

The complete Android native Mock journey passed on API 35 x86_64 in 2m28s
after restarting the existing isolated emulator without wiping its data. Input,
file-picker and lifecycle steps in that journey are injected. Its log is
`/tmp/homeverb-body-android.log`; no complete native capture archive was retained.
No real provider, appliance, physical phone or iOS acceptance is claimed.

The ordinary dual-ABI development-release APK rebuilt and passed 16 KiB ZIP
alignment. Both Core libraries are byte-identical to `c6231cc`; packaged font
metrics, atlas data and decorative faces match source. Native pointer clicks
opened Settings and Connections, saved Auto, discovered the fictional Mock
appliance, opened its card and refreshed target 26/cool/power off. The mode
sheet's Cancel remains above Android navigation and dismisses without changing
mode. The Connections paragraph, climate value and sheet screenshots were
inspected. Evidence: `android/body-ink-release-{settings,connections,device,mode}.png`
and `android/body-ink-release-clicks.json` under `output/flutter-verification/`.
Build log: `/tmp/homeverb-body-release.log`.

## Native form and reorder alignment — October 16, 2026

Form labels and validation messages now retain fractional line-box heights and
the prototype's two-logical-pixel horizontal inset. Native TextField content
padding centers the visible glyph bounds for each value or hint without changing
total padding or field height. A controller listener updates the padding for
both typed and programmatic changes; caret, selection and editing remain native.
The shared font metadata adds the 15-point Regular variant used by inputs.
Editable-specific bounds use nonempty glyph-mask rows: a dedicated bullet check
reproduced a two-physical-pixel offset caused by blank rows in the font's layout
bounds, then passed with the visible-ink bounds. Prototype text retains its
separate font-anchor bounds, matching the reference renderer's anchor behavior.
Existing control-ink checks also pass.
Unsupported glyphs and system text overrides retain native padding fallback.
The same padding is applied to device-parameter and protected account fields.
Password geometry is measured from the masking bullet, never from hidden glyphs;
a fixture assertion verifies identical padding for different hidden characters.
Existing masking, IME-learning restrictions, validation and protected-buffer
clearing remain intact. No real sign-in was performed.

Displayed verification codes collapse repeated separator spaces in the same way
as the independent reference renderer. This changes presentation only and does
not implement provider verification. Reorder handles use the reference's narrow
three inclusive strokes, with the corresponding value spacing. Native room and
device reorder rows use centered labels; their drag/menu hit areas are retained.

The final **84 Flutter tests** and static analysis pass. The new raster check
covers numeric and descender-bearing values, live editing, selection position,
programmatic clearing, hint centering and unchanged 50-logical-pixel field height.
The complete suite covers all 82 captures, 201 simulated atlas branches, native
Mock/fixture interactions, forms, keyboard avoidance and large-text layouts.
A high-contrast P03 heading initially failed the contrast gate after fractional
form alignment. One extra logical pixel of heading padding separates text from
the thicker card border. The unchanged contrast gate now passes on all states,
including 200% text. All default captures remain byte-identical after that
high-contrast-only fix; P03 is retained as an additional accessibility capture.

Strict visual acceptance remains failed: **0/82 exact**, mean channel error
**0.6725/255**, previously 0.7289. Twelve states improve, 70 remain unchanged,
and none regress. Reference images and the exact-match gate were not changed.
No Core API, routing, approval, credential or execution semantics changed.

Evidence: `output/flutter-verification/fields-comparison.json`, the full
`index.html` report and `fields-{C14,P05,R05,R01,D03,P03}.png` review panels.
Logs: `/tmp/homeverb-fields-targeted.log`, `/tmp/homeverb-fields-tests-final.log`,
`/tmp/homeverb-fields-analyze-final.log`, `/tmp/homeverb-fields-contrast.log`
(reproduced failure) and `/tmp/homeverb-fields-contrast-fixed.log`.

The full Android native Mock journey passed on API 35 x86_64 in 2m25s after
all three field paths were integrated and before the final glyph-mask asset
refinement. Its input, picker and lifecycle transitions are injected; it does
not establish physical-phone, real-provider, real-appliance or iOS acceptance.
The log is `/tmp/homeverb-fields-android-final.log`.
The final ordinary dual-ABI development-release APK rebuilt, passed 16 KiB ZIP
alignment and retained unchanged Core libraries and source-matching packaged
assets. The installed APK was exercised with real pointer and OS keyboard events:
create Living Room, save with the keyboard open, open the reorder/menu handle,
rename and cancel, force-stop and cold-launch, and verify the saved room persists.
Long-press selection, the native Select all toolbar and replacement with Kitchen
were verified; Cancel then preserved Living Room. An earlier keyboard-shortcut
selection attempt appended text instead and was canceled; it is not counted as
a successful replacement. Screenshots and UI label readbacks were inspected.

Evidence: `output/flutter-verification/android/fields-release-{keyboard,rooms,restarted,selection-menu,selected-edit}.png`
and `fields-release-clicks.json`. Build log: `/tmp/homeverb-fields-release-final.log`.
Final APK SHA-256: `abc631f0a48b13891531a54838ada7bb140001913dcf8fecb23fb757ed274b15`.
Full Android screen-reader acceptance remains unverified; the empty EditText's
uiautomator node did not expose the wrapper label. No real account, appliance,
physical phone or iOS execution was performed.

## Shared decorative glyph alignment — October 16, 2026

Navigation, header, appliance and stepper icons now use 25 small tintable masks
exported from the approved prototype's drawing methods. The atlas is 7,787 bytes;
its generator reads only the Canvas drawing methods and never imports the
renderer or regenerates reference pages. Running it twice produces identical
assets. Masks preserve the independent renderer's integer outlines and endpoints;
paint origins snap to physical pixels. Native controls retain hit targets, focus,
semantics and high-contrast tinting. Other icon sizes retain the vector fallback.
The masks contain no labels, complete controls or page captures.

Choice checkmarks retain their specified 14-logical-pixel size instead of being
stretched by the circular parent, and their right inset matches the reference.
The home dropdown's existing text symbol now has font-anchor metadata, correcting
its downward fallback offset. The native body-ink regression includes that symbol.

All **84 Flutter tests** and static analysis pass. The complete suite captures
all 82 states, clicks all 201 explicitly simulated atlas branches, and exercises
native Mock/fixture interactions, forms, large text and high contrast. Final
strict comparison remains **0/82 exact**, with mean channel error **0.6451/255**,
compared with 0.6725 before this phase. All 82 states improve; none regress.
Intermediate icon scaling briefly regressed the reduced S11 preview; the final
physical-pixel placement and geometry corrections remove that screen regression.
Four independently cropped D01 icon regions (Search and all three navigation
icons) are exactly equal. This does not establish whole-screen equivalence.

Evidence: `output/flutter-verification/glyph-comparison.json`,
`glyph-pixel-audit.json`, `glyph-{D01,C02,C09,C11,P05,S11}.png` review panels and
the full `index.html` report. Logs: `/tmp/homeverb-glyph-position.log`,
`/tmp/homeverb-glyph-tests-final.log` and `/tmp/homeverb-glyph-analyze.log`.
No reference image, comparison tolerance, Core API, credential or execution
semantics changed.

The ordinary dual-ABI APK rebuilt in 26.9s and passed 16 KiB ZIP alignment.
Its Core libraries match the previous phase byte for byte; packaged glyph masks,
font metrics, atlas data and decorative button faces match source. The installed
APK passed actual pointer checks for Platforms, Devices, Search, Back, Add Device,
explicit Mock discovery, the saved card, Refresh State and Mode. Selecting heat
moves the native checkmark; Cancel preserves the observed cool mode, target 26
and power off. The resulting screenshots and UI labels were inspected.

Evidence: `output/flutter-verification/android/glyph-package-check.json`,
`glyph-release-clicks.json` and
`glyph-release-{platforms,devices,search,home,device-read,mode,selected,cancel}.png`.
Build log: `/tmp/homeverb-glyph-release.log`. APK SHA-256:
`3ef0c9f3598e7025cea550a8f1e3b70c8df96b9551d8dd6539338dbbbae2ca2b`.
The full Android integration journey was not repeated for this decorative phase;
the prior journey's scope and limitations remain recorded above. No real account,
appliance, physical phone, iOS or new Keystore acceptance is claimed.


## Source-only paused checkpoint — October 16, 2026

At the owner's request, the existing unfinished surface experiment is committed
on `codex/flutter-ui` so another computer can recover it directly from Git.
This checkpoint adds the current `skin.dart`, its generator and 24 decorative
corner/border masks (11,946 bytes total), plus portable instructions in
[README.md](README.md). No further pixel correction was implemented. APKs, AARs,
build caches, screenshots, reports, logs, IDE settings and the historical local
archive are excluded; historical evidence paths remain workstation-local.

Checks repeated for this exact source checkpoint:

- `cargo build -p homeverb-ffi --offline --locked`: passed.
- `fvm flutter analyze --no-pub`: passed with no issues.
- `fvm flutter test --no-pub --reporter expanded`: all 84 tests passed, including
  the 82-state widget capture and existing synthetic/native Mock regressions.
- The surface generator ran in an isolated temporary directory; all 24 output
  PNGs match the committed assets byte for byte.
- Documentation, contract, repository and existing commit-history validation
  passed. The contract check used the existing development virtual environment;
  the system Python first refused validation because its date-time checker was
  missing. On another computer, install `requirements-dev.txt` before validation.
- `git diff --check`: passed.

Local logs: `/tmp/homeverb-portable-{native-build,analyze,tests}.log`.
The earlier comparison remains 0/82 exact, mean channel error 0.5268/255, with
the known S11 regression and endpoint-rounding issue described in README.md.
The image comparison was not rerun for this source-only checkpoint. Android/iOS
builds, GPU rendering, native pointer checks, physical devices and remote CI were
not repeated. Passing the Flutter suite does not complete visual or mobile
acceptance. Native API 1.14.1, storage formats and Core semantics are unchanged;
main's newer Core integration remains separate work.


## Flutter session control permissions

The Meiju app starts read-only. Device Information -> Review control options now
renders exact model-scoped power, temperature, vendor fan and explicit power-on/mode
choices from API 1.16 inspection. A separate review and consent click constructs
only those existing startup grants. The trusted controller cancels work, closes
Core and reopens the same private profile. Grants remain in process memory and are
never imported from preferences or ordinary requests. Navigation/backgrounding
invalidates pending consent, including while reopening; failures remain closed
with an explicit Reopen Read-only recovery action.

Consent itself performs no discovery, provider request or appliance write. The
separate Retrieve Control Capabilities button explicitly refreshes the selected
account and descriptor. Core and the adapter still check every command. Revoke
Session Controls closes/reopens without grants; retained descriptors and receipts
are preserved. Process restart also starts read-only. Dishwasher and high-risk
permission setup are not added by this AC phase.

Five focused Flutter tests passed, covering actual native startup with fictional
exact grants, two-step UI consent, model-specific limits, unknown constraints,
separate authority, no implicit networking, revocation and lifecycle races. The
existing account/current-contract/trusted-approval regression selection also
passed (18 tests before the two new tests were added). This establishes host
integration, not real-provider or physical AC success. Native API 1.22, SQLite 5,
protected payload 3 and execution outcomes are unchanged.


## Flutter internationalization: October 16, 2026

The app adopts the LocalSend pattern using pinned slang/slang_flutter 4.19.0:
generated typed catalogs, TranslationProvider, LocaleSettings, Material delegates
and a persisted nullable language choice. Settings > Language supports system
default, en-US and zh-CN. The generator stages 1,120 shared messages, validates
keys/placeholders and preserves the existing English approval asset contract.
Chinese authoring remains in the shared zh-CN JSON; generated Dart uses escapes.
User/provider names remain literal, while authored labels, parameterized messages
and known error explanations are localized. Core request IDs, action tokens,
arguments, receipts, native API and storage formats are unchanged.

Executed source checks include Flutter analysis; 25 focused account, approval,
appliance, diagnostics and localization tests; then 13 affected inventory,
organization, control, full Mock journey and localization tests after fixing the
exact connection/card join and the privacy dialog intrinsic layout. Language
checks exercise actual Settings clicks in both directions, persisted preference
reopening, OS changes/manual overrides, unsupported-language fallback, 320-wide
layouts at 2x text and Chinese unknown receipts/errors without changing commands.
Seven shared presentation tests, 20 contract schemas with 45 positive and 250
negative fixtures, documentation/repository validators and commit conventions
passed. The final `fvm flutter test --reporter expanded` run passed 94 tests and failed
only the four previously recorded macOS glyph-pixel assertions: one button-label,
two control-label and one field-alignment case. Their one-pixel discrepancies,
references and strict tolerances remain unchanged. This is not full pixel-equality
acceptance. No Rust source or ABI changed in this phase.

Android packaging and actual process language persistence are recorded in the
subsequent delivery entry. The user's current emulator login attempt returned
`SECRET_STORE_UNAVAILABLE`; provider credentials and actual AC operation are not
validated by these presentation checks. No real-device writes, remote push or
artifact upload occurred in this phase.


### Android localization corrections

The first bilingual APK passed archive checks and installed after the owner
explicitly authorized removing the differently signed old app and its local data.
Actual Android Settings clicks selected Simplified Chinese. Force-stop removed
process 3093; new process 3313 restored the Chinese preference. Inspection found
two uppercase section labels missing from the catalog: HOME and PREFERENCES.
They are now included, bringing the catalogs to 1,122 messages; the five focused
localization tests pass with explicit assertions for both Chinese section labels.
All seven shared presentation tests also pass after regeneration.

An isolated XDG configuration had generated a different temporary development
key. Builds now set ANDROID_USER_HOME to the existing persistent Android directory;
the build guide records this requirement. The rebuilt APK certificate was checked
against that persistent local development key. No key is copied into this repo.
The final rebuilt artifact and updated runtime inspection follow below.


### Final bilingual Android artifact

The final release-mode Meiju APK was built from clean source
`312eeb5dab2b48325b9792ff5fdf4362a33209fe` with Flutter 3.47.7, the existing
native dual-ABI AAR and persistent local development signing. Archive verification
passed: arm64-v8a/x86_64, 17 Core exports per ABI, signature, disabled backup and
16 KiB ZIP/ELF alignment. The app remains a development artifact, not a store release.

- Local artifact: `output/flutter-verification/android/homeverb-0.2.0-meiju-i18n.apk`
- Bytes: 68061872
- SHA-256: `e48661d8e5f95770464d2771390c0b66c7c77030f9412dc90718691cd81f62a5`
- Commands: `fvm flutter build apk --release --target-platform android-arm64,android-x64 --dart-define=HOMEVERB_ADAPTER=midea.meiju`,
  `scripts/verify_flutter_apk.py --adapter midea.meiju` with the artifact/toolchain
  paths, and explicit `adb -s emulator-5586 install -r` followed by cold start.

The final APK successfully updated the persistent-key installation without wiping
its preferences. It reopened in Chinese. Actual UI inspection and screenshot review
confirmed Chinese navigation, language preference and both corrected section labels.
The app remains open on its Chinese Settings page. The earlier force-stop/new-process
check established preference persistence; the final update independently retained it.
Local `i18n-runtime-evidence.json`, the archive report and
`i18n-settings-zh-CN-final.png` accompany the APK. No artifacts were uploaded.

Real provider login remains blocked by the earlier `SECRET_STORE_UNAVAILABLE`
attempt, which was not retried during this localization-first phase. No real-device
write, physical-phone acceptance or remote push is claimed. Core API 1.22, SQLite 5,
protected payload 3 and read-only startup authority remain unchanged.


## Android credential-readiness correction

Read-only Android system inspection found `secure=false` on the control emulator,
matching the production backend's deliberate unsecured-device refusal. After the
owner configured the lock directly in Android Settings, the system reported
`secure=true`, `showing=false` and `inputRestricted=false`. No screen-lock secret
was read or stored in the project.

Trusted Application startup now retains the backend-install result. A local
Flutter method channel exposes only that result and secure/unlocked metadata,
with an explicit system-security-settings action. The sign-in page hides credential
fields until this preflight passes, rechecks before submission and after foreground
return, and invalidates late checks on background/disposal. Calls have a three-second
deadline. Fields clear before dispatch/backgrounding, temporary buffers clear after
completion, and missing/invalid/rejected replies fail closed. Core remains the
actual authority for Keystore and provider operations; this metadata check neither
opens stored credentials nor changes any grant or error contract.

Executed checks: 17 focused Flutter credential/account/localization tests passed,
including unsecured-device gating, changed protection before dispatch, background
races, field clearing and zeroed temporary buffers. Flutter analysis passed;
seven shared presentation tests, documentation/repository validators and contract
fixtures passed. The dual-ABI release APK compiled successfully with the existing
persistent development signature. The first regression run exposed an old test's
assumption that backgrounded credential fields stay mounted; the updated assertion
checks both retained controllers are empty and fields are absent until rechecking.
No Core/Rust change or real provider write occurred in this correction phase.
Actual APK installation/channel and real-login results are recorded separately.


### Protected sign-in prerequisite on the emulator

The correction APK passed signature, both complete ABIs, 17 Core exports per ABI,
backup exclusions and 16 KiB ZIP/ELF alignment. `adb install -r` succeeded with
preferences preserved. The actual Chinese Meiju sign-in page exposed two empty
credential fields and the submit action, proving that the release APK's local
Android channel reported installed backend plus secure/unlocked state. This does
not establish a Keystore round trip or successful provider login. The owner was
asked to enter credentials directly in the app; none were read or entered by tooling.

Artifact: `output/flutter-verification/android/homeverb-0.2.0-meiju-credential-ready.apk`.
Source: `67fa391bb9232f0c8bb47ce578824f8fe7e3c615`; bytes: 68062104.
SHA-256: `a45aedd66f6d3745484f92c4c03491bda52b7fab08f6fa306d460d1581f6cde5`.
The adjacent report and `credential-readiness-runtime.json` record packaging and
prerequisite evidence. Earlier bilingual artifacts remain historical builds.
No real AC write, external upload or new remote push occurred in this phase.


## Default device controls: owner-directed revision

The owner requested removing the complete session-control option and making
supported controls available without extra checkboxes. The Flutter host now
prepares every reviewed low-risk AC value for the device being opened, using
exact saved control-option inspection and existing native startup fields. Only
that device's account is refreshed to publish admitted actions. The old permission
widget and technical permission/revocation/capability-refresh UI are removed.
Startup remains local-only, and no appliance command is part of device setup.
See [ADR-0065](../../docs/adr/0065-default-app-device-controls.md).

Seven focused tests passed: actual native startup with fictional exact grants,
model constraints, automatic preparation with no startup refresh or write,
same-device reuse, failed-refresh retry, foreign/late inspection rejection,
backgrounded reopen cleanup/recovery and absence of the former UI. Flutter
analysis passed. The earlier combined account/current-contract/localization run
passed 22 tests and had one test-only header capitalization mismatch; correcting
the expectation to the actual prototype header made the full focused group pass.
The complete Flutter suite passed 101 tests and retained only the four known
macOS glyph-pixel failures (button labels, two control-label cases and field
alignment). References and tolerances were unchanged. Documentation/repository
checks and all seven shared presentation tests passed. Rebuilt Android results
are recorded separately below.

Before this revision, the owner completed real Meiju login in the App. Actual UI
showed complete device retrieval and the selected wall AC; explicit refresh
returned power off and 27.4 degrees Celsius. This proves the app's real account
and read path on this emulator, not physical effects or appliance-write success.
No command was sent before the owner requested this policy revision.


### Default-control APK and owner-observed power acceptance

Source `493b8f2199a85d7047945d2d3e61873915d15fad` built successfully with
`fvm flutter build apk --release --target-platform android-arm64,android-x64
--dart-define=HOMEVERB_ADAPTER=midea.meiju` using the persistent development key.
`scripts/verify_flutter_apk.py` passed both complete ABIs, 17 Core exports per ABI,
signature, backup exclusions and 16 KiB ZIP/ELF alignment.

- Artifact: `output/flutter-verification/android/homeverb-0.2.0-meiju.apk`
- Bytes: 67931172
- SHA-256: `ce7e0b7a65d26429142ab38a1ac8e187becb0dbf7b58e5787d2b409f18e46611`
- Archive report: `output/flutter-verification/android/homeverb-0.2.0-meiju.json`

`adb -s emulator-5586 install -r` succeeded on the Android 15 arm64 emulator.
The existing real account, retrieved cards and Chinese preference survived the
upgrade and new process. Opening the owner's selected wall AC prepared power,
mode and fan controls without checkboxes or a permission page. Device Information
showed only ordinary metadata and recent operations. This target's temperature
setter remains unavailable; opening a device cannot invent unsupported mappings.

The owner separately authorized operations on this AC. An explicit pre-write read
reported power on and 26.0 degrees Celsius; the older saved power-off observation
was not used as current evidence. Exactly two commands were sent through the App:

| Command | UTC start / recorded | Retained outcome | Independent evidence |
|---|---|---|---|
| Power off | 2026-10-16 14:33:26.343977 / 14:33:36.355521 | `unknown`, `WRITE_OUTCOME_UNKNOWN`, one attempt, no verification evidence | Explicit subsequent read reported off and 23.3 degrees Celsius; the owner confirmed the physical AC had switched off |
| Power on, restoring the pre-test state | 2026-10-16 14:36:57.478409 / 14:37:01.980193 | `accepted`, `VERIFICATION_PENDING`, one attempt, no verification evidence | Explicit subsequent read reported on and 22.9 degrees Celsius; physical restoration was not independently observed by tooling |

No uncertain command was replayed. Later reads did not rewrite either receipt.
This establishes real App power actuation with an owner-observed shutdown for
this selected appliance, not protocol-level `verified`, broad model certification,
physical-phone acceptance, temperature/fan/mode actuation or off-LAN acceptance.
Cloud observations lack trustworthy source time. No native API, schemas, routing
or execution semantics changed. No credentials or live device identifiers were
captured, and no artifact was uploaded or newer commit pushed.


## In-place climate controls and prominent selectors

The owner requested keeping AC operations on the control page and showing separate
Mode, Air direction and Fan speed buttons. The climate-and-power view now uses
these buttons with observed values and bottom-sheet choices from exact descriptors.
Unsupported direction control stays explicitly unavailable. Mode activation states
that it also powers on the device. Other descriptor controls remain accessible.
No new protocol mapping, native grant, Core outcome or schema is introduced.

Power and supported climate actions retain the current page and display their
outcome inline, with optional View Details. Quick card power stays on the home
page with a dismissible status message. One bounded foreground read updates observed
state after a command; it never rewrites the retained outcome or repeats the write.
Selection/background/navigation and device/component boundaries invalidate late work.
Temperature-only components retain their previous specialized behavior.

`fvm flutter analyze` passed. The complete `fvm flutter test --reporter expanded`
run passed 107 tests and retained the same four known macOS glyph-pixel failures
(button labels, two control-label cases, field alignment), without changing reference
images or tolerances. Six new focused tests cover retained unknown outcomes, one
read/no replay, explicit detail navigation, exact mode/fan values, unsupported
direction, backgrounded selectors, navigation and background/resume during a pending
write, failed post-write reads, Chinese large text and denied/coupled controls.
The native click journeys now exercise stay-on-page outcomes and explicit details.

The first selector tests exposed Flutter ChoiceChip dry-layout incompatibility
with paragraph Copy; native localized Text labels fixed it. The existing native
journey exposed a status message covering a later page; navigation now clears it.
The final complete suite includes both corrections. Shared presentation tests passed
7/7; document/repository checks and contracts (20 schemas, 45 positive and 250
negative fixtures) passed. Android packaging and installed UI results follow below.

Actual Android visual inspection then exposed a misleading generic icon and an
unmapped observed fan code. The final view reuses the existing Window Light fan
glyph, uses direction arrows, and labels unrecognized values explicitly while
retaining their raw value. It does not invent an automatic-speed meaning or add
that observation to the allowed choices. The final focused climate/localization
selection passed 11 tests and analysis passed after this presentation correction.

### Climate interaction APK delivery

Final source: `612a296854c2faa5042acbd188c741df19e13365`.
The dual-ABI Meiju release build passed with the existing persistent development
signature. The APK verifier passed complete arm64-v8a/x86_64 contents, 17 Core
exports per ABI, signature, backup exclusions and 16 KiB ZIP/ELF alignment.

- Artifact: `output/flutter-verification/android/homeverb-0.2.0-meiju-climate.apk`
- Bytes: 68324780
- SHA-256: `5b59edbc02151dad19d3b42ea744aa399133ed9cca3bf286e75a9e609e6d2212`
- Report: `output/flutter-verification/android/homeverb-0.2.0-meiju-climate.json`
- Screenshot: `output/flutter-verification/android/climate-controls-zh-CN.png`

`adb -s emulator-5586 install -r` succeeded and the app cold-started with its
existing account/cards and Chinese preference. Actual UI inspection confirmed
three adjacent buttons, enabled mode/fan selectors and explicitly unavailable
direction. Selector runtime checks used `c13791d`; the final `612a296` upgrade
repeated saved-account opening and screenshot review after the icon/value fix.
The mode sheet showed five translated modes and the power-on notice;
the fan sheet showed six reviewed translated choices. Apply stayed disabled until
selection, and cancellation returned to the device page. Final screenshot review
confirmed the corrected fan glyph and explicit unrecognized reading without
clipping. The app remains on the selected wall-AC control page.

This revision's real-account checks opened/cancelled selectors only: no new
household-appliance command was sent. Stay-on-page execution and outcome recovery
were exercised by widget and actual native Mock tests; the earlier owner-observed
physical power evidence retains its original scope. No phone acceptance, new
physical mode/fan/direction support, remote push or artifact publication is claimed.

## Meiju-style climate composition phase

The owner requested a fuller Meiju-style panel after reviewing the previous
inline controls. The live AC page now has a light background, original vector
appliance illustration, separate indoor reading, native temperature stepper and
slider, and a two-column power/mode/fan/direction grid. Additional independent
low-risk setters receive dedicated tiles when present in actual descriptors.
The integration-status sheet distinguishes missing HomeVerb work from missing
hardware. Full parity remains unfinished; see the
[comparison and evidence gate](../../docs/development/10-meiju-ac-parity.md).

Validation on October 16, 2026:

- `fvm flutter analyze`: passed.
- Focused `climate_controls_test.dart`, `localization_test.dart`, `app_test.dart`
  and `outcome_click_test.dart`: all 17 passed, including native Mock execution.
- Complete `fvm flutter test --reporter expanded`: 109 passed, four existing
  macOS glyph-pixel failures in button labels, two control-label checks and field
  alignment. The references/tolerances are unchanged. This is not a fully passing
  visual suite or pixel parity claim.
- Shared presentation tests: all seven passed.
- Documentation and repository validators passed. `.venv/bin/python
  scripts/validate_contracts.py`: 20 schemas, 45 positive and 250 negative
  fixtures passed. The system Python lacks the contract validator dependency;
  the existing virtual environment supplied it.
- Drag-only changes do not send; release sends one target, validates exact bounds
  and step, and retains observed state until a read. Background/resume invalidates
  a late release. Coupled required fields hide the simple slider. Additional
  fictional sleep control stays in place and cannot bypass its descriptor.

This phase changes no Rust code, native ABI, startup grants, storage or admitted
Meiju control. No new physical AC write is part of these checks.

The first emulator review prompted a small presentation correction: keep the
Celsius unit beside the target, select the mode icon from the observed semantic
mode, and describe the unconnected direction control as an integration gap.
Static analysis and the final climate/localization subset passed (13 tests).
The full suite above was not repeated for these label/icon/layout-only changes.

## Optional-subtype temperature correction

The owner's wall AC had protected product and explicit wall-category evidence,
but no nonzero subtype. The earlier mandatory-subtype check omitted its
`climate.set` option. Temporary emulator logging emitted only fixed readiness
booleans; the diagnostic was removed before this source phase. No raw product
values, device identifiers or credentials were exported.

ADR-0066 corrects this for independently protected exact reviewed wall models.
Option inspection, descriptor admission, execution and read decoding use the
same selection. Missing model/product/category, ambiguous placeholders and excluded
profiles remain denied. Known subtype priority, exact grants, Celsius bounds,
steps, encoding and uncertain-write rules are unchanged.

Checks actually run:

- Temperature-focused Rust tests passed, including additional missing-subtype
  cases across all four exact models, encoding and negative evidence fixtures.
- `cargo +1.96.0 clippy --workspace --all-targets --offline --locked -- -D warnings`
  passed on the documented baseline. The workstation's stable Rust 1.99 check
  encounters four pre-existing `chunks_exact_to_as_chunks` warnings in unchanged
  codec/LAN files; these were not suppressed or included in this fix.
- The unfiltered workspace suite encountered 14 LAN tests requiring an unconfigured
  `127.0.0.2` address on macOS (`AddrNotAvailable`). A second workspace run excluded
  exactly those 14 platform-blocked tests: 467 passed, one existing test ignored,
  no failures. This is not an unfiltered workspace pass.
- Both Android native ABIs built with 17 exports each and 16 KiB alignment.
- Contract validation passed: 20 schemas, 45 positive and 250 negative fixtures.
  Documentation/repository validation and formatting passed.

This source phase changes no public schemas, native API 1.22.0, SQLite schema 5
or protected payload 3. APK/runtime temperature evidence is recorded separately.

## Temperature APK delivery and selected-AC read-back

Delivery completed on October 17, 2026 (Asia/Shanghai), from clean source
`6c232a3c2f89b53e0ef3dd3bfa7473d3b7f5be68`. The local artifact is
`output/flutter-verification/android/homeverb-0.2.0-meiju-temperature.apk`, with
its archive report at the same path ending in `.json`. This supersedes the
earlier `homeverb-0.2.0-meiju-panel.apk`.

- Size: 68,333,888 bytes.
- SHA-256: `eb8b0cff3bcb1e7b631063b1cd5d136ff02c63a46a8819a98f274f70fc210c20`.
- Adapter: `midea.meiju`; arm64-v8a and x86_64, 17 Core exports per ABI.
- APK signature and 16 KiB ZIP/Core ELF alignment passed archive verification.
  Signing uses the existing local development key, not a distribution credential.
- Release Flutter build and both native ABI builds completed. `adb install -r`
  succeeded on the Android 15 arm64 emulator; the saved login and Chinese
  preference remained available. The final binary contains no temporary
  temperature-admission diagnostic logging.

The owner had authorized controls on the selected wall AC. On this final APK,
the initial cloud observation was power on, cooling, automatic fan and target
28 Celsius. Each action below was one explicit stepper tap and one recorded
attempt, followed by a separate bounded state read:

| Action | Start (UTC, October 16, 2026) | Receipt recorded (UTC) | Receipt | Subsequent target observation |
|---|---|---|---|---|
| Decrease once | 15:58:33.816603 | 15:58:38.064950 | `accepted`, `VERIFICATION_PENDING` | 27.5 Celsius |
| Restore once | 16:00:44.192753 | 16:00:45.200310 | `accepted`, `VERIFICATION_PENDING` | 28 Celsius |

Both operations remained on the AC page. Receipt details were opened manually
only for inspection, then closed. Neither operation was replayed. Power, cooling
mode and automatic fan remained observed after the adjustments. These are cloud
command/read-back results, not protocol verification or proof of physical
temperature behavior; no verification evidence was added to either receipt.

Local captures `output/flutter-verification/android/climate-temperature-readback-zh-CN.png`
and `output/flutter-verification/android/climate-temperature-restored-zh-CN.png`
show the observed targets, separate controls and inline accepted outcome. The
emulator is left on the restored 28 Celsius AC page. No physical-phone, full
Meiju feature parity, remote push or artifact publication is claimed. The preceding
source-phase test limitations remain applicable; this delivery record does not
turn the filtered Rust run or Flutter visual suite into an unfiltered pass.
