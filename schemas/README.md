# JSON Schema contracts

The schemas use JSON Schema Draft 2020-12. They are HomeVerb runtime contracts, not official vendor protocols. Runtime acceptance validates actual CLI results against these schemas.

| Schema | Purpose |
|---|---|
| [device.schema.json](device.schema.json) | Devices, components, multiple connections |
| [room.schema.json](room.schema.json) | Local rooms and display ordering |
| [room-assignment.schema.json](room-assignment.schema.json) | Bounded distinct device selection and explicit nullable destination |
| [device-preferences.schema.json](device-preferences.schema.json) | Nullable alias/room, favorite, and card ordering |
| [diagnostics.schema.json](diagnostics.schema.json) | Closed allowlist for local counts, bounded receipt summaries, and reviewed error codes |
| [capability.schema.json](capability.schema.json) | Property and action descriptions |
| [command.schema.json](command.schema.json) | Command structure |
| [cli-meiju-grants.schema.json](cli-meiju-grants.schema.json) | Explicit trusted Linux CLI startup permissions for exact local devices and power, fan, mode activation and exact temperature values |
| [batch-command.schema.json](batch-command.schema.json) | Explicit commands for 1-32 distinct devices; Core also enforces identity and request-ID uniqueness |
| [batch-result.schema.json](batch-result.schema.json) | Ordered per-device receipts and aggregate status without a fabricated device identity |
| [command-result.schema.json](command-result.schema.json) | Execution outcomes and evidence invariants |
| [adapter-manifest.schema.json](adapter-manifest.schema.json) | Adapter metadata; daemon/backend requirements forbidden |
| [state.schema.json](state.schema.json) | State with source, timestamps, and cache semantics |
| [native-open.schema.json](native-open.schema.json) | Trusted Mock/Meiju startup configuration and optional exact AC permissions and elevated dishwasher arguments |
| [native-approval.schema.json](native-approval.schema.json) | Separate trusted approval prepare/execute inputs and result definitions; no ordinary dispatch method |
| [native-request.schema.json](native-request.schema.json) | Closed host envelope and all 31 callable method inputs |
| [native-response.schema.json](native-response.schema.json) | Response bundle; select `$defs/<method>` or `$defs/open`, preserving query/admitted-execution distinctions |

After validating a Command envelope, validate `arguments` using the target `capability.actions[action].parameters`. Core still enforces permissions, unit conversions, conditional constraints, identity relationships, timestamp trust, idempotency, and networking. JSON Schema cannot establish these business or physical guarantees.

Counterexamples include verified outcomes without evidence, retryable unknown writes, required daemons, and out-of-range temperatures. Embedded action parameter schemas are also checked as valid schemas.

The CLI Meiju grant lists match native startup definitions exactly; contract
validation checks their parity. Schema validation cannot establish file ownership
or permissions; the Linux CLI checks these before opening a profile.

Batch schemas embed the existing Command/CommandResult definitions for offline self-contained validation. `validate_contracts.py` checks exact parity, with recursive execution references rebased into the batch schema. Update these definitions together when their underlying contracts change. Schema validation cannot prove the batch's preflight, identity uniqueness, shared budget, or concurrency behavior; integration tests cover those rules.

Native schemas reference shared definitions using `urn:homeverb:schema:<name>`. Resolve them with [native_contract.py](../scripts/native_contract.py), which registers only local files and rejects external retrieval. Run `python3 scripts/verify_native_contract.py` after building the native library; it validates actual method responses and the shared examples. The response file is a definition bundle, not a catch-all root validator. See the [Flutter contract](../docs/native-ui-contract.md).

The Meiju membership correction keeps existing response fields. The shared receipt
description and [not-listed fixture](../examples/command-result.meiju-not-listed.json)
document a failed/not-sent outcome. Schema validation does not prove inventory
membership; the account lease and protected payload enforce it. Native account
examples include both membership errors, and negative fixtures reject caller
membership claims and evidence-free promotion to verified.
