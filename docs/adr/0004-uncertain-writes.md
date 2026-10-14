# ADR-0004: Represent uncertainty in physical writes

Status: accepted. Date: October 14, 2026.

## Context

A network timeout does not prove non-execution; LAN/cloud retries may start a physical action twice.

## Decision

Separate accepted, verified, failed, and unknown. Never automatically replay possibly-sent non-idempotent actions. Verification needs evidence, and batches retain per-item outcomes.

## Consequences

Some requests return unknown rather than a simple success/failure. Explain that in the UI instead of sacrificing correctness for a reassuring status.

## Change control

Changing this decision requires explicit owner approval and a superseding ADR. Implementation details must not silently change the product's scope.
