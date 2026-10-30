# ADR-0119: Exact Mijia hood preferences

Status: Core/native API 1.45 implemented and synthetic-tested; App adoption and physical acceptance pending.
Date: October 30, 2026.

## Evidence and decision

The digest-pinned October 22 reference specification for `xiaomi.hood.cyp10`,
`urn:miot-spec-v2:device:hood:0000A01B:xiaomi-cyp10:2`, admits five additional
readable/writable properties on service 2. Use the existing independent reference
setting family with exact device/component/property/value grants and protected
full model/service/property/type/unit/domain checks. Default sessions stay read-only.

| PIID | Public property on hood-2 / vendor.xiaomi.mijia.hood | Domain |
|---|---|---|
| 9 | power_on_with_light_9 | Boolean |
| 10 | power_off_with_light_10 | Boolean |
| 11 | clean_remind_on_11 | Boolean |
| 12 | clean_remind_time_12 | 20, 30, 40, 50 hours |
| 18 | gestures_18 | Boolean |

These are low-risk idempotent preferences, with unavailable verification. They
configure the appliance's existing light linkage, cleaning reminder and gesture
input. They are not gas-response/safety controls or a guarantee of ventilation.
The public labels preserve the source names; no additional causal direction,
gesture vocabulary, timer-reset behavior or execution ordering is inferred.
One explicit target produces one property write. Threshold changes do not enable
the reminder; linkage changes do not send companion light/power commands; gesture
preferences do not synthesize gestures. Firmware may apply existing configuration
following an accepted write. HomeVerb starts no timer, polling or background work.

## Compatibility and safety

Native API 1.45 adds five closed tuples to `mijia_setting_grants`. There is no new
method/export, transport, command outcome, SQLite version or protected-payload
version. Existing complete read-only, illumination-only and six-control hood
mappings remain valid; explicit refresh upgrades them. Arbitrary partial mixes
and edited domains remain invalid. Older libraries reject new tuples. App startup
must negotiate API 1.45 before supplying them; App adoption is a separate phase.

Core and the adapter require independent reference admission. Fan/outlet grants
cannot authorize these targets. Correlated success means accepted, never verified.
Timeout/cancellation/lost replies remain unknown without replay, retry, renewal,
protocol fallback or companion writes. Delayed shutdown, schedules, stove linkage,
gas behavior, display clocks and cleaning execution remain separate coverage.

## Verification requirements

Exercise all twelve targets through legacy and modern protected routes; inspect
locally and test read-only defaults, wrong grants/families, exact one-field wire
requests, source/domain mutations, historical mapping upgrades and unknown receipts.
Native schema/ABI admission must reject foreign scope, malformed values and
20/30/40/50-hour off-grid values before storage. No real appliance operation is
required or authorized by this implementation phase.

## Executed verification

Rust 1.96 offline/locked Mijia and Host regressions passed 666 tests. The broader
workspace passed 742 tests with one ignored, excluding `homeverb-midea` because
its known macOS LAN-alias failures remain unresolved. Strict all-target workspace
Clippy, formatting and workspace build passed. Tests cover all twelve new target
values on both protected protocols, source/domain mutations, independent grants,
legacy mappings and uncertain single-property writes without replay.

Contract validation passed 26 schemas, 89 positive and 2,118 negative fixtures;
three reference projection tests passed. Existing Flutter hood/auxiliary/settings/
session-permission compatibility passed 50 tests against the rebuilt API 1.45
library. This does not establish new App controls or real-provider support.
Actual C ABI verification passed 38 ordinary methods and 86 live examples,
including both Mijia/combined startup modes and malformed new preference tuples
rejected before profile creation. The modern-account C boundary retained its
76 refusal/buffer/lifecycle checks. Documentation, repository hygiene, projection
and diff checks passed. No real account, provider, physical-device operation,
installation, push or publication was performed.
