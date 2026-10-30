# ADR-0106: Exact hood manual controls

Status: Core/native API 1.39/App implemented and synthetic-tested; Android archive checks passed. Real acceptance pending.
Date: October 30, 2026.

## Evidence and scope

The digest-pinned October 22 reference archive supplies exact readable/writable
fields for xiaomi.hood.cyp10, device specification
urn:miot-spec-v2:device:hood:0000A01B:xiaomi-cyp10:2:

| SIID/PIID | Existing public target | Exact domain |
|---|---|---|
| 2/1 | hood-2 / vendor.xiaomi.mijia.hood / on_1 | Boolean |
| 3/1 | fan_control-3 / vendor.xiaomi.mijia.fan_control / fan_level_1 | Integer 1, 2, 3, 4 |
| 13/1 | screen-13 / vendor.xiaomi.mijia.screen / display_state_1 | Integer 0, 1 |
| 13/2 | screen-13 / vendor.xiaomi.mijia.screen / auto_screen_off_2 | Boolean |

The pinned Apache-2.0 community implementation
[f78158257e19a21902adaf6bcc8440c3ba172a91 fan entity](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/fan.py)
uses on and fan-level property converters but may combine power with speed and
convert enumerated levels to percentages. HomeVerb retains original scalar targets,
never synthesizes percentages or issues companion writes. The pinned
[customizations](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/device_customizes.py)
expose display-state and auto-screen-off on another hood model; that is semantic
context, not cyp10 compatibility evidence. Exact cyp10 admission comes only from
the protected pinned specification. No community runtime or polling is required.

## Decision

Add four independent low-risk, idempotent single-property settings to the existing
reference-setting family. Each requires its own exact local device/component/
property/value grant, separate adapter opt-in and protected model/spec/domain proof.
Existing fan and outlet family grants cannot enable these controls. Native defaults
remain read-only. These are explicit manual appliance settings, not a safety
controller, gas-response policy or guarantee that ventilation is running.

Preserve existing vendor identifiers; do not remap hood.on to generic power.state
or make selecting a fan level turn on the hood. Codes 1 through 4 remain original
levels (not percentages); level 0 is not an off command. Display-state codes 0/1
retain their exact source meaning, Time/Pm2.5. Auto-screen-off is its own boolean;
it grants no timer or schedule setting. Device firmware can apply its pre-existing
configuration after an explicit write; HomeVerb does not promise physical effect
or alter prerequisites to force it. Correlated acknowledgement is accepted only.
Uncertain outcomes are retained without retry or cross-protocol fallback.

Repeated display fields 13/4 and 13/6, timing fields 13/3, 13/5 and 13/7, delayed
shutdown, stove linkage, gas-triggered behavior, cleaning, toggles, resets and
pairing remain separate contracts. In particular, a 0..2359 numeric range does not
establish valid clock encoding. Do not infer units or clamp values into a schedule.

## Native, storage and App compatibility

Native API 1.39 adds four closed mijia_setting_grants tuples and three component
scopes to local control-option responses. Ordinary methods and C exports remain
unchanged. Flutter must negotiate 1.39 before reopening with these tuples, preserve
all ten exact target values, and refuse broadened schemas. Presentation uses the
existing capability-driven controls, without provider wire encoding in the UI.

Explicit refresh upgrades old protected mappings without changing account/device/
connection identity. The previous hood mapping with only its two illumination
write domains remains valid, as do older read-only derivations. New implementations
must not accept arbitrary partial mixtures or edited domains as historical mappings.
Older binaries refuse the new derivation. SQLite 5 and protected payload 3 remain
unchanged; this phase adds no automatic refresh, account access or real-device write.

## Required evidence

Verify exact model/service/property/type/unit/access/domain admission and saved
mapping upgrades. Exercise all ten targets on legacy and modern protected routes,
independent family opt-ins and grants, zero companion writes, acknowledgement and
uncertainty handling. Native positive/negative fixtures must cover exact schemas,
startup and local inspection. App tests must prove pre-open API gating, all values,
foreign/malformed option refusal, ordinary control clicks and bilingual narrow
layouts. Run affected regression checks and record packaging separately. No
physical compatibility claim follows from these fictional checks.

## Exact hood manual controls -- October 30, 2026

ADR-0106 adds four exact xiaomi.hood.cyp10 properties through legacy and modern
Core paths: hood on-state, fan levels 1/2/3/4, display content codes 0/1 and the
independent automatic screen-off Boolean. The 33 reviewed scalar controls still
cover five exact models. Existing vendor IDs and raw codes remain stable; no
percentage conversion, companion power-on, inferred timer or schedule is added.
Core and the protected adapter independently require exact reference-setting
grants; fan/outlet authority does not transfer. Uncertain writes are never replayed.

Native API 1.39 adds four closed startup tuples and three inspection component
scopes. Flutter probes the native version before any granted reopen and preserves
all ten target values. Old illumination-only hood mappings remain valid; explicit
refresh upgrades their domains while keeping identity and organization. Invalid
partial domain mixtures refuse. Methods, C exports, SQLite 5 and protected payload
3 are unchanged. Older native libraries refuse new grants/derivations.

Scoped evidence: 233 Mijia unit and 14 compile-fail tests passed, including all
33 modern controls and every new legacy target. Host account/grant suites passed
53 and 3 tests. The hood/auxiliary/settings/permissions Flutter suite passed 50
tests, the related control/approval/device/FFI suite passed 61 tests, and analysis
reported no issues. All material is fictional. The API 1.38 APK from 2e776a9
predates the four new controls. Final workspace/native checks and API 1.39
packaging are recorded separately; provider/phone/model acceptance remains pending.

Final software verification: the workspace excluding the known unrelated
Meiju/macOS LAN-alias suite passed 642 tests with one ignored. Strict all-target
Clippy and formatting passed. Rebuilt FFI and actual C boundary checks passed
36 methods/60 live examples, including new hood startup acceptance/refusal and
76 modern-account cases. The checked-in examples were regenerated from that run.
Documentation, source hygiene, 25 schemas with 73 positive/1320 negative fixtures,
two coverage-projection tests and seven presentation tests passed. The existing
eight full Flutter font/pixel/legacy-click failures were not rerun or claimed fixed.
No real account, appliance, installation, remote push or publication was used.

The clean 503ef1f API 1.39 Android package subsequently passed dual-ABI/current-AAR,
31-export, signature and 16 KiB alignment checks. See the
[delivery record](../../apps/mobile/verification.md#exact-hood-controls-api-139-android-package--october-30-2026).
No installation, provider query or physical-device write was performed.
