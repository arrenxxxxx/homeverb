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

## 6. Preserve other zone controls

`review_compartment_mode_test.dart` first failed because the supported normal
and eco options were absent (`review-6-before.log`). Standalone, authorized enum
arguments now appear inline beside the existing target editor; a selection sends
only its exact argument, retains the observed value until readback, and still
passes Core planning. Other supported action shapes retain their existing form.
The focused compartment and expanded-device selection passed 11 tests
(`review-6-after.log`), followed by a targeted rerun after duplicate-selection
guarding.

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
