# ADR-0027: Admit scoped native AC power grants at trusted startup

Status: accepted for the requested implementation. Date: October 15, 2026.

## Problem

The experimental Meiju AC power adapter and Core grants are callable by a developer
probe, while Flutter's native host has only Mock write configuration. A broad real
write switch would discard the device/argument/route boundary in ADR-0010. Ordinary
requests must not edit their own authority. The owner deferred physical operation;
this phase uses synthetic controls only.

## Decision

Native API 1.3.0 adds optional `meiju_power_grants` to trusted `homeverb_open`
configuration. Each item has exactly `device_id` (canonical lowercase UUID v4 from
this local profile) and `state` (boolean). The array contains at most 64 distinct
pairs, enough for both states of a 32-device Core batch. Omitted/empty grants retain
read-only Meiju behavior. Nonempty grants require explicit `adapter: "midea.meiju"`,
false Mock permission and normal scenario. Invalid configuration fails before
profile creation. There is no wildcard, action schema, provider ID, secret, principal
or confirmation flag in a grant.

The host maps each pair to a Core grant for that exact device's `main / power.set`,
the exact `{state: boolean}` object, and cloud transport only. It opts the adapter
into experimental power descriptors; Core and protected binding/model checks still
apply. An unsupported model cannot gain capability from a grant. `auto` can resolve
only to cloud; an explicit LAN request is denied. No other appliance/control, nor
an elevated/high-risk action, is admitted by these permissions.

Startup is a trusted application boundary. Flutter must obtain an explicit user
decision about the selected device/states before supplying grants. A tool argument,
device descriptor, external JSON or model-generated `confirmed: true` must never
be copied into startup authority. MCP remains read-only and does not expose startup
editing. Opening a granted session alone sends nothing and does not access a saved
session or discover devices.

Grants are immutable for the host lifetime and are not persisted in SQLite. To
change them, stop new submissions, close the old handle (cancelling active work),
and open a new handle with the intended full list. Do not silently widen a denied
request's authority or convert old receipts into new commands. Foreground/cancel/
close behavior and unresolved-write recovery are unchanged. Accepted/unknown cloud
outcomes remain unverified; closing the session is not physical rollback.

## Compatibility and acceptance

The seven C symbols, 24 methods, Command/Execution schemas and SQLite schema 3 are
unchanged. Existing startup objects preserve their behavior. Older libraries reject
the new field; read `get_api_info` on an ordinary read-only handle and require API
1.3 or later before enabling this startup option. Default discovery still publishes
read-only descriptors; a granted session needs an explicit account refresh/discovery
to publish experimental action descriptors. Cached metadata alone grants nothing.

Tests cover schema/runtime agreement for valid/invalid startup, refusal before local
profile creation, no account/network work on startup, exact policy mapping,
read-only reopening, host plans and synthetic accepted/unknown executions, receipt
replay, cancellation/background/close and account-lease cleanup. Real household
writes, official-app/physical comparison, off-LAN/mobile operation and Flutter
consent UI remain unverified. This implementation does not override the owner's
decision to defer real operation.
