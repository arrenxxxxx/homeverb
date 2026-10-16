# Window Light visual system and Flutter handoff

Status: selected by the owner on October 15, 2026. This is the default and **only included skin**. The complete [prototype](../output/prototypes/homeverb-window-light-v1/index.html) applies it to all 82 specified screen states. See the [functional handoff](ui-functional-spec.md) for behaviors and the [screen inventory](ui-screen-inventory.md) for branches.

## Approved visual character

Use the approved [01 / Window Light study](../output/prototypes/homeverb-window-light-v1/assets/approved-window-light.png): warm plaster illumination, diffuse diagonal window shadows, cool reflected light, softly tinted porcelain surfaces, a charcoal active device card, and terracotta action controls. Depth comes from directional light, surface tone and restrained contact shadows. Avoid flat single-color page backgrounds, harsh black drop shadows, broad glossy effects or a different accent on every card.

Apply the same background and material tokens to Devices, Platforms, Settings, device controls, forms, empty/error states, sharing flows, sheets and dialogs. System/browser/permission/secure-storage/export surfaces remain owned by their providers or operating system. The atlas's illustrative provider consent page does not promise that HomeVerb can reskin a third-party website.

### Owner-directed AC composition revision

On October 16 the owner requested a Meiju-style AC panel and eventual feature
parity for the selected appliance. The live climate-and-power page now uses a
pale blue-to-white background, white rounded control cards and a blue interaction
accent within the existing skin. Its original vector appliance illustration is
decorative; power remains a separately labelled observation. This supersedes the
charcoal climate card for the live panel, not the historical prototype or other
screens. No second selectable skin is introduced.

