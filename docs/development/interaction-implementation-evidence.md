# Interaction implementation evidence

This record tracks the November 9 interaction handoff. The design and its exit
criteria are in [the handoff](15-interaction-experience-handoff.md); historical
results remain in the existing verification records. This is fictional-fixture
evidence, not account, appliance or physical-phone acceptance.

## M0 baseline and coverage map

The baseline was `b0923807530a7cf1fb5d72f39fa5a3b6817025dd`, with a clean
worktree and `main` two commits ahead of `origin/main`. The pinned Flutter is
3.47.7; the local macOS debug `homeverb-ffi` library exists. The baseline
`state_refresh_test.dart` passed eight tests: three-second cadence, filtered
components, silent errors, slow-cycle exclusion, lifecycle fencing and user-work
drain. This does not establish the new opening-read requirements.

| Surface and branches | Requirements | Existing or planned evidence |
| --- | --- | --- |
| Device home, search, rooms; normal, compact and large text in both skins | UX-01, 02, 05, 06, 08, 12 | `device_cards_test.dart`, `compact_device_cards_test.dart`, `state_refresh_test.dart`, `organization_test.dart` |
| AC, refrigerator, dishwasher idle/running, unsupported and generic devices | UX-01–09 | `climate_controls_test.dart`, `compartment_overview_test.dart`, `refrigerator_modes_test.dart`, `dishwasher_interaction_test.dart`, `device_detail_controls_test.dart`, `controls_test.dart` |
| Approval, execution, partial and uncertain receipts | UX-09 | `approval_test.dart`, `outcome_click_test.dart`, `receipts_test.dart` |
| Accounts, sign-in and identity recovery | UX-01, 10, 12 | `account_surfaces_test.dart`, `auth_experience_test.dart`, `account_identity_test.dart`, `identity_recovery_test.dart` |
| Toasts and refresh feedback | UX-04–06, 11 | `feedback_test.dart`, `refresh_feedback_test.dart` |
| Settings, cleanup, history and diagnostics | UX-12 | `app_test.dart`, `cleanup_test.dart`, `diagnostics_view_test.dart`, `diagnostics_export_test.dart`, `receipts_test.dart` |
| Both locales, both skins, enlarged text, safe areas and contrast | UX-03, 08, 10–12 | `localization_test.dart`, `skin_selection_test.dart`, `accessibility_test.dart`, `safe_area_test.dart`, `contrast_test.dart` |

The production `NativeApp` baseline capture suite passed five tests using
fictional AC, refrigerator and idle/running dishwasher clients. Its Window Light
captures are under `apps/mobile/build/verification/device-design/window_light/`;
the command used the local Hiragino Sans GB font as a CJK fallback and the
bundled Inter assets. These are pre-implementation captures, not proof of the
approved appearance or final platform font rendering. The reported unknown
temperature on a physical phone was not reproduced by this capture.

An opt-in `interaction_baseline_capture_test.dart` also passed against the
pre-navigation-change controller: with a cached read held for five seconds,
the production App remained on home with operation progress; a terminal
fictional refrigerator failure showed per-zone errors and `Unknown`. Its
captures are in `apps/mobile/build/verification/interaction-baseline/`. Card
icons in those two captures include concurrent, uncommitted M3 work, so they
are evidence only for the baseline navigation and failure behavior. The test
records historical behavior and is not a final acceptance test.

The deterministic `interaction_fixture_test.dart` exercises five-second held
reads, independent component failure, bounded timeout, method counts, and
fictional per-profile/device cache restoration across closed sessions. It makes
no network, native or physical-device calls. Further milestone tests must
exercise these states through the production controller and widgets.

Commands run at baseline: `fvm flutter --version`, `fvm flutter test
--reporter expanded test/state_refresh_test.dart`, the opt-in baseline capture
and fixture tests, and the opt-in device design capture with the font arguments
above. `python3 scripts/validate_docs.py`, `python3 scripts/validate_repository.py`,
`python3 scripts/validate_commits.py --all`, contract validation in an isolated
temporary Python environment, and `node packages/presentation/test/i18n.test.mjs`
passed. No emulator or Simulator was booted when checked. Performance remains
unmeasured; the existing screenshots do not satisfy M6.

## M1 navigation and local restoration

Device and account destinations now render before local descriptor or account
detail work completes. Device descriptions and Core component snapshots are
restored locally; current control authority is inspected separately. Opening
reads run before any required control-session replacement, and native work is
cancelled and drained before reusing or replacing a session. Local root
preference writes are serialized without occupying the native operation lock.
Device, session and lifecycle checks fence asynchronous results.

