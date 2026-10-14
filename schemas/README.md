# JSON Schema contracts

The schemas use JSON Schema Draft 2020-12. They are homeverb design contracts, not official vendor protocols.

| Schema | Purpose |
|---|---|
| [device.schema.json](device.schema.json) | Devices, components, multiple connections |
| [capability.schema.json](capability.schema.json) | Property and action descriptions |
| [command.schema.json](command.schema.json) | Command structure |
| [command-result.schema.json](command-result.schema.json) | Execution outcomes and evidence invariants |
| [adapter-manifest.schema.json](adapter-manifest.schema.json) | Adapter metadata; daemon/backend requirements forbidden |
| [state.schema.json](state.schema.json) | State with source, timestamps, and cache semantics |

After validating a Command envelope, validate `arguments` using the target `capability.actions[action].parameters`. Core still enforces permissions, unit conversions, conditional constraints, identity relationships, timestamp trust, idempotency, and networking. JSON Schema cannot establish these business or physical guarantees.

Counterexamples include verified outcomes without evidence, retryable unknown writes, required daemons, and out-of-range temperatures. Embedded action parameter schemas are also checked as valid schemas.
