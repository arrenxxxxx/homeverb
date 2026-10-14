# ADR-0005: Native-client authentication is an admission gate

Status: accepted. Date: October 14, 2026.

## Context

The project does not operate a backend and must not embed confidential shared application secrets in clients.

## Decision

Admit only viable authentication paths without a confidential project backend. If a vendor requires one and has no native alternative, mark the integration blocked.

## Consequences

Not every vendor platform can be supported. Research authentication feasibility before investing in large adapter implementations.

## Change control

Changing this decision requires explicit owner approval and a superseding ADR. Implementation details must not silently change the product's scope.
