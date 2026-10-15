# HomeVerb Flutter application

The native application embeds Core using native UI contract 1.0. The separate visual preview contains fictional Window Light fixtures and never opens Core. Account authentication, nearby authorization sharing and destructive cleanup remain unavailable in the native contract. Their preview states are not live features.

## Development

Use FVM 4.3.1 and Flutter 3.47.7, pinned in `.fvmrc`.

```sh
fvm install
fvm flutter pub get
fvm flutter analyze
fvm flutter test test/native_test.dart
fvm flutter test test/screens_test.dart
```

Build the library from the repository root before running native tests:

```sh
cargo build -p homeverb-ffi --offline --locked
```

Run a Linux native application with an absolute library path:

```sh
fvm flutter run -d linux --dart-define=HOMEVERB_LIBRARY=/absolute/path/to/libhomeverb_ffi.so
```

The explicit Mock host permits fictional writes only. Native metadata and presentation preferences live in the OS application support directory. Opening a page does not discover or refresh devices. Use Add Device to discover the Mock appliance and Refresh State to read it.

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
- Diagnostics export only the exact report displayed in the preview.

## Verification boundary

Seven Dart integration tests passed against the locally built Linux library: organization, persistence/reopen, validation, observations, receipts, foreground gating, outcome preservation and no write replay during refresh. This establishes fictional Linux integration, not a verified physical appliance.

Android/iOS scaffolding alone does not establish native library packaging, secure-store integration, physical-phone lifecycle, accessibility or performance. No real provider is enabled. Project release licensing remains undecided. Bundled Inter fonts use the adjacent SIL Open Font License.
