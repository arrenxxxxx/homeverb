# Interaction review fixes

The review baseline is `248e3d5`. These phases use fictional clients and do not
change native contracts, device authority or adapter behavior. Captures and
logs under `apps/mobile/build/verification/` are local, unversioned artifacts.

## 1. Distinguish a new command error from an earlier receipt

`review_command_error_test.dart` first reproduced the issue: the new
`POLICY_DENIED` error was present in the controller but missing from the actual
device page (`review-1-before.log`, failed). The controller now identifies the
current command's error independently, so a prior receipt cannot hide it or
appear as the new outcome. Retained receipts remain available, and a later failed
readback still leaves an accepted result accepted.

The focused selection `review_command_error_test.dart`,
`generic_command_feedback_test.dart`, `write_fencing_test.dart` and
`climate_controls_test.dart` passed 30 tests (`review-1-after.log`).

## 2. Open a device during manual refresh

`review_navigation_refresh_test.dart` reproduced a held refresh dropping the
card tap (`review-2-before.log`, failed). The destination and its local device
metadata now appear immediately; its opening work waits for the cancelled refresh
to drain before using the native session. Generation checks still reject work
after navigation or backgrounding. The focused navigation, refresh and lifecycle
selection passed 29 tests (`review-2-after.log`).

## 3–5. Foreground and session recovery

`review_lifecycle_recovery_test.dart` first reproduced five independent
failures: interrupted device opening, history and diagnostics interrupted by
backgrounding, and history and diagnostics after replacing a closed session
(`review-3-5-before.log`). The visible device view now retries its interrupted
opening once after the prior operation drains; permission inspection is not
part of the three-second timer. Interrupted history and diagnostics loads enter
their existing retryable failure states. Their work binds the current client
and epoch only after session restoration.

Five additional tests reproduced the same stale-binding problem for card,
component, device and receipt refresh and receipt lookup
(`review-5-refresh-before.log`). Those operations now bind after restoration as
well. The combined focused lifecycle, session, history, diagnostics, refresh
and receipt selection passed 54 tests (`review-3-5-after-final.log`). No write
is replayed by these recovery paths.

Additional tests found that a failed session reopen could itself leave history
or diagnostics loading forever (`review-5-reopen-failure-before.log`). The
failure is now retained as retryable even when no work callback was entered,
with request and lifecycle generations preventing stale publication. The focused
recovery selection passed 45 tests (`review-5-reopen-failure-after.log`), and
Flutter analysis passed after this follow-up.

The automatic foreground recovery has an explicit device-page Retry if session
preparation fails again; the retry test first failed because that control was
absent (`review-3-retry-before.log`). While recovery is pending, the UI does not
claim that controls were denied. If another cancelled operation is still
draining when history or diagnostics opens, each page now enters its existing
retryable state instead of remaining idle/loading. Both cases were reproduced
before the fix (`review-busy-history-before.log`); the lifecycle selection then
passed 46 tests (`review-busy-history-after.log`).

## 6. Preserve other zone controls

`review_compartment_mode_test.dart` first failed because the supported normal
and eco options were absent (`review-6-before.log`). Standalone, authorized enum
arguments now appear inline beside the existing target editor; a selection sends
only its exact argument, retains the observed value until readback, and still
passes Core planning. Other supported action shapes retain their existing form.
The focused compartment and expanded-device selection passed 11 tests
(`review-6-after.log`), followed by a targeted rerun after duplicate-selection
guarding.

Production screenshot review exposed a separate template-label collision for
the inline mode (`review-6-label-before.log`). The two reviewed mode names and
the field title now use canonical localized messages, while unknown values stay
literal. A native-session fixture additionally verifies that a target-only grant
cannot enable a mode command (`review-final-security.log`).

## 7. Keep supported refrigerator readings after failure

`review_refrigerator_failure_test.dart` reproduced 15 loading rows disappearing
after the initial read failed (`review-7-before.log`). Readable properties in the
current descriptor now retain their rows and render a dash when absent or failed;
undeclared and unreadable properties are not added. The old assertion that hid
declared but unreported features was updated to require the truthful missing value
and to keep undeclared groups absent. The focused refrigerator and reading-state
selection passed 19 tests (`review-7-after.log`).

## 8. Keep privacy content and Close accessible

`review_privacy_dialog_test.dart` reproduced the missing scroll container in
both locales at 320 by 844 logical pixels and 200% text
(`review-8-before.log`). The existing dialog now scrolls its title and content
while keeping Close accessible. The bilingual layout, settings and accessibility
selection passed 16 tests (`review-8-after.log`).

## Integrated captures and compatibility

The opt-in `review_capture_test.dart` renders actual `NativeApp` routes with
fictional clients. It captures the new command error, opening during refresh,
foreground recovery and retry, zone modes in both languages, refrigerator read
failure, history and diagnostics retry, and the privacy dialog at the top and
bottom of its content. Captures use 320 by 844 logical pixels; privacy uses 200%
text. The CJK test fallback was the local Hiragino Sans GB font. They do not
prove physical-phone font, keyboard, provider or appliance behavior.

```sh
cd apps/mobile
fvm flutter test --no-pub test/review_capture_test.dart \
  --dart-define=HOMEVERB_REVIEW_CAPTURE=true \
  '--dart-define=HOMEVERB_CJK_FONT=/System/Library/Fonts/Hiragino Sans GB.ttc' \
  --reporter expanded
```

The screenshots are in `apps/mobile/build/verification/interaction-review/`;
they are local build artifacts and not versioned. No provider capability, native
API, Core schema, storage format, write retry, or background scheduling authority
was added. Existing physical-phone and provider acceptance remain open.

The opt-in capture selection and the card-tap regression passed 12 tests
(`review-capture-final.log`). The integrated review selection passed 73 tests
(`review-integrated-focused.log`); subsequent session and exact-grant tests passed
separately (`review-busy-history-after.log`, `review-final-security.log`).
Final `fvm flutter analyze` passed. The final full
`fvm flutter test --no-pub --reporter expanded` run on macOS had 369 passes,
51 skips and four failures (`review-full-suite-final.log`): the same four strict
macOS glyph-pixel cases recorded before this review in `docs/testing.md`.
Those failures and the skipped tests are not counted as passes. The 11 review
capture scenes among the skips were run explicitly as described above. The
Linux result from the earlier interaction phase predates these review changes;
no final Linux or physical-phone run is claimed for this review.
