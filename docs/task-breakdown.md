# Actionable task breakdown

Each task should fit a small pull request. T00 is delivered as a documentation baseline; every application task is `planned` unless explicit implementation evidence is added.

| ID | Task | Dependencies | Acceptance criteria |
|---|---|---|---|
| T00 | Documents, diagrams, contracts, repository templates | None | Documentation and contract checks pass |
| T01 | Select language/toolchain/layout; create a compilable workspace | T00 | At least one real executable unit test; toolchain recorded |
| T02 | Device/Capability/Command/Execution DTOs | T01 | Positive/negative, unit, range, and version tests |
| T03 | Mock adapter and controlled clock/network | T02 | Reproducible reads, writes, rejection, lost ACK, cancellation |
| T04 | Core execution state machine and routing | T03 | No duplicate uncertain writes; hard route policy |
| T05 | SQLite, secret storage, refresh locks | T02 | No plaintext credentials; no crash replay |
| T06 | CLI queries, capabilities, writes, exit codes | T04,T05 | Stable JSON; commands exit without a daemon |
| T07 | Minimal Android Flutter in-process probe | T02,T03 | Same Core on a physical phone; cancellation/lifecycle verified |
| T08 | Local MCP stdio reads and allowlisted writes | T04,T05 | Protocol-only stdout; EOF exit; default denial of writes |
| T09 | First adapter authentication feasibility | T00 | Evidence of no-project-backend authentication, or blocked |
| T10 | First real-device discovery/read/authorized write | T04,T05,T09 | Model-specific hardware and verification evidence |
| T11 | App dynamic device controls | T07,T10 | Independent control without any project service |
| T12 | Second heterogeneous adapter | T10 | Same capability calls; no brand branches in UI |
| T13 | Credentials, policy, diagnostics, supply-chain review | T06,T08,T11 | Security cases and documented residual risks |
| T14 | Packages and public release readiness | T12,T13 | Platform matrix, license decision, actual CI success |

## First implementation tasks

Start with T01, T02, T03, and a minimal T07 probe. Do not implement every platform or build an HTTP service first. T09 can run alongside prototype work; authentication research must not bypass the boundary through a "temporary" backend.

## PR completion record

Task ID; changed files; interface changes; commands actually executed and results; skipped tests and reasons; physical-device evidence; security impact; documentation and compatibility updates. Write the entire record in English and use a Conventional Commit PR title.

## Blockers

Record precise reasons for native-authentication incompatibility, missing authorized accounts, unavailable hardware, or unresolved upstream licensing. Mock and contract work can proceed, but do not claim support. A stack change updates the architecture and ADR first; avoid indefinitely maintaining two separate cores.
