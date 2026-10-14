# ADR-0003: Unify capabilities; integrate platforms and protocols

Status: accepted. Date: October 14, 2026.

## Context

A brand is not a protocol, and regions may use different platforms. Common method names alone discard device constraints.

## Decision

Use Device/Component/Property/Action/Event structures and parameter schemas shared by all entry points. Vendor-specific behavior uses a vendor namespace.

## Consequences

Maintain capability semantics and versions. Do not promise identical device features or replace model-specific evidence with a brand label.

## Change control

Changing this decision requires explicit owner approval and a superseding ADR. Implementation details must not silently change the product's scope.
