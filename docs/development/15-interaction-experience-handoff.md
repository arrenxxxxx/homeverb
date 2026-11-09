# Interaction experience implementation handoff

Status: **design approved; implementation in progress**. Owner review: November 9,
2026. Audited source baseline: `8cf791b`. Milestone evidence is tracked in the
[implementation record](interaction-implementation-evidence.md); pending stages
do not claim that the Flutter App already behaves like the approved preview.

## Scope and source of truth

Improve the whole foreground App journey: home/search/room cards, every device
detail family and generic fallback, accounts/sign-in, organization, settings,
history and diagnostics. The AC is a prominent example, not the scope boundary.
Use the approved preview's simple grouping, local loading and compact feedback.
Borrow interaction principles from Mijia without adding provider support or
copying proprietary assets.

The reviewed visual artifacts are `homeverb-interaction-preview.html` and
`homeverb-interaction-preview-standalone.html`, supplied separately with the
handoff. They contain fictional data and demonstrate eight surfaces plus cached,
initial-loading, initial-failure, silent-failure and uncertain-command scenarios.
Their browser checks prove the mockup only. This specification remains usable
without those workstation-local files; production Flutter captures are required
for implementation acceptance. Mockup values, ranges and feature combinations
are not device contracts.

Follow [AGENTS.md](../../AGENTS.md), the
[native UI contract](../native-ui-contract.md),
[execution contract](../routing-and-execution.md) and
[ADR-0067](../adr/0067-foreground-state-refresh.md).
This approved revision replaces the following earlier presentation instructions:

| Previous presentation | New implementation requirement |
| --- | --- |
| Homepage product artwork, including compact Sky Blue cards | The approved outlined appliance icons on every home-card variant and both skins |
| Fixed homepage Refresh button | Pull-to-refresh with pull, release and refreshing states |
| Device refresh button and last-read footer | Remove routine refresh controls and freshness captions from device pages |
| Refrigerator extra information in an expansion | User-facing information in flat, always-expanded groups |
| Dishwasher More programs/options and secondary details reveal | Every supported choice and information group visible by ordinary page scrolling |
| Missing numeric readings rendered as Unknown or explanatory prose | A spinner while first loading, then a number or an em dash |
| Blocking navigation while preparing controls | Open the destination immediately; prepare and load within it |

These replacements supersede the conflicting portions of the
[October device design](14-device-interaction-design.md),
[compact home design](compact-blue-home.md) and
[refresh experience](auth-refresh-experience.md). Their historical verification
records remain historical evidence. Preserve the November 9 home-card
missing-power **appearance** fallback: the off-looking icon does not turn a null
observation into false or enable a denied shortcut.

## Frozen acceptance requirements

| ID | Requirement | Observable completion condition |
| --- | --- | --- |
| UX-01 | Immediate navigation | A device/account tap shows its destination and working Back action while a delayed data fixture is still held; no global operation overlay precedes entry |
| UX-02 | Local restoration | Reopening restores that device's groups and saved readings without waiting for the network, including after restarting with a fictional persisted profile |
| UX-03 | Honest first reading | Value positions show spinners during loading; missing or failed values show `—`; no `Unknown`, `unknow`, `null`, `NaN`, invented zero or target appears as a numeric reading |
| UX-04 | Bounded initial failure | A terminal first-read failure with no saved reading ends loading and emits one concise localized Toast per opening attempt, including multi-component failure |
| UX-05 | Quiet periodic updates | Existing foreground 3,000 ms scheduling remains non-overlapping; automatic failures keep readings and produce zero new notices or Toasts |
| UX-06 | Intentional refresh | Home uses pull-to-refresh; device pages contain no routine Refresh or last-read footer; partial manual failure keeps successes and reports one combined error |
| UX-07 | No folded device content | All currently eligible user-facing functions, choices and device information are inline; no accordion, More functions/programs/options/information or overflow menu is required to see them |
| UX-08 | Icons on home | Home/search/room cards use the preview's outlined icon family, with no product image in normal, compact or enlarged-text variants |
| UX-09 | Local command feedback | Only the relevant control/group shows activity; repeated taps cannot duplicate a command; requested values do not replace observations |
| UX-10 | Centered Cancel | Sign-in Cancel's visual and touch-target centers match the form content center within one logical pixel at each tested width |
| UX-11 | Compact Toast | A short message sizes to its content, centered within one logical pixel; a long message wraps with at least 24 logical pixels of side clearance |
| UX-12 | Complete secondary flows | Empty/loading/error/content states are distinct; every retained settings entry works; account and organization recovery stays in context |

