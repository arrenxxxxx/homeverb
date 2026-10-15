# Documentation index

Start with the progressive development path. Reference documents define invariants and future requirements; the [task table](task-breakdown.md) records what is actually implemented.

## Development path

| Step | Guide | Outcome |
|---|---|---|
| 1 | [Run the Mock MVP](development/01-run-mock.md) | Build, control, inject faults, verify |
| 2 | [Work on Core](development/02-core.md) | Understand responsibilities and execution invariants |
| 3 | [Connect the UI](development/03-ui-integration.md) | In-process requests, cancellation, en-US/zh-CN |
| 4 | [Add a real adapter](development/04-add-adapter.md) | Authentication gate, contract tests, physical evidence |
| 5 | [Integrate Midea/Hualing](development/05-midea.md) | Account-device retrieval, direct cloud control, additional LAN routes, and physical acceptance |
| 6 | [Verify a Meiju account locally](development/06-meiju-probe.md) | Hidden-input read-only login, protected session and automatic inventory |
| 7 | [Preview a Meiju AC control](development/07-meiju-control.md) | Exact-device power grant, tracked cloud outcomes and separately authorized physical acceptance |

## Reference

- Product: [requirements](product-requirements.md), [architecture](architecture.md), [lifecycle](process-lifecycle.md).
- Contracts: [domain](domain-model.md), [capabilities](capabilities.md), [adapters](adapter-contract.md), [routing/execution](routing-and-execution.md), [errors](errors.md), [schemas](../schemas/README.md).
- Entry points: [CLI](cli.md), [native SDK](sdk.md), [mobile requirements](mobile-app.md), [MCP](mcp.md).
- Flutter handoff: [versioned native contract](native-ui-contract.md), [callable catalog and examples](../packages/native-contract/README.md), [native contract decision](adr/0019-native-ui-contract.md).
- UI delivery: [Window Light prototype](../output/prototypes/homeverb-window-light-v1/index.html), [functional handoff](ui-functional-spec.md), [82-state inventory](ui-screen-inventory.md), [Flutter visual system and one-skin extension](ui-window-light.md).
- Persistence/security: [storage](storage.md), [authentication](authentication.md), [native account vault](adr/0021-persistent-account-vault.md), [local account removal](adr/0028-local-account-removal.md), [saved-session recovery](adr/0030-explicit-meiju-account-recovery.md), [pending-account reauthorization](adr/0031-pending-account-reauthorization.md), [cache/history clearing](adr/0029-local-cache-and-history-clearing.md), [security/privacy](security-and-privacy.md).
- Delivery: [repository layout](repository.md), [testing](testing.md), [task status](task-breakdown.md), [roadmap](roadmap.md), [compatibility](compatibility.md), [commit conventions](commit-conventions.md), [private publishing](publishing.md).
- Evidence: [references](references.md), [fictional examples](../examples/README.md), [diagrams](diagrams/README.md).

## Decisions

[No daemon](adr/0001-no-daemon.md) | [Embedded core](adr/0002-embedded-core.md) | [Capability first](adr/0003-capability-first.md) | [Uncertain writes](adr/0004-uncertain-writes.md) | [Native authentication](adr/0005-native-auth.md) | [Project name](adr/0006-project-name.md) | [Mock implementation and composite receipts](adr/0007-mock-mvp.md) | [Foreground authorization sharing](adr/0008-foreground-authorization-sharing.md) | [Local organization and receipt retention](adr/0009-local-organization-and-retention.md) | [Scoped write grants](adr/0010-scoped-write-grants.md) | [Native secret storage](adr/0011-system-secret-store.md) | [Explicit action decomposition](adr/0012-explicit-action-decomposition.md) | [Rediscovery identity](adr/0013-preserve-device-connection-identity.md) | [Local receipt lookup](adr/0014-local-receipt-lookup.md) | [Property freshness](adr/0015-property-freshness.md) | [Correlated completion](adr/0016-correlated-completion.md) | [Bounded device batches](adr/0017-bounded-device-batches.md) | [Component capability scopes](adr/0018-component-capability-scopes.md)
