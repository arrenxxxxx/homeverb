# ADR-0084: Exact Mijia light and purifier controls

Status: implemented and synthetic-tested through Core, native and App; physical acceptance pending.
Date: October 25, 2026.

## Decision

Extend the community-compatible legacy MIoT single-property write path for two
exact October 22 reference models: `abhome.light.wy0a01` and
`zhimi.airpurifier.v6`. The pinned device, service and property URNs, SIID/PIID,
format, unit, read/write access and entire target domain must all match.

| Model | Admitted SIID/PIID | Meaning |
|---|---|---|
| abhome.light.wy0a01 | 2/1, 2/2, 2/3, 2/7 | Power, brightness, color temperature, mode |
| abhome.light.wy0a01 | 2/9, 2/12, 2/13 | Power-on state, flex switch, night-light switch |
| zhimi.airpurifier.v6 | 2/1, 2/2, 2/3 | Power, fan level, mode |
| zhimi.airpurifier.v6 | 5/1, 6/1, 7/1, 8/1 | Indicator, buzzer, physical-control lock, favorite fan level |

Preserve existing observation IDs and vendor semantics. Only equivalent on-state
uses `power.state`; auxiliary settings retain vendor scope. Favorite fan level
does not select favorite mode, color temperature does not turn the light on, and
no setter inserts prerequisite or companion writes. A correlated success means
accepted, not that the selected mode currently makes the setting effective.

The adapter requires a separate trusted composition opt-in for these reference
settings. Fan admission alone cannot enable them and reference admission alone
cannot enable fan writes. Core independently requires local device/component/
action/exact-value grants. The protected specification is revalidated independently
of public risk metadata or native startup arguments. Default sessions stay read-only.
These reviewed settings are low-risk, idempotent, single-property writes.
Socket/wall-switch loads, locks/security cameras, custom composite fields, timers,
reservations and MIoT actions require their own admission and remain separate work.

## Native and App rollout

Native API 1.30 adds `mijia_setting_grants` with local device ID, component ID,
property and exact scalar value. It admits only the closed set above, validates
values against the pinned target domains before opening a profile, bounds entries
and rejects duplicates. Protected runtime evidence and Core policy remain independent.
The ordinary control-option response admits the corresponding exact component
scopes; JSON method count and C exports remain unchanged. Older callers retain
existing fan startup behavior. New callers negotiate 1.30 before new startup fields.
App controls must retain the full reviewed domains, including 2700–6500 K in
steps of 1 K; reducing them to convenient presets is not full implementation.

SQLite schema 5 and Mijia protected payload 1 remain unchanged. Old complete
read-only derivations remain readable; explicit refresh derives the new options.
Older readers reject unsupported new derivations. No credential crossover to the
modern SID/origin, background processing, uncertain-write retry or real-device
write is introduced or authorized.

## Acceptance

Check exact source/domain correlation and saved-mapping upgrades, every admitted
setting with representative/boundary values, missing/foreign grants, independent
adapter opt-ins, wrong model/route/identity, and lost/mismatched replies without
replay. Exercise native startup positive and negative fixtures plus unchanged
fan/Meiju behavior. Add App API gating, full-domain forms and bilingual small-screen
checks before claiming App completion. Each completed layer must be recorded
accurately; synthetic tests never establish physical support.
