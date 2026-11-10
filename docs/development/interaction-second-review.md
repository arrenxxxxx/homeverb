# Second interaction review fixes

The baseline was `505a6a5`. All reproductions use fictional clients; no provider
account or appliance command was used. The original six assertions failed before
these changes (`apps/mobile/build/verification/review2-before.log`): three manual
refresh paths suppressed a reopen error, a pending quick command swallowed a card
tap, a combined grant had no form, and an enum option violated its minimum.

1. Manual reads distinguish failure before their work callback from failure
   after it starts. The pre-work path checks the foreground, navigation generation,
   cancellation and the original device or receipt context. The widget regression
   also checks a single home refresh feedback and checks that background and
   navigation suppress a late failure. Existing automatic-read tests cover silent
   failures. Session restore does not perform a write.
2. A card tap opens the device view while a quick command drains. The opening
   waits for that work before calling the native session. A sent result remains
   associated with its command; backgrounding invalidates a late result. Tests
   check the immediate route, no intervening calls, one execute, an accepted
   receipt, and no replay after a background transition. A cancelled wait is not
   evidence of a physical rollback.
3. A grant requiring combined arguments retains the capability form. The
   synthetic exact-grant test supplies both values and asserts one combined
   execute; separate tests cover target-only, mode-only and unrestricted modes.
   Core planning remains mandatory and the form does not create authority.
4. Inline enum choices now intersect the action parameter schema with current
   authorization and existing observation and lifecycle conditions. Tests exercise
   integer type, minimum, maximum, step and valid 50/60 choices. A separate
   regression verifies that a missing required observation prevents the option
   from appearing and sends no command (`review2-observation.log`). Core validation
   remains in place.

The focused integrated selection in `review2-integrated.log` passed 65 tests and
skipped two opt-in captures. The separate zone selection in
`review2-controls-final.log` passed five. The opt-in `review_capture_test.dart`
selection in `review2-capture.log` passed 15 production `NativeApp` capture
scenarios, including the first review scenes. Captures are unversioned local
artifacts in `apps/mobile/build/verification/interaction-review/`, with fictional
Core clients, 320 x 844 logical pixels, a local CJK fallback font, and a neutral backing for transparent Flutter layers. The
`review2-reopen-feedback.png`, `review2-quick-opening.png`,
`review2-combined-form.png`, and `review2-enum-bounds.png` files cover the changed views. They are not phone or
physical-appliance evidence.

Flutter analysis passed after removing a test import warning. The complete macOS
Flutter run (`review2-full.log`) had 379 passes, 54 skips and four failures:
`button_labels_test.dart`, two cases in `control_labels_test.dart`, and
`field_alignment_test.dart`. These are the previously recorded strict macOS
glyph-pixel differences, not passing results. The explicit capture run above is
separate from the default skipped captures. The final rerun (`review2-full-final.log`) had 381 passes, 55 skips and the same four failures.

The system Python initially could not run contract validation because its
requirements were absent; `.venv/bin/python scripts/validate_contracts.py`
passed with 21 schemas, 52 positive and 341 negative fixtures. Documentation,
repository and commit checks passed. An attempted i18n `--check` invocation was
invalid because the generator has no such option; no locale catalogs changed.
No physical phone, production provider session, native device write, Linux
rerun, deployment, publication or push was performed. No native contract,
provider authority, background polling or retry policy was changed.

## Third review: inline parameter schemas

The `4077c4a` baseline still allowed an inline enum member at an exclusive
minimum. `review3_inline_schema_test.dart` first failed three scenarios
(`review3-before.log`): exclusive minimum, the intersection of exclusive maximum
and const, and an unknown `not` constraint. The new validator applies only to
inline single-argument controls and accepts scalar string, boolean and finite
number values. It interprets `type`, `enum`, `const`, `minimum`, `maximum`,
`exclusiveMinimum`, `exclusiveMaximum` and `multipleOf`; step checks use the
represented decimal values. Presentation annotations `title`, `description`,
`default`, `examples`, `deprecated`, `readOnly`, `writeOnly` and `$comment`
are ignored as non-validation annotations. Other keywords, malformed bounds,
invalid steps and unsupported types disable direct selection. The enclosing
argument object is checked for its supported object keywords and single-argument
cardinality; unsupported constraints do not enable inline actions. This is not
a general JSON Schema implementation. Existing authorization, observation and
Core plan/execute validation are retained.

The focused Flutter selection passed 49 tests (`review3-regression.log`),
including 40 and 50 disabled and one planned and executed fictional command
for 60, exclusive maximum, const, unknown and malformed constraints, decimal
steps, previous bounds, observation prerequisites, exact single and combined
grants, and component routing. `fvm flutter analyze` passed
(`review3-analyze.log`). The full Flutter suite was not rerun for this focused
change; its preceding macOS failures and skips are recorded above. No physical
appliance, provider account or real device write was used.