UX-07 permits ordinary vertical page scrolling. It does not remove actual command
confirmation, account removal, cleanup or identity-recovery confirmation. User
consent must remain separate from merely viewing a setting. Unknown vendor codes
remain unknown in the domain; do not guess their meaning to remove English text.
Translate their user-facing fallback and keep raw protocol explanations out of
daily reading rows. Necessary authorization or uncertain-command recovery remains
visible and actionable.

Eligibility still follows the capability and observation contracts. In particular,
`control_requires_observation` may omit a setter until its required observation
exists. A cached descriptor must not expose that setter early. Keep a stable
noninteractive group placeholder where useful, then reveal eligible controls
automatically without requiring an expansion tap. Declared cycle choices that
are merely locked during a running program remain visible and disabled.

## State and feedback contract

Track layout/metadata readiness, reading readiness, control readiness and command
progress independently. Do not encode all of them as one page-wide `busy` flag.
Native work may still require serialization; UI independence is not permission
for concurrent session replacement or unbounded Core calls.

| Situation | Display | Feedback and work |
| --- | --- | --- |
| No descriptor loaded yet | Immediate detail shell, stable title/Back, section placeholder | Load local metadata inside the destination; no invented controls |
| Descriptor ready, no reading yet | Real groups; spinner in each unresolved value position | Start one bounded opening read promptly; do not wait for the first periodic tick |
| Successful read with absent property | `—` only in the absent field | No error Toast for a legitimately missing optional field; do not borrow a setpoint as a measured value |
| First read fails for a component with no prior completed snapshot | Failed values become `—`; successful values remain | At most one short Toast across such failures in that opening attempt; the next regular silent cycle may recover |
| Cached component on re-entry | Saved layout and readings immediately; previously confirmed absent fields remain `—` | Refresh silently; no full-page skeleton or disappearing groups |
| Automatic refresh fails | Keep prior values; unresolved fields stay `—` | No Toast, banner, spinner, receipt change or retry burst |
| Manual pull refresh | Keep cards; show the pull indicator | Publish completed items promptly, continue after per-item failure; one combined failure Toast; no routine success Toast |
| Command pending | Old observation plus local pending indicator | Disable duplicate/conflicting actions; keep navigation usable; no optimistic verified state |
| Command accepted, readback fails | Retain accepted outcome and old observation | Do not relabel the command failed or verified; keep concise pending-confirmation feedback where needed |
| Command failed or unknown | Accurate result with a useful recovery action | Unknown offers state inspection, never automatic command replay |
| Account/history/diagnostics first load | Destination-local loading | Empty results appear only after successful empty retrieval; errors offer the relevant retry |

Once an opening attempt has ended, periodic failures must not repeat its Toast.
Leaving the page invalidates pending Toasts and results. Successful periodic reads
may update values without silently clearing an unrelated manual error or receipt.
Deduplicate errors at the user-operation boundary, not separately in every zone.
For a partially cached device, apply this distinction per component: retain cached
snapshots silently, and resolve first-time components with their own loading and
failure state, combining any first-time failures into one opening Toast. A field
confirmed absent by an earlier completed snapshot is not a never-loaded component.

The immediate opening read is user-triggered work. The automatic schedule still
starts a fresh three-second interval once that user operation has drained and the
session is eligible under ADR-0067; it does not overlap the opening read, accumulate
missed ticks or prepare controls. Record this distinction
in the implementation documentation rather than changing the timer to a shorter
interval. A cycle lasting more than three seconds does not promise a fresh
response from every device every three seconds.

## Cache and session boundaries

Reuse saved Core descriptors, existing SQLite observation caches and local
presentation preferences. Before adding any cache, demonstrate what cannot
already be restored from those sources. Device-specific layout is derived from
actual capabilities; cache derived presentation only when useful.

