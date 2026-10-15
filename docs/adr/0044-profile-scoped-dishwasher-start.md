# ADR-0044: Start reviewed dishwasher programs through trusted confirmation

Status: accepted for offline implementation; physical acceptance remains deferred.

## Evidence and admission

The pinned [E1 mapping](https://github.com/sususweet/midea_auto_cloud/blob/ce0c4b88224def0f6b53714312c19f71aa9c2a50/custom_components/midea_auto_cloud/device_mapping/T0xE1.py)
uses different program encodings and centralized fields by product. Starting a
program can consume water and heat. Admit it as elevated and non-idempotent,
requiring both an exact startup grant and ADR-0043's trusted confirmation path.
Ordinary JSON/CLI/MCP plan and execute cannot approve it.

Continue to require the exact experimental dishwasher model already admitted for
reads, plus protected product evidence from the account inventory. Require a known
subtype before considering the product code; the pinned E1 file has no subtype
or category override. Never use the family default for unknown/missing products.

| Protected product code | Program encoding | Additional command requirements |
|---|---|---|
| 7600020L | Reviewed short names, such as `eco` and `normal` | No additional field |
| 76006481, 7600649Q | Reviewed names such as `eco_wash` | No additional field |
| 7600V1E7 | Its nine-option list; `soft_wash` encodes `glass_wash` | No additional field |
| 760Y0026 | Its twelve start options; exclude `neutral_gear`, which cancels | Explicit `additional` code: 0, 9, 10, 13 or 14 |

7600016L also requires wash-region and automatic-door fields whose complete
writable domains have not been established here. Leave that profile unavailable;
do not drop those fields or copy stale cloud state to pretend preservation.
The supported 760Y0026 path explicitly sets the additional option instead of
silently copying it. Selection establishes a reviewed experimental wire mapping,
not that the owner's product has any specific code or is physically verified.

## Contract and implementation

Expose `main / vendor.midea.dishwasher.start_cycle` with a descriptor-specific
`program` enum. The 760Y0026 action also requires `additional`; all other admitted
profiles reject it. Preserve vendor program names as identifiers and supply
English/Chinese presentation labels. Include the exact program and any additional
option in both the command and confirmation. No implicit default, cached setting,
resume, pause, cancellation, delayed start or automatic restoration is introduced.

Native API 1.14 adds `meiju_dishwasher_grants`, at most 64 distinct exact local
UUID-v4 / `arguments` pairs. Each grants only this action on `main` over cloud.
The grant schema admits the union of reviewed arguments; the selected descriptor
and protected profile further restrict them. Absence preserves read-only startup.
Do not add this elevated permission to CLI/MCP startup files, whose dispatchers
cannot perform trusted user confirmation. Keep the 31 methods and ten C functions.

The adapter independently requires Core's approval marker before account access,
then checks protected identity, complete-inventory membership, family, exact model,
selected profile and exact arguments before constructing one cloud control.
The payload contains `work_status: work`, the selected wire `mode`, and only the
explicitly required `additional` option. It performs no pre/post read, retry or
route fallback. It does not mutate observed cache. Cloud code zero is accepted,
never physical verification; uncertain transmission remains unknown. Existing
journaling, operation deadlines, approval expiry and lifecycle cancellation apply.
The unreleased Rust `MeijuControlGrants` bundle gains a `dishwasher` slice;
existing helper functions keep their behavior and struct callers may use
`..Default::default()`. SQLite schema 4 and protected payload 3 are unchanged.

## Acceptance

Use fictional accounts and transport only. Verify every profile/program body,
missing/conflicting profile refusal, centralized-field requirements, exact grant
scope, ordinary denial, unforgeable adapter approval enforcement, protected family
and model checks, inventory membership, changed preview, replay prevention,
accepted/unknown/rejected outcomes, pre-send failure, cancellation and retained
receipts. Check native startup schema/runtime parity and both locale catalogs.
Build the Android libraries with unchanged export count. Real account access,
physical/off-LAN operation and Flutter consent UI remain separate acceptance.
