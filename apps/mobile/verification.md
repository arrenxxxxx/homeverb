# Flutter implementation verification

Date: October 15, 2026. Baseline: native UI contract 1.0.0 and the approved Window Light atlas. This is an implementation checkpoint, not completion of the owner's pixel-identical acceptance requirement.

## Implemented scope

The application pins Flutter 3.47.7 through FVM 4.3.1. It loads the native library in process, negotiates API availability, runs blocking work in isolates, gates background calls, cancels outstanding operations and preserves uncertain outcomes. No production HTTP bridge, CLI parsing, periodic refresh or account service was added.

The native Mock path provides first launch, three roots, local search, room filters, room create/rename/reorder/delete, atomic assignment, alias/pin/order, explicit discovery and reads, component-described climate controls, explicit selected-device power batches, exact receipts/history, local route/name/skin preferences and allowlisted diagnostics preview/export. Loading and failed local reads do not display fictional preview cards. Modal backgrounds use the selected native device state.

All 82 atlas states have native-widget implementations in an isolated, visibly labelled preview. The 198 event buttons in that preview simulate atlas branches. They are **not 198 successful native operations**. Real accounts, authorization, sharing, identity merge and cleanup remain unavailable under the owner's approved current-contract scope. Unsupported non-climate appliance examples remain preview data, not physical-device support.

## Executed verification

Commands below ran in the isolated worktree; Flutter commands used the pinned FVM SDK. Temporary profiles contained only fictional data.

