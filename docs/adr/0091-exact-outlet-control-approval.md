# ADR-0091: Exact Mijia outlet control with trusted approval

Status: typed adapter/Core, native API 1.34 and App implemented with synthetic tests; Android packaging checked; provider/physical acceptance pending.
Date: October 29, 2026.

## Evidence and scope

The owner-supplied, digest-pinned October 22 reference contains exact outlet
`cuco.plug.v3`, device type
`urn:miot-spec-v2:device:outlet:0000A002:cuco-v3:2`. Its reviewed scalar source
provides readable/writable switch SIID 2 properties 1 and 2, physical-control
lock SIID 7 property 1 and indicator SIID 13 property 1. Service/property URNs,
wire formats, units and full domains are available in the pinned reference.
This establishes a candidate protocol mapping, not physical actuation evidence.

Apache-2.0 community revision `f78158257e19a21902adaf6bcc8440c3ba172a91`
[customizations](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/device_customizes.py)
explicitly identify `cuco.plug.v3` and its switch service.
The [switch entities](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/switch.py)
and [device writer](https://github.com/al-one/hass-xiaomi-miot/blob/f78158257e19a21902adaf6bcc8440c3ba172a91/custom_components/xiaomi_miot/core/device.py)
use independent property values rather than a composed plug command. HomeVerb
already implements the bounded legacy MIoT property-set transport and correlated
acknowledgement checks. No upstream Python, scheduling or optimistic state is
required for this implementation.

## Decision

Add a separate trusted outlet-control admission, independent of fan and
light/purifier settings. Only the exact model/device/service/property evidence
above is admitted; similar plug models, wall switches, timers, charging protection,
load limits, automatic cycles and opaque composites do not inherit it.

| SIID/PIID | Existing component / property | Target domain | Risk |
|---|---|---|---|
| 2/1 | switch-2 / power.state | Boolean | elevated |
| 2/2 | switch-2 / vendor.xiaomi.mijia.switch.default_power_on_state_2 | Integer 0, 1 or 2 | elevated |
| 7/1 | physical_controls_locked-7 / vendor.xiaomi.mijia.physical_controls_locked.physical_controls_locked_1 | Boolean | elevated |
| 13/1 | indicator_light-13 / vendor.xiaomi.mijia.indicator_light.on_1 | Boolean | low |

A plug does not establish the kind of connected load. Changing its power can
start or stop that load; power-loss recovery behavior can affect a later restart;
locking physical controls can remove immediate local operation. These three
settings therefore require both an exact trusted startup grant and a fresh
immutable trusted confirmation under ADR-0042/0043 for every command. The public
risk field is not the enforcement boundary: the adapter re-derives risk from the
protected exact outlet mapping and checks Core's approval marker before sending.
Changing metadata to `low` or supplying `confirmed=true` cannot bypass it.

The indicator is an independently reviewed low-risk setting. No setting sends
companion power, mode, reset or prerequisite operations. Vendor numeric recovery
codes retain their exact reported labels only where evidence establishes them;
unknown semantics must remain numeric rather than being guessed from another model.
These are set-value commands, but an uncertain acknowledgement is never replayed
or routed elsewhere automatically. A correlated success is `accepted`; stale
read-back, HTTP success or an optimistic value is never physical verification.

Use a new closed `mijia_outlet_grants` startup field with exact local device ID,
component, property and scalar value. Do not widen `mijia_setting_grants`:
physical-control-lock identifiers overlap the purifier's, so sharing that startup
admission would obscure the different risk and provider/model policy. Each family
also retains its own adapter policy: combining both families never lets a grant
from one family authorize the same identifier in the other. Native
configuration validates the exact outlet tuple and value before opening a profile;
protected binding/model/domain checks remain independent during execution.

Publish new native API 1.34 only when this field and the relevant inspection
response scopes are implemented and tested. C export count, ordinary method count,
SQLite schema 5 and Mijia payload envelope 1 stay unchanged. Existing read-only
saved derivations remain valid; explicit selected-account refresh upgrades the
mapping. Old fan/light/purifier grants do not authorize an outlet. Default and
CLI/MCP sessions retain read-only outlet behavior; no JSON approval method is added.

The App must negotiate API 1.34, inspect exact options, construct only the new
startup grants and retain all target values. Low-risk indicator requests use the
ordinary flow. Elevated requests use the existing separate review/confirmation
bridge, with explicit connected-load, future-restart or physical-button effects.
Navigation, background, expiry and session replacement invalidate pending previews;
a lost reply retains its request ID for lookup without replay. Upper layers
render capability/approval metadata and do not infer semantics from a model name.

## Required implementation and evidence

1. Exact source/domain projection and a separate outlet control family; old saved
   mappings still readable and explicit refresh produces the new options.
2. Protected adapter/Core tests for all four properties, representative/boundary
   values, independent opt-ins/grants, protected identity/route/model refusal,
   actual trusted approval consumption, forged low-risk descriptors, cancellation,
   uncertain/mismatched acknowledgements and replay guards.
3. Closed native startup/schema fixtures, new component inspection scopes,
   API 1.34 negotiation and unchanged fan/light/purifier/Meiju admission.
4. App full-domain selection, separate consent, localized effects, lifecycle and
   narrow enlarged-text tests, followed by a clean-source Android package.
5. Owner-run exact-model/provider/physical acceptance. Development tests use only
   fictional material; this decision authorizes no real appliance writes.

The reference coverage ledger must mark these fields implemented only when the
corresponding runtime is present. This design does not complete event history,
modern SID login, other outlet programs or the overall reference feature request.