| Data | Restore behavior | Invalidation or limitation |
| --- | --- | --- |
| Names, room, device/component metadata | Render locally before network work | Apply local edits; isolate profiles and local device/component IDs |
| Capability-derived sections and options | Reuse for immediate layout | Reconcile against the current descriptor revision, or a stable fingerprint of relevant descriptor content if no revision exists |
| Previous observations | Display while refreshing | Preserve source/time/stale metadata; never use retrieval time as measurement time or a cache hit as online evidence |
| Current grants and control prerequisites | Reinspect independently before enabling/executing | Cached descriptors cannot grant permissions; session replacement, revocation or identity changes invalidate readiness |
| Credentials, provider identifiers and approval previews | Never add to a presentation cache | Existing protected stores and trusted single-use approval lifecycle remain authoritative |

Account/device removal, profile switch and relevant local-cache cleanup must
invalidate corresponding in-memory/derived entries. Unchanged capabilities
should not make whole sections disappear during refresh. Changed capabilities
must remove obsolete controls without retaining their prior authority. If valid
cached metadata is unavailable, show a shell and recoverable local error rather
than another device's layout.

Fence all asynchronous publication by page, selected device/component, profile
and session/lifecycle generation as appropriate. A late result from device A
must never populate B, resurrect a removed account, show a stale Toast or restore
an invalid approval. Preserve cancel-and-drain ordering around session reuse.

## Milestones and dependency order

Each milestone is a separately verified local commit. Implementation sessions
must update its evidence before changing its status. A screenshot alone cannot
complete a state-management milestone; a widget test alone cannot complete visual
or physical-phone acceptance.

| Milestone | Status | Depends on | Deliverable and exit gate |
| --- | --- | --- | --- |
| M0 | Complete | None | Baseline and deterministic delayed/error fixtures; requirement-to-test map and screen inventory recorded |
| M1 | Complete | M0 | Immediate device/account entry, local restoration and independent readiness; delayed navigation and restart/cache isolation tests pass |
| M2 | Complete | M1 | First-reading states, partial refresh, pull-to-refresh and 3-second integration; state/feedback/lifecycle matrix passes |
| M3 | Complete | M0; integrate after M2 | Icon cards and capability-specific summaries; all card variants and hit regions pass visual/behavior checks |
| M4 | Complete | M1, M2 | All device groups expanded, local command feedback and truthful outcomes; all device-family and approval tests pass |
| M5 | Complete | M1, M2 | Secondary flows, centered Cancel and content-width Toast; route/geometry/empty/error tests pass |
| M6 | Pending | M1-M5 | Integrated production captures, regressions and measured performance; evidence and unresolved physical acceptance explicitly recorded |

M3 presentation work may run alongside M1/M2. M4 device-widget work and M5
account/feedback work can be assigned separately after their common state
contracts stabilize. One implementer should own `main.dart` and `controller.dart`
integration to avoid incompatible state/scheduling changes.

### M0: establish fixtures and baseline

- Recheck the repository revision, worktree, pinned Flutter and available native
  dependencies. Read current task/compatibility records and preserve other work.
- Map every production route, device family and reusable card branch to UX IDs.
  Include unsupported/generic devices, search, rooms, both skins and both locales.
- Extend existing fictional clients with controllable completion/deadlines,
  partial component/device errors, session replacement and restartable local
  caches. Record native method counts to prove absence of replay/network work.
- Capture current production entry/loading/error behavior with synthetic data.
  Distinguish the user's reported `Unknown` temperature from a reproduced case:
  the audit found a generic null formatter, but did not reproduce that exact
  reported screen on a phone.
- Exit: fixtures can hold a read for five seconds, release it successfully or
  fail it, simulate a bounded timeout and inspect calls without a real account.

### M1: navigation, local restoration and readiness

- Refactor device opening so navigation does not await descriptor retrieval,
  `_prepareDeviceControls`, account inventory or a native session reopen. Render
  available local card metadata first and keep Back responsive.
- Apply the same principle to account details; avoid awaiting a full `loadLocal`
  before showing the selected account. Preference saving must not block tabs.
- Separate loading layout/readings from preparing permitted controls. Preserve
  all existing exact grant and adapter checks when executing. Do not simply
  remove `busy` guards around native operations.
- Restore local information first. When the existing session admits reads, start
  the opening read without unnecessarily waiting for control-grant preparation or
  account inventory. Serialize it with session replacement; if replacement is
  genuinely required for the read, retain local reading progress until it drains.
  A held-control-preparation fixture must demonstrate this ordering and no
  overlapping use of a native session being replaced.
