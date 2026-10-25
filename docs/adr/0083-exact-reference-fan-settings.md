# ADR-0083: Exact reference fan settings

Status: implemented with synthetic Core, native and App verification; physical verification pending.
Date: October 25, 2026.

## Context

The owner's request to complete documented Mijia functionality includes the
October 22 reference fan. Its existing five standard fan control families do not
cover the reference's separate included-angle, vertical swing and ionizer fields.
The reference currently exposes these as read-only scalar observations.

## Decision

Extend the existing one-property, exact-grant fan control path for precisely
`jdjz.fan.2072fc`, device URN
`urn:miot-spec-v2:device:fan:0000A005:jdjz-2072fc:1:0000D062`, service 2 and its
pinned service URN. Admit only these additional reference fields:

| PIID | Stable property | Target domain |
|---|---|---|
| 6 | `horizontal_swing_included_angle_6` | Integer enum 60, 90, 120 degrees |
| 7 | `vertical_swing_7` | Boolean |
| 8 | `vertical_swing_included_angle_8` | Integer enum 60, 105, 120 degrees |
| 9 | `anion_9` | Boolean |

Retain the existing public observation identifiers. Included swing angle is not
absolute horizontal orientation; it must not be renamed to `horizontal_angle`.
The protected current specification must retain the exact model, device/service/
property URNs, SIID/PIID, format, unit, read/write access and domain from the pinned
reference. A wider, narrower or otherwise changed domain does not admit a write.
Other models, manufacturer services, timer/reservation fields and MIoT actions
remain separate work. These four settings are idempotent scalar low-risk commands;
none schedules HomeVerb background work or bypasses a device's own firmware rules.

Each setting requires its own trusted startup exact-value grant. A grant neither
proves model support nor changes saved device evidence. The adapter independently
checks protected identity, model/spec/route and arguments. Each request performs
one property-set send; a correlated acknowledgement is accepted, never verified.
Existing possibly-sent/unknown handling and replay guards remain mandatory.

## Compatibility

Native API 1.29 extends only the `mijia_fan_grants.property` enum and the matching
control options; all 22 C functions and 34 ordinary JSON methods stay unchanged.
All current API manifests/example version fields follow 1.29; historical records
remain unchanged. The iOS acceptance version assertion follows the compiled API,
without claiming an executed iOS run. Flutter checks API 1.29 for new grants and
keeps existing Mijia controls compatible with API 1.27/1.28.

SQLite schema 5, Mijia protected payload 1 and Meiju payload 3 remain unchanged.
Old reference read-only mappings must still validate exactly; explicit inventory
refresh can derive the new options. Existing read-only sessions stay read-only.
Newly derived protected mappings require this implementation; an older reader
refuses unsupported derivations rather than silently granting or rewriting them.

## Acceptance

Test exact values and wire property correlation through Core; deny absent grants,
wrong values, source/domain/model changes, foreign identities and read-only hosts.
Retain uncertainty on lost replies with no replay. Exercise old saved mappings,
contract positive/negative cases, native grant admission, Flutter exact permission
choices and old-version refusal. Synchronize routing, errors, compatibility and
coverage records. No real-device write or physical acceptance is authorized by
this implementation decision.