The [official public reference](https://www.smartmidea.net/projects/downloads-app/image/change/a66.png)
informs the temperature, environmental reading and grouped control hierarchy.
It is a marketing example, not evidence of the selected model's current app
functions. See the [AC parity record](development/10-meiju-ac-parity.md) for
implementation gaps and the exact-model reference still needed.

## Tokens and layout

| Token | Initial value / use |
|---|---|
| Skin ID | `window_light`; user-facing name Window Light |
| Background | Bundled [window-light-background.png](../output/prototypes/homeverb-window-light-v1/assets/window-light-background.png), deterministic warm/cool light field with low-amplitude texture and blurred window shadows |
| Main text | `#292B2D`; use strong text on pale surfaces |
| Supporting text | `#626566`; validate contrast over the final background |
| Primary accent | `#B94E29`; terracotta buttons, selected tabs and key actions |
| Light surface | Gradient `#FDFAF3` to `#EEEFEB`; subtle white edge |
| Active surface | Gradient `#42413E` to `#2B2E2F`; light text `#FFFAF1` |
| Navigation surface | `#F6F2EA`; three destinations only |
| Semantic status | Success, warning, error and neutral have dedicated text/icon/background pairs; device outcomes retain their labels under every skin |
| Radius | Device/surface cards 22 logical pixels; compact groups 18; buttons 15; sheets 28 |
| Spacing | Page inset 20; card gap 12; group gap 12–16; keep generous space around the primary control |
| Typography | Platform-native UI font or a properly licensed bundled font; typical body 14–16, supporting 12–13, section title 24–30. Prototype uses local Inter for review. Do not ship a font without checking its license. |
| Interaction | At least 48 logical pixels for interactive targets, even if the visible icon is smaller. Card power, menu and card navigation have independent hit targets and semantics. |
| Adaptation | Two-column cards when content fits; one column for narrow widths/large text; wide rail/grid and optional master/detail layout. Breakpoints derive from usable width and text metrics, not device names. |

The raster prototype uses logical 390 × 844 phone layouts and 1024 × 768 wide layouts rendered at 2×. These dimensions are reference compositions, not fixed Flutter canvas sizes. Do not render the screenshots as the production interface: all text, controls, focus and accessibility semantics must be native widgets. Product cutouts and the decorative background can be image assets.

Button backgrounds may use the small generated Window Light corner assets with
nine-slice stretching. These contain only fill/border pixels, never labels or
page content. Keep the native FilledButton, focus, semantics, state feedback and
48-logical-pixel minimum target. At 2x, the reference's terminal rectangle edges
are inclusive; preserve that one-physical-pixel paint coverage without changing
layout size. High contrast uses the native solid outlined fallback.

## Control alignment revision — October 16, 2026

The owner requested vertical centering throughout the prototype, explicitly
including but not limited to C01–C10. Center labels and values within row content
areas, button labels within button faces, input text within fields, and option
labels within their selectable surfaces. Center wrapped label/value pairs as one
group. Card section labels, slider headings, pairing codes and appearance labels
follow the same rule. Titles, descriptions and status paragraphs retain their
normal document flow.

The prototype measures visible text bounds rather than applying a fixed top
inset. Flutter uses bundled-font ink metrics and centered native control
containers to align visible glyph bounds; unknown glyphs and system bold/spacing
overrides retain native fallback layout. Body copy retains its separate layout. Stepper symbols are centered vector marks in both implementations. A
32-pixel visible pill retains a 48-pixel interaction area, including wrapped runs.
The change does not alter actions, state or native permissions.

Explanatory paragraphs reserve their specified 1.4 line-height multiple using
the actual wrapped line count and system text scale. Keep fractional logical
heights: three 13-point lines occupy 54.6 logical pixels, not 54. This prevents
small rounding errors from shifting later controls. Native text and its semantics
remain intact; this rule does not replace the separately centered control text
or change the existing title layout.


All 82 raster references and 21 boards were regenerated for this explicit design
revision. This is not automatic acceptance of Flutter differences. The strict
comparison remains a separate gate in the [verification report](../apps/mobile/verification.md).

## Flutter feasibility and rendering strategy

The table records the original rendering strategy. The Flutter app now has
Linux and Android emulator evidence; its current surface-mask experiment and
physical-phone performance remain unaccepted. See the [checkpoint](../apps/mobile/README.md).

| Visual effect | Flutter implementation | Acceptance concern |
|---|---|---|
| Window shadows, subtle texture, reflected ambient light | Bundled image in a full-page `Stack`, `Positioned.fill`, `Image.asset` with `BoxFit.cover`; use `Alignment` to preserve the light direction across sizes | Keep decoration out of semantics and pointer hit testing. Decode/cache a suitable resolution; avoid visible tiling or oversized decoded textures. |
| Porcelain and charcoal surfaces | Native gradient/shadow plus committed decorative corner/border masks; high contrast retains the solid native fallback | Explicitly clip child imagery only where needed. Decoration radius does not automatically clip descendants. |
| Soft contact shadow | One low-opacity `BoxShadow`, modest blur/spread/offset | Avoid per-card live backdrop blur and deep nested opacity layers. Profile on a lower-end phone. |
| Selected controls and navigation | Material 3 buttons/navigation widgets with custom color, radius and elevation tokens | Keep focus/pressed/disabled/error states and minimum target sizes; do not replace interactive semantics with painted icons. |
| Sheets/dialogs | Transparent outer route and a rounded themed surface, safe insets and keyboard-aware scroll content | Verify background dimming, barrier labels, back behavior, input visibility and large text. |
| Dynamic capability forms | Stateful native switches, sliders, inputs and selection controls built from descriptors | Preserve observed/requested distinction and per-component limits; skin affects presentation only. |
| Wide layout and accessibility | `LayoutBuilder`, `MediaQuery`, `SafeArea`, lazy grids/lists and a rail at sufficient width | Reflow with text scaling; wrap long names, preserve scroll position and focus. Do not cap text scale as a layout shortcut. |
| Extensible materials | `ThemeData` plus `ThemeExtension<HomeVerbSkin>`; implement `copyWith` and `lerp` | Keep all surfaces on resolved semantic tokens. Animation is optional and honors reduced motion. |

Official sources checked for this design:

- [BoxDecoration](https://api.flutter.dev/flutter/painting/BoxDecoration-class.html): colors, gradients, images, borders, rounded corners and shadows.
- [ThemeExtension](https://api.flutter.dev/flutter/material/ThemeExtension-class.html): custom theme fields and interpolation.
- [Adaptive and responsive design](https://docs.flutter.dev/ui/adaptive-responsive/general): layouts that respond to available space and input.
- [Performance best practices](https://docs.flutter.dev/perf/best-practices): avoid excessive offscreen layers, opacity/clipping costs; build lists/grids lazily and profile the result.

The earlier [isolated Flutter reference](../output/prototypes/homeverb-window-light-v1/flutter-reference/README.md) demonstrates the theme using fictional data. The subsequent [Flutter application](../apps/mobile/README.md) pins its SDK through FVM, implements all 82 isolated preview states, and integrates the Linux Mock Core. Compilation and Linux process tests are recorded in the [verification report](../apps/mobile/verification.md). Strict screenshot equality remains failed; physical-phone screenshots and frame-time measurements are unverified.

## One-skin extension contract

The settings entry ships now in the design: Settings > Appearance > Skin. Show one Window Light tile, marked **Current**, with Preview. Preview returns without changing data. Do not add disabled skin thumbnails or a dark/system picker. Text follows OS accessibility settings independently of skin selection.

The presentation-layer registry has this shape:

```json
{
  "schema_version": 1,
  "default_skin_id": "window_light",
  "available_skin_ids": ["window_light"]
}
```

Use a stable skin definition containing display name/localization key, light/dark surface roles where explicitly designed, background asset, semantic text/accent/status colors, card gradients, borders, radii and shadows. Current Window Light uses a single light appearance with charcoal cards; a charcoal card is not an application dark mode. Keep device behavior and capability metadata outside the skin object.

Persist only the selected ID and preference schema version in local application preferences, not in credentials or an adapter. Missing, corrupt, or unknown IDs fall back to `window_light` and leave account/device/room data intact. A failed save retains the previous effective selection and shows a local error. Resolve the skin before presenting the application to avoid an unthemed flash. The Flutter app implements private local preference persistence and fallback. The earlier isolated reference uses in-memory state only; it is not the application or its persistence evidence.

Later reviewed skins extend the registry and asset bundle, reuse the same components and pass all contrast/accessibility checks. A selection can update theme tokens without rebuilding navigation or reauthorizing platforms. No backend, executable download, credential access, wallpaper importer or marketplace is part of this extension point. Additional appearance modes require their own designed and reviewed tokens rather than automatic color inversion.

## Validation gates

1. **Prototype completeness:** all 82 IDs rendered in Window Light; all 201 branches resolve; all 14 review journeys remain available. Appearance contains exactly one registered option.
2. **Flutter compile and interaction:** use FVM, run dependency resolution and analysis, then build the preview and production UI. Linux and isolated web-preview checks are recorded in the application verification report. Android APK/emulator and selected OS-keyboard/export checks have passed; the latest surface checkpoint, physical phones, iOS app integration and full accessibility still need mobile acceptance.
3. **Visual parity:** compare screenshots on narrow/regular phones and wide layouts against the approved home and representative control/platform/settings/share pages. Tune image fit and material tokens together.
4. **Accessibility:** verify 100%, 150%, 200% text scaling, screen readers, long names, zh-CN/en-US, focus order, 48-pixel targets and contrast over the final composite background. Use an opaque readable surface when enhanced contrast requires it.
5. **Performance:** use a profile build on representative physical phones; measure raster/UI frame time, memory and scrolling after images are decoded. Avoid claiming a frame-rate target from static artwork or desktop rendering.
6. **Native behavior:** independently validate Core embedding, foreground cancellation, secure storage and real-device compatibility. Visual parity is not proof of those capabilities.
