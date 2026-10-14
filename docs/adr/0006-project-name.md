# ADR-0006: Adopt HomeVerb as the project name

Status: accepted by the project owner. Date: October 14, 2026.

## Context

The cross-brand appliance-control project needs one identity for its embedded core, terminal interface, mobile app, and local MCP entry point. The owner selected HomeVerb. The name combines home with verb: different appliances share a common vocabulary of actions.

## Decision

| Surface | Name |
|---|---|
| Project and mobile app | HomeVerb |
| Git repository slug | `homeverb` |
| Proposed CLI executable | `homeverb` |
| Proposed core package | `homeverb-core` |
| Proposed adapter API package | `homeverb-adapter-api` |
| Proposed platform adapter package | `homeverb-adapter-<provider>` |
| Proposed MCP package | `homeverb-mcp` |
| Requested private repository target | `arrenxxxxx/homeverb` |

Tagline: **One language for your appliances. No daemon required.**

Package identifiers are naming conventions for future implementation, not claims that binaries or published packages exist. Confirm ecosystem-specific identifier requirements when creating the implementation workspace.

## Consequences

Update maintained documentation, command examples, MCP host configuration, diagrams, publishing defaults, and repository tests together. Preserve existing English Git history and record the rename in a new typed commit rather than rewriting historical commits.

The name is final for this design baseline. Name selection is not trademark clearance, domain registration, package-name reservation, or proof of remote repository creation. Those remain separate actions.

This decision does not change the product boundary: no persistent service, no project-operated backend, no NAS gateway, and no remote HTTP MCP. CLI, mobile app, and local MCP each embed the same core independently. All real-device adapters remain unimplemented and unverified.

## Change control

A future project rename requires an explicit owner decision and updates to all affected public identifiers and documentation.