`fvm flutter analyze` passed. The combined focused run of
`device_opening_test.dart`, `current_contract_test.dart`,
`session_permissions_test.dart`, `device_detail_controls_test.dart`,
`state_refresh_test.dart`, `component_test.dart` and
`account_surfaces_test.dart` passed 40 tests. These include held local metadata,
account details and preferences, reading before held control preparation,
session replacement without overlap, fictional restart/cache isolation, rapid
device switching, backgrounding and removal. No real account or appliance was
used. Reading-state feedback, partial manual refresh and final visual and
performance acceptance remain work for M2 and M6.

## M2 reading and refresh states

Opening reads distinguish unresolved components from completed snapshots. An
initial failure without a saved snapshot produces one localized notice; cached
values survive later failures. Manual refresh continues across component errors
and publishes each completed result. Home and search use one pull-to-refresh
operation and an accessible action, while ordinary device refresh buttons and
reading-time captions are removed. The initial read is explicit user work;
the next automatic three-second interval starts after that work drains.

The integrated focused Flutter selection passed 82 tests, with two opt-in
CJK-font capture tests skipped. After extending the generic climate fallback,
the generic controls and reading-state selection passed 11 tests. Tests cover
five-second holds, initial and partial failures, cached reopening, progressive
publication, repeated pulls, the 2,999/3,000 ms boundary, ten silent failures,
background/resume, localization and uncertain-receipt retention. Flutter
analysis, catalog checks and documentation/repository/contract validators
passed. These are fictional tests; final screenshots and performance remain M6.

## M3 card icons and summaries

Home, search and room cards render locally drawn outlined appliance symbols.
Card summaries use the available component observations: AC target and mode,
refrigerator zone targets or current temperatures with their meanings retained,
and dishwasher program, phase and remaining time. Missing observations remain
missing, and the separate quick-power path still checks existing grants.

`device_summary_test.dart`, `device_cards_test.dart` and
`compact_device_cards_test.dart` passed 22 tests; four opt-in font-dependent
captures were skipped. The actual `NativeApp` home capture ran for both skins
at 390 logical pixels and 100%/200% text using fictional AC, refrigerator and
dishwasher clients. Files are under
`apps/mobile/build/verification/interaction/<skin>/en-US/390px-<scale>pct/`.
The capture uses production refresh to obtain fictional observations; it does
not establish physical-device or M6 cross-width acceptance. The screenshot
fixture also contains other scenes that have not yet been accepted in M3.

## M4 expanded device controls and outcomes

Eligible AC extras, refrigerator information and component actions, dishwasher
choices and readings, and generic multi-component information now appear inline.
Running dishwasher choices remain visible but disabled. Component actions still
enter the existing reviewed form, with current descriptor, observation, grant
and lifecycle checks. Temperature adjustments retain observations during local
progress and preserve accepted or uncertain receipts if a readback fails.

The integrated Flutter selection passed 72 tests, including device-family and
mixed-component layouts, observation and permission gates, navigation races,
duplicate input, retained outcomes, native fictional receipt recovery and trusted
confirmation. `fvm flutter analyze` passed. These results do not establish a new
physical device capability or authorize a real appliance write.

## M5 secondary journeys and feedback

Account sign-in Cancel and floating feedback are centered; Toast surfaces size
to their contents with bounded wrapping. Search and room empty states, move
selection, room actions, settings destinations, and history/diagnostics loading
and retry now have distinct paths. Returning from a newly created room preserves
the in-progress move. Device route options are limited to saved connections.

The focused Flutter selection passed 96 tests; two opt-in font-dependent captures
were skipped. It includes 320/390/430 logical-pixel Toast and Cancel geometry,
both locales, large text, screen-reader feedback, exact diagnostic export,
local cleanup and fictional native request-retirement checks. Documentation,
repository and contract validators, catalog tests and Flutter analysis passed.
The current retained receipt contract has no original action field, so history
uses the truthful generic label `Operation` rather than inferring an action from
returned data; raw evidence remains available. Physical and M6 acceptance remain
separate.

## M6 integration and verification

Integration review extended the loading, failure and retained-outcome treatment to
the specialized appliance pages and generic action forms. Lifecycle and navigation
fences now cover pending reads, account and receipt lookup, plans, batches and
trusted approvals. A cancelled plan cannot send after navigation or backgrounding;
an interrupted approval must be resolved against its original receipt before a
new preview is prepared. The tests use fictional clients and never send a real
device command. Core contracts, storage schemas and provider authority are
unchanged. M0 through M5 are recorded in local commits `6613530`, `db96c4a`,
`03b749d`, `8ae87bf`, `adee987` and `25e881c`, respectively.

