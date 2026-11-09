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
