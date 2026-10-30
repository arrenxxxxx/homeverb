# Selectable Sky Blue skin: October 30, 2026

Local implementation retains Window Light as the default and adds Sky Blue via
`HomeVerbSkin` / `skinRegistry`. Settings > Appearance saves `skin_id` in the
existing presentation preference file. Missing or invalid IDs fall back to the
original skin. Skin selection does not grant permissions or invoke Core.

Executed in the isolated Linux workspace with the repository-pinned Flutter SDK:

- `flutter analyze`: passed with no issues.
- `flutter test`: 210 passed; five opt-in font/capture cases skipped by default.
- `device_design_capture_test.dart` with explicit regular/bold SC font paths:
  five passed for Sky Blue and five passed for Window Light. Actual NativeApp
  Home, Appearance, AC, refrigerator and dishwasher pages use fictional clients;
  image decoding completes before capturing the homepage illustration.
- Skin tests cover restart persistence, unrelated preference saves, unknown and
  malformed IDs, both-direction switching with no Core calls, and repeated
  switching at 320 logical pixels with 200 percent text.
- Documentation/repository validators and the seven presentation i18n tests:
  passed. Contract validation with the installed development dependencies:
  21 schemas, 52 positive and 341 negative fixtures passed; Python tests: 70 passed.
- Final focused appliance controls, device-detail controls and skin selection
  regression: 14 passed after the last color/contrast corrections.
- `git diff --check`: passed.

Recovery checks initially exposed the Appearance ListTile's missing local Material,
a deprecated semantics test's bool/Tristate mismatch, and a narrow enlarged-text
home heading overflow. All were corrected and retested; failed intermediate runs
are not counted as passes. This is local UI evidence, not an APK build, physical
phone test or appliance acceptance. No account, provider or real device was used.
Publication remains separate from the owner's preview acceptance; this record does
not assert that remote CI, including the earlier independent iOS job, passed.

