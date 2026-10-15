# ADR-0010: Bind write grants to components, arguments, and routes

Status: accepted for the requested Core implementation. Date: October 14, 2026.

## Problem

The initial Mock policy binds only device, capability, and action. A device with multiple components could reuse that grant on another component. A host also cannot restrict a granted action's argument range or route. This falls short of the authorization boundary in the security design.

## Decision

Each trusted in-memory Grant must explicitly name a device ID, component ID, capability, action, allowed transports, and an argument JSON Schema. Command arguments must satisfy both the device action schema and one complete matching grant. Different grants cannot supply separate parameter permissions for a single command. Routes may be combined only from grants that each independently authorize the entire command.

Core intersects these authorized transports with the action's supported transports and the requested route policy. An explicit forbidden route returns `POLICY_DENIED`; `auto` can use only the authorized intersection. Fallback after a proven-unsent failure must stay within it. Each non-atomic step requires independent authorization, with routes restricted to the parent/step intersection. All steps are preflighted before any send; dry-run reuses this preflight and returns per-step candidate transports. Possibly-sent writes remain non-replayable. Elevated and high-risk writes remain denied even if a matching grant exists.

Grants are supplied by the trusted host, never deserialized from an ordinary request, model input, or device descriptor. Malformed or externally unresolved grant schemas confer no authority. No wildcard identity, implicit component, or implicit route permission is added. The current host principal is the Core instance's trusted configuration; no selectable principal field is introduced.

## Compatibility and acceptance

This changes the unreleased Rust Grant construction and Policy authorization return type. All in-repository hosts and tests must specify the added fields together. The version 1.0 Command/result envelopes and the C ABI stay compatible. MCP remains read-only; adding this policy support does not expose new write tools or grant editing.

Acceptance covers independent component, argument, route, and risk denial; valid bounded writes; strict LAN-only and cloud-only grants; no route expansion on failure; full-command grant matching; invalid grant schemas; and composite authorization before any transmission. No physical-device support is implied.
