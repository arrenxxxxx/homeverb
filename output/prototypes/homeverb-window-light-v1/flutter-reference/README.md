# Flutter Window Light visual reference

An isolated widget reference for the approved material system: bundled ambient/window-light background, porcelain/charcoal gradients, shadows, responsive cards, three root destinations and Appearance with exactly one included skin. All data is fictional. It makes no network/Core calls and holds no credentials. The full [82-state atlas](../index.html) remains the product design specification; this reference implements only representative visual components and local navigation/filtering.

## Verification boundary

Flutter SDK is unavailable in the authoring environment. This source has **not** been compiled, analyzed, run on a phone, or performance-tested. The supplied images were locally rendered by Python, not captured from Flutter. Official painting/theme documentation supports the implementation approach; actual widget and native integration acceptance remains required.

## Run in a Flutter development environment

The directory deliberately contains only the widget source, SDK-only package definition and bundled assets. Generate host scaffolding in a disposable copy so the handoff remains unchanged:

```bash
cp -R flutter-reference /tmp/homeverb-window-light-reference
cd /tmp/homeverb-window-light-reference
flutter create --platforms=android,ios,web --project-name homeverb_window_light_reference .
flutter pub get
flutter analyze lib
flutter run -d chrome
```

Use a supported configured device instead of Chrome for phone acceptance. These are suggested commands, not executed results. Host scaffold generation may add Flutter's stock test; it is not a test of this reference. Review generated files before integrating into a real app.

## Skin extension point

`HomeVerbSkin` extends `ThemeExtension`; `skinRegistry` contains only `window_light`. `resolveSkin` falls back to that default for absent/unknown IDs. `ThemeData` carries semantic colors and the extension. `WindowLightBackground` and `Porcelain` consume shared tokens. Future reviewed skins can add registry entries while keeping device behavior unchanged.

Appearance currently shows one Current option and a preview. There is no dark/system-mode picker, download catalog or wallpaper importer. Production local preference persistence and Core integration are specified in [the visual handoff](../../../../docs/ui-window-light.md), not implemented here. The sample uses in-memory room/navigation state only.

Use native Flutter text and controls; do not embed prototype screenshots as pages. Device cutouts are derived locally from earlier fictional HomeVerb artwork; they are review assets, not verified product illustrations. Background assets are locally generated. Validate licensing/branding before release.

Before production use, run the [UI acceptance checklist](../../../../docs/ui-functional-spec.md), test all text scales and platform lifecycle paths, add genuine widget/interaction checks, and profile actual mobile frames. No physical-device compatibility is established by this reference.
