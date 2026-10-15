# Documentation index

Design baseline v0.1.1-docs, October 14, 2026. Application features remain planned. Current executable content is limited to documentation, contract, and repository checks and an authenticated publishing helper.

| Document | Contents |
|---|---|
| [product-requirements.md](product-requirements.md) | Positioning, journeys, scope, and exclusions |
| [architecture.md](architecture.md) | Runtime, dependencies, and technology proposal |
| [process-lifecycle.md](process-lifecycle.md) | No-daemon definition and cleanup rules |
| [domain-model.md](domain-model.md) | Accounts, devices, components, connections, observations |
| [capabilities.md](capabilities.md) | Common capabilities and vendor extensions |
| [adapter-contract.md](adapter-contract.md) | Authentication, discovery, execution, evidence |
| [authentication.md](authentication.md) | Native-client admission and on-demand sessions |
| [routing-and-execution.md](routing-and-execution.md) | Route policy, retries, verification, uncertain writes |
| [errors.md](errors.md) | Structured errors and outcome invariants |
| [cli.md](cli.md) | Commands, JSON, exit codes |
| [mobile-app.md](mobile-app.md) | Flutter screens, custom rooms and cards, appliance platforms, Settings sharing, lifecycle |
| [Window Light prototype](../output/prototypes/homeverb-window-light-v1/index.html) | Complete visual atlas: 82 screen states, 14 journeys, 198 branches |
| [ui-functional-spec.md](ui-functional-spec.md) | Functional coverage, recovery paths, integration snapshot and acceptance checks |
| [ui-screen-inventory.md](ui-screen-inventory.md) | Every screen ID, visible content and review destination |
| [ui-window-light.md](ui-window-light.md) | Flutter rendering strategy and one-skin extension contract |
| [mcp.md](mcp.md) | Local stdio tools, permissions, lifecycle |
| [sdk.md](sdk.md) | Embedded Core interfaces and FFI |
| [storage.md](storage.md) | SQLite, secret stores, concurrency, recovery |
| [security-and-privacy.md](security-and-privacy.md) | Threats, authorization, redaction, supply chain |
| [compatibility.md](compatibility.md) | Candidate platforms and model-specific evidence |
| [testing.md](testing.md) | Current checks, future tests, release gates |
| [roadmap.md](roadmap.md) | M0-M4 milestones |
| [task-breakdown.md](task-breakdown.md) | T00-T15 tasks and acceptance criteria |
| [repository.md](repository.md) | Layout and Git workflow |
| [commit-conventions.md](commit-conventions.md) | English commit types, scopes, labels, and checks |
| [publishing.md](publishing.md) | Private GitHub creation and safe upload procedure |
| [faq.md](faq.md) | Common questions and scope clarification |
| [references.md](references.md) | Primary sources and evidence boundaries |

## Diagrams and contracts

[Architecture SVG](diagrams/architecture-overview.svg) | [PNG preview](diagrams/architecture-overview.png) | [Mermaid source](diagrams/architecture-overview.mmd) | [Command sequence](diagrams/command-sequence.mmd) | [JSON Schemas](../schemas/README.md) | [Fictional examples](../examples/README.md)

## Architecture decisions

- [ADR-0001: No persistent service or project backend](adr/0001-no-daemon.md)
- [ADR-0002: Embed the same core in all three entry points](adr/0002-embedded-core.md)
- [ADR-0003: Unify capabilities; integrate platforms and protocols](adr/0003-capability-first.md)
- [ADR-0004: Represent uncertainty in physical writes](adr/0004-uncertain-writes.md)
- [ADR-0005: Native-client authentication is an admission gate](adr/0005-native-auth.md)
- [ADR-0006: Adopt HomeVerb as the project name](adr/0006-project-name.md)
- [ADR-0008: Allow explicit foreground authorization sharing on a local network](adr/0008-foreground-authorization-sharing.md)

## Repository entry points

[README](../README.md) | [AI contributor rules](../AGENTS.md) | [agent.md](../agent.md) | [Contributing](../CONTRIBUTING.md) | [Security reporting](../SECURITY.md)
