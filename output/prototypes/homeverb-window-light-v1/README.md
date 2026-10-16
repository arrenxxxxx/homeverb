# HomeVerb — Window Light complete prototype

Owner-selected visual system: **01 / Window Light**, October 15, 2026. This is the current full prototype, superseding the earlier atlas for visual implementation. Historical image-generation studies remain unchanged.

Open [index.html](index.html) to browse **21 boards, 82 screen states, 14 review journeys and 201 action branches**. Search by screen ID or term; open a screen to follow its action branches. Arrow keys move between screens and Escape closes the preview. These controls navigate design images, not appliances. The atlas uses bundled assets and requires no account or external service.

## Review artifacts

- [Six representative pages](key-pages.jpg): Devices, climate controls, Platforms, Settings, Appearance and authorization scope.
- [All-screen overview](overview.jpg), [coverage map](coverage.md) and [asset manifest](assets.json).
- [Functional specification](../../../docs/ui-functional-spec.md), including existing Core contracts versus remaining integration work and acceptance checks.
- [82-state inventory](../../../docs/ui-screen-inventory.md), with visible content and branch destinations.
- [Flutter visual system and skin contract](../../../docs/ui-window-light.md).
- [Isolated Flutter visual reference](flutter-reference/README.md), SDK-only package and local assets.
- [Verification record](review.md).

## Skin scope

Settings > Appearance shows exactly one option, **Window Light**, marked Current, with a preview. The stable ID is `window_light`. The visual/theme extension point is documented now so later reviewed skins can reuse the components. No alternate skin, dark/system selector, custom wallpaper or skin download feature is included. Preview does not change accounts, rooms or device state.

## October 16 alignment revision

At the owner's request, text is vertically centered in rows, buttons, fields,
choices, pills, stepper values and related control bands across the complete atlas.
Wrapped content is centered as a group. Titles and body paragraphs keep their
normal flow. The shared renderer measures text bounds; all 82 screens, 21 boards
and overview images were rebuilt. See the [alignment rule](../../../docs/ui-window-light.md#control-alignment-revision--october-16-2026)
and [Flutter checks](../../../apps/mobile/verification.md).

## P08 account-contract revision

The owner-selected native contract combines sign-in and automatic retrieval.
P08 now depicts its completed result for two fictional cards and five explicit
actions. It no longer asks the user to start a second discovery step. This
independent raster update matches the production result content and brings the
atlas to 201 branches. Busy, empty, partial and interrupted variants still need
separate visual acceptance; see the [P08 inventory](../../../docs/ui-screen-inventory.md)
and [executed Flutter checks](../../../apps/mobile/verification.md).

## Rendering and provenance

All 82 screens are freshly composed locally from declarative screen content. Text, controls, cards, sheets and dialogs are drawn by Pillow. The ambient background is a deterministic NumPy light field based on the approved [Window Light study](assets/approved-window-light.png). Four appliance cutouts reuse existing fictional artwork from [the earlier C direction](assets/source-device-home.png); other icons are locally drawn outlines. No new external image-generation request or asset upload was made for this revision.

Phone layouts use logical 390 × 844 and wide layouts use 1024 × 768, rendered at 2× for legible review. These are not Flutter screenshots. Do not use whole page images as production Flutter UI. Text and controls must remain native, accessible widgets. Product cutouts are review assets and do not establish model support or production licensing.

## Rebuild locally

Requires Python with Pillow and NumPy. From the repository root:

```bash
python3 output/prototypes/homeverb-window-light-v1/prepare_spec.py
python3 output/prototypes/homeverb-window-light-v1/render.py
python3 output/prototypes/homeverb-window-light-v1/build_handoff.py
python3 output/prototypes/homeverb-window-light-v1/validate.py
```

`prepare_spec.py` preserves the approved earlier inventory and defines the content for every screen. `render.py` applies shared visual primitives, exports all panels/boards, and records layout bounds. `build_handoff.py` builds the offline gallery, documentation inventory, coverage, reference assets and hashes. `validate.py` checks integrity and coverage. All source dependencies are bundled: `source-inventory.json`, `requirements.json`, `atlas-template.html`, the approved study and the fictional device artwork. Earlier experiment directories are not required. The renderer uses locally installed Inter, with DejaVu Sans as a fallback; font differences may change text metrics.

## Implementation boundary

The prototype and Flutter reference use fictional devices/accounts/codes. Real platform authentication, sharing, mobile secure stores and real-device compatibility remain separate implementation gates. The original isolated reference was not built in its authoring environment. Subsequent FVM application checks are recorded separately in the [verification report](../../../apps/mobile/verification.md). This raster atlas does not establish physical-phone visual parity, frame-time performance or appliance support.
