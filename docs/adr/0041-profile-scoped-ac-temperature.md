# ADR-0041: Admit profile-scoped AC temperature setting

Status: accepted for offline implementation; physical acceptance remains deferred.

## Evidence and selection

The pinned [AC mapping](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/device_mapping/T0xAC.py)
and [climate setter](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/climate.py)
define different temperature limits, steps and field encodings. Use protected
product evidence from ADR-0040, not display metadata, to select a reviewed mapping.
Continue to require one of the four exact experimental AC models.

| Source selector | Celsius range / step | Encoding |
|---|---|---|
| Subtype 524, before any product-code/category selection | 16–30 / 1 | Integer `temperature` plus `small_temperature: 0` |
| Product code 23096245 | 17–30 / 1 | Integer `temperature` only |
| Product codes 22396961, 22396963, 22396965, 22396969, 22396973, 22397057, 22397059, 22397061, 22397063, 22397065, 22397067, 22397091 | 16–30 / 0.5 | Integer `temperature` plus `small_temperature: 0` or `0.5` |
| Product codes 22040055, 22040023, 22040053, 22396505, 22396517, 22396521, 22396525, 22396533, 22259015, 22019031, 22019035, 22019045, 23096653, 23096633, 22012369, 22270043, 22251077 | 17–30 / 0.5 | Same two-field encoding |
| No product-code override, explicit `wall-air-conditioner` category | 17–30 / 0.5 | Same two-field encoding |

Require a known subtype before considering a lower-priority selector and a known
product code before category fallback. Unknown evidence never selects the generic
family default. Explicit water-temperature, fan-only and centralized-setting
profiles (22012895, 21293193, 17497071, 10693145, 106J6363, 26093137, 26093139,
26096947, 22012227, 22013167, 22013285, 22251749) never fall through to a category mapping.
Category selection follows the source's hyphen-to-underscore normalization, so
`wall_air_conditioner` selects the same mapping as `wall-air-conditioner`.
The inspected centralized-setting profiles require preserving additional values;
no stale-cache copy is admitted here. A selected mapping is protocol evidence,
not a physical model/firmware guarantee.

## Command and permission contract

Expose `main / climate.set {target_temperature}` after explicit temperature
opt-in and discovery/refresh. The action carries its selected minimum, maximum
and multipleOf constraints. Read properties retain their observed-value domain;
write bounds must not hide an otherwise valid historical reading. Never infer
power, mode, fan or auxiliary-heat changes. Both atomicity and independent-field
flags are false. The adapter reselects the protected profile and validates exact
arguments before transport, even if public descriptors were weakened.

Native API 1.12.0 adds optional `meiju_temperature_grants`: at most 64 distinct
canonical local UUID-v4 / exact Celsius target pairs. The startup representation
admits the union of reviewed setpoints (16–30 in half-degree increments), not a
universal appliance range. Both the actual action constraints and the exact grant
must permit a command. Other actions, components, targets and LAN remain denied.
The same list is available to explicit CLI/MCP grant files; missing/empty lists
preserve prior behavior. Ordinary command JSON cannot edit authority.

Retain all 31 methods, seven C functions, SQLite schema 4 and protected payload
version 3. Keep existing Rust host helpers as wrappers; add a typed borrowed grant
bundle for Core and native sessions. Negotiate API 1.12 before supplying the new
field. Older binaries reject it. Previously saved product evidence survives
omitted later fields and supplies discovery descriptors without extra provider
calls; permissions remain session-local and require explicit host restart to change.

## Outcomes and verification

One request contains only the encoded temperature field(s). Do not round invalid
inputs, copy cached status, read after acceptance, change optimistic state, retry,
fall back or restore settings automatically. Code-zero acknowledgement remains
accepted; uncertain transmission remains unknown. Current cloud observations
cannot verify temperature or preservation of unrelated settings without status
context. Physical and off-LAN acceptance remain outstanding.

Test selector priority and refusals, whole/half-degree boundaries and exact bodies,
protected evidence retention, descriptor tampering, model/identity/membership
checks, independent startup policy, native/CLI/MCP admission, receipts/replay and
foreground interruption using fictional profiles only. No real-account or
appliance operation is authorized by this phase.