- Restore saved per-device presentation across reopen and process restart;
  reconcile changed descriptors and all invalidation paths listed above.
- Exit: one pump after tapping a known card/account, its destination shell is
  visible while a five-second fixture remains unresolved. Cached groups/readings
  render before any network response; a revoked grant remains disabled. A-to-B,
  Back, background and removal fixtures reject late results.

### M2: readings and refresh

- Introduce explicit never-loaded/loading/available/missing/failed reading state
  per relevant component or field, separately from cached/refreshing status.
  Bind spinners and `—` consistently across AC targets/room readings, refrigerator
  zones, cycle values and generic detail rows. Keep geometry stable.
- Start the bounded opening read within the destination. A failure must leave
  no indefinitely spinning value and emit no duplicate zone/global errors.
- Keep cached values during re-entry and refresh. Publish each completed manual
  card/component update without waiting for every other item. Individual failures
  must not abort the remaining refresh targets.
- Replace fixed home refresh controls with pull, release and progress states,
  including empty/short lists. Preserve search/room context. The Chinese release
  label must express the owner's exact release-to-refresh wording in the catalog.
  Expose a non-gesture accessibility refresh action without restoring the fixed
  visual button. Prevent repeated pulls from enqueueing duplicate cycles.
- Remove routine device refresh buttons/footers and technical measurement-time
  caveats. Keep evidence metadata internally and purpose-specific recovery such
  as inspecting an uncertain command's state.
- Exit: measure the fresh interval from eligibility after the opening/user work
  drains, or from resume eligibility; count periodic calls separately from the
  opening read. Virtual-time tests show no periodic call at 2,999 ms and one
  eligible cycle at 3,000 ms; slow cycles skip ticks. Ten failed automatic cycles create
  zero notices, preserve values/receipts and do not refresh inventory or grants.
  Background/non-device navigation stops reads; resume creates a fresh interval.
  User work cancels and drains automatic work before native session use.

### M3: icon cards and useful summaries

- Use one consistent outlined vector family matching the preview. The reference
  shapes are Lucide `air-vent`, `refrigerator` and `washing-machine` for the
  approved AC, refrigerator and dishwasher tiles. Retain the actual device name;
  the illustrative icon does not identify its protocol family or capabilities.
  Provide coherent fan/light/media/sensor and generic-device fallbacks.
- Bundle or code-render icons locally with appropriate third-party notices when
  needed. Do not introduce runtime icon downloads. Replace product image calls
  in normal, compact and enlarged-text home/search/room cards across both skins.
  Do not delete assets still used on other surfaces.
- Derive summaries from actual semantics: AC target/mode where available;
  refrigerator declared zone temperatures with set/current meaning preserved;
  dishwasher program/phase/remaining time where available. Do not use a climate
  target-temperature lookup for every appliance. Unknown fields use `—`.
- Preserve separate open/power/menu hit targets, no false online badges, pending
  observation colors and the already approved missing-power appearance fallback.
- Exit: production captures show no product artwork in any card branch; a widget
  assertion checks no product `Image` is built in the card subtree. Each icon and
  summary corresponds to the fixture's semantics; open/power/menu taps dispatch
  only their intended action. No icon change expands shortcut eligibility.

### M4: expanded device pages and command feedback

- AC order: power and target, observed room reading, mode, fan, supported swing
  and direction controls, supported comfort/energy groups, then light/sound and
  device information. Extra groups follow basic controls without any reveal tap.
  Keep the primary target editor unique and use exact descriptor bounds/steps.
- Refrigerator: declared zones with set/current distinction, supported function
  states, door/reminder readings and device information, all inline. Omit empty
  groups, keep unclassified supported information visible, and show a necessary
  read-only limitation once. Meiju refrigerator writes remain unadmitted.
- Dishwasher: observed program/phase/time, all supported program/argument
  choices, applicable actions, consumable readings and information. While
  running, keep choices visible and disable only invalid edits. Preserve scalar
  additional-option semantics; do not copy multi-select artwork into an enum.
- Apply the same always-expanded rule to generic and future capability layouts.
  Do not hide fields behind an ellipsis, secondary information page or accordion
  merely to shorten the page. Rename/edit actions may still have dedicated flows.
- Replace routine page-wide command activity with control/group-local progress.
  Avoid routine accepted/verified bottom-card clutter; retain useful pending or
  uncertain outcome and recovery feedback. `accepted`, `verified`, `failed`,
  `unknown` and per-item partial results remain distinct in the underlying data.
