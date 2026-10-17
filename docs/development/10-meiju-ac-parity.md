# Meiju AC parity review

Status: in progress. The owner requested complete functional and interaction
parity with the selected AC's Meiju page. The redesigned Flutter panel is only
the presentation phase; it does not complete that request.

## Reference and evidence

The [official public AC example](https://www.smartmidea.net/projects/downloads-app/image/change/a66.png)
was viewed in a browser on October 16, 2026. It groups an appliance illustration,
environmental readings, target adjustment, primary selectors and additional
functions. The earlier HomeVerb composition used original vector artwork; the
current row layout uses native widgets without the appliance illustration. It neither
embeds that screenshot nor invents outdoor temperature or humidity readings.

The October 17 owner-approved [Window Light proposal](../../output/imagegen/homeverb-ac-detail-window-light-v2/README.md)
supersedes the blue/white presentation: the live Flutter view now uses expanded,
capability-dependent rows and the shared application materials. It does not turn
the illustrative controls in that board into model support.

This reference is not the selected device's live Meiju page. Its exact current
screens, expanded function drawer, selector options and app version are still
needed to define and verify one-to-one parity. Do not use a generic marketing
image as a complete model-specific feature inventory.

The existing pinned [AC protocol mapping](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/device_mapping/T0xAC.py)
contains materially different temperature, swing, display and comfort mappings.
For example, the portable subtype lacks horizontal swing, display writes can be
numeric or string-valued, and some profiles require preserving additional fields.
These are integration evidence, not proof that the selected appliance supports
every mapping. Preserve the profile-selection rules in ADR-0040 and ADR-0041.

## Current comparison

| Function | HomeVerb implementation | Selected AC evidence / remaining work |
|---|---|---|
| Power | Direct card, in-place receipt and one bounded follow-up read | Earlier owner-observed shutdown; this redesign sends no new physical write |
| Mode | Expanded localized choices with explicit power-on effect and exact local grant checks | Existing exact-model actions; individual modes not physically verified |
| Fan | Expanded reviewed presets or descriptor-bounded numeric stepper; preserves unknown observed codes | Current Meiju codes are presets, not a percentage or an inferred ordered scale. Earlier code 101 has no reviewed label; later temperature acceptance read the reviewed automatic value. Do not relabel unknown codes |
| Temperature | Descriptor-bounded plus/minus and slider; one command on gesture release | ADR-0066 resolves the optional-subtype admission defect. Selected AC accepted 28 to 27.5 to 28 Celsius with separate matching cloud observations; physical temperature behavior remains unverified |
| Swing / fixed positions | Each axis independently shows actual swing, exact positions, both or neither; unsupported rows are omitted | Current Meiju adapter has no swing/position action or corresponding startup grants. UI fallback and exact commands have fictional widget evidence only |
| Anti-draft, sleep, energy saving, boost, display, button sound, auxiliary heat | Independent descriptor-driven rows when actual low-risk setters and permission exist; known booleans use compact switches, unknown values use explicit choices | Meiju action encoding, model admission, exact grants and physical behavior remain unimplemented; these controls stay hidden for current Meiju devices |
| Timer / self-clean | No operational controls | Need selected-model app and protocol evidence, risk classification and native device execution contract; no project background scheduler |
| Indoor temperature | Real observation shown separately from target | Existing stale cloud observation semantics retained |
| Outdoor temperature / humidity / energy | No invented readings | Require protected model mappings, bounded reads and evidence |

The earlier Function support sheet and disabled direction placeholder are removed
from the live page. Missing integration is not a claim that appliance hardware
lacks the feature. Unconnected functions cannot send commands. Generic descriptor presentation is
not Meiju support; future adapter work must update the table and compatibility
record with actual tests before enabling the corresponding controls.

## Presentation acceptance

Temperature bounds and steps come exclusively from the action descriptor. Coupled
required fields, missing bounds and unsupported risk/idempotency omit the slider.
Dragging changes only a local draft; release sends one validated target. Leaving
the foreground invalidates the draft and late release, including after resume.
Replies do not turn the requested value into an observation. Inline mode/preset
selection and setting switches send one exact command through Core planning and
execution; operations remain on the appliance page. Old callbacks across navigation,
component changes or background/resume send nothing. Native session grants are
inspected locally on opening; no new polling, account login or control grant is
introduced by that inspection. Read-only reopenings hide stale saved actions.

The English/Simplified Chinese catalogs share the existing slang generation path.
Large text wraps row controls. Native controls retain focus and semantics;
high contrast retains outlined surfaces. The historical 82-state prototype is
unchanged and no longer a pixel-parity target for the owner-revised live AC panel.

APK identity, checks and emulator captures belong in the app's
[verification record](../../apps/mobile/verification.md). The presentation phase
changes no Rust ABI, native grants, storage, adapter model admission or transport
behavior. The subsequent narrow temperature-admission correction is documented
in [ADR-0066](../adr/0066-optional-wall-ac-subtype.md); it does not implement the
remaining Meiju functions above.
