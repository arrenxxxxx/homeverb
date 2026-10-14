# ADR-0001: No persistent service or project backend

Status: accepted. Date: October 14, 2026.

## Context

The owner explicitly requires no HA/NAS central instance and no persistent service.

## Decision

Use one-shot CLI processes, in-process foreground app control, and host-owned MCP stdio sessions only. No daemon, gateway, background automation, remote HTTP MCP, or project cloud service.

## Consequences

Automation without a running client and project-provided remote LAN access are unavailable. Do not quietly restore a daemon architecture as an optional feature.

## Change control

Changing this decision requires explicit owner approval and a superseding ADR. Implementation details must not silently change the product's scope.
