# Device interaction design

This is the reusable interaction contract approved for the HomeVerb device pages
on October 30, 2026. It applies to the current Meiju presentation and to future
Hisense, Mijia/Xiaomi and other provider adapters. It does not admit a provider,
model, command or new authority. The domain capability and native UI contracts
remain authoritative.

## Five distinct visual roles

| Meaning | Presentation | Interaction |
| --- | --- | --- |
| Reported state | Plain label and value in a flat reading group | No button elevation, switch, chevron or tap handler |
| Exactly one choice | Segmented choices or radio rows, with a visible selection mark | One value from the exact descriptor; no implied execution success |
| Independent multiple choices | Checkboxes in a labelled group | Only for a descriptor that actually supports independent combinations |
| Independent binary setting | A labelled switch | Only for an admitted boolean action; unknown is not off |
| Execute an action | A verb-labelled button | Explicit command boundary, with trusted approval when required |

An enum of numeric codes is still **single choice**, not a bitmask. In particular,
the current Meiju dishwasher `additional` argument is a single scalar enum; the
visual proposal's multiple-checkbox illustration is not a claim of protocol
support. The implemented panel must show a single choice. Do not infer safe
combinations, ranges, units or action semantics from the artwork or brand name.

## Page hierarchy

### Air conditioner

Keep the power switch and target temperature prominent. Label the target
separately from the observed room temperature. Mode and fan choices show their
selected state without relying on color alone. Fan presets stay in a single
horizontal row: compact padding may fit all values at normal width; otherwise
provide discoverable horizontal scrolling with full-size touch targets and keep
the observed selection visible. Do not wrap fan presets or shrink their font.
Display a numeric speed as a
level, not an unexplained number. Group swing settings; a corresponding fixed
position cannot be selected while swing is on or its state is unknown. Lower
frequency settings belong in an expandable section. Descriptor bounds, exact
value grants and current observations still determine availability.

### Refrigerator and other read-only multi-zone devices

State the read-only limitation once at the top when no admitted controls exist.
Use flat zone and feature rows. Distinguish **set temperature** from **current
temperature**. Missing observations are "Not received", never an invented value
or a reused setpoint. Preserve actual per-component errors and underlying cache
metadata. Do not add repeated freshness captions to every feature. Refresh and
device-information navigation are real actions and remain visually distinct.

A future writable compartment must use the existing exact component-scoped
validation and authority. A generic Mock refrigerator with admitted writes must
not be made read-only merely because the real Meiju refrigerator is read-only.

### Dishwasher and cycle devices

Show the current observed program, phase and remaining time before controls.
Standby configuration is separate from the current running program. Expose only
the applicable primary action: start when idle, pause while running, resume when
paused. Cancel is secondary and retains trusted confirmation. Unknown state must
not be presented as a known idle state. Provider readings without a measurement timestamp remain visibly unverified; the existing Meiju
adapter marks even newly received reads stale, so this UI does not add an
arbitrary freshness gate or claim that Refresh makes them verified. Do not imply
that selecting a program starts it. While running, lock settings that the current
protocol cannot change and explain why. Existing exact argument forms and immutable trusted
approval previews remain the final execution boundary.

## Visual layout contract

The accepted proposal is a visual target, not just a list of control meanings.
Use the real shared device-page shell: warm white background, centered device
name, real Back navigation, roughly 16 logical pixels of horizontal inset, and
10–12 pixel gaps. Do not replace it with screenshot-only chrome. Never add an
online badge without reliable connection evidence.

At a normal 430-pixel phone width, keep the AC hero near 190–210 pixels tall,
with a 64-pixel target and 56-pixel step controls. Mode and fan each have their
own compact white card; selection type sits beside the heading. Swing switches
share a row when space permits; fixed-direction controls and the fine slider
belong under More settings. Preserve a stacked layout for enlarged text.

Read-only refrigerator temperature zones use a three-column table (zone, set,
current), not three repeated large cards. Group function readings together and
door/reminder readings separately. Extra protocol details remain accessible in
an expansion. The refresh action is orange outlined, distinct from plain state.

Dishwasher standby foregrounds three supported primary programs and two options;
all remaining declared choices remain available under More programs/options.
Do not silently discard extra descriptor values. Standby does not need the dark
running hero. Running uses a compact dark status card, a white reading group,
orange primary action, orange outlined cancellation and a grey locked-settings
notice. Low-frequency readings belong in secondary details. Actual pending or
receipt evidence may extend the page; never fabricate the proposal's timeline.

Acceptance compares actual production-page renders at the same viewport to the
approved reference. Inspect card grouping, density, typography, spacing and
hierarchy, not merely absence of overflows. Screenshots must not conceal a
production-layout difference behind a custom preview wrapper.

## Command feedback and lifecycle

- Reported values are evidence; selected/draft values are requests.
- Sending, accepted/waiting for confirmation, verified, failed and unknown are
  different outcomes. HTTP success or a tapped switch is not verification.
- Keep the old observation until a new observation arrives. Do not overwrite it
  optimistically, auto-replay uncertain writes or silently fall back to a route.
- Busy, background, navigation and changed device identity invalidate local
  selection/confirmation as defined by the existing controller contract.
- Unknown provider codes remain unknown. Localized labels require an exact
  reviewed mapping; never guess whether `0` or `1` means an open door or low salt.

## Accessibility and localization

Use native control semantics, keyboard focus and at least normal Material touch
targets. Include a check/radio mark as well as color for selected items. Read-only
rows have no control semantics. Long translated labels and enlarged text must
wrap at a 320 logical pixel width. High contrast must retain visible boundaries.
`DeviceTypography` defines the device-only scale: 22 for page titles, 18 for
section headings, 16 for body text, 18 for cycle options, 20 for primary cycle
actions, 17 for refrigerator readings and 13 for secondary labels, with a 1.25
line height for short labels. Paragraphs retain 1.4. The AC target uses 64/600;
refrigerator targets use 24/600. Do not globally enlarge unrelated `Copy` text.
Latin text and numerals use the bundled Inter Regular and SemiBold (600). CJK
uses explicit system fallbacks (Noto Sans CJK SC / PingFang SC); platform glyphs
can differ. Capture tests must load Inter as Inter and CJK as a separate family,
never replace Inter with a CJK font to make missing glyphs disappear.
All authored translations live in the canonical presentation catalogs; public
IDs and exact command argument values are never translated.

## Adapter onboarding checklist

Before applying this design to Hisense, Xiaomi or another platform:

1. Map device/component identity and typed properties/actions in the adapter.
2. Record exact-model read, write, enum, combination and risk evidence.
3. Classify each field into the five visual roles above from those semantics.
4. Keep unsupported fields read-only or absent; do not copy Meiju wire codes.
5. Bind existing foreground, permission, plan/execute and trusted approval paths.
6. Test missing/stale/unknown values, denied grants, pending/failed/uncertain
   outcomes, lifecycle interruption and device switching with fictional clients.
7. Exercise state-specific actions, single/multiple selections, narrow enlarged
   text, both locales and actual Flutter-rendered screenshots.
8. Record synthetic evidence separately from physical-device acceptance. Obtain
   owner authorization before any real login, appliance write or publication.

## Verification scope

The implementation changes presentation and local selection only; it does not
expand provider admission, startup grants, background work or device authority.
See the [current execution record](device-interaction-verification.md) for the
exact executed checks and limits. The older application verification history is
unchanged.
