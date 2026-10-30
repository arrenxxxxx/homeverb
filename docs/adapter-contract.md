# Platform and protocol adapter contract

The Rust `Adapter` trait in [Core](../crates/core/src/lib.rs) is implemented by Mock,
experimental Meiju and experimental read-only Mijia cloud adapters. Meiju and
Mijia have separate trusted Rust account/login functions; Mijia C/App entry points
remain pending. The generic authentication/manifest/subscription interfaces below remain
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


## Historical numeric inspection and read contract

ADR-0089 supplies additive typed `history_options` and `read_history` hooks.
Adapters default to no options and unsupported reads. Local option inspection must
avoid networking and derive target, unit and bounds from protected exact-profile
bindings. Reads independently repeat identity/model/connection admission and
retain account leases for the bounded request. A single explicit connection is
used without fallback or paging. Returned historical aggregates require Core
validation; public metadata alone cannot admit another model or protocol key.
The first exact protocol candidate is [daily energy](adr/0088-mijia-exact-daily-energy-history.md).

ADR-0105 extends these hooks with numeric `reported_samples`. Derive the exact
pinned field's numeric domain independently of public descriptor claims; retain
signed values and the original unit, including null when absent. Return separate
options when one target supports daily aggregates and samples. Core independently
checks descriptor type/domain/unit, request identity and received timestamps.
Mijia binds the request to the protected selected legacy/modern protocol and root
identity, holds the account lease through the reply, and never falls back, renews,
writes cache/receipts or mutates saved credentials during historical reads.

ADR-0106 admits four exact hood manual/display fields through the existing
reference-setting family. Preserve the complete pinned model/service/property/
type/unit/access/domain evidence. Fan/outlet opt-ins cannot enable them. Historical
illumination-only mappings are accepted only as a complete known derivation;
partial new-domain mixtures refuse. Inspection may upgrade a temporary mapping
without changing saved authority; explicit refresh performs the protected upgrade.


## Event occurrence history contract phase

[ADR-0116](adr/0116-bounded-event-history.md) defines a separate domain/schema
contract for occurrence-time inspection and reads. It does not yet add Adapter
hooks. The forthcoming default is empty inspection and unsupported reads. Opt-in
adapters must independently derive event support from protected bindings, retain
account leases through one request and admit event-only service components during
explicit descriptor refresh without inventing state. Public event metadata never
substitutes for protected evidence. Numeric history and existing adapters are
unchanged in this contract-only phase.


## Implemented typed event-history hooks

ADR-0116 adds event_history_options and read_event_history Adapter hooks. Default
inspection is empty and reads are unsupported. Core requires a declared event with
history.projection=occurrence_times and validates every option and returned row.
Mijia's explicit typed opt-in derives namespace/service/event identities from the
pinned catalog and retains account leases through exact selected-protocol requests.
Old saved scalar layouts require explicit refresh before event options appear;
complete event-enriched layouts remain valid for ordinary scalar reads even when
a host disables event history. No native/App entry point is added in this phase.

## Native event-history composition

ADR-0117/API 1.44 exposes the implemented event hooks through ordinary Session
queries. MijiaSessionOptions.event_history and the trusted native startup Boolean
mijia_event_history default to false; enabling them changes no write policy.
Only explicit connect/refresh publishes enriched descriptors. Existing constructors
remain compatible, and disabled hosts preserve ordinary scalar reads for complete
enriched layouts. Flutter and CLI/MCP entry points remain separate adoption work.

## CLI/MCP event-history composition

ADR-0118's explicit open_mijia_read_only_with_events composition installs reviewed
event history and both protected legacy/modern transports with default read-only
policy. Existing open_mijia_read_only preserves its legacy-only behavior. CLI/MCP
select the new composition; explicit CLI account connect/refresh opts into event
publication. No constructor reads credentials, logs in, refreshes accounts or
schedules work. Adapter identity/model/membership/protocol checks and Core
occurrence-time validation remain the same as native/App event reads.

## Exact hood preferences

ADR-0119 adds five independent protected xiaomi.hood.cyp10 preference targets to
the reference-setting family: power_on_with_light_9, power_off_with_light_10,
clean_remind_on_11, clean_remind_time_12 and gestures_18 on hood-2. The first,
second, third and fifth are Boolean; the threshold is exactly 20/30/40/50 hours.
Preserve exact service/property/type/unit/access evidence. One set-value request
never enables its companion or invokes a cleaning/power action. No physical
verification or complete model support follows from an accepted response.