- Exit: every currently eligible user-facing option is reachable by scrolling
  without a reveal tap, respecting the observation-gated exception above. Rapid
  repeated input sends one command. Draft/accepted values never
  overwrite observations; readback failure preserves accepted. Unknown, refresh,
  reopen and lifecycle recovery cause zero command replays. Elevated operations
  still consume an immutable preview only after actual trusted confirmation.

### M5: accounts, feedback and secondary journeys

- Account overview/detail: lead with name/platform/devices; retain devices on
  retrieval failure. Distinguish a network retry from expired authorization and
  re-sign-in. Keep existing in-place refresh and its single button-local spinner.
- Sign-in: center Cancel; preserve field-specific validation, duplicate-submit
  prevention and responsive cancellation. Keep protected buffers, credential
  clearing and late-result guards. An uncertain credential persistence outcome
  offers account reload instead of blindly repeating login.
- Toast: remove full-width constraints in both the Toast child and its host;
  use intrinsic content width with bounded wrapping. Preserve deduplication,
  the existing maximum queue of three, navigation/background cleanup, contrast
  and accessible announcement. Preserve the extended accessibility duration,
  actionable-feedback persistence until dismissal for screen-reader users and
  reduced-motion behavior. Persistent actionable errors remain inline.
- Search/rooms: separate no devices, empty room and no matches. Preserve filters
  and scroll context on Back. Default move selection to the actual room,
  including Unassigned; show selection count and disable an empty submission.
  Make room actions visible separately from drag handles.
- Settings: group preferences, management and help; remove ordinary dead-end
  sharing/full-profile-deletion entries. Connect retained Privacy/Licenses items
  to real local content. Connection choices reflect admitted routes; do not add
  unsupported controls. Inspect prototype-driven rows as well as dynamic code.
- History/diagnostics: distinguish loading/error/empty/content. In history/receipt
  UI, show device/action, concise outcome and local time first, retaining execution
  evidence. Diagnostics keep their existing identifier-free allowlist and exact
  reviewed export; do not add device/account names or identifiers to that report.
  Retrying record retrieval never executes the recorded command.
- Exit: Cancel center error is at most one logical pixel. At 320/390/430 widths,
  short Toast width equals its measured content plus declared padding (within
  rounding tolerance), is centered, and is narrower than its max-width bound;
  long Toasts wrap without overflow. Both locales, 200% text and keyboard/screen
  reader navigation remain usable; actionable Toasts remain available until
  dismissal with accessible navigation enabled. Every retained settings row has a
  tested destination/action. Empty-state actions recover the corresponding situation.

### M6: integrated verification and handoff

- Run the full App regression suite after the phase-specific checks. Capture
  actual production routes with fictional fixtures, real Back/header/widgets
  and the production fonts. No HTML or screenshot-only wrapper substitutes for
  App verification. Review visual grouping as well as overflow assertions.
- Capture home, AC, refrigerator, dishwasher idle/running, accounts, sign-in,
  settings and history/diagnostics. Include first loading/failure, cached reopen,
  partial refresh, command pending/unknown, centered Cancel and short/long Toast.
- Check both skins and languages at 320, 390 and 430 logical pixel widths;
  exercise 200% text, high contrast, safe areas and keyboard insets. Inspect
  complete scrolling content so hidden extras cannot evade acceptance.
- Measure a profile build separately from widget tests. The proposed engineering
  target is tap-to-detail-shell p95 at most 150 ms over 20 transitions on a named
  reference emulator, both with cached readings and with a held five-second
  network response. Record raw durations, device/OS/GPU/build, sample method and
  cold versus warm runs. The deterministic M1 gate remains mandatory regardless
  of benchmark tooling. These are targets, not measurements already achieved.
  Record unmet or unmeasured targets explicitly; an unavailable reference runtime
  leaves the performance gate blocked rather than passed.
- Use the existing scroll-performance fixture on the same named configuration
  before/after; record frame build/raster distributions and investigate regression.
  An emulator result does not establish physical-phone smoothness. Do not reuse
  historical GPU measurements as current results.
- Update current verification with commands/results, captures, commit IDs,
  compatibility effects and precise blockers. Keep historical records intact.
  Reconcile obsolete visual assertions against the approved new design; do not
  silently bless changed golden files or claim pixel parity with the old layout.