Additional fictional tests explicitly exercise changed descriptors and cache
clearing (`descriptor_cache_invalidation_test.dart`), revoked native session
grants with saved observations (`grant_revoke_cache_test.dart`), and recovery of
an unknown receipt after recreating the Flutter App and native session over the
same isolated profile (`unknown_restart_recovery_test.dart`). The recovery test
asserts no new plan or execute call; it does not claim an OS process restart.
Those three suites and `outcome_click_test.dart` passed seven tests together.

The final scenario audit maps the handoff to the following tests. All paths are
under `apps/mobile/test/`; the tests use fictional data unless stated otherwise.

| Acceptance area | Direct evidence |
| --- | --- |
| Immediate device/account entry and isolated disk restoration | `device_opening_test.dart` |
| First reading, failure Toast, partial results and ten silent failures | `interaction_reading_test.dart`, `appliance_reading_states_test.dart` |
| Three-second cadence, slow reads, cancellation and foreground transitions | `state_refresh_test.dart`, `interaction_fencing_test.dart` |
| Pull feedback, repeated pulls and accessibility action | `home_pull_refresh_test.dart` |
| Expanded options, disabled cycle choices and observation prerequisites | `device_expanded_test.dart`, `dishwasher_interaction_test.dart`, `dishwasher_permissions_test.dart` |
| Outlined cards, separate hit targets and observed summaries | `compact_device_cards_test.dart`, `device_cards_test.dart`, `device_summary_test.dart` |
| Local command progress, accepted/unknown outcomes and no replay | `generic_command_feedback_test.dart`, `write_fencing_test.dart`, `unknown_restart_recovery_test.dart` |
| Current grants, descriptor change and cache clearing | `grant_revoke_cache_test.dart`, `descriptor_cache_invalidation_test.dart` |
| Centered sign-in Cancel and bounded Toast | `auth_experience_test.dart`, `feedback_test.dart` |
| Search, rooms, settings, account, history and diagnostic recovery | `secondary_journeys_test.dart`, `account_surfaces_test.dart` |
| Production-route width, language, skin and text-size captures | `interaction_capture_test.dart` and the opt-in matrix below |

