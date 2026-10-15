# ADR-0007: Implement the embedded Mock MVP and preserve composite receipts

Status: accepted for the requested framework and verifiable MVP implementation.

## Decision

Use a Rust 1.96 Cargo workspace for domain, Core, simulator, SQLite storage, host composition, and the CLI. Add a small C ABI for in-process native UI integration. Use the owner-selected Flutter framework for mobile UI; phone integration still requires prototype evidence. Supply framework-independent en-US/zh-CN catalogs and presentation helpers so visual UI work can proceed separately.

No vendor adapter or credential store is simulated as production support. The Mock adapter is explicitly selected, operates only on fictional data, and uses a separate SQLite file to model appliance state independently of the Core journal.

## Composite receipt clarification

The design schema previously allowed `items` only for `partial`. A successful non-atomic command also needs per-item evidence. Keep the already-defined optional `items` field for any aggregate outcome, require it for `partial`, and preserve `verified` evidence on each successful item. All verified items produce `verified`; all accepted items produce `accepted`; all failed items produce `failed`; differing outcomes produce `partial`.

This broadens the version 1.0 schema's accepted result shapes without removing or renaming fields. Consumers using the earlier baseline validator must update it to accept successful composites. No earlier runtime was released. A composite verification summary describes separate item observations and makes no atomicity or causality claim.

## Consequences

The prototype is testable without hardware. Linux ABI validation does not establish mobile support. Real devices still require native authentication, secure credentials, model-specific tests, and explicit owner authorization for writes. Successful Mock outcomes must remain visibly labeled as simulated.

The optional `warnings` array records persistence failures that occur after verification or while a request is already uncertain. It preserves evidence and the original physical outcome instead of turning a storage problem into a misleading device failure. Earlier strict validators must load the updated schema.