- Exit: all required behavioral/layout and profile-performance gates pass;
  unresolved platform/physical acceptance is explicitly listed. Packaging or
  installing an App is a separate
  requested deliverable; preserve existing profiles with the documented retention
  procedure whenever runtime testing uses an installed App.

## Source and regression map

Paths in the next table are relative to `apps/mobile/`. Test names identify
existing coverage to extend, not claims that the new acceptance cases exist.
`test/device_loading_test.dart` is a proposed new suite if the existing tests
cannot express the combined navigation/readiness fixtures clearly.

| Area | Implementation entry points | Existing regression suites |
| --- | --- | --- |
| Entry, cache and control readiness | `lib/main.dart` device/account navigation; `lib/controller.dart` `selectDevice`, `_prepareDeviceControls`, `selectAccount`, `perform`, `loadLocal`; `lib/session_permissions.dart` | `current_contract_test.dart`, `session_permissions_test.dart`, `account_surfaces_test.dart`, `credential_access_test.dart` |
| Read scheduling and manual refresh | `lib/controller.dart` automatic timer, `refreshCards`, `refreshComponents`; `lib/screens.dart`; `lib/refresh_feedback.dart` | `state_refresh_test.dart`, `refresh_feedback_test.dart`, `component_test.dart` |
| Cards and summary formatting | `lib/blocks.dart` `DeviceGrid`/`DeviceTile`/`product`; `lib/device_presentation.dart`; `lib/main.dart` `deviceCards` | `device_cards_test.dart`, `compact_device_cards_test.dart`, `skin_selection_test.dart` |
| Loading, missing and device layout | `lib/controls.dart` `propertyValue`; `lib/device_readings.dart`; `lib/device_page.dart`; `lib/climate_controls.dart`; `lib/compartment_controls.dart`; `lib/appliance_panels.dart`; `lib/device_detail_controls.dart` | `climate_controls_test.dart`, `climate_rows_test.dart`, `device_detail_controls_test.dart`, `compartment_overview_test.dart`, `refrigerator_modes_test.dart`, `control_labels_test.dart` |
| Commands, confirmation and outcomes | `lib/controller.dart` execute/approval paths; `lib/approval_review.dart`; `lib/appliance_controls.dart`; `lib/receipts.dart` | `dishwasher_interaction_test.dart`, `dishwasher_permissions_test.dart`, `dishwasher_approval_copy_test.dart`, `approval_test.dart`, `outcome_click_test.dart`, `receipts_test.dart`, `native_test.dart` |
| Accounts and sign-in | `lib/account_surfaces.dart`; `lib/account_views.dart`; `lib/account_identity.dart`; `lib/credential_access.dart` | `account_surfaces_test.dart`, `account_feedback_test.dart`, `auth_experience_test.dart`, `accounts_test.dart`, `account_identity_test.dart`, `identity_recovery_test.dart` |
| Toast and retained errors | `lib/feedback.dart` `FeedbackToast` and `FeedbackQueue`; callers in `lib/main.dart` | `feedback_test.dart`, `refresh_feedback_test.dart`, `account_feedback_test.dart` |
| Organization, settings and records | `lib/main.dart` search/room/settings/history routes; `assets/prototype.json`; `lib/diagnostics_view.dart`; `lib/receipts.dart` | `organization_test.dart`, `app_test.dart`, `screens_test.dart`, `cleanup_test.dart`, `diagnostics_view_test.dart`, `diagnostics_export_test.dart` |
| Cross-cutting layout and copy | Shared widgets, `lib/skin.dart`, canonical catalogs under `packages/presentation/locales/` at repository root | `localization_test.dart`, `accessibility_test.dart`, `contrast_test.dart`, `safe_area_test.dart`, `field_alignment_test.dart`, `button_labels_test.dart` |

The audit found that `selectDevice` currently waits for descriptor/control/cache
work before navigation; first live detail values can arrive on the periodic
tick; and manual card refresh can stop on the first exception. The existing
three-second scheduler, account refresh spinner and Toast lifecycle already
provide useful behavior and should be retained. Source symbols are more stable
handoff references than line numbers; recheck them against the implementation
revision before editing.

## Mandatory scenario matrix