`fvm flutter analyze` passed. The latest macOS full
`fvm flutter test --no-pub --reporter expanded` run had 347 passes, 40 skips and
four failures: the existing macOS glyph-pixel cases in `button_labels_test.dart`,
two cases in `control_labels_test.dart`, and `field_alignment_test.dart`.
Their strict assertions remain intact; see [testing](../testing.md#merged-flutter-verification-on-macos).
The output is `apps/mobile/build/verification/interaction-scenarios-full-suite.log`.
The prior run also found a stale cancellation-error assertion; after updating it
to require no late error while preserving the no-send assertion, that case passed
in that run. The unmodified strict glyph tests were then verified on Linux as
described below; the macOS failures remain failures on that host.

A separate Debian 12 x86_64 guest under QEMU TCG ran the unmodified Flutter
3.47.7 suite with a locally built x86_64 FFI library and Rust 1.96.0. The first
full run with default timeouts had 337 passes, 40 skips and 14 failures: two
large tests exceeded Flutter's ten-minute limit, and the subsequent cases in
one file failed after the timeout. A single-concurrency retry of the two files
with `--ignore-timeouts` passed all 14 tests. A subsequent complete run of
`flutter test --no-pub --reporter expanded --ignore-timeouts` passed 351 tests,
skipped 40 opt-in tests, and exited 0 after 53 minutes 26 seconds. The timeout
setting accommodates software emulation and does not erase the first failure.
The unchanged independent glyph tests also passed separately, five out of five.
The 24 opt-in production capture configurations were run separately as below;
skipped tests in the full run are not counted as passes. Original logs, source
identity and environment details are in
`apps/mobile/build/verification/linux-vm-attempt/qemu-x86/`.

An isolated Ubuntu 24.04 ARM64 virtual machine was also attempted on this host
with the existing Linux Rosetta support, a read-only source snapshot and separate
temporary data. The guest booted and reached its package sources, but its 6.8
kernel twice faulted in `seccomp_run_filters` during dependency installation;
the guest could not disable its fixed BPF JIT. No HomeVerb test ran in that
guest. Logs are retained in `apps/mobile/build/verification/linux-vm-attempt/`;
the temporary VM was stopped and removed. This is an environment failure, not a
passing or failing product test.

A second isolated guest used Debian 12 with its different 6.1 ARM64 kernel.
Its official image passed the published SHA-512 check; its source snapshot was
`4a57eb9`. It booted, installed initial dependencies and registered Rosetta, but
reported an illegal instruction and corrupted shared-library errors during Rust
toolchain setup. It likewise ran no HomeVerb tests. Its logs are in the `debian/`
subdirectory of the same artifact directory. The guest and downloaded files were
removed; neither attempt changed the host configuration.

The actual `NativeApp` capture fixture uses production routes and fictional
clients. A matrix covers both `window_light` and `sky_blue`, `en-US` and `zh-CN`,
320/390/430 logical-pixel widths and 100%/200% text. Each configuration captures
20 scenes, including first load/failure, cached reopen, partial refresh and
pending/unknown commands. Additional 320-pixel, 200% Chinese captures exercise
high contrast and top/bottom safe areas; English and Chinese 390-pixel captures
exercise a simulated 300-pixel keyboard inset. Complete-content captures use a
taller test viewport; interaction tests separately exercise scrolling. The
keyboard inset does not render or verify an operating-system keyboard. The
Chinese test fallback was the local Hiragino Sans GB font, not a phone font.
Captures and per-configuration logs are local build artifacts under
`apps/mobile/build/verification/interaction/` and
`apps/mobile/build/verification/interaction-final-*.log`; build artifacts are not
versioned and can be regenerated with `test/interaction_capture_test.dart`:

```sh
cd apps/mobile
fvm flutter test --no-pub test/interaction_capture_test.dart --reporter expanded \
  --dart-define=HOMEVERB_INTERACTION_CAPTURE=true \
  --dart-define=HOMEVERB_CAPTURE_SKIN=window_light \
  --dart-define=HOMEVERB_CAPTURE_LOCALE=zh-CN \
  --dart-define=HOMEVERB_CAPTURE_WIDTH=320 \
  --dart-define=HOMEVERB_CAPTURE_TEXT_PERCENT=200 \
  '--dart-define=HOMEVERB_CJK_FONT=/System/Library/Fonts/Hiragino Sans GB.ttc' \
  '--dart-define=HOMEVERB_CJK_BOLD_FONT=/System/Library/Fonts/Hiragino Sans GB.ttc'
```

Vary the skin, locale, width and text percentage for the matrix. The fixture also
accepts `HOMEVERB_CAPTURE_HIGH_CONTRAST`, `HOMEVERB_CAPTURE_SAFE_TOP`,
`HOMEVERB_CAPTURE_SAFE_BOTTOM` and `HOMEVERB_CAPTURE_KEYBOARD_INSET` defines.

Profile measurements used Flutter 3.47.7 on a disposable Android 15 API 35 arm64
Pixel 7 emulator, Apple M1 Pro and the emulator's OpenGL ES translator on Metal.
The measured source's visible detail shell took 36.273 ms p95 for 20 warm cached
transitions and 52.714 ms p95 for 20 transitions with reads held for at least
five seconds; one cold transition took 35.991 ms. All were under the proposed
150 ms p95 target. In one before/after scroll run, build p95 was 1.063/0.864 ms
and raster p95 was 5.178/4.062 ms; neither run had a build or raster frame over
16.667 ms. The single-run difference does not establish a cause. Raw durations,
distributions, commands and source identity are in
`apps/mobile/build/verification/interaction-profile-final/`; the baseline and
earlier intermediate measurements are in the adjacent `interaction-profile/`.
After measurement, seven view files changed only their import from the native
client implementation to its shared contract, allowing the web preview to
compile. The benchmark did not rerun after those import-only changes. These
results do not establish physical-phone performance.

On this macOS host `cargo fmt --all -- --check` and the offline workspace build
passed. The workspace test run outside the sandbox stopped at 14 existing Midea
LAN tests because `127.0.0.2` was unavailable; the sandboxed attempt also denied
local sockets. Clippy with Rust 1.99.1 and `-D warnings` stopped on four existing
`chunks_exact_to_as_chunks` warnings in Midea code. The CI-pinned
`cargo +1.96.0 clippy --workspace --all-targets --offline --locked -- -D warnings`
subsequently passed. `verify_mvp.py` passed with
seven Linux-only skips, `verify_native_contract.py` passed, and Python unit tests
passed 70 cases. Documentation, contract, repository and presentation catalog
validators passed after the evidence update.
The documented `fvm flutter build web -t lib/main_preview.dart --no-pub` first
failed because the existing preview import chain reached `dart:ffi`. Replacing
unneeded native implementation imports with the shared client contract made the
rerun pass. Both logs are in `apps/mobile/build/verification/`; this preview build
is not the source of the mobile captures or profile measurements.
Linux Flutter integration/builds, physical Android/iOS accessibility and keyboard
behavior, real provider acceptance and appliance writes remain unverified here.
No physical device, live account, publication or real device write was used.
