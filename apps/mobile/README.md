# HomeVerb Flutter application

The native application embeds Core using native UI contract 1.0. The separate visual preview contains fictional Window Light fixtures and never opens Core. Account authentication, nearby authorization sharing and destructive cleanup remain unavailable in the native contract. Their preview states are not live features.

## Development

Use FVM 4.3.1 and Flutter 3.47.7, pinned in `.fvmrc`.

```sh
fvm install
fvm flutter pub get
fvm flutter analyze
fvm flutter test --reporter expanded
```

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

The explicit Mock host permits fictional writes only. Native metadata and presentation preferences live in the OS application support directory. Opening a page does not discover or refresh devices. Use Add Device to discover the Mock appliance and Refresh State to read it. Device menu > Select Devices offers explicit batch power actions; results remain per item. Unsupported or ambiguous capabilities fail admission before execution.

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
- Diagnostics export only the exact report displayed in the preview.

## Verification boundary

See [verification.md](verification.md) for executed checks and remaining acceptance gates. All 82 native-widget preview states render and all 198 simulated atlas event branches are clicked separately from the real Core tests. The actual Linux Flutter process also discovers, reads and writes the fictional Mock appliance through FFI. Screenshot equality has **not passed**; do not describe the application as a pixel-identical or phone-verified delivery.

Generate the local comparison report from the repository root after screenshot tests:

```sh
python3 apps/mobile/tool/compare_screens.py --require-exact
```

The command generates `output/flutter-verification/index.html`, reference/actual/difference images and JSON metrics. `--require-exact` exits nonzero while any pixel differs. The generated image collection is ignored by Git; the reference remains unchanged.

Android/iOS scaffolding alone does not establish native library packaging, secure-store integration, physical-phone lifecycle, accessibility or performance. No real provider is enabled. Project release licensing remains undecided. Bundled Inter fonts use the adjacent SIL Open Font License.
