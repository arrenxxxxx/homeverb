# 2. Work on Core

## Dependency direction

`domain` defines DTOs and argument schemas. `core` depends on domain and defines Adapter, Store, AccountStore, Clock, SecretStore, and host-policy interfaces. `mock`, `storage`, and `secret-store` implement those interfaces. `host` assembles the Mock application and exposes `open_accounts` for trusted native account flows; provider authentication remains pending. CLI/MCP and FFI call Core directly. No entry point invokes another entry point.

Read [repository layout](../repository.md) and [execution semantics](../routing-and-execution.md) before editing the implementation. `crates/host/tests/core_flow.rs` exercises the assembled business logic with isolated SQLite stores and fault-injected adapters.

## Write flow

1. Start the total budget, then validate the envelope, stored descriptor, exact device ID, component, action arguments, and trusted grant. Recheck the deadline before polling adapter work.
2. Acquire the device lock. An existing request ID returns its recorded result or rejects a different command.
3. Select action-supported routes. `lan` and `cloud` are strict; `auto` prefers LAN.
4. Persist a conservative pending result before calling the adapter. The adapter marks possible transmission before sending.
5. Preserve rejection, acceptance, or uncertainty. Only a proven-unsent reachability error can select another route.
6. Apply the declared verification method: fresh matching observations for state matching, or a validated correlated completion from this bounded adapter execution.
7. Persist the outcome and release the lock. Recovery never retransmits an unresolved write.

[Descriptor validation](../../crates/core/src/descriptors.rs) shares the existing device/capability schemas, then checks reference uniqueness and resolution and compiles action parameter schemas locally. Discovery applies it to a complete adapter response before saving that response. Description and operational lookup recheck stored descriptors before use. Keep valid read-only capabilities and multi-component metadata supported; do not infer a common range from a brand. See [descriptor integration tests](../../crates/host/tests/descriptors.rs) for invalid batches, preserved metadata, operational refusal, and valid vendor properties.

Use `Descriptor::capability(component_id, capability_id)` to resolve definitions. A Capability's omitted scope preserves the shared legacy definition; an explicit `component_id` restricts it to one referencing component. Shared/scoped mixing of one ID, duplicate definition keys, nonexistent components, and unresolved references fail admission. Never use the first definition with a matching capability ID alone. [Component tests](../../crates/host/tests/components.rs) exercise two compartments with different ranges, steps, freshness, and grants, through reads, cache, writes, verification, invalid rediscovery, and restart. See [ADR-0018](../adr/0018-component-capability-scopes.md); existing Rust literals add `component_id: None`.

Rediscovery uses `Device::validate_rediscovery` to retain established connection bindings while allowing reordered connections and updated labels/capabilities. The whole adapter response is preflighted before any member is saved. Store implementations must repeat that identity check atomically with each update, as SQLite does; a read followed by an unconditional upsert is vulnerable to concurrent discovery. Binding changes require a future explicit confirmation flow. See [ADR-0013](../adr/0013-preserve-device-connection-identity.md) for limits and compatibility.

Use `resolve_device_id` for local organization/history selectors that need only identity resolution. It preserves exact-ID priority and rejects ambiguous aliases without requiring operational metadata to be valid. It grants no device access; Core still validates descriptors and policy for actual operations. This keeps local history and preferences accessible when rediscovery is needed.

[Observation preparation](../../crates/core/src/observations.rs) is shared by normal reads and write verification. It validates the entire snapshot before caching, then preserves or computes stale flags and warnings. Verification applies additional post-transmission target checks. Keep these paths aligned when adding an adapter; one matching property must never legitimize malformed fields elsewhere in the response. The [fault-injection tests](../../crates/host/tests/observations.rs) cover version, identity, source, clock, cache, partial-state, and acceptance-preservation cases.

The same validation resolves each observation against the requested component's readable property definitions. Property shape comes from the existing capability schema; device-specific type, enum, bounds, and step then validate the value. Tests include invalid unrelated values, metadata changes that invalidate cached state, and read-only vendor properties with their own ranges. Keep the descriptor captured for a write through verification rather than reloading potentially changed metadata mid-operation. No property value is coerced or synthesized. Preparation also resolves each observation's `max_age_ms` from the same validated property descriptor, using 30000 ms only when omitted. The computed limits are shared by ordinary reads and verification. Use deterministic clocks to test equality at expiry, mixed limits, stored stale flags, and post-send requirements; see [ADR-0015](../adr/0015-property-freshness.md). Cache queries apply current limits without rewriting the stored snapshot.

