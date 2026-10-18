# ADR-0074: Protected Mijia fan property controls

- Status: Accepted for implementation; physical acceptance pending
- Date: 2026-10-18

## Context

The requested fan adaptation needs controls bounded by each device's actual MIoT
specification. Public descriptors and online hints cannot establish writable
properties, permissions or physical completion.

## Decision

Admit one scalar property per command for the exact standard fan service already
mapped under ADR-0072: power, level, mode, horizontal swing and horizontal angle.
The property must be readable and writable, with an explicit bounded enumeration
or aligned numeric range, or boolean type. No universal level/mode range is added.
Unrepresentable ranges remain read-only. Other service types, MIoT actions, vendor
properties, doorbells and locks remain unavailable for writes.

The Core adapter defaults to read-only. Trusted composition must separately enable
fan controls and provide Core grants for exact local devices, components, actions
and arguments. The adapter independently loads protected account/model/spec/route
evidence and revalidates arguments against its derived action. Forged public
capabilities, risk metadata or grants cannot select another wire property.
Account leases cover transmission and reply handling; lifecycle cancellation
releases the lease and preserves possibly-sent receipts.

Local control-option inspection projects protected spec evidence without network
traffic, publication, grants or current-provider authorization. Enabled discovery
publishes only spec-admitted actions. Saved descriptors do not retain authority
after reopening a default read-only session.

Each command uses the separate fixed-endpoint property-set transport exactly once.
Only an encrypted single-item reply matching DID/SIID/PIID can establish accepted
or explicitly rejected. Accepted has no completion evidence. Transport failures,
malformed/global replies and cancellation after send preserve uncertainty; no
implicit login, redirect, alternate route or retry is permitted.

## Consequences

The Rust adapter path is additive. Native API 1.25 adds `mijia_fan_grants` only to
trusted Mijia/combined startup, each with a local device, `fan-SIID` component,
property and exact scalar value. Its 1024-entry bound and duplicate checks apply
before opening a profile. Ordinary requests cannot edit grants. Existing
`inspect_control_options` routes by saved adapter and returns protected fan options
without networking or mutation. API 1.24 read-only callers remain supported.
Flutter must negotiate API 1.25 before submitting these startup grants.
SQLite schema 5 and protected Mijia payload 1 retain the saved original MIoT spec;
no new secret storage format is needed. Physical-device support remains
experimental until the owner completes model-specific acceptance.
