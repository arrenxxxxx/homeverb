# Account connection and foreground refresh experience

The November 9 [interaction handoff](15-interaction-experience-handoff.md)
defines pending follow-up changes to immediate account navigation, centered
Cancel, pull-to-refresh and compact feedback. Its requirements supersede the
conflicting refresh presentation here; the historical checks below do not prove
implementation of that follow-up.

## Scope

The mobile app separates adding a supported platform from viewing locally saved
accounts. Account details group authorization state, device-list synchronization,
and locally associated devices. A saved authorization is not proof that an
account remains authorized or that its appliances are online. Account-list
summaries do not invent device counts; the detail count uses `get_account`
associations, not a reachability estimate.

Only the currently supported Meiju mainland-China connection is offered. The
provider picker does not imply that browser authorization, verification codes,
Xiaomi, or Hisense login is implemented. The original and Sky Blue appearances
share the same behavior and semantic controls.

First login and reauthorization use a provider card, clearly labelled protected
fields, an inline busy indicator, an actionable failure message, and a separate
Cancel action. Reauthorization identifies the selected local account and explains
that names and rooms are preserved. Restoring an identity binding remains a
separate reviewed operation with its existing explicit same-account consent.
Credentials are still cleared when dispatched, canceled, backgrounded, or
disposed, and temporary byte buffers are zeroed. No credential is placed in
ordinary configuration, diagnostics, or screenshots.

Manual homepage refresh keeps existing device cards and readings in view. The
refresh entry shows progress and prevents duplicate requests. A failure offers a
compact retry/dismiss card without exposing provider response text. Navigation
and foreground epochs fence late refresh results and errors. This does not change
the separate silent foreground-read schedule, route policy, permission grants,
write receipts, or cancellation semantics of appliance commands.

## Validation boundary

The regression fixtures use fictional accounts and appliances. Screenshot tests
render actual Flutter pages with the production Inter faces and a separate Noto
CJK fallback, including both appearances. Narrow-screen and enlarged-text tests
exercise the new surfaces. Account removal, credential readiness, identity
recovery, account refresh, and lifecycle suites remain mandatory regressions.

No real provider login, appliance operation, APK release, deployment, or physical
phone acceptance is part of this change. Rust APIs, storage formats, grants, and
the historical mobile verification record remain unchanged.

## Local verification on October 31, 2026

- Flutter 3.47.7: analyze passed; the complete suite passed 226 tests, with
  13 explicitly optional font/capture tests skipped in the ordinary invocation.
- Explicit capture runs produced and inspected 16 actual images across both
  appearances: six account surfaces, eight login/reauthorization states, and
  two homepage refresh failures. They use fictional data and empty credentials.
- Rust 1.96.0 `cargo build -p homeverb-ffi --locked` passed to supply the native
  library for Flutter regressions. This is not a claim of local full Rust-suite
  execution.
- Python 70 tests, presentation i18n 7 tests, documentation links, schemas,
  repository hygiene, and diff checks passed.
- Historical `apps/mobile/verification.md` is byte-for-byte unchanged.

## Reproduce

From `apps/mobile`, with the pinned Flutter toolchain:

```sh
fvm flutter analyze
fvm flutter test --reporter expanded
fvm flutter test test/account_surface_capture_test.dart \
  --dart-define=HOMEVERB_ACCOUNT_CAPTURE=build/verification/account-surfaces \
  --dart-define=HOMEVERB_CJK_FONT=/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc
fvm flutter test test/refresh_feedback_test.dart \
  --dart-define=HOMEVERB_CJK_FONT=/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc
```

The capture tests require the regular and bold Noto CJK fonts in addition to the
repository's Inter assets. See each fixture for its explicit capture opt-in and
output paths. The ordinary suite skips optional font-dependent captures when
those inputs are not supplied; a skip is not visual acceptance.
