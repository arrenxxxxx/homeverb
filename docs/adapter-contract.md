# Platform and protocol adapter contract

The Rust `Adapter` trait in [Core](../crates/core/src/lib.rs) is implemented by Mock
and the experimental Meiju cloud adapter. Meiju has separate native account/login
functions; the generic authentication/manifest/subscription interfaces below remain
design pseudocode. Native LAN helpers are not registered device routes; follow the
[current integration boundaries](development/05-midea.md#remaining-route-admission-work).

## Responsibilities

Adapters own viable authentication and pairing, discovery, identity evidence, capability descriptions, state reads, protocol writes, acknowledgement mapping, bounded subscriptions, and cleanup. Core owns common permissions, routing, total budgets, execution state, and log redaction. Action arguments stay together unless the descriptor explicitly declares independent setter fields under [ADR-0012](adr/0012-explicit-action-decomposition.md); an adapter must justify that declaration with protocol evidence.

The following is language-neutral pseudocode, not a compilable interface:

```text
manifest() -> AdapterManifest
begin_auth(AuthRequest, Context) -> AuthChallenge | AuthSession
continue_auth(AuthFlowId, AuthResponse, Context) -> AuthChallenge | AuthSession
refresh_session(AccountRef, Context) -> SessionResult
discover(DiscoveryRequest, Context) -> DiscoveryBatch
describe(DeviceRef, Context) -> DeviceDescriptor
read_state(ReadRequest, Context) -> ObservationSet
execute(ValidatedCommand, SelectedConnection, Context) -> AdapterOutcome
observe(ObserveRequest, Context) -> bounded async stream   [optional]
close(Context) -> CleanupReport
```

Context injects deadline, cancellation, credential access, clock, rate limits, and redacted logging. Do not create unbounded threads or timers. Authentication, reads, and writes all obey the operation budget.

## AdapterManifest

Declare ID, version, platform/protocol, actual authentication methods, supported operating systems, LAN/cloud routes, cloud requirements for onboarding or LAN credentials, external-service requirements, and support evidence status. Implementations with `requires_project_backend` or `requires_daemon` set to true are not admitted.

Region, account type, and exact device model are compatibility conditions, not inherited brand guarantees. Evidence includes model, firmware, vendor app, region, adapter version, test date, verified actions, and known limitations.

## Execution evidence

Report proven-not-sent, sent/possibly-sent, protocol acknowledgement, a request token when available, vendor error classification, and verification suggestions. A network-library exception does not prove that nothing was transmitted. If non-transmission cannot be proved, report `possibly_sent`. Do not label every exception `retryable=true`.

Verification returns real observed values and timestamps. An enqueue acknowledgement maps to `accepted`, not `verified`. Core determines the final outcome and persists attempts.

For an action declaring `correlated_completion`, the bounded `execute` call may return `AdapterOutcome.completion`. The adapter must first validate a successful protocol completion correlated to the entire supplied command. A local request ID or HTTP success alone is insufficient. `CorrelatedCompletion` contains only exact local request/device/component/connection IDs, local receipt time, and an optional trustworthy device completion time. Keep provider correlation tokens and raw responses internal. Core checks bindings and post-attempt chronology before projecting public verification evidence; invalid or missing evidence preserves acceptance. This does not create or refresh a state snapshot.

Mark `WriteProgress::mark_accepted` as soon as acceptance is known, before waiting for completion within the existing Operation. For `wait_for: accepted`, do not start additional completion work. There is no background wait or new subscription API. Rejection and uncertainty cannot be promoted by a conflicting completion object. State-match and unavailable actions ignore it. Existing adapter implementations must initialize the new optional field to `None`; see [ADR-0016](adr/0016-correlated-completion.md).

Every observation returned by one live `read` must identify the selected connection's transport. Return the requested device/component under schema 1.0, with `observed_at <= received_at` when the source timestamp is trustworthy; otherwise use null for `observed_at`. Do not substitute last-change time for observation time. Core rejects future live receipt times and validates the whole snapshot before caching or verifying it, including fields outside the requested write. See [state contract](../schemas/state.schema.json) and [observation tests](../crates/host/tests/observations.rs).

Observation keys use the exact `capability.property` name declared for the requested component. Return only readable properties and values satisfying their declared type, enum, bounds, and step. Convert protocol-specific representations inside the adapter; Core never coerces a string such as `"on"` into a boolean. Read-only vendor properties are supported when declared. Missing values are omitted; nullable property values are not implemented by the current capability contract. Invalid unrelated fields also prevent verification of an otherwise matching target. Declare a property's justified `max_age_ms` when the 30000 ms Mock default is inappropriate. The limit applies to live/cache/verification observations, never creates a refresh timer, and cannot override explicit stale flags or post-send evidence requirements. A state timestamp must still describe an observation, not the last time the value changed.

## Discovery and identity

Core validates every descriptor in one adapter's discovery response before saving any descriptor from that response. Device and capability shapes must satisfy the supported schemas. Component IDs, connection IDs, capability definition keys `(component_id, id)`, and each component's capability references must be unique in their respective scopes. Every reference must resolve, full `capability.property` names must be unambiguous per component, and action parameter schemas must compile locally. Duplicate device IDs within one response and connections claiming another adapter are rejected. Validation failure preserves stored metadata for that batch; this does not provide a transaction across adapters or across later storage failures.

Omit a Capability's `component_id` only when every referencing component uses that same complete definition. Otherwise supply separate scoped definitions for each referenced component. Never mix shared and scoped definitions of one capability ID or depend on list order for selection. Scoped definitions must name an existing component that references that ID; explicit null is invalid. Core resolves action constraints and readable properties consistently through `Descriptor::capability`. See [ADR-0018](adr/0018-component-capability-scopes.md). Component-specific limits still need actual model evidence.

Device description and operational lookup repeat descriptor validation, so malformed persisted metadata cannot authorize a read, dry run, or write. Return `CAPABILITY_UNSUPPORTED` before adapter calls; an explicit valid rediscovery may replace the metadata. Local listing, organization, execution history, and diagnostic projections remain available independently of operational admission. Descriptor validation establishes internal consistency, not physical identity or model compatibility. Core also checks established connection bindings for the whole adapter response before saving any member. An existing device ID cannot acquire a different connection ID, adapter, transport, account, or identity reference through rediscovery; adding or removing a binding also returns `DEVICE_IDENTITY_CONFLICT`. Keep bindings when routes are temporarily unreachable. The separate typed same-identity append operation is defined by [ADR-0052](adr/0052-trusted-connection-linking.md). Adapters default to refusing `validate_connection_link`; opt-in requires current local protected identity/model/route evidence and subsequent discovery that retains every linked connection. Do not network or mutate credentials during validation. User confirmation does not establish that evidence. Cross-account/provider merging and binding replacement remain unsupported. The ordinary guard remains [ADR-0013](adr/0013-preserve-device-connection-identity.md).

LAN scans have deadlines and user-authorized network boundaries; never scan the public internet. A URL or IP returned by a discovered device is untrusted input. Validate allowed networks and vendor endpoints to prevent malicious responses from triggering arbitrary requests. Cloud enumeration is account/region scoped, paginated, rate-limited, and deduplicated.

## Plugin packaging

Version one uses compile-time registration and explicit feature switches. Mobile packages contain reviewed adapters. Do not promise runtime code downloads on every mobile platform. A later desktop-specific bounded subprocess mechanism may be evaluated separately, but reusing a Python library must not require a companion service.

## Definition of done

Authentication feasibility evidence -> fictional/redacted fixtures -> contract tests -> one physical device in read-only mode -> owner-authorized safe writes -> verification and fault cases -> compatibility matrix update. Without a real device, use `experimental` or `mock-tested`, not `verified`.

Libraries such as `midea-local` can inform protocol work. Citing one does not establish a port, mobile compatibility, or support for every model. [R5](references.md)
