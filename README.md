# HomeVerb

**Cross-brand appliance control without a persistent service. One embedded core, three independent entry points: CLI, mobile app, and local MCP.**

> **Status: design baseline v0.1.1-docs, October 14, 2026.** HomeVerb is the owner-approved project name. This repository contains design documents, contract examples, architecture diagrams, and validation tooling. **It does not contain a working appliance-control application.** All `homeverb` commands are proposed interfaces. Every real-device adapter remains unimplemented and unverified.

## Project identity

**One language for your appliances. No daemon required.**

Use **HomeVerb** for the project and mobile app, and `homeverb` for the repository and CLI executable. Proposed modules use the `homeverb-` prefix. See the [naming decision](docs/adr/0006-project-name.md).

## What this project solves

Connect platform accounts and appliances from a single client, translate vendor protocols into common capabilities, and control devices from a terminal, a phone, or a local AI host. Users do not deploy Home Assistant, a NAS gateway, Docker services, or a project-operated backend.

A shared core means shared **code**, not a shared central service. Each entry point loads its own Core instance and performs authentication, discovery, routing, execution, and verification inside its process. Direct access to vendor clouds is allowed; universal offline operation is not promised.

## Product boundaries

| Required | Explicitly excluded |
|---|---|
| One-shot CLI commands that exit when complete | Daemons, keep-alive workers, and startup services |
| An embedded mobile core with foreground, on-demand networking | Background phone gateways and central control servers |
| A local MCP stdio process owned by its host session | Remote HTTP MCP and public project APIs |
| Common devices, capabilities, and actions | Automation engines and unattended scheduling |
| Direct LAN or vendor-cloud connections | Project-operated relays and hosted credentials |
| Local configuration and timestamped state snapshots | Automatic cross-device sync and continuous collection |

An appliance continuing an already-started wash or cleaning cycle is not a project background task. Continuously evaluating conditions and issuing new commands is automation and is out of scope.

## Architecture

![Independent embedded-core architecture without a persistent service](docs/diagrams/architecture-overview.svg)

[Mermaid source](docs/diagrams/architecture-overview.mmd) | [PNG preview](docs/diagrams/architecture-overview.png) | [Architecture details](docs/architecture.md)

## Proposed usage - not implemented

```bash
homeverb auth login midea --profile home
homeverb device discover --profile home --connection auto --timeout 10s
homeverb device list --json
homeverb device describe bedroom-ac --json
homeverb device state bedroom-ac --fresh --json
homeverb climate set bedroom-ac --temperature 26 --mode cool --connection lan --json
homeverb power set bedroom-ac --off --connection auto --json
homeverb mcp --stdio --profile home
```

Authentication, capability ranges, and control paths require model-specific evidence. Unreachable LAN-only devices must return a clear error; no implicit relay is created. Cloud adapters that cannot meet the native-client authentication requirement are not admitted.

## Reading order

[Documentation index](docs/index.md) -> [Product requirements](docs/product-requirements.md) -> [Architecture](docs/architecture.md) -> [Capabilities](docs/capabilities.md) -> [Adapter contract](docs/adapter-contract.md) -> [Task breakdown](docs/task-breakdown.md).

AI contributors must read [AGENTS.md](AGENTS.md). The requested [agent.md](agent.md) filename is retained as a pointer to the canonical instructions.

## What is available now

Design documents, fictional JSON fixtures, JSON Schemas, Mermaid/SVG/PNG diagrams, issue and pull-request templates, and local documentation/contract checks. There is no Cargo workspace, Flutter application, control binary, or real account configuration.

```bash
python3 scripts/validate_docs.py
python3 -m pip install -r requirements-dev.txt
python3 scripts/validate_contracts.py
```

The first check only needs Python's standard library. Passing these checks does not establish real-device compatibility.

## Repository policy

All repository documentation, diagram labels, examples, issue/PR templates, and commit messages are written in English. Use Conventional Commits such as `feat(core): ...`, `fix(adapter): ...`, `docs: ...`, and `test: ...`; see [Commit conventions](docs/commit-conventions.md).

Read [CONTRIBUTING.md](CONTRIBUTING.md), [SECURITY.md](SECURITY.md), the [test plan](docs/testing.md), and the [roadmap](docs/roadmap.md). The project license has not been selected; see [License decision](LICENSE-DECISION.md). [Private publishing](docs/publishing.md) describes the authenticated GitHub upload procedure. A local checkout or this documentation package is not evidence that a remote repository has been created.
