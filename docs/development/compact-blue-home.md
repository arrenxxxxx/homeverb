# Compact Sky Blue device home

The November 9 [interaction handoff](15-interaction-experience-handoff.md)
specifies a pending replacement of product artwork with the approved outlined
icons across both skins and all home-card variants. The layout and verification
below describe the earlier implementation; the separate November 9 missing-power
appearance fallback remains required.

The October 31 owner phone review showed that the Sky Blue homepage retained the
original tall device-card composition. A large product image, separate room line,
and bottom power row allowed only two complete rows on the reviewed phone. This
revision is a skin-specific presentation change, not a new device capability.

## Reusable skin layout

`HomeVerbSkin.compactDeviceCards` selects compact device cards and homepage
insets. Window Light retains its existing card layout. Sky Blue uses a small
product image and a 48-logical-pixel power target in the top row, followed by the
device name, room, existing summary reading and any existing limitation caption.
Cards have a minimum height, rather than a clipping fixed height. Enlarged text
uses the existing single-column fallback and grows naturally.

A visible menu target is separate from the power target; neither overlays the
other. Tapping the rest of the card still opens the device, and long press still
opens its menu. Pending, unknown and denied power states retain the existing
presentation metadata, disabled behavior and command path. No temperature,
permission, freshness, account or appliance state is invented by the skin.

The skin owns pale mint active and blue-gray inactive button surfaces with dark
icons. State remains available through icon shape and accessible semantics, not
just color. High contrast retains black/white controls and card outlines.
Inactive cards use the skin's cool translucent surface instead of the old
hard-coded warm gray. The background uses code-rendered overlapping blue light
fields; no vendor screenshot or private account content is embedded.

## Preview and verification boundary

Comparison images render production `ScreenView`, `DeviceHome` and `DeviceGrid`
with fictional devices at identical viewports. They are UI renders, not physical
phone or provider acceptance. The uploaded owner screenshot is reference input
only and must not be published in the repository or included in preview assets.
Publication, APK generation and real-device actions are separate tasks.

## November 9 power presentation

At the owner's request, home cards render missing power observations with the
same off icon, colors, tooltip and accessibility state as an off observation.
There is no third power appearance or added unknown-state caption. This display
fallback does not change the nullable Core observation, shortcut eligibility,
command arguments or execution receipts. A shortcut that was disabled remains
disabled until the existing control conditions are met.

Local verification passed: all eleven compact-card cases, including four captures
across both skins; Flutter analysis; documentation, repository and contract
validators; and diff checks. The updated captures use fictional data. Physical
phone verification remains blocked pending selection and setup of the owner's
test phone; no APK or iOS package was built for this change.

## Executed checks: October 31, 2026

- Flutter 3.47.7 analyze: no issues. Full Flutter suite: 233 passed and 17
  opt-in screenshot cases skipped by default.
- Compact-card suite with explicit Inter and separate Noto Sans CJK SC fonts:
  seven regressions and four actual homepage captures passed. Coverage includes
  enabled/off/unknown/denied/pending power, separate open/menu/power hit regions,
  independent disabled accessibility nodes, pale icon contrast, and 320-pixel
  double-text-scale/high-contrast reflow.
- Identical fictional 360 by 800 viewports: card height 233 to 154 logical pixels;
  four to six completely visible cards. New card width is 161 pixels, row gap 10,
  third-row bottom 681, and navigation begins at 702. The normal-text target does
  not cap enlarged-text height or discard observations.
- Window Light baseline and new captures at both 360 by 800 and 430 by 956 are
  byte-identical. Blue navigation sampled small-label contrast is 4.86:1; its
  opacity was increased after review rather than copying an unreadable reference.
- Documentation, repository hygiene, contract fixtures, seven presentation i18n
  tests, 70 Python tests and `git diff --check`: passed. The initial contract check
  lacked the declared date-time validator; installing `requirements-dev.txt` in
  an isolated environment resolved it and the complete check passed.
- Native FFI was rebuilt for local fictional-client tests. No real account,
  appliance, APK, deployment, remote push or physical-phone test was performed.

An intermediate test used a deprecated semantics API; it was replaced with
`flagsCollection`, then focused and full tests were rerun. Neither intermediate
failure is represented as a pass.