After validation, `matches_target` accepts equal integer/float numbers without converting large integers to potentially rounded floats. Its boundary tests cover signed/unsigned limits and precision beyond 2^53. Host and CLI regressions verify both numeric representations, preserve observed values, reject mismatches, and keep the existing request-ID conflict behavior. Do not introduce a universal epsilon or rewrite cached values to match requested formatting.

For `correlated_completion`, adapters return the optional `CorrelatedCompletion` only after establishing successful protocol completion for the entire command. Core checks the exact local request/device/component/connection binding and timestamp interval. It never interprets an acknowledgement as completion or synthesizes state observations. Mark acceptance before a bounded completion wait so interruption preserves it. Existing adapters use `completion: None`; `wait_for: accepted` requests no extra completion work. See [ADR-0016](../adr/0016-correlated-completion.md) and [completion tests](../../crates/host/tests/completion.rs), including non-idempotent actions with no readable properties and invalid evidence without replay. Real protocol correlation remains an adapter admission requirement.

A non-atomic multi-field action is split only with the adapter's explicit `independent_fields: true` declaration and when every single-field command validates independently. The declaration requires `idempotency: set_value`; even repeatability alone does not establish field independence. Without it, Core passes all arguments together. See [ADR-0012](../adr/0012-explicit-action-decomposition.md) for compatibility and regression coverage. Steps share one deadline and stop after failure, uncertainty, or a storage error. Remaining items are `failed` with `NOT_EXECUTED`. Receipts preserve already known outcomes and verification evidence; storage warnings do not erase them. This is not a transaction across an appliance and SQLite, and there is no automatic compensation.

