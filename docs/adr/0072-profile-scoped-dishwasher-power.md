# ADR-0072: Add explicit dishwasher power requests

Status: accepted for offline implementation under the owner's appliance-button
request; physical acceptance remains deferred.

## Evidence and scope

The pinned community E1 mapping at
`ce0c4b88224def0f6b53714312c19f71aa9c2a50` explicitly maps `power_on` and
`power_off` to the corresponding `work_status` values for all five products
already reviewed by ADR-0055 and ADR-0063. The official Android E1 base Lua
independently declares both values. The owner's official App screenshot shows
an explicit dishwasher power-on button. None of this proves the selected owner's
protected product identity or physical operation.

Admit these two requests only for the existing five protected product profiles
and experimental exact-model boundary. Product 760Y0026 requires an explicit
`additional` value from the same reviewed domain as cycle controls. The other
four profiles require empty arguments. Product 7600016L and family defaults stay
excluded. Do not derive values from cache, select a program or substitute cancel
for power-off. Power-on is not a claim that washing has started; power-off is not
a guarantee of electrical isolation, safe door opening, draining or rollback.

## Contract

Native API 1.24 extends the existing `meiju_dishwasher_cycle_grants` action enum
with `power_on` and `power_off`. Preserve the field and Rust type names for
compatibility; they now cover explicit dishwasher lifecycle requests. The
64-entry bound and exact device/action/argument matching are unchanged. Existing
start/pause/resume/cancel grants do not authorize either power action, and power
requests do not authorize one another or any cycle action.

Both actions remain elevated, non-idempotent and cloud-only with unavailable
verification. They require the existing separate trusted preview/confirmation
boundary and independent adapter approval check. Ordinary JSON, CLI and MCP
cannot confirm them. No automatic read, retry, route fallback, restoration or
optimistic state update is added. Uncertain writes retain receipts and replay
protection. Capability revision 6 includes these actions only when host control
admission is enabled and protected profile evidence is present.

Older hosts reject new enum values; App grant preparation must use only actions
returned by the running library's reviewed control-option inspection. New static
startup callers must negotiate API 1.24. No new C function, ordinary method,
SQLite schema or protected-payload format is introduced.

## Acceptance

Compare all admitted product/power payloads and centralized options with the
pinned source. Extend actual adapter execution tests across accepted/rejected/
unknown and lifecycle paths, plus absent/conflicting profiles, forged public
metadata, independent grants, immutable previews and unchanged state. Update
native startup schema/fixtures and actual C admission checks. Exercise App
permission preparation and power-button confirmation without device traffic.
Physical power behavior and the owner's protected profile remain separately
unverified until explicitly authorized testing.
