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