Validate the caller ID before deriving child receipt IDs. Appending the internal step suffix must not reduce the supported 256-byte input limit. The boundary tests exercise ASCII and multibyte UTF-8 IDs, persisted child receipts, replay after process exit, and rejection of oversized caller inputs. Long device IDs use bounded digest lock filenames while short IDs retain the legacy mapping; see [storage concurrency](../storage.md#concurrency).

Hosts that already started an operation use `Core::execute_with_operation` so setup and selector resolution cannot reset its deadline. The effective deadline is the earlier of the host deadline and the command budget. `Core::execute` remains a convenience for callers starting a new operation. Both paths preserve cancellation and validate before sending.

## How to change behavior

For multiple devices, use `Core::plan_batch` and `Core::execute_batch` from [batch.rs](../../crates/core/src/batch.rs), passing the existing Operation. BatchCommand contains explicit full Commands for 1-32 distinct exact device IDs. Whole-list preflight checks grants, routes, existing requests, and composite child-ID collisions before writes. Groups of at most four futures share the original host budget and return items in input order. Each item reuses ordinary execution, locking, and journaling. Errors on independent devices do not stop the selection; host cancellation/deadline still stops further sends. See [batch tests](../../crates/host/tests/batch.rs) and [ADR-0017](../adr/0017-bounded-device-batches.md). Do not implement a background queue or infer commands from mutable room membership.

Add a failing integration case first. Update the relevant contract and positive/negative fixtures, implement the smallest change, then run formatting, Clippy, Rust tests, runtime acceptance, and contract checks. Keep diagnostics independent of locale; the UI translates stable codes.

Do not add a brand switch to Core or an entry point. Do not add automatic authentication, a periodic refresh task, or a retry that bypasses uncertainty rules. The Linux secret-store library is available to trusted native hosts; unavailable platforms fail closed. Use AccountVault leases for session reads/updates; real provider authentication and rotation still belong to the first real-adapter milestone.

## Trusted write policy

The host injects a `Policy` when constructing Core. A Grant binds an exact device ID, component, capability, action, allowed transports, and an argument schema. No wildcard identity or implicit route permission exists. One grant must accept the entire command; separate field-specific grants cannot be combined to authorize a larger command. The device action schema still applies independently. Malformed or externally unresolved grant schemas add no authority.

For example, a trusted host can permit only a bounded fictional temperature setting over LAN:

```rust
use homeverb_core::Grant;
use homeverb_domain::Transport;
use serde_json::json;

let grant = Grant {
    device_id: "dev-mock-ac".into(),
    component_id: "main".into(),
    capability: "climate".into(),
    action: "set".into(),
    transports: vec![Transport::Lan],
    argument_schema: json!({
        "type": "object",
        "properties": {
            "target_temperature": {"type": "number", "minimum": 20, "maximum": 24}
        },
        "required": ["target_temperature"],
        "additionalProperties": false
    }),
};
```

These bounds describe this example permission, not universal device limits. `auto` may choose only transports authorized for that complete command. Unsent fallback cannot escape the grant. Non-atomic commands require both whole-command and per-step authorization; each step uses their route intersection. A grant requiring both fields may intentionally prevent splitting unless separate matching step grants are also supplied. Dry-run uses the same authorization preflight and includes per-step `items` with their allowed transports.

Grants are trusted host configuration, never ordinary CLI command JSON, native method arguments or MCP tool inputs. Native startup can supply exact Meiju power/fan grants; the Linux CLI can explicitly load the same scopes from an [owner-controlled startup file](../cli.md#scoped-meiju-controls). The Mock host's explicit write option grants `main` power/climate actions over both simulated transports; MCP supplies no grants. Elevated/high-risk actions remain denied. See [ADR-0010](../adr/0010-scoped-write-grants.md) for the unreleased Rust API migration and [policy tests](../../crates/host/tests/write_policy.rs).

Next: [Connect the UI](03-ui-integration.md).

## Local organization and persistence

Core implements room creation/rename/order/delete and device aliases, room assignments, favorites, and ordering. Metadata changes do not alter physical identity, routes, credentials, or grants. Names are trimmed and bounded; room uniqueness uses Unicode lowercasing. Preference updates replace all three editable fields, including explicit null values. Ordering requires every current ID exactly once. See the [native request contract](03-ui-integration.md).

`assign_devices_to_room` changes membership for 1-256 exact device IDs in one storage transaction, preserving all other preferences. The RoomAssignment destination is explicitly nullable, never omitted. Missing references and storage errors roll back the whole selection. `device_executions` reads one device's retained receipts with the device filter applied before the query limit; it neither refreshes state nor retransmits commands.

The storage implementation upgrades schema 1, 2 or 3 to 4 with a verified local backup and exclusive migration access. It retains 100 finalized detailed receipts plus active records, while keeping retired request IDs to prevent replay. The ID ledger can grow. See [storage](../storage.md) and [ADR-0009](../adr/0009-local-organization-and-retention.md). Organization, migration, retention, and fault-injected execution tests exercise these boundaries.

The Meiju native flow now uses the account vault. The next acceptance needs real account identity evidence, production host integration, and read-only device testing. Authorization sharing is governed separately by ADR-0008 and must remain disabled until its contract and security gates are satisfied.

## Inspect recorded outcomes

`Core::execution(request_id, operation)` reads one retained receipt without resolving a device, requiring a write grant, acquiring its lock, calling adapters, or mutating the journal. Pass the exact parent or child request ID; child correlation IDs may exceed the caller-command input limit. Missing and retired IDs have distinct errors. The existing operation controls cancellation/deadline refusal. See [ADR-0014](../adr/0014-local-receipt-lookup.md).

This is useful after a lost acknowledgement or restart. A receipt remains the evidence recorded for that operation; a later state read does not rewrite it or establish causality. Active records may still change while their owning operation runs. Keep query success separate from the nested command outcome.

## Local diagnostics

`Core::diagnostics(&operation)` returns dedicated allowlisted DTOs from local metadata and retained receipts. It shares the caller's budget and checks it between synchronous storage reads. It does not call adapters or the credential store, and it does not mutate the journal. The projection is in [diagnostics.rs](../../crates/core/src/diagnostics.rs); the closed [schema](../../schemas/diagnostics.schema.json) and marker-injection tests define the export boundary. Add export fields deliberately rather than serializing private records wholesale. See [privacy and sampling limits](../security-and-privacy.md#local-diagnostic-reports).
