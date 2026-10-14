# ADR-0002: Embed the same core in all three entry points

Status: accepted. Date: October 14, 2026.

## Context

The product needs terminal, phone, and MCP interfaces without another central process.

## Decision

Core is a library accessed through functions or in-process bindings. Do not make App/MCP shell out to CLI or introduce a default local HTTP bridge.

## Consequences

Validate mobile FFI, permissions, and secret stores early. Shared code does not provide automatic cross-client state synchronization.

## Change control

Changing this decision requires explicit owner approval and a superseding ADR. Implementation details must not silently change the product's scope.
