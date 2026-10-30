# Device interaction verification

This standalone report contains only the current synthetic UI work. The existing
`apps/mobile/verification.md` historical record remains byte-for-byte unchanged.
Run Flutter commands from `apps/mobile`. No historical account, authentication,
emulator or production-operation record is copied into this report.

### Device interaction clarity -- October 30, 2026

The reusable interaction contract is recorded in
[`14-device-interaction-design.md`](14-device-interaction-design.md).
AC controls now distinguish power switches, single selections, set temperature,
room readings and secondary settings. Fixed direction requires the corresponding
swing state to be off. A manual Refresh State action restores the existing
read-only recovery path on the specialized AC page. Read-only refrigerator zones
separate set and measured
temperatures, retain missing data and show one connection limitation. Dishwasher
configuration uses the actual scalar additional-option schema, not invented
multi-select combinations; state-specific actions retain exact grants and
trusted review. Raw unreviewed door/supply codes remain unknown. Submitted or
accepted commands do not replace observed state or imply verification.

The work uses synthetic clients and local Mock native execution only. No real
account, appliance, production configuration or deployment was used. The native
API, Core, adapters, model admission and authority are unchanged. Future Hisense
and Xiaomi adapters must first establish equivalent capability/protocol evidence;
this presentation work does not establish their device support.

Four full-content Chinese captures are reproducible through the actual Flutter
test renderer, using production widgets and fictional fixtures:

```sh
fvm flutter test test/device_design_capture_test.dart \
  --dart-define=HOMEVERB_CJK_FONT=/path/to/NotoSansCJKSC-Regular.otf \
  --dart-define=HOMEVERB_CJK_BOLD_FONT=/path/to/NotoSansCJKSC-Bold.otf
```

The opt-in font is not bundled. Without that define, these four artifact tests
are skipped; ordinary behavior tests still run. The output is
`build/verification/device-design/{ac,fridge-readonly,dishwasher-standby,dishwasher-running}.png`.
The screenshot harness asserts no client calls while rendering or selecting
standby radio options, and no trusted approval consumption. These are headless Flutter captures, not browser, phone,
live-device or physical-control acceptance. Missing measured temperatures and
unverified timestamps are deliberately visible rather than filled from targets.

Executed checks on the final presentation changes:

- `cargo build -p homeverb-ffi --locked`: passed, producing the local Mock FFI
  used by Flutter native-client tests (Rust stable 1.99.1 in this workspace).
- `fvm flutter analyze`: no issues.
- Explicit CJK screenshot run: all four captures passed, with visual review.
- Repository Python tooling: 70 tests passed; shared presentation i18n: seven
  tests passed. Documentation, repository hygiene and contract validators passed
  (21 schemas, 52 positive fixtures and 341 negative fixtures).
- Independent read-only safety review found no authority expansion or optimistic
  observation mutation. `git diff --check` passed.

The first full Flutter run exposed stale integration selectors for home-card
power and the platform chooser, and the missing manual AC refresh entry point.
The tests now assert disabled quick power before observation, use the actual
power switch, preserve all receipt/no-replay checks, and verify refresh performs
only a read and rejects an old background callback. No production login flow
or unavailable-platform gate was changed to make these tests pass.

The initial component-only captures did not faithfully match the accepted visual
proposal. The corrected implementation uses the actual `NativeApp` route and a
shared `DevicePage` shell for these appliance pages: centered real device title,
Back navigation, warm white background, compact separate cards and the existing
error/busy handling. The capture harness now renders that exact production path
at 430 by 932 logical pixels, with only a small fictional-data disclosure outside
the app footprint. It also saves optional `*-full.png` artifacts for scrolling
content. No screenshot-only app chrome is substituted.

Independent visual review compared all four final viewports against the approved
reference at equal width, including selected radio styling, right-aligned status
rows, refrigerator columns and conditional swing summaries. The remaining
intentional differences are actual schema choices, unverified/missing data,
access to additional supported options, and existing refresh/details actions.
No online badge or confirmed-operation timeline is fabricated to match artwork.


Typography review found an artifact-path issue: the original capture registered
Noto CJK Regular under the Inter family, changing Latin/numeric glyphs as well as
Chinese. The corrected harness loads the bundled Inter Regular/SemiBold and
separately registers Noto CJK Regular/Bold. `HOMEVERB_CJK_BOLD_FONT` can select the
bold file explicitly; otherwise the harness expects `NotoSansCJK-Bold.ttc` beside
the regular file. It checks Latin/numeric metrics with the fallback chain against
Inter at both weights. The production fallback is shared, not a screenshot-only
font substitution. System CJK on iOS/Android may still differ; this does not claim
physical-phone typography acceptance. Device-specific 16-point body text and
18-point section headings now use the reusable scale without globally changing
other application pages.

The final font inputs use the SC faces extracted without glyph changes from the
installed Noto CJK collections, rather than relying on TTC face zero (JP).
Font metadata confirms Inter weights 400/600 and Noto SC weights 400/700; CJK
semibold requests use the available bold fallback, not a claimed bundled CJK
600 face. These host verification fonts are not added to the application bundle.

Final regression: `fvm flutter test --reporter expanded --concurrency=1` passed
194 tests, with the five explicit font/capture checks skipped without their font
defines. During the typography pass, the balanced six-choice fan layout and
24-test AC subset
passed, including 3-by-2 geometry, 16-point labels, enlarged text and stale
callback guards. The complete suite also renders all 82 catalog states and
exercises 201 simulated navigation branches. No skipped font artifact check is
counted as a behavioral-test pass.
The final explicit SC-font run passed all five font/capture checks (font-family
and weight metrics plus the four production-page screenshots).
The bounded final typography refinement uses 18-point cycle options, 20-point
primary cycle actions (18-point secondary actions) and 17-point refrigerator
readings. All 19 affected dishwasher/detail/refrigerator tests passed afterward,
including enlarged Chinese text, disabled actions and exact component authority.


The owner subsequently requested a single-row fan selector. This supersedes the
intermediate balanced-grid visual evidence above: fan presets must not wrap;
compact padding or explicit horizontal scrolling preserves all declared values,
readable text, touch targets and observed-selection visibility.
The single-row correction passed all 26 AC tests, followed by three focused
layout/arrow tests. At 430 logical pixels all six declared presets fit without
scroll arrows; narrow or enlarged-text cases retain 16-point labels, at least
44-pixel touch targets and explicit horizontal navigation. Every externally
observed selection is revealed, both scroll ends are reachable, and scrolling
never issues a device command.
