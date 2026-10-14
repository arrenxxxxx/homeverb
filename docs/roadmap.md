# Roadmap

No calendar deadlines are promised. Advance by testable deliverables. Only the documentation baseline and repository checks are available; application milestones are planned.

## M0 - Design and contract baseline

Product boundaries, architecture, agent instructions, schemas/fixtures, diagrams, checks, and repository templates. There is no control program. Acceptance means consistent documentation, a no-daemon architecture, and honest implementation status.

## M1 - Minimal shared-core flow

Create the actual language workspace, mock adapter, device/capability/read/write/error contracts, CLI JSON output, minimal Android Flutter FFI probe, and a local read-only MCP catalog. The goal is proving that all three hosts can use the same in-process core before writing extensive vendor code.

Acceptance: fictional devices produce consistent results from CLI, App, and MCP; cancellation and exit leave no background work. Mock success is not device support.

## M2 - First real platform

Verify native authentication first, then reads and safe owner-authorized writes. Add applicable LAN/cloud paths, verification, and secret protection. Choose an available real device rather than promising a model in advance.

Acceptance: exact-model evidence; one client operates independently; no HA/NAS/project backend; timeouts do not duplicate writes; unverified items are explicit.

## M3 - Heterogeneous platform and mobile experience

Add a genuinely different protocol family, not only another brand alias. Improve dynamic controls, cross-entry-point policy parity, recovery, and compatibility records. If iOS is included, validate its native authentication, permissions, bindings, and build independently.

## M4 - Public trial readiness

Packaging, install/uninstall, tested documentation, redacted diagnostics, private security reporting, dependency review, and license selection. No daemon, gateway, background automation, or remote HTTP MCP is added.

See the [task breakdown](task-breakdown.md) for actionable work.
