# October 22 Meiju reference integration

Status: partially implemented in Core and App; full interface parity is blocked.
This document tracks the owner's complete-integration request, including gaps.
It does not redefine completion as importing a route list or displaying buttons.

## Source and reproducibility

The [owner-supplied archive](../adapter-references/2026-10-22/meiju-11.7.10.6-analysis-20261022.zip)
contains the six-device report/contract, generic integration contract, five model
configuration files and summaries of 22 latest successful read snapshots. It
omits actual authentication material, raw responses, original plugins/Lua and
some referenced resources. Its own independent client did not implement login,
renewal or any controls. A shared plugin route or Lua decoder field is not proof
of a feature on every product. The source reports identify unresolved settings
and transport details; they cannot support an assertion of complete parity.

`python3 scripts/extract_meiju_reference_facts.py --check` verifies the archive
SHA-256 and reproduces the four compiled exact AC catalogs without extracting or
executing bundled programs. The generated file contains product selectors and
reviewed protocol facts, not session data or downloaded executable plugins.
The generated [interface ledger](meiju-interface-coverage.json) tracks all 67
generic contract operations individually; the table below groups their blockers.
See [ADR-0074](../adr/0074-exact-meiju-reference-profiles.md).

## Implemented in the current App

Existing independent HomeVerb login and the admitted community Lua transport
remain the account/control path. No new secret is required for these changes.
Exact profiles supersede the old broad optional-AC default:

| Protected product | Exact public model | Candidate optional controls |
|---|---|---|
| 22012559 | KFR-26G/N8VJC3 | Vertical swing, display, overcooling protection, shutdown drying, anti-draft |
| 22040089 | KFR-35G/N8HE1ⅡPro | Both swing axes and directions, display, overcooling protection, shutdown drying, anti-draft, cool energy saving, strong cooling, light sensing |
| 22013331 | KFR-35G/WF2 | Both swing axes and directions, display, overcooling protection, shutdown drying, breezeless, cool energy saving, light sensing |
| 220F4057 | KFR-72G/N8HA1Pro | Both swing axes and directions, display, overcooling protection, shutdown drying, cool energy saving, light sensing |

These are inspected candidates, not a count of physically verified functions.
The App requires a reported property and current exact grant. Execution reads
prerequisites again, retains any reported buzzer preference, sends once and
preserves accepted/unknown outcomes. Cool energy saving, strong cooling and light
sensing are separate switches; ECO, comfort saving and boost are not aliases.
Existing basic power/mode/fan/temperature admission is unchanged. The new source
also exposes coupled mode/ECO/default-heat behavior that still needs a dedicated
review; it does not justify turning those operations into independent toggles.

The exact BCD-600WUFIPZM(E), product 310A2111 and subtype 56 now has a separate
read-only variable-temperature component, alongside fridge/freezer. Its power,
target and current temperature come from reported left-flex-zone fields only.
The separate details capability adds reported deep cooling, plasma sterilization,
odor purification/progress, quick ice making, door alerts, saving-star state and
three door-state tokens. Missing readings stay absent, all cloud readings retain
unproven physical sample time, and no writable ranges/defaults are inferred from
the 6/2/0 presets or the deep-cooling marketing name. The modern/old extension
configuration conflict is unresolved; a reported field is not control admission.

The Flutter panels use descriptor fields and component IDs, without vendor/model
branches. English/Simplified Chinese labels are generated from the shared catalogs.
Explicit inventory refresh publishes new scopes while preserving local identity
and preferences. New detail fields are omitted for older saved descriptors. The
existing automatic foreground state-read policy does not refresh inventory or
enable missing controls.

## Existing-session compatibility experiment

After direct user sign-in in the isolated Android emulator, the opt-in
[ADR-0075 probe](../adr/0075-one-shot-meiju-read-compatibility-probe.md) tried
`/v1/device/status/lua/get` and `/v1/appliance/home/list/aggregate` once each.
Both returned business code zero with an object in `data`, using the existing
protected HomeVerb session and already-admitted POST signer. No new signing
material or second login was required. The initial blanket authentication blocker
was too broad and is withdrawn for these two tested requests. This does not prove
every response field, other endpoints, renewal or another account/region works.
The [closed evidence record](meiju-read-probe-evidence.json) contains no account
or device identifiers, credentials, state values or raw responses.

