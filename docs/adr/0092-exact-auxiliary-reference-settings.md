# ADR-0092: Exact auxiliary reference settings

Status: native API 1.35 implemented with synthetic tests; App integration and acceptance pending.
Date: October 30, 2026.

## Evidence

The digest-pinned October 22 archive contains these exact readable/writable
properties with complete scalar domains:

| Exact model/spec | Service/property | Existing public target | Domain |
|---|---|---|---|
| `xiaomi.hood.cyp10`, `urn:miot-spec-v2:device:hood:0000A01B:xiaomi-cyp10:2` | light 6/1 | `light-6`, `power.state` | Boolean |
| Same hood | light 6/2 | `light-6`, `vendor.xiaomi.mijia.light.brightness_2` | Integer enum 0, 1, 2 |
| `jdjz.fan.2072fc`, `urn:miot-spec-v2:device:fan:0000A005:jdjz-2072fc:1:0000D062` | function 3/5 | `function-3`, `vendor.xiaomi.mijia.function.quiet_5` | Boolean |

The pinned Apache-2.0 community implementation at
`f78158257e19a21902adaf6bcc8440c3ba172a91` provides a
[property converter](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/converters.py)
that emits DID/SIID/PIID/value. Its
[light entity](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/light.py)
may combine power and brightness during `turn_on`. HomeVerb uses the already
implemented single-property transport; it neither copies that companion write
nor its range-based 0-255 brightness conversion. The hood's three codes have no
percentage unit and remain exact vendor enum values. The source's `Drak` label
is not grounds to infer a physical power-off effect for brightness code 0.

The fan's custom `quiet` boolean stays vendor-scoped. It does not imply a mode,
fan speed, timer, reservation or power transition. No upstream scheduled job,
Python runtime, relogin retry or optimistic state is adopted.

## Decision

Extend the trusted reference-setting family with these three independent low-risk
settings, retaining the exact existing component/property IDs and source evidence.
Each command requires its own device/component/property/value grant and the
separate reference adapter opt-in. Fan and outlet grants never enable these
settings. Existing reference grants cannot target `light-6` or `function-3` until
native API 1.35 explicitly admits the tuples. The independent family policies
introduced in ADR-0091 remain in force.

Hood main power, ventilation, gas-triggered behavior, stove linkage, cleaning,
filters, toggles and timing are separate contracts; no companion or prerequisite
write is allowed. Fan timing and reservation remain unavailable. This does not
create project scheduling, a keep-alive or any work outside a foreground operation.

Native API 1.35 expands `mijia_setting_grants` by three closed tuples and local
control-option component scopes. It retains 36 JSON methods, 26 C exports plus
the Android initializer, SQLite 5 and Mijia payload envelope 1. Default descriptors
remain read-only. Old protected read-only derivations are valid; explicit account
refresh upgrades controls while preserving local account/device/route identity.
New derivations require the matching updated implementation.

The App negotiates API 1.35 before sending these tuples, enumerates all seven
exact targets, retains integer enum codes without percentage conversion, and
uses the ordinary low-risk single-property flow. Presentation remains driven by
capabilities and component metadata. Unknown codes and missing observations are
never guessed or replaced with zero. A success acknowledgement is `accepted`;
read-back remains stale cloud evidence, and uncertain writes are not replayed.

## Required evidence

1. Protected exact model/spec/service/property/type/unit/domain admission and
   negative mutation fixtures, including rejection of unreviewed sibling controls.
2. Core tests for all seven targets, independent opt-ins/grants, exact encrypted
   request bytes, zero companion writes, rejection and uncertain outcomes.
3. Native schema/startup/inspection fixtures and API version manifests; old fields
   and other platform grants remain unchanged.
4. App version gating before additive startup, explicit target selection, native
   startup tests and narrow enlarged-text presentation in both supported locales.
5. Verified packaging from clean source. Provider/physical acceptance remains a
   separate owner-run gate; this decision authorizes no real appliance write.

Modern SID authentication, events, BLE/LAN, health/media and other coverage gaps
remain open. A public specification and passing fictional tests do not establish
actual device behavior or complete official-App parity.
