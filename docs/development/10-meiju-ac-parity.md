# Meiju AC parity review

Status: in progress. The owner requested complete functional and interaction
parity with the selected AC's Meiju page. The Flutter panel now includes profile-scoped native scalar controls under
ADR-0069. Full official-App parity and physical feature acceptance remain open.

## Reference and evidence

The [official public AC example](https://www.smartmidea.net/projects/downloads-app/image/change/a66.png)
was viewed in a browser on October 16, 2026. It groups an appliance illustration,
environmental readings, target adjustment, primary selectors and additional
functions. The earlier HomeVerb composition used original vector artwork; the
current row layout uses native widgets without the appliance illustration. It neither
embeds that screenshot nor invents outdoor temperature or humidity readings.

The October 17 owner-approved [Window Light composition](../ui-window-light.md#owner-directed-ac-composition-revisions)
supersedes the blue/white presentation: the live Flutter view now uses expanded,
capability-dependent rows and the shared application materials. It does not turn
the illustrative controls in the original study into model support.

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
| Swing / fixed positions | Each axis independently shows actual swing, exact positions, both or neither; unsupported rows are omitted | Native profile-scoped actions and exact startup grants now encode each supported axis/position, preserving other swing axes from prerequisite reads. Fictional Rust and widget evidence; physical acceptance pending |
| ECO, comfort saving, boost, anti-draft, display, sound, auxiliary heat | Independent profile-scoped native actions and exact scalar grants; reported booleans use switches | Paired ECO fields, numeric/string display formats, anti-draft polarity and reported-buzzer preservation/absent-preference omission have fictional tests; physical behavior pending |
| Breezeless | Separate exact 0/1/2/3 choices, distinct from anti-draft | Only admitted profiles with actual state expose off/both/upper/lower; no blanket brand support |
| Sleep | Active product-specific energy-save pairs are integrated | Ordinary wall sleep presets are commented out in the pinned source; those remain unavailable pending reviewed evidence |
| Dry after shutdown, follow airflow, odor removal, fresh-air/humidity/purification settings | Independent scalar rows for the reviewed selected profile and reported field | Follow-airflow enable field and product-specific domains are preserved; conditional/coupled functions below remain excluded; physical acceptance pending |
| Timer / self-clean | No operational controls | Need selected-model app and protocol evidence, risk classification and native device execution contract; no project background scheduler |
| Indoor temperature | Real observation shown separately from target | Existing stale cloud observation semantics retained |
| Outdoor temperature / humidity / energy | No invented readings | Require protected model mappings, bounded reads and evidence |

The earlier Function support sheet and disabled direction placeholder are removed
from the live page. Missing integration is not a claim that appliance hardware
lacks the feature. Unconnected functions cannot send commands. The native scalar mappings below now supply actual candidate actions. Missing
protocol integration still must not be labelled absent hardware.

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
[verification record](../../apps/mobile/verification.md). The earlier presentation-only phase changed no native permissions. The narrow
temperature correction is documented in [ADR-0066](../adr/0066-optional-wall-ac-subtype.md).
The subsequent native optional-feature integration uses API 1.23 while retaining
method/symbol counts, SQLite 5 and protected payload 3.


## Reviewed native scalar catalog (October 17)

[ADR-0069](../adr/0069-profile-scoped-ac-features.md) retains the four exact AC
models and selects the optional catalog from protected subtype/product/category.
Portable subtype 524 takes priority. Explicit product overrides never inherit
wall-only functions; excluded coupled/centralized profiles remain excluded.
Candidate option inspection is local and does not prove live field availability.
The App hides properties flagged `control_requires_observation` until reported.
A command independently rechecks the private profile, membership, exact permission
and prerequisite state, then sends once. Other supported swing axes must be reported
in that operation. A valid reported buzzer preference is preserved; an absent one
is omitted, never filled from the upstream client default or a saved snapshot.
Concurrent vendor-client changes can race these reads. Normal state uses the switch
or selected choice directly, without a redundant observation caption; unknown
values retain an explicit indication.

The checked-in [catalog](../../crates/midea/src/ac_features.json) records scalar
protocol facts from the pinned public mapping above. Reproduce it by downloading
that exact revision outside the repository, then running
`python3 scripts/extract_ac_feature_facts.py /path/to/T0xAC.py`. The extractor checks
the source SHA-256 and parses Python AST literals; it never imports/executes the
upstream module. HomeVerb contains its own validation, permission, lifecycle and
execution code. Source facts are integration evidence, not verified firmware
support. Public source provenance remains subject to the personal MVP boundary
in ADR-0022; no confidential app secrets or downloaded runtime plugins are used.

The ordinary wall profile provides ECO, comfort saving, boost, horizontal/vertical
swing, each axis's discrete direction codes, breezeless levels, display, buzzer,
anti-draft, overcooling protection, shutdown drying, auxiliary heat, body-following
airflow and odor removal. Product overrides can omit these or add lower-outlet
swing, an active sleep pair, fresh-air speed/automatic/CO2 controls, electrostatic
dust removal, manual humidity/target or purifier PM threshold. The catalog and
exact action schemas, not this union, determine any selected device's options.
Read-only optional groups are skipped when their relevant fields already exist;
unmapped environmental/energy query groups are not requested by this integration.
Native startup confines enrichment to devices in its exact feature grants.

Still blocked on additional reviewed selected-model evidence: ordinary wall sleep,
self-clean/disinfection, timers, conditional exhaust/odor/left-right outlet pairs,
conditional fresh-air mode, coupled intake/exhaust selectors, kitchen cooking
helpers and the unidentified `remove_arofene` field. These are not claimed as
implemented or absent hardware. The owner's selected official-App function list
and authorized physical validation remain required to establish complete parity.


## October 22 exact-profile correction

The supplied official model configuration supersedes the broad category union
for the four exact products. See the [reference integration](13-meiju-reference-integration.md)
for the candidate matrix, new independent cool-energy-saving/strong-cooling/light-
sensing settings and unresolved coupled controls. The historical scalar union
above is no longer the feature list of each known exact product. Full parity and
physical optional-control acceptance remain incomplete.