These are development-probe results, not ordinary App integration of the two
new routes. The ordinary APK retains its existing inventory/state paths. Actual
response mapping, list completeness, model-specific semantics and remaining
GET/query/business wrappers still require implementation and verification.

## Remaining interface groups and concrete blockers

| Interface/function group | HomeVerb status | Missing evidence or dependency |
|---|---|---|
| Native account login, saved session, legacy inventory and Lua reads | Existing implementation retained; same-session compatibility proven for two modern POST reads | Other endpoints and independent modern renewal remain untested |
| `/v1/appliance/home/list/aggregate` | Same-session probe passed; no replacement of existing inventory | Complete response/card flattening, completeness and membership fixtures |
| `/v1/device/status/lua/get` | Same-session empty-command probe passed; existing `/mjl/...` reads remain | Model-specific response semantics, decoding and complete positive/negative fixtures |
| TSL GET/POST properties | Unimplemented and untested | Exact GET/query signing and serialization, current-session admission and response fixtures |
| TSL PUT properties and module actions | Blocked | Endpoint-specific authentication, per-model argument/response/effect review and transmission tracking; never generic URL/body access |
| AC energy/statistics, fault history, sleep curves and filter records | Blocked | Exact app2base UID/dataKey/service wrapper; business error/units/pagination/response fixtures |
| AC sleep/ECO/default heat, dual anti-draft, child/mother-baby modes, self-clean and timers | Incomplete | Conditional/coupled effects, precise current-profile encoding and trusted-risk review; no project scheduler |
| Refrigerator targets and modes | Read-only | Cloud configuration conflict, exact encoder behavior/step/effects, prerequisite preservation and grants; no generic centralized replay |
| Refrigerator config, reports and existing diagnostic results | Blocked | Endpoint-specific compatibility plus independent business signature and authorization wrapper; original signing constant is excluded |
| GX1200Max 760064BA program/action controls | Blocked; old five legacy profiles unchanged | Protected current firmware branch, full state-dependent TSL conversion, program-option constraints and endpoint-specific compatibility |
| Dishwasher settings/statistics/history/consumables | Blocked | E1 msg/action envelopes, user binding, complete business response fixtures; settingInfo returned empty |
| Device templates/plugins/Lua resources | Not downloaded or executed | Endpoint/source admission and resource integrity; runtime executable plugins remain prohibited |
| Firmware upgrade/reset, shared cloud services and unclassified route fragments | Unadmitted | Exact device applicability, complete semantics and consequential/high-risk policy review |

The package explicitly withholds masKey/mucSecret values. Its read-only client
loads an existing official-App session instead of implementing independent
login/renewal. The owner-requested live experiment subsequently demonstrated
that the already-admitted signer/session works for the two fixed POST reads;
it does not establish equivalence for all app/business signatures. No raw
session was exported, confidential developer secret added or generic route
executor published. Live acceptance used only the owner's direct in-App sign-in
and bounded reads. No appliance control was sent.

## Compatibility and validation

Native API 1.26.0 adds the three Boolean feature names to startup permissions.
Older libraries reject them; App preparation still derives choices from inspected
definitions. The method/function counts, JSON envelope, SQLite 5 and protected
payload 3 do not change. Existing receipts and identity bindings are preserved.
Saved generic optional actions cannot override a newly selected exact profile.
New optional fields require the current descriptor; refresh explicitly after
upgrading before attempting these controls.

Rust regressions cover all four exact selectors, mismatched model/category/
subtype, independent scalar encodings, invalid/missing prerequisite state,
accepted/unknown replay, three-zone isolation, old-descriptor behavior and
malformed-read cache preservation. Flutter checks cover switch dispatch, retained
observed state, lifecycle invalidation, the read-only third zone and localization.
Executed fictional and owner-authorized read results and package identity are
recorded in the
[App verification record](../../apps/mobile/verification.md). Real optional
controls, physical meaning/freshness of refrigerator observations, phone policy
and full parity remain unverified. Actual three-zone cloud readings and rendering
passed after direct sign-in in the emulator. No real appliance command is
authorized by this source integration.
