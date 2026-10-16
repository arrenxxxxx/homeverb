# Flutter implementation verification

Date: October 16, 2026. Baseline: native UI contract 1.14.1 (pinned Core `7db8e9b`) and the approved Window Light atlas. This is an implementation checkpoint, not completion of the owner's pixel-identical acceptance requirement.

## Implemented scope

The application pins Flutter 3.47.7 through FVM 4.3.1. It loads the native library in process, negotiates API availability, runs blocking work in isolates, gates background calls, cancels outstanding operations and preserves uncertain outcomes. No production HTTP bridge, CLI parsing, periodic refresh or account service was added.

The native Mock path provides first launch, three roots, local search, room filters, room create/rename/reorder/delete, atomic assignment, alias/pin/order, explicit discovery and reads, component-described climate controls, explicit selected-device power batches, exact receipts/history, local route/name/skin preferences and allowlisted diagnostics preview/export. Loading and failed local reads do not display fictional preview cards. Modal backgrounds use the selected native device state.

All 82 atlas states have native-widget implementations in an isolated, visibly labelled preview. The 198 event buttons in that preview simulate atlas branches. They are **not 198 successful native operations**. Flutter account metadata, protected input, automatic-retrieval results, pending-session recovery/reauthorization and scoped removal are implemented. Their successful provider flows are tested with isolated contract fixtures, not real credentials. Nearby sharing, identity merge and full-profile deletion remain unavailable. Native cache/history cleanup is implemented with separate scope confirmations. Non-climate native forms consume described scalar properties/actions. A selected component with one light, fan, media, sensor or cycle family now uses a dedicated Window Light layout; mixed families retain the descriptor list. These isolated fixtures are not registered native adapters or physical-device support. The multi-compartment overview now groups exact component readings and target controls. Strict layout equality remains pending.

## Executed verification

Commands below ran in the isolated worktree; Flutter commands used the pinned FVM SDK. Temporary profiles contained only fictional data.

| Check | Evidence / result |
|---|---|
| `fvm flutter analyze` | No issues found |
| `fvm flutter test --reporter expanded` | Full 72-test suite passed, including four trusted confirmation scenarios and two saved-inventory checks, two native account and five synthetic account checks, three cleanup regressions and all 11 prior native/controller checks and the inline-stepper interaction check. Other coverage: one extended native click scenario, three native outcome scenarios, two component checks, four compartment-overview checks, five accessibility checks, three system-contrast checks, two receipt checks, two descriptor-control checks, six dedicated appliance-layout checks, three organization checks, nine screen/interaction/reflow checks, two Android export-channel checks and three diagnostics-card checks |
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

The latest complete comparison has **0 of 82 exact matches**, with mean absolute channel error **1.6529 / 255**, down from 6.1482 before rendering refinements. Native switches now use the prototype's accent/gray tracks and light thumbs. A lower average alone does not establish correct states or layouts. These metrics measure differences, not an acceptance tolerance. Text rasterization, icons, surfaces, controls and layout still differ. P08 intentionally follows the amended automatic-retrieval contract; the approved atlas still depicts the superseded manual step.

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
| UI-01 | 82 states and 198 simulated branches covered; native available routes clicked. System-owned authorization/permission transitions unavailable. |
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
| UI-15 | Meiju Core and explicit read-only Flutter startup/account integration are present; real account and off-LAN acceptance remain pending. P08 preview copy follows automatic retrieval; replacement authoritative artwork and real off-LAN acceptance pending. |

## Compatibility and environment

The frontend merged Core API 1.9 and SQLite schema 4 from pinned commit `00d44b7`, including the protected seventh C symbol. This cleanup UI uses the existing native methods without further schema changes. Presentation preferences are app-private JSON containing no credentials. No real-platform compatibility row is promoted. No real-device write, diagnostic upload or package publication was performed. The owner-authorized frontend implementation has been pushed to `origin/codex/flutter-ui`; the branch is retained. This is an incomplete acceptance checkpoint, not a merge into main or a claim that the pixel gate passed.

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
