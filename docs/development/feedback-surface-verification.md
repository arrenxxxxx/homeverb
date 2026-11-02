# Feedback surface verification

## Scope

Cloud Linux development validation on 2026-11-02, using Flutter 3.47.7,
Dart 3.13.5 and Rust 1.96.0. All account and appliance data are fictional.
No provider account, physical device, desktop connection, deployment or APK
was used. Historical verification records were not edited.

The approved design is implemented with real Flutter widgets, not a painted
mockup. Dark floating feedback distinguishes explicit operation results from
settings; persistent slim-edge notices retain account remediation. Only labeled
actions are clickable. Homepage feedback clears the bottom navigation, including
larger text. The compact Sky Blue cards, softer power colors and no-extra-menu
behavior are retained; the no-menu behavior was independently reimplemented
from the requested behavior rather than restoring an unavailable old artifact.

## Validation

- `flutter analyze`: no issues.
- `flutter test --reporter expanded`: 249 passed; 19 optional capture cases skip
  without their explicit capture flags. Existing native Mock integration, all
  82 screen states and 201 simulated atlas branches remain covered.
- Explicit Chinese capture/regression run: 11 passed. Real account pages captured
  at 390 by 844 logical pixels, with Inter regular/semibold and Noto CJK regular/
  bold loaded separately. Both skins cover complete, failed and expired-auth
  states; homepage refresh feedback is also captured at 430 pixels.
- Dedicated tests cover 320-pixel doubled text, high contrast, independent action
  hit targets, queue deduplication/bounds/clearing, screen-reader retention,
  reduced motion, late navigation/background responses, account identity,
  unknown credential persistence and unsupported-only refresh outcomes.
- `validate_docs.py`, `validate_contracts.py`, `validate_repository.py` and
  `validate_commits.py --all` pass. Contract checks cover 21 schemas, 52 positive
  fixtures and 341 negative fixtures.
- Repository Python tooling: 70 tests pass. Shared presentation catalogs:
  seven Node tests pass. The Linux native FFI library was rebuilt successfully.

Screenshots use actual Flutter shadow rendering, restoring test debug globals
before each test completes. They contain no real account identifiers or secrets.
The first inspection caught a test-only solid-shadow artifact and homepage
feedback overlapping navigation; both were corrected and captures rerun.

Independent review identified stale errors leaking into device-detail shells,
a success message on unsupported-only homes, and a loose action layout. Each
was fixed; navigation and no-read success regressions were added. Unknown write
receipts and elevated confirmations retain their existing semantics and never
receive a retry shortcut. Silent automatic reads remain silent.

## Reproduce the optional captures

From `apps/mobile`, with the pinned SDK and built native FFI available:

```sh
flutter test test/feedback_capture_test.dart test/refresh_feedback_test.dart \
  --dart-define=HOMEVERB_ACCOUNT_CAPTURE=build/verification/feedback-v2 \
  --dart-define=HOMEVERB_CJK_FONT=/path/to/NotoSansCJK-Regular.ttc \
  --reporter expanded
```

Place `NotoSansCJK-Bold.ttc` beside the regular font, or provide the explicit bold
font parameter supported by the account capture test. Captures are ignored build
artifacts, not application assets. This evidence does not establish Android/iOS
physical-phone, accessibility-service or appliance compatibility.

See the [shared feedback contract](../design/feedback-surfaces.md) before adding
future platform-specific account flows.
