# Window Light verification record

Date: October 15, 2026. Scope: visual prototype and English implementation handoff. This record does not establish a production Flutter application or real-device support.

## Visual inspection

All 21 boards were inspected as local images, covering all 82 phone/wide states. Review included normal, empty, permission-denied, expired-authentication, uncertain-write, sharing-failure, confirmation, large-text and wide layouts. Follow-up corrections addressed dialog height after text wrapping, sheet bottom space, device-cutout edges, larger-text power targets, explicit power switches/steppers, the unknown-result freshness label, and the obsolete dark-appearance board subtitle.

The only available skin is Window Light. S02 contains one Current option; S11 previews that skin. All page backgrounds, cards, forms, dialogs and sheets use the shared light/material system. Provider-owned/system screens are illustrative and outside HomeVerb skin control.

## Publication checks

The checks below ran against an isolated export of the exact UI documentation staging area. This export excludes unrelated Core implementation work and historical image-generation directories.

| Command / check | Result |
|---|---|
| `prepare_spec.py`, `render.py`, `build_handoff.py` | Passed in the isolated export: all 82 states, 21 boards, 14 journeys and 198 branches rebuilt using bundled sources. No recorded content overflow. Rebuilt image bytes match the staged images. |
| `python3 output/prototypes/homeverb-window-light-v1/validate.py` | Passed: 41 requirement groups, one skin option, 138 images with dimensions/hashes, 534 local links, matching embedded metadata and complete screen inventory. |
| Bundled Node `--check /tmp/homeverb-window-light-atlas.js` | Passed: gallery JavaScript syntax. This is not a browser interaction test. |
| `python3 scripts/validate_docs.py` | Passed: 53 Markdown files, 664 local links, 20 JSON files. External URLs, anchors, runtime behavior and devices are outside this check. |
| `python3 scripts/validate_repository.py` | Passed: 97 maintained text files satisfy English-language and basic hygiene checks. |
| `python3 scripts/validate_contracts.py` | Passed: 6 schemas, 8 positive fixtures, action arguments and 8 negative fixtures. This validates documentation contracts only. |
| `git diff --cached --check` | Passed: no staged whitespace errors. |
| `python3 scripts/validate_commits.py --message-file <message-file>` | Passed: English Conventional Commit subject with the `docs(ui)` prefix. |
| `command -v flutter`, `command -v dart` | Neither SDK executable is available. |

Rebuild and check commands are listed in [README.md](README.md). The renderer used the bundled Python runtime with Pillow and NumPy; the system Python was sufficient for the handoff builder and integrity checker. Earlier checks of the broader development workspace have a different file/dependency scope and are not the publication result above.

## Publication preparation

The documentation delivery bundles its source inventory, 41-group requirement map, gallery template, approved Window Light study and fictional device artwork. Rebuilding no longer requires historical experiment directories. The Core integration table explicitly describes a separately developed local workspace snapshot; unrelated Core/Rust implementation changes are excluded from this UI documentation delivery. Existing documentation files include only the reviewed UI additions in this commit.

## Unverified boundaries

- Flutter and Dart are absent from PATH. No Flutter analyzer, build, widget test, phone screenshot comparison, physical-device test or performance measurement was executed.
- The isolated Flutter source demonstrates the intended implementation primitives; it is not a compiled deliverable or the complete application.
- The offline atlas has static navigation/data checks. Browser-rendered layout and real click/keyboard behavior were not re-tested for this revision; local-file browser inspection is unavailable in this session.
- Rendering performed no external image-service request/upload, authentication, credential transfer or appliance write. The owner separately authorized the Git documentation publication; repository history records that delivery.
- Existing unrelated workspace changes were preserved. Changes here are the prototype/reference artifacts and their linked UI documentation; no production Core behavior or public contract was modified.


## October 16, 2026 — owner-requested vertical centering

Regenerated all 82 screens, 21 boards, thumbnails and overview images after
replacing fixed text-top offsets in shared controls with vertical centering.
Reviewed C01–C10 and representative room/platform/settings/sharing controls.
The atlas validator checked 138 images, 547 local links, 14 journeys and 198
branches with no layout overflow. Native application interaction results and
remaining pixel differences are tracked in
[the Flutter verification report](../../../apps/mobile/verification.md).

The screen content and action graph remain unchanged. Existing contract
amendments in `docs/ui-screen-inventory.md`, including P08 and P10, were preserved;
regeneration does not roll those requirements back to historical artwork.

## P08 combined account result revision — October 16, 2026

P08 was independently rerendered from the selected native connection/retrieval
contract. Its five visible action branches replace the superseded Find Devices
and Done branches. The base reference and Flutter catalog now have identical
content; there is no second discovery step after sign-in. No screenshots were
copied from Flutter into the reference. The regenerated atlas validates 82
screens, 21 boards, 14 journeys, 201 unique branches, 138 images and 547 links,
with no overflow. The base P08 image was inspected at full resolution.

Flutter's 74-test full suite and static analysis passed. After the native
account-title correction, seven account tests and static analysis passed again. Strict raster comparison remains
failed (0/82 exact; mean channel error 1.3958/255). The base P08 result is aligned
with the contract, but separate busy/error/interrupted reference variants and
physical-phone acceptance remain outstanding. Historical entries above record
the previous 198-branch atlas and are superseded for current counts.