| Scenario | Required evidence |
| --- | --- |
| First entry, empty cache, delayed success | Immediate destination; per-value loading; real value after completion; no placeholder temperature |
| First read rejects or times out | Loading ends at terminal completion; `—`; one Toast; no duplicate retry |
| Partial card/component failure | Completed successes publish while another request is held; failed fields preserve prior values or `—` |
| Warm reopen and restart from disk | Correct layout/readings before any network response; no cross-profile/device leakage |
| Cached layout with revoked grants | Layout remains readable; denied controls stay disabled; execute preflight still refuses |
| Rapid A-to-B, Back, filter change, removal | Late responses cannot change current content, restore removed data or show stale feedback |
| Ten failed periodic cycles | Zero new Toasts/banners; same cached values and receipt; bounded non-overlapping reads |
| Slow read plus repeated pull or command | No queued timer burst or duplicate manual work; user operation gets safe native priority |
| Background/resume and non-device route | Admit no new network work while backgrounded; cancel existing work immediately, drain within existing lifecycle bounds and reject late publication; start a fresh eligible resume interval |
| Descriptor change and local-cache clearing | Removed features disappear safely; cleared readings do not reappear from a UI cache |
| Accepted command then failed readback | Accepted remains accepted; prior observation retained; no fabricated verification |
| Unknown command, reopen and restart recovery | Unknown remains accurate; zero automatic execute/replay calls; approval cannot survive invalidation |
| Empty search/room/account/history | Distinct successful-empty UI and working contextual action; loading is never mislabeled empty |
| All currently eligible device options | Visible through page scrolling, including disabled running-cycle choices; zero reveal taps; observation-gated setters stay unavailable until their prerequisite is met |
| Narrow/large text and both languages | No clipped value/actions, usable targets, centered Cancel and bounded Toast; no raw English numeric fallback in Chinese |

## Commands and phase evidence

Check tool availability first; Flutter is pinned by `apps/mobile/.fvmrc`.
Build the local native test dependency as documented in
[testing](../testing.md) and the [mobile README](../../apps/mobile/README.md).
Run relevant focused suites during each phase, then the integrated gates:

```sh
# From the repository root.
python3 scripts/validate_docs.py
python3 scripts/validate_repository.py
python3 scripts/validate_contracts.py
python3 scripts/validate_commits.py --all
node packages/presentation/test/i18n.test.mjs

# From apps/mobile, using the pinned toolchain and prepared native library.
fvm flutter analyze
fvm flutter test --reporter expanded
```

Edit canonical English/Chinese catalogs, then run
`python3 scripts/generate_flutter_i18n.py --generate` from the root. Do not edit
generated translations manually or put Chinese prose into other source/docs.
Use `fvm install` and `fvm flutter pub get` when dependencies need preparation.
Screenshot font/capture opt-ins and platform integration commands are documented
in the existing test fixtures and mobile README; skipped captures are not passes.
Run the Rust/runtime checks in the testing guide if native execution/contracts
change, rather than reporting UI tests as sufficient evidence for those changes.

For each milestone, record: status, implementation commit, UX IDs covered, exact
commands and results, test/capture paths, reference device/toolchain, remaining
blockers and compatibility effects. Mark a milestone complete only after its
exit gates pass. A missing physical test is recorded separately from successful
synthetic evidence. Keep local commits small and use English Conventional Commit
messages. No push, publication, diagnostics upload or real appliance write is
part of this handoff.

Expected compatibility effect: presentation and foreground host orchestration
only; no added provider/model admission, native API/schema change, new authority,
background work, backend or CLI/MCP polling. If implementation requires a public
contract or execution-semantics change, update the corresponding ADR/contracts,
errors and positive/negative fixtures before claiming that larger change.

## Starter instruction for an implementation session

> Implement the approved November 9 HomeVerb interaction revision from
> `docs/development/15-interaction-experience-handoff.md`. Read AGENTS.md and the
> required current project documents first. Begin at the first pending milestone,
> preserve existing work, and record actual verification for each local phase
> commit. Treat UX-01 through UX-12 and the scenario matrix as acceptance gates.
> Match the supplied preview while deriving all controls from real capabilities.
> Use icons on every home-card variant and keep every device information/control
> group expanded. Preserve authority, trusted confirmation, foreground scheduling
> and uncertain-write semantics. Do not claim mockup evidence as Flutter or phone
> acceptance, and do not perform remote publication or real-device writes.