| Check | Evidence / result |
|---|---|
| `fvm flutter analyze` | No issues found |
| `fvm flutter test --reporter expanded` | 21 tests: 9 native/controller tests, one extended native widget click scenario, three native outcome click scenarios, two isolated component fixture checks, six screen/interaction/reflow checks |
| `fvm flutter test integration_test/native_smoke_test.dart -d linux` under Xvfb | Actual Linux process loads the bundled Core library without a path override, discovers Mock, reads state, sets mode and verifies the returned receipt; process screenshot captured |
| `fvm flutter build linux --release` | Linux bundle includes `libhomeverb_ffi.so`; the bundled Rust library is the checked debug build |
| `fvm flutter build web -t lib/main_preview.dart` | Isolated preview compiles; web preview does not call native Core |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --offline --locked -- -D warnings` | Passed |
| `cargo test --workspace --offline --locked` | Passed |
| `cargo build --workspace --offline --locked` | Passed |
| `python3 scripts/verify_native_contract.py` | 22 native methods and 42 live examples passed |
| `python3 scripts/verify_mvp.py` | 24 runtime checks passed |
| `python3 scripts/validate_contracts.py` | 15 schemas, 24 positive and 86 negative fixtures passed |
| `node packages/presentation/test/i18n.test.mjs` | 5 shared-catalog tests passed; this does not establish Flutter localization |
| `python3 -m unittest discover -s tests -v` | 23 repository tests passed |

The native click scenario exercises discovery, read, mode and target writes, quick power before/after reading, card long press, invalid target rejection, blank selection rejection, batch execution, blank/overlong/control-character/duplicate room rejection with retained input, room rename/drag/menu reorder/delete, exact-room assignment, All/custom/Unassigned filters, alias/pin/unpin, search/clear/exit, unchanged route persistence, sharing-name persistence, receipt refresh with no current device selection, history, diagnostics export cancel/save/failure, unavailable gates and foreground lifecycle. Room-menu assignment preselects its exact room and rejects empty device selection; device-menu assignment visibly selects the current device. Corrected input clears outdated validation messages before the next submission. The file picker is replaced by a controlled platform implementation; saved file bytes must equal the exact preview. The actual Linux integration separately exercises the real Flutter process, not the system save chooser.

The native widget click test now loads the bundled Inter/Material fonts and fails missed pointer targets. Its latest successful run saves 79 viewport captures across 27 screen IDs, including errors and intermediate states. Generate `output/flutter-verification/native-gallery.html` with `python3 apps/mobile/tool/native_gallery.py`. Entries identify clicks, drags and direct test navigation; all records are fictional native Mock data. Phone insets are simulated and OS status icons are absent. This is separate evidence from the 82-state isolated atlas and from the actual Linux-process screenshot. The native room page now uses the prototype's grouped rows and exposes assignment through its row menu; its drag handle and menu ordering are both exercised.

Three additional click scenarios use the native `lost-ack`, `accept-only`, and `reject` fixtures. They click discovery, read, power, receipt refresh and history; assert the displayed `unknown`, `accepted`, or `failed` outcome; retain observations until an explicit read; and verify the original request ID and unchanged history count after refresh. These are fictional native operations, not physical-device evidence.

Two isolated contract-fixture tests load the repository's refrigerator and compartment capability examples. They verify independent cached observations, exact component references, absence of unrelated shared capabilities, blocked selection changes while busy, distinct cold-storage ranges and steps, plan-before-execute, and disabled read-only temperature controls. The fixture exists only in tests; it is not a registered native adapter. The real native Mock host still exposes one climate appliance. Multi-selection preview rows also retain every selected item and support independent selection/deselection.

Controller/FFI tests additionally cover persisted reopen, accepted/verified/failed/unknown results, nested partial batch decoding, no write replay on read, numeric bounds/steps/component scopes, preference corruption and rapid background/resume races. The Mock host exposes only one appliance. Multi-device mixed outcomes use native fixtures; no physical multi-device batch was performed.

## Visual gate: failed

The screenshot test captures every ID at the reference size: 780 x 1688, or 2048 x 1536 for W01/W02. It loads the licensed Inter fonts and approved product/background assets. All text, controls and semantics are widgets; complete screen images are never used as the app interface. Test status/navigation chrome is deterministic artwork, not a captured phone OS.

The latest complete comparison has **0 of 82 exact matches**, with mean absolute channel error **1.9495 / 255**, down from 6.1482 before rendering refinements. Native switches now use the prototype's accent/gray tracks and light thumbs. A lower average alone does not establish correct states or layouts. These metrics measure differences, not an acceptance tolerance. Text rasterization, icons, surfaces, controls and layout still differ. P08 intentionally follows the amended automatic-retrieval contract; the approved atlas still depicts the superseded manual step.

This refinement aligns the surface gradient, text spacing, root navigation, form heights, sheet corners, sliders, climate panel, large cards and skin miniature. The bottom sheet no longer applies the keyboard inset a second time after Scaffold resizing. All 82 states also render at 844 x 390 logical pixels with 150% text; all phone states render at 320 logical width with 200% text. Every atlas form accepts edited text while its input remains above a simulated 280-logical-pixel keyboard. These widget checks do not establish physical keyboard or screen-reader behavior.

Generate the full reference / actual / amplified-difference report with:

```sh
python3 apps/mobile/tool/compare_screens.py --require-exact
```

The command verifies the complete 82-file set, writes `output/flutter-verification/index.html` and `metrics.json`, and exits 1 while differences remain. The reference is never overwritten and generated artifacts are ignored by Git. Passing screenshot rendering is not passing visual equality.

## Acceptance status

| Requirement | Current status and remaining evidence |
|---|---|
| UI-01 | 82 states and 198 simulated branches covered; native available routes clicked. System-owned authorization/permission transitions unavailable. |
| UI-02 | Native first launch, three roots, search/clear, separate card/quick-control clicks, long press and room filters tested with actual fonts and strict pointer hits. Full keyboard/screen-reader audit pending. |
| UI-03 | Native room/card CRUD, room drag/menu sorting, exact-room assignment and preservation tested. Empty device selection and blank, overlong, control-character and duplicate room names are click-tested with retained input. Stale permutations and multi-device conflict cases still require broader Flutter coverage. |
| UI-04 | Exact-component selection, capability references, cache isolation, climate bounds/enums and unknown/read-only values implemented. Isolated fridge/freezer fixture clicks verify distinct ranges and steps; non-climate control forms and physical multi-component acceptance remain pending. |
| UI-05 | Native outcomes preserved, including nested partial results. Explicit batches plan before execution; no aggregate rollback/retry. Physical evidence unavailable. |
| UI-06 | Explicit bounded Mock discovery implemented. Real permission, identity-merge and nearby-pairing paths unavailable. |
| UI-07 | Unavailable under native contract 1.0; isolated preview only. |
| UI-08 | Unavailable under native contract 1.0; no raw storage deletion or fabricated revocation. |
| UI-09 | Unavailable under native contract 1.0; no peer networking or credential transfer. |
| UI-10 | Dart/native foreground, cancellation and rapid-resume checks passed. Physical Android/iOS background and termination acceptance pending. |
| UI-11 | One registered skin, local persistence/fallback and preview implemented and tested. |
| UI-12 | Exact diagnostics preview/export and retained receipt navigation tested. Real OS save chooser acceptance and safe cleanup methods pending. |
| UI-13 | Reference phone/wide captures, 320-width/200% text, all-state landscape/150% text and all-form simulated keyboard checks passed. Full zh-CN wiring, high-contrast/reduced-motion/screen-reader and physical-keyboard acceptance pending. |
| UI-14 | Linux process and bundle verified. Android/iOS builds, phone profiling and pixel equality not passed. |
| UI-15 | Real Meiju integration unavailable. P08 preview copy follows automatic retrieval; replacement authoritative artwork and real off-LAN acceptance pending. |

## Compatibility and environment

Native API requests, schemas, C symbols and SQLite schema are unchanged. Presentation preferences are app-private JSON containing no credentials. No real-platform compatibility row is promoted. No real-device write, diagnostic upload, Git push or publication was performed.

Android/iOS project scaffolding is present, but the full Android SDK/NDK and Xcode/Apple environment were unavailable. ADB successfully listed devices and found none attached. No device choice is required to continue shared Flutter UI work; Android/iOS native packaging and actual OS lifecycle acceptance remain separate outstanding checks. Linux GTK/Clang development packages were extracted into a temporary local sysroot for this run; no system package installation was performed. Standard installed Linux dependencies are sufficient for normal development. FVM owns the SDK; no project-local SDK copy is committed.
