# Routing and execution semantics

The owner-requested [one-shot Meiju compatibility experiment](adr/0075-one-shot-meiju-read-compatibility-probe.md)
is disabled in ordinary builds. An explicitly compiled development library may
append at most two fixed authenticated reads to foreground legacy operations.
They inherit the caller's cancellation/deadline, never retry, and never update
inventory, cached observations, permissions, routes or execution receipts.
This is not an alternate control route or uncertain-write recovery path.

API 1.24 routes account operations by registered provider and saved account ID
under [ADR-0077](adr/0077-mijia-native-account-routing.md). Combined sessions keep
one Core policy with exact Meiju grants and read-only Mijia admission. Positive
Mijia partial inventory publishes cards with explicit partial status. Ordinary
requests cannot change provider registration or authority. Device execution and
uncertain-write semantics are unchanged.

The Flutter account flow now selects a registered provider for protected sign-in
and uses the saved provider for reauthorization. Unsupported control-option
inspection does not prevent read-only device views; it supplies no grants. Mijia
primary appliance components precede auxiliary batteries so card summaries read
the appliance service. Unknown fields, raw lock codes and stale cloud evidence
retain their existing meaning; no Mijia write is added by this App phase.

## Experimental Mijia Core reads

[ADR-0076](adr/0076-mijia-protected-core-reads.md) adds a read-only cloud adapter
with protected account/model/component/connection checks before each property
request. Positive partial inventory can publish named cards through explicit
account connection/refresh. Generic discovery refuses populated Mijia accounts
before networking because it cannot represent partial coverage. Missing historical
bindings remain retained; no complete-inventory timestamp is invented.

Reads hold the account lease through one bounded cloud operation and correlate
every returned device/SIID/PIID. Snapshots carry cloud source, local receipt time,
no trustworthy observation time and stale values; Core still validates the entire
snapshot before replacing cache. Invalid replies preserve prior readings.
There is no automatic login, route fallback or background worker. The default
read-only adapter refuses writes even with forged public actions and Core grants.
Native API, snapshot schemas and existing execution outcome semantics are unchanged.

## Mijia write protocol foundation

The low-level Rust protocol uses a separate progress-tracked, single-property
transport. Default/read transports refuse controls before sending. The first
possible send records `possibly_sent`; only an encrypted, exact single
DID/SIID/PIID acknowledgement can return accepted or explicitly rejected. Global
codes, malformed replies, cancellation and transport failures after send remain
uncertain. Accepted acknowledgements have no completion evidence. No retry,
reauthorization or route replay occurs.

[ADR-0078](adr/0078-mijia-fan-control.md) adds explicit trusted Rust fan admission.
Core grants remain independently required. The adapter selects wire properties
only from the protected original model/spec, validates exact route and arguments,
and retains the account lease through the operation. Public descriptor edits
cannot widen the protected domain or add lock/doorbell writes. Local control-option
inspection grants no authority. Native API 1.25 accepts exact `mijia_fan_grants` only at trusted Mijia/combined
startup. Default sessions remain read-only; ordinary JSON cannot edit grants.
Option inspection permits protected `fan-SIID` component scopes. Existing outcome
schemas and semantics are unchanged.

## Route policy

`connection=lan` permits LAN only, with no cloud fallback. `cloud` permits vendor-cloud control only. `auto` chooses among action-supported routes with valid credentials, user permission, and current reachability; prefer LAN by default. A readable LAN route is not automatically writable for every action. Route by action, not merely by device.

Read/discovery operations may race safe routes within one shared budget. **Never race a write over LAN and cloud.** After an uncertain write on the primary route, do not duplicate it on a backup route.

The App-owned [foreground state timer](adr/0067-foreground-state-refresh.md) issues
new bounded reads using this same route policy. Its three-second ticks do not
overlap or enqueue retries. User operations cancel and drain automatic reads
before planning; backgrounding and navigation fence delayed replies. A silent
read failure preserves prior observations, while successful reads retain source
freshness evidence and never rewrite accepted/unknown receipts or replay a write.
Core/adapter execution semantics and request/response schemas are unchanged.

Write routes must also satisfy the trusted grant for the complete command, including component and argument bounds. Explicitly requesting an ungranted route returns `POLICY_DENIED`. `auto` cannot use an ungranted route, even after a proven-unsent failure. Each non-atomic step must independently satisfy a complete grant and use only transports allowed for both parent and step. Dry-run performs this same preflight and returns per-step candidate transports. See [trusted policy](development/02-core.md) and [ADR-0010](adr/0010-scoped-write-grants.md).

Native `inspect_session_grants` projects current local device/transport rules
without credentials, adapters or mutation. Saved actions can outlive their prior
session's grants; inspect current policy after reopening. Neither returned rules
nor control-option definitions replace the execution gates above. See
[ADR-0057](adr/0057-session-grant-inspection.md).

## Trusted native connection linking

[ADR-0052](adr/0052-trusted-connection-linking.md) adds typed Rust
`prepare_connection_link` / `confirm_connection_link` to Core and Session. This
local metadata operation appends one adapter-admitted route to an exact device,
anchored to an existing connection with identical adapter/account/identity
references. It cannot merge accounts, replace bindings or change capabilities.
Ordinary discovery still refuses connection-set changes.

Show the immutable descriptor, anchor, proposed connection and
`transport_grants()` before trusted user consent. These are existing potential
grants for the new transport, not new permission or physical compatibility.
The new route participates in automatic reads and may activate existing matching
write grants; all action, argument, risk and transport checks still apply.
Cloud-only authority cannot authorize LAN writes.

The preview expires after 60 monotonic seconds and is consumed on every confirm
attempt. Core checks its instance/generation, locks the device, revalidates the
effective descriptor and adapter evidence, then checks cancellation, deadline
and preview again before transactional storage admission. Background/close
invalidates previews and cancels active operations. Dropping a preview dismisses
it. Adapter validation is asynchronous within the inherited Operation;
confirmation caps its budget at preview expiry. An in-flight wait reaching that
cap returns `DEADLINE_EXCEEDED`. A commit already admitted is not rolled back by later cancellation; inspect
local state after interruption. No appliance request, retry or execution receipt
is generated. Mock/Meiju do not opt in; only synthetic tests exercise linking.
API 1.20 exposes separate trusted C prepare/confirm/discard functions under
[ADR-0061](adr/0061-native-connection-link-bridge.md). Ordinary JSON, CLI and MCP
still cannot link. The session retains at most eight previews; background/close
clears them. Confirmation consumes the ID before decoded request admission,
without broadening adapter evidence, grants or Core's transactional rules.

## Reviewed dishwasher cycle operations

[ADR-0055](adr/0055-profile-scoped-dishwasher-cycle-controls.md) admits separate
`pause_cycle`, `resume_cycle` and `cancel_cycle` actions only for reviewed protected
products with no centralized settings to preserve.
[ADR-0063](adr/0063-explicit-dishwasher-cycle-options.md) additionally admits product
760Y0026 with an explicit `additional` choice (0, 9, 10, 13 or 14). Its grant and
immutable preview bind that exact value; never infer it from cached state.
Omitted grant arguments mean exactly `{}` and grant no additional-option operation.
Each requires its own exact
cloud grant and trusted single-command confirmation, remains elevated and
non-idempotent, and independently checks the approval marker in the adapter.
Program-start permission and these three permissions do not imply one another.

Send one action-specific cloud payload with no implicit state read, settings copy,
program selection, retry, fallback or optimistic observation. Preserve accepted,
failed and unknown outcomes and request-ID replay protection. Pause/cancel are
protocol requests, not safe-door, completed-drain or physical rollback guarantees.
Optional raw `operator` observations retain cloud staleness; neither a returned
operator nor `work_status` proves command causality. All existing execution,
approval expiry and foreground lifecycle semantics remain in force.

## Trusted foreground LAN candidate scan

[ADR-0054](adr/0054-trusted-meiju-lan-candidate-scan.md) composes discovery and
protected reconciliation for one saved Meiju device through typed Rust Session
methods. The host selects an assigned interface and handles platform permission.
Preparation validates exact device/connection IDs and protected mainland account
membership without networking. Its immutable, non-cloneable preview exposes the
device/network, belongs to one Session/lifecycle generation and expires after
60 monotonic seconds. Actual trusted consent is required before consuming it.

Confirmation rechecks metadata, holds the account lease through one collector
invocation, and revalidates protected identity/membership and current metadata
before returning. The inherited budget is capped by preview expiry; the production
collector retains its five-second window and two fixed probes. Cancellation,
background, close and dropped futures release operation resources; background and
close also invalidate outstanding previews. There is no automatic rescan.

`LanScanOutcome` returns only the selected candidate or `NotObserved`, `Incomplete`
or `Ambiguous`. Native redacted/nonserializable hints establish no authenticated
endpoint, model, credential or route authority. This flow performs no cloud
request, token lookup, TCP/state query, connection mutation or write. Ordinary
JSON, C/JNI, CLI and MCP do not expose it. Mobile permission and consent integration
remain pending; no physical acceptance follows from fictional integration tests.

## State machine

```mermaid
flowchart LR
  V[Validate] -->|Valid| P[Prepare]
  V -->|Invalid| F[failed]
  P -->|Proven unsent failure| F
  P -->|Transmit| S[Sent]
  S -->|Explicit acceptance| A[accepted]
  S -->|Explicit rejection| F
  S -->|Acknowledgement lost| U[unknown]
  A -->|Fresh evidence matches target| OK[verified]
  U -->|Later qualifying evidence| OK
  A -->|Verification deadline| A
```

A later observation after `unknown` may prove that the target is now satisfied without proving this request caused it. Record `verification.kind=state_match` and that limitation. Do not promise exactly-once execution without causal evidence. A pure action with no correlated completion receipt or observable state remains `accepted` or `unknown`.

## Outcomes

| Status | What it establishes | What it does not establish |
|---|---|---|
| accepted | The interface accepted the request | The appliance reached the target |
| verified | Fresh qualifying state or a correlated completion receipt | A universal exactly-once guarantee |
| failed | Explicit rejection or a proven-unsent failure | Non-execution inferred from a generic timeout |
| unknown | Execution may have occurred; confirmation is missing | Permission to replay on another route |
| partial | Batch or non-atomic sub-operations had different outcomes | Global rollback or complete success |

## Retry matrix

| Situation | Automatic retry |
|---|---|
| Read timeout | Allowed within attempt and time budgets |
| Write proven not transmitted | Rerouting allowed within policy |
| Unacknowledged `set_value` write | Verify first; retry only with sufficient protocol/idempotency guarantees |
| Unacknowledged start / toggle / increment | Never automatically replay |
| Accepted request whose verification times out | Keep `accepted`; do not resend |
| Permission, argument, or identity error | Do not retry or switch accounts |

A local `request_id` correlates records; it is not automatically a vendor idempotency key. No protocol-level deduplication means no exactly-once promise.

## Budgets, cancellation, and crashes

A timeout includes validation, local storage/lock work, connection, necessary session refresh, transmission, and verification. Do not reset it per attempt. Check cancellation and the monotonic deadline before polling adapter work, including after synchronous storage operations. The budget prevents new sends after expiry; synchronous SQLite work and final receipt persistence are not preemptible and may delay return. Interactive login uses a separate explicitly initiated budget. Unsent work can be cancelled safely. Cancelling a sent write stops client waiting, not the physical operation; retain `unknown` or known `accepted` as appropriate.

Persist a pending phase before transmission and update after acknowledgement. On restart, unresolved records permit a warning or explicit read-based verification, never automatic replay. There is no atomic transaction spanning local storage and the appliance.

## Trusted single-command approval

[ADR-0042](adr/0042-trusted-single-command-approval.md) adds typed Core/Session
preparation and execution for one elevated action. Ordinary plan/execute remain
restricted to low-risk grants. Preparation does not send, journal, authenticate
or prove live preconditions. It requires a complete existing grant and one explicit
transport with exactly one admitted connection. The immutable preview expires
in 60 seconds; only the trusted UI may execute that same preview after consent.
No batch, independent-field decomposition, automatic routing or high-risk action
is admitted. No ordinary request field can supply approval.

Execution consumes the preview even on failure, revalidates descriptor/authority
and rejects a different Core, invalidated generation or expired preview before
send. Its budget is capped by the monotonic expiry. Existing device locks and
journaling prevent concurrent duplicate sends and preserve uncertainty on crashes.
A fixed receipt warning records that approval was supplied for the exact command.
The adapter receives Core's approval marker only on this path and must check it
before consequential effects, independently of cached risk metadata.

Native background/close invalidates pending previews and cancels in-flight work;
direct Rust hosts must implement the same lifecycle. Invalidation cannot undo a
sent command. Pending expiry/refusal stays unsent; expiry/cancellation after a
possible send stays unknown. Inspect the original request receipt rather than
creating an automatic retry. [ADR-0043](adr/0043-native-trusted-approval-bridge.md)
adds separate trusted C preparation/execution/dismissal functions. Session holds
at most eight pending previews and prunes expired entries during preparation.
A decoded execution consumes its matching ID before admission, including refusal;
malformed undecodable input does not. Missing/consumed/foreign/pruned IDs return
`APPROVAL_INVALID`; a retained expired preview returns `APPROVAL_EXPIRED`.
Background/close clears the registry; a generation check prevents publishing a
preview across a background/foreground race. No registry lock spans execution.
Ordinary JSON/CLI/MCP remain unchanged. Flutter consent UI has fictional native integration evidence; real-platform consent and physical acceptance remain separate work.
[ADR-0071](adr/0071-app-dishwasher-command-preparation.md) lets the foreground App
prepare exact reviewed dishwasher grants on device opening. That preparation
does not prepare or consume a command preview and cannot send an appliance command;
the separate trusted confirmation and adapter admission rules below still apply.
[ADR-0044](adr/0044-profile-scoped-dishwasher-start.md) uses this path for reviewed
experimental dishwasher starts: elevated, non-idempotent, exact arguments, one
cloud request, no replay/readback/optimistic cache. Explicit additional settings
are part of the confirmed command; stale cached state is never silently copied.

## Composite and batch operations

Treat multi-field `climate.set` as atomic only when the adapter declares it so. Split and track sub-steps only with an explicit `independent_fields: true` declaration, which requires a non-atomic `set_value` action. Otherwise pass the complete argument object to the adapter; absence of atomicity is not evidence that options may become separate commands. Batches return per-device outcomes with bounded concurrency. Do not compensate or roll back automatically: that would perform more physical changes without a reliable transaction boundary.

`Core::execute_batch` accepts 1-32 complete commands for distinct exact device IDs. Every command, composite step, permitted route, and request record is preflighted before sending. Duplicate caller IDs or collisions with derived child IDs are refused. A preflight failure returns an error with no sends or new receipts. Later metadata changes, lock contention, or storage failures can still affect individual items; this is not atomic admission across processes.

Execution uses groups of at most four borrowed futures and returns results in input order. A failed or uncertain device does not prevent independent devices from running. Every group shares the host's original cancellation/deadline; each command's own budget may shorten its execution window. After cancellation or expiry, queued items still return unsent outcomes through the normal journal path. Synchronous validation and storage are not preemptible, so cleanup may outlast the network budget. There are no detached tasks or automatic retries.

Batch status equals the common item status when all items agree, otherwise `partial`. Evidence belongs to each device or composite child, never to a fabricated batch device. Only per-command receipts are persisted. A forced exit can leave later items unstarted and without records; inspect individual IDs and never automatically resubmit the batch. See [ADR-0017](adr/0017-bounded-device-batches.md).

## Local account removal and late results

Local account removal under [ADR-0028](adr/0028-local-account-removal.md) preserves
execution receipts and retired IDs. It never rolls back or resends an appliance
action. Meiju retains its account lease through network work; cleanup refuses a
busy account. Receipts can still be finalized after the account lease is released,
even when cleanup has removed the device metadata. Their known outcomes remain
available through exact receipt lookup and global history.

SQLite rejects descriptor/snapshot saves referencing an account with removal
intent, including completed removal. Snapshot saves also require an existing
device. These checks share the write transaction, preventing late inventory or
read results from recreating cleaned data. They do not retract already delivered
results or make a multi-device publication atomic. A cache-save failure after
acceptance/verification follows the existing strongest-outcome preservation rules.

Under [ADR-0033](adr/0033-delete-unreadable-account-credentials.md), explicit account
removal deletes the exact protected entry without reading/decrypting it. A confirmed
absent entry causes no mutation; refused or uncertain deletion preserves pending
cleanup and metadata. This local idempotence does not permit retrying appliance writes.

## Explicit history clearing

[Local-data cleanup](adr/0029-local-cache-and-history-clearing.md) refuses history
clearing while any journal is active, including a composite parent. Completed
receipt details are retired atomically with their request IDs preserved. Clearing
a finalized unknown receipt does not resolve uncertainty or authorize a replay.
`REQUEST_RETIRED` still prevents sending after reopen. Cache clearing is independent:
it removes snapshots without cancelling or rolling back device operations, and a
concurrent read/verification may save new observations after the transaction.

## Implemented Mock behavior

Capability resolution includes the selected component. Shared definitions omit `component_id`; scoped definitions bind one exact component, and shared/scoped mixing of one capability ID is invalid. The same resolver supplies action parameters, risk/routes/verification policy, live and cached property constraints, and property freshness. A fridge grant cannot authorize its freezer. Both remain one physical device with one write lock and separate component snapshots. See [ADR-0018](adr/0018-component-capability-scopes.md).

Operational lookup checks device/capability schema versions and shapes, unique identities and references, unambiguous property names, and locally compilable action schemas. Invalid metadata returns `CAPABILITY_UNSUPPORTED` before reads, dry runs, journal reservation, or writes. Discovery validates its complete per-adapter response before saving any descriptors from that response; an invalid response leaves existing metadata intact. This validation gate does not make storage commits across multiple devices or adapters atomic. Rediscovery also compares the entire response against stored connection identities before saving. `DEVICE_IDENTITY_CONFLICT` prevents silent binding changes under an already granted device ID. SQLite repeats each comparison within its write transaction to catch concurrent changes; earlier saved devices remain saved if a later transaction fails. This guard does not change command execution outcomes or authorize route merging.

The experimental Meiju adapter additionally requires complete account inventory
retrieval and checks current plus historical protected appliance bindings across
all selected accounts before returning descriptors. Native account connect/refresh reuses
`Core::publish_discovery` for the same registered-adapter, whole-batch validation
without a second cloud inventory request; no public JSON imports descriptors. Partial inventory and ambiguous
cross-account identity publish no new descriptor batch. Only explicitly reviewed
exact model/family pairs expose read properties. Unknown models remain visible with
unsupported status and no actions. Every Meiju observation remains stale while its
source timestamp is unproven; it cannot verify a write. Construction, local queries
and cached reads do no account or network work.

An explicit trusted Meiju adapter opt-in additionally admits experimental AC
`power.set` for the four exact read models. Default hosts retain read-only adapters
and no Meiju write grants. Native API 1.3 can receive exact-device/state power grants
only from trusted startup under [ADR-0027](adr/0027-native-power-grants.md); ordinary
requests cannot change them. Control repeats the fixed argument contract and protected
account/device/connection/model checks before sending; public metadata cannot supply
a provider target. The separate HTTPS control path marks possible transmission
before executing the request. Only a valid code-zero envelope establishes acceptance;
HTTP 401/403 and recognized authorization codes establish rejection. Other provider
codes, malformed replies, HTTP failures and post-send cancellation remain unknown.
No retry, route fallback or automatic login follows. Verification is unavailable,
so accepted controls requested with `wait_for: verified` retain `VERIFICATION_PENDING`
without a state read or cache mutation. Existing Execution schemas and replay rules
apply unchanged; see the [control probe](development/07-meiju-control.md).

API 1.9 additionally admits a separately opted-in vendor AC fan action under
[ADR-0034](adr/0034-scoped-meiju-ac-fan-control.md). It shares the same transport,
protected model/binding checks and conservative outcomes, sending only one reviewed
`wind_speed` code. Power and fan permissions remain independent. It never changes
power, mode or temperature to force an effect; mode-dependent effectiveness remains
unverified. No read, optimistic cache update, fallback or retry follows acceptance.

API 1.11 adds explicit `vendor.midea.ac.activate_mode` under
[ADR-0039](adr/0039-explicit-meiju-mode-activation.md). Each independent startup
grant permits one exact device/mode and its combined power-on effect. The adapter
sends `power: on` and that mode in one request, never separate power/mode calls.
Neither power nor fan grants admit it. `atomic` and `independent_fields` are false;
accepted/unknown handling, protected checks and replay rules remain unchanged.
It neither copies cached status nor writes temperature/fan values. Model behavior
and preservation of unrelated settings remain unverified without physical evidence.

API 1.12 adds profile-scoped `climate.set {target_temperature}` under
[ADR-0041](adr/0041-profile-scoped-ac-temperature.md). After explicit discovery,
only admitted AC models with reviewed protected product evidence expose the action.
Its parameters carry the selected Celsius bounds and step; property observation
domains stay readable outside write limits. Authority names one device and exact
target. The adapter rechecks the protected selector and arguments before transport,
even when public descriptors were weakened. It sends only the selected temperature
field(s), without rounding, cached status, extra provider reads, optimistic cache
updates, replay or restoration. Missing evidence and explicit unsupported overrides
refuse before send. Atomicity/independent fields stay false; accepted/unknown
outcomes and physical-preservation limitations are unchanged.

The Linux CLI can supply these same independent grants through an explicit
owner-controlled file under [ADR-0037](adr/0037-cli-meiju-control-grants.md).
It requires explicit request IDs for single writes and previews, and never
refreshes capabilities implicitly before execution. Removing the file from a
later invocation revokes its permissions without erasing recorded receipts.
Default startup remains read-only; transport and outcome semantics are unchanged.

Under [ADR-0038](adr/0038-scoped-meiju-mcp.md), local Meiju MCP hosts can snapshot
the same grants at startup. Only those sessions expose `command_plan` and
`command_execute`; tool arguments cannot change authority. Commands retain their
explicit request IDs and budgets. RPC IDs identify cancellation/response messages,
not execution receipts. EOF/SIGINT and stdio failure cancel active work and drain
cleanup; lookup after restart preserves outcomes without replay. Removing a grant
file affects subsequent startup, not a currently running session.

The current Core attempts each permitted route at most once. Only a proven-unsent reachability failure allows the next write route. A possibly-sent write returns `unknown` immediately; an explicit later read does not silently rewrite its receipt. State-match verification performs one bounded live read. All live snapshots, including verification reads, must use schema 1.0, match the requested device/component, and have every observation sourced from the selected transport. A source observation cannot be later than its local receipt, and a live receipt cannot be in the future. Empty warning strings are invalid. The entire snapshot is checked before saving; a valid target field cannot hide malformed unrelated fields.

The live-read rule applies to `state_match`. For `correlated_completion`, Core instead uses successful completion evidence returned by the same bounded adapter execution. The adapter validates protocol correlation; Core requires an exact match of request, device, component, and selected connection IDs. Receipt time must be at or after attempt start and no later than the current clock. If available, trustworthy completion time must be between attempt start and receipt. Evidence is used only after acceptance and when the caller requests verification. Missing or invalid evidence keeps `accepted` with `VERIFICATION_PENDING`, without a state-read fallback or another send. An enqueue acknowledgement is never completion.

Completion verification does not update observed state. It remains evidence of that operation even if the appliance subsequently changes. Cancellation or deadline expiry while waiting retains previously marked acceptance. Core projects only method, transport, timestamps, and a fixed explanation, never raw provider tokens. Already acquired completion evidence survives subsequent journal errors; the persisted row may still be conservative. See [ADR-0016](adr/0016-correlated-completion.md).

Invalid ordinary reads return `INVALID_OBSERVATION`, retain the old cache, and do not fall back to another route. Invalid verification leaves a known acknowledgement as `accepted` with `VERIFICATION_PENDING`, with no replay or cache replacement. A valid but stale unrelated property does not invalidate fresh target evidence. Missing observations remain absent, and an empty valid snapshot cannot verify a target.

Every returned observation must resolve to one readable property of the requested component. Core validates its value against that property's type, enum, bounds, and step, including unrelated fields in a verification response. Unknown properties, unreadable properties, ambiguous definitions, and malformed property descriptors fail closed. Vendor names are matched as complete `capability.property` identifiers; dots in a vendor capability ID are preserved. The current property contract does not permit null values; omit missing readings. Cached values are checked against the currently stored descriptor without refreshing, coercing values, or rewriting the cache. A valid descriptor update can therefore make an older cached value invalid until an explicit fresh read succeeds.

State-match verification compares validated numeric values independently of their integer or floating representation: `25` and `25.0` match. It does not coerce strings or booleans, round to a property step, or apply an approximate tolerance. Mixed integer/float comparison checks exact integer bounds before conversion so distinct large integers cannot match through floating-point rounding. Cached observations keep the adapter's original representation; verified result data keeps the requested arguments. This comparison does not change command equality for request-ID replay protection: changing a recorded argument from `25` to `25.0` still conflicts with that request ID.

Live reads, cached reads, and verification apply each property's `max_age_ms`, measured from `observed_at` when known and otherwise `received_at`. Omission retains the 30000 ms Mock baseline. Age at or above the limit is stale. Existing stale flags remain true; warnings explain property expiry, previous flags, or clock changes. Invalid limits fail descriptor admission before reads or writes. Limits need real-adapter evidence and are not a universal freshness promise; see [ADR-0015](adr/0015-property-freshness.md). Verification additionally requires target observations and receipt times at or after transmission. Cached reads recheck version, identity, and chronology without network fallback. A future cached receipt may reflect a local clock rollback: preserve it as stale with a warning instead of pretending it was just observed. Cached sources remain historical evidence even when the caller supplies a route preference. Cache reads do not rewrite persisted observations.

Non-atomic multi-field commands with `independent_fields: true` must validate as individual single-field commands before any send. Omitted or false retains all arguments in one adapter call per permitted attempt. Conflicting `atomic` or `idempotency` declarations return `CAPABILITY_UNSUPPORTED` before operational use. See [ADR-0012](adr/0012-explicit-action-decomposition.md). Their receipts contain `items` for both success and failure. All verified items produce `verified`, all accepted items produce `accepted`, all failed items produce `failed`, and different outcomes produce `partial`. Completed fields are not rolled back. The aggregate verification summary refers to the separate item evidence. See [ADR-0007](adr/0007-mock-mvp.md).

Exact receipt lookup is also available without invoking the write path: `Core::execution`, CLI `execution`, native `get_execution`, and MCP `execution_get` return a retained receipt unchanged. They do not require current write authorization or operational device metadata and do not read state, lock the device, or alter the journal. Missing/retired IDs give `REQUEST_NOT_FOUND` / `REQUEST_RETIRED`, never permission to resend. A later observation remains separate from the saved outcome; see [ADR-0014](adr/0014-local-receipt-lookup.md).

An identical request ID and command returns the saved receipt without sending, including after restart. A different command under the same ID fails with `REQUEST_CONFLICT`. When detailed receipts are retired, their IDs remain reserved; any reuse fails with `REQUEST_RETIRED` before transmission. The old outcome cannot be inferred from this error. See [retention and migration](storage.md).

Caller-supplied command identifiers are nonblank and bounded to 256 UTF-8 bytes. Composite steps validate each argument set and grant using the already valid caller ID before appending `:<step-index>` for internal receipt correlation. This preserves the full caller limit for composite commands; derived child IDs may be longer and must not be submitted as new standalone requests. Parent replay protection and child-ID collision checks still apply. Device locks also support the full accepted device-ID length through the bounded filename mapping in the storage contract.

Database failure before transmission prevents sending. Infrastructure failures after acceptance or verification preserve the strongest known outcome. A failed snapshot save cannot erase already obtained verification evidence. For a composite, any storage failure stops later writes and records remaining items as `failed` / `NOT_EXECUTED`, with no attempts. Parent and child receipts retain known results even if final persistence fails. Verified, unknown, or partial receipts record infrastructure errors in `warnings`; their existing evidence, uncertainty, and item results remain intact. An existing rejection, cancellation, or skipped-item error is also preserved, with the subsequent storage error added as a warning. A failed journal update may leave an earlier conservative stored outcome. Never replay or compensate to repair local storage.

CLI and native sessions pass their existing operation into Core. Starting execution never extends the earlier host deadline; a shorter command budget can only reduce it.

## Meiju inventory membership admission

Under [ADR-0035](adr/0035-meiju-inventory-membership.md), each live bound read or
control checks protected membership while holding the account lease through the
network operation. A complete inventory marks missing historical bindings absent;
partial/failed retrieval preserves established membership. Identity records and
public descriptors remain retained. `MIDEA_DEVICE_NOT_LISTED` or
`MIDEA_INVENTORY_REQUIRED` fails before transmission without implicit enumeration,
login, fallback or retry. Reappearance preserves IDs and does not replay receipts.

Membership is never copied back from descriptors. Late publication cannot undo a
newer protected exclusion. Static plans and cached reads remain local and do not
check membership; execution can therefore refuse a previously valid plan. This
is last-retrieval evidence, not an online signal or current authorization proof.
No change is made to accepted/unknown writes, existing receipt lookup or journaling.

## Midea LAN candidate collection

The [bounded LAN collector](adr/0045-bounded-lan-discovery-decoding.md) uses the
caller's existing Operation and a selected private/link-local subnet. It sends
two fixed probes once, caps setup/collection at five seconds, and closes its
socket on completion, error, cancellation or future drop. Source filtering,
datagram/candidate limits and exact-tuple deduplication do not authenticate a
peer. A normal or limited report is not a complete inventory and must not remove
cards, merge identities, add routes or authorize TCP connections. CLI, native
JSON and MCP do not invoke this foundation yet. Public request/result schemas,
cloud routing and write outcome semantics remain unchanged.

The [V3 codec](adr/0046-bounded-v3-lan-codec.md) supplies only handshake and frame
processing. Key confirmation does not authenticate an advertised appliance ID or
authorize endpoint reuse. A valid response digest/sequence is not fresh state or
command acknowledgement. A future TCP adapter must retain identity, account
membership, operation lifetime, request correlation and write-progress checks;
codec success cannot change routing or replay an uncertain command.

The native `exchange_v3` helper now owns one socket for one inherited Operation.
It rejects invalid subnet endpoints/payloads and reused progress markers before
network access, then performs one handshake and command exchange with no retries.
Only the command write marks `possibly_sent`; connection/handshake failures leave
it unsent. Valid framing does not mark acceptance. Disconnects, cancellation,
deadline and future drop close the socket while preserving transmission evidence.
No adapter or entry point invokes this helper yet, so public schemas and cloud
write behavior remain unchanged; protected identity, grants and model evidence
still gate future route admission.

The [account-bound v2 credential helper](adr/0047-account-bound-lan-credentials.md)
uses an exact protected local binding and fresh unique cloud home membership.
It returns native handshake material without selecting or contacting a LAN peer.
No route, permission, inventory change or write outcome follows from a successful
lookup. Discovery hints cannot select provider IDs for it. A future registered
route must separately reconcile the endpoint and validate its model/permissions
before sending the token. Cloud failure does not trigger default keys, v1 lookup,
alternate UDP methods or automatic route fallback.

The [ordinary packet envelope codec](adr/0048-bounded-lan-packet-envelopes.md)
requires the expected native appliance ID and rejects mismatched wire identity
before exposing decrypted bytes. Its public checksum/key are forgeable. Valid
framing and matching identity fields do not establish endpoint ownership, an
inner model response, freshness or causal completion. The codec performs no I/O,
returns no outcome/observation and never changes existing transmission markers.
Model-aware validation remains necessary after `exchange_v3` releases its socket.

The [standard AC inner codec](adr/0049-ac-lan-query-and-c0-decoding.md) emits one
explicit query and decodes only query-kind C0 candidates. Strict AA framing and
checksum checks do not correlate an acknowledgement or establish freshness. Raw
mode/fan codes remain native protocol values; a model-aware adapter must admit
their semantics and reject known stale/alternate temperature layouts before
publishing observations. No default query, route, outcome or retry is added.

[Protected candidate reconciliation](adr/0050-protected-lan-candidate-reconciliation.md)
compares one exact saved account binding with a bounded report and explicit subnet.
It returns `NotObserved`, `Incomplete`, `Ambiguous` or a borrowed `UniqueCandidate`.
Conflicting target addresses/protocols/families and endpoint identity collisions
never choose a winner. Report limits prevent a unique result; no result changes
cards, membership, grants or connections. A unique hint is neither authenticated
nor globally unique and cannot authorize sending credentials or publishing state.
This native-only helper performs no networking and retains the caller's Operation.

The [composed AC V3 query](adr/0051-bounded-ac-lan-read-sequence.md) takes an already
admitted native endpoint, protected handshake and expected target. On one TCP
connection it queries the appliance's message version, then builds one AC query
with that version. Both replies require expected-ID envelope validation before
inner decoding. The injected clock supplies each packet's timestamp; neither
timestamp is observation evidence. One inherited deadline covers both reads;
errors, cancellation and drop close the connection without retry or fallback.
It returns C0 candidates only and never publishes a route, Snapshot or write
outcome. Existing one-command `exchange_v3` transmission phases are unchanged.

The standard E1/CA sequences in
[ADR-0053](adr/0053-dishwasher-refrigerator-lan-reads.md) reuse that same bounded
handshake/negotiation/query flow through `query_dishwasher_v3` and
`query_refrigerator_v3`. They validate the expected family/version at both stages
and return only native candidates. E1 retains raw program/status/temperature
codes; CA admits only its main-compartment Celsius prefix. Setter replies,
notifications, alternative headers, short bodies and unsupported units cannot
become observations or completion evidence. Model/endpoint admission and Core
route publication remain separate; no caller or grant is implicitly enabled.

## Trusted native scan bridge

[ADR-0058](adr/0058-native-lan-scan-bridge.md) exposes ADR-0054's typed scan through
three separate trusted C functions in API 1.18. Preparation stores at most eight
immutable device/network previews; confirmation consumes one before admission.
The controller must obtain assigned-interface information, platform permission
and actual foreground consent. Background/close clears previews and cancels work.
The original bounded operation, account lease and metadata checks still apply.
Only local IDs and a non-authoritative status leave the bridge; candidate
endpoints/provider identifiers are dropped. Ordinary JSON, CLI/MCP discovery,
cloud routing, storage and appliance write outcomes remain unchanged.

## Optional subtype in wall-AC temperature admission

[ADR-0066](adr/0066-optional-wall-ac-subtype.md) corrects an over-restrictive
selector: absent subtype is permitted only with an independently saved exact
reviewed wall model, explicit wall category and non-placeholder product code.
Existing product overrides and exclusions still determine the temperature encoding
and bounds. Execution rechecks the same protected evidence as option inspection
and discovery. No inferred subtype is stored; grants, schemas, one-send behavior,
accepted/unknown outcomes and the separation of later observations are unchanged.


## Optional AC prerequisite reads

[ADR-0069](adr/0069-profile-scoped-ac-features.md) adds independent profile-scoped
scalar setters. Core validates exact permissions and routing first; the adapter
rechecks protected membership/model/profile and its immutable argument schema,
then reads prerequisites within that same operation/deadline before its one
control send. Missing or unrecognized required state refuses before transmission.
Other supported swing axes must come from that operation, never Core cache.
A valid reported buzzer preference is preserved, an absent preference is omitted,
and a malformed report refuses before send. No upstream client default is copied.
These reads do not verify the later command or update Core's observation cache.
There is no atomic transaction with another vendor client; concurrent changes can
race the reads. Source-specific paired ECO fields stay in one request.

No retries or route fallback are added. A control request marked sent retains
possibly-sent/accepted evidence through timeout, cancellation and malformed reply;
only reviewed explicit rejections prove rejection. Reusing its request ID returns
the stored receipt without more prerequisite reads or sends. Separate foreground
state refresh remains observation evidence, never physical-causality proof.


## Explicit dishwasher power requests

[ADR-0072](adr/0072-profile-scoped-dishwasher-power.md) admits `power_on` and
`power_off` through the same trusted elevated boundary as dishwasher cycles.
Each exact action/argument grant is independent. Profile 760Y0026 requires an
explicit additional option; absent/default/cached options never authorize a send.
Power requests use one `work_status` control attempt with no wash-program field,
implicit read, retry, fallback or compensation. Neither power assignment is
assumed replay-safe. Provider acknowledgements remain accepted; possibly-sent
failures remain unknown. Power-off is not physical isolation or rollback.

## Account identity recovery and command authority

The trusted API 1.25 legacy Meiju recovery in
[ADR-0073](adr/0073-meiju-account-identity-recovery.md) sends login/inventory reads
only. A recovery preview is not a command grant or appliance approval. Confirmation
requires a quiescent foreground host, seals it before local persistence and
invalidates existing approvals. Reopen read-only; do not copy command grants into
the replacement session. Uncertain credential persistence stays pending and is
never automatically replayed. Device/connection IDs and existing receipts remain
unchanged; neither restoration nor cancellation proves any physical state.


## Exact October 22 AC scalar profiles

[ADR-0074](adr/0074-exact-meiju-reference-profiles.md) replaces the generic
optional catalog for four exact protected AC product/model pairs. Model/category
conflicts and portable-subtype contradictions refuse optional controls. A saved
old action or a startup grant does not override this protected selection.
`cool_energy_saving`, `strong_cooling` and `light_sensing` independently encode
0/1, 0/1 and 0/3 on their reviewed profiles. Each uses the existing scoped grant,
prerequisite read and single tracked Lua send. Any reported buzzer preference is
preserved, never invented. Prerequisite failures are proven not sent; a possibly
sent failure remains unknown. Replay returns the original receipt without
additional reads or writes. No transport, fallback, retry, verification or
background behavior changes. The new refrigerator scopes contain reads only.

## Exact Mijia reference observations

[ADR-0080](adr/0080-mijia-exact-reference-readings.md) admits additional exact-model
scalar reads through the existing protected account lease and cloud operation.
Pinned public facts avoid optional public-spec requests on explicit account
retrieval; they are neither current firmware evidence nor account authority.
Reference properties retain device/SIID/PIID correlation, local receipt time,
unknown physical observation time and stale flags. ADR-0083 separately admits
four exact reference fan setters; other new fields have no setters.
Existing fan grants, one-send outcomes, replay protection and lifecycle rules are
unchanged. No credential is forwarded from the legacy SID to the modern origin.

## Exact reference fan settings

[ADR-0083](adr/0083-exact-reference-fan-settings.md) extends the existing fan path
only for the pinned `jdjz.fan.2072fc` device/service/property specification.
Horizontal included angle (60/90/120), vertical swing, vertical included angle
(60/105/120) and ionizer retain their existing property identifiers. Each exact
value requires its own trusted startup grant and protected adapter admission.
Included angle never selects the separate absolute-orientation command.

A command sends one correlated MIoT property over the existing legacy session
route. Its acknowledgement is `accepted`; no physical verification is inferred.
Possibly-sent failures remain `unknown` with no automatic replay or route fallback.
Old reference mappings remain readable and gain options only after explicit
inventory refresh. Foreground cancellation and independent Core policy checks
apply unchanged; no timer, reservation, MIoT action or modern-origin route is added.

## Exact reference light and purifier execution

[ADR-0084](adr/0084-exact-light-and-purifier-controls.md) admits 14 exact light
and purifier settings through the same correlated single-property legacy route.
`with_reference_controls` is independent of `with_fan_controls`; both default
off. Native `mijia_setting_grants` supplies only exact local targets. Protected
model/specification/identity/connection and arguments are checked again at send.

One setting sends once without automatic mode/power companions. An acknowledged
setting can depend on the appliance's current mode to have an observable effect;
acceptance does not assert effectiveness or physical verification. Timeout,
malformed response and cancellation after possible transmission remain unknown
with no replay or fallback. Nothing schedules HomeVerb work or uses the modern
origin. Socket/wall-switch power and custom composite fields are not admitted
by this opt-in. Saved older derivations stay readable until explicit refresh.


## Historical numeric reads

[ADR-0089](adr/0089-bounded-numeric-history.md) separates explicit historical
queries from current reads and all execution outcomes. Typed Core now validates
one local connection, one inspected numeric target and a bounded half-open UTC
interval, then holds the device lock while the adapter rechecks protected bindings.
It validates the complete returned history before publication, repeats descriptor
admission after I/O and rejects late results. Public native/App integration is
pending; no ordinary Host method is added by the typed Core phase.
No route fallback, retry, cache write, execution receipt, write verification or
new write authority follows from a historical query. Missing dates remain absent.


## Exact outlet approval enforcement

Under ADR-0091, typed Rust hosts may independently opt into the four reviewed
`cuco.plug.v3` settings. Protected mapping re-derivation determines risk and wire
identity. Power, recovery state and physical controls require Core's trusted
approval marker even when public action metadata claims low risk. Exact grants
alone cannot send those operations. The indicator is independently low-risk.
Ordinary execution never consumes an approval from JSON; callers use ADR-0042's
immutable typed preview after actual user consent. Cancelling after send leaves
an unknown result and the original request ID; no automatic retry or route fallback.
Current native API 1.33 and CLI/MCP keep their previous read-only outlet behavior.


## Native exact outlet controls -- October 29, 2026

API 1.34 adds closed `mijia_outlet_grants` with exact local device/component/
property/value targets and a 64-entry limit. Only Mijia and combined hosts accept
these startup grants. All four exact outlet options can be inspected locally
before granting authority. Existing light/purifier and fan fields retain their
scopes. Separate adapter policies prevent overlapping physical-lock identifiers
from transferring authority between purifier and outlet grants, even when both
families are enabled.

Power, power-loss recovery and physical-button locking require the separate
immutable trusted preview/confirm bridge for every operation; indicator control
is low-risk. Default startup and CLI/MCP remain read-only for outlets. Ordinary
JSON cannot confirm elevated commands. Unknown replies preserve the request ID
without replay, and backgrounding invalidates previews. C exports (26 plus the
Android initializer), 36 ordinary methods, SQLite 5 and protected payload envelope
1 are unchanged. Explicit account refresh in an outlet-enabled session upgrades
old read-only mappings. Older implementations may refuse those upgraded protected
contents; retain the matching implementation with that profile. App wiring, an
updated APK and real-provider/physical acceptance remain pending.


## Native auxiliary settings -- October 30, 2026

API 1.35 implements [ADR-0092](adr/0092-exact-auxiliary-reference-settings.md):
the exact `xiaomi.hood.cyp10` illumination switch and brightness enum 0/1/2,
and `jdjz.fan.2072fc` vendor quiet boolean. These seven targets use independent
`mijia_setting_grants`; existing fan and outlet admissions do not enable them.
Only exact protected source evidence permits the properties. No percentage
brightness conversion, companion power request, ventilation, timing or cleaning
authority is added. Each command sends one property and preserves accepted/
unknown outcomes without replay. Local native inspection works before grants.

Existing reference read-only derivations remain valid. Explicit selected-account
refresh upgrades the mapping; use the matching implementation with newer saved
derivations. There are still 36 JSON methods and 26 C exports plus the Android
initializer, with SQLite 5 and protected payload envelope 1 unchanged. Flutter
API 1.35 negotiation and presentation are pending. The delivered API 1.34 APK
does not gain these controls. No real provider/account/device acceptance is claimed.


## Independent modern Mijia authentication

ADR-0093 adds a separate typed Rust SID=mijia QR/renewal protocol. It is not a
new device route or a conversion of saved legacy credentials. One protected
challenge inherits the original cancellation/deadline and is consumed by one
foreground completion attempt. A single bounded poll can be followed by one
exact STS exchange. Renewal independently validates the same protected subject;
no automatic relogin, retry, device request or write replay follows authentication.
Native account payloads, ordinary request schemas and write outcomes are unchanged.


ADR-0094 admits three typed Rust read routes at the exact modern reference origin,
using its continuous-stream codec. This creates no Core device connection and
no alternate write route. Logical directory entries remain bound to the protected
modern subject. Per-entry mapping checks and post-decode budget checks precede
returned evidence; malformed or stopped operations publish nothing. A later
ordinary page failure may return positive partial membership, never deletion
permission. Reads do not refresh credentials, fall back to community/legacy
origins, retry properties or alter execution receipts. Modern controls and native
account/Host/App integration remain pending.


ADR-0095 persists the modern session in a separate tagged protected account slot.
Legacy device routes continue to require the legacy slot; neither directory
entries nor successful renewal selects another route or changes write authority.
Modern entries bind to their local account/credential reference and protected
provider identity. They remain logical entries until independent reconciliation.
QR continuation retains the original lifetime through persistence and refuses
changed account snapshots. Renewal performs authentication only. Explicit recovery
validates the exact pending bytes through a bounded modern directory read before
confirming them; it never renews, rewrites unchanged secrets or replays commands.


ADR-0096 adds foreground Host ownership for the modern account flow. Pending QR
handles share the existing bounded sign-in registry and are consumed before any
continuation attempt; they are not general JSON authority. Cancel/background/close
invalidate presentation, cancel the relevant registered work and release leases
on completion/drop. Explicit directory/recovery and renewal follow the same Host
operation boundary. A late lifecycle result is refused without pretending to undo
already-completed protected storage. No physical Device, fallback route, write
grant or automatic credential renewal follows from a logical directory entry.


ADR-0097's dedicated C bridge forwards explicit modern account work through the
Host lifecycle. Protected QR data is not an ordinary request/result field. Native
reports summarize logical entries without publishing physical cards, selecting
routes or granting controls. Directory failure after successful protected login
keeps the saved account, while uncertain storage still requires explicit recovery.
No C caller may turn a transport failure into an automatic renewal or login replay.

## Modern selected-protocol Core bindings

[ADR-0100](adr/0100-mijia-modern-protected-core-reads.md) adds explicit typed
modern device adoption before Core publication. Each protected binding selects
legacy or modern reads; public connection metadata cannot choose the codec,
credentials or origin. Explicit inventory can select its validated protocol while
preserving local IDs and organization. A failed read never selects the other slot,
renews credentials or refreshes inventory. Modern observations retain absent
source time, local receipt time and stale=true, with Core scalar validation.

Modern-selected descriptors have no control actions or historical options.
Independent adapter checks also refuse stale descriptors and trusted grants before
send, retaining failed/not-sent semantics. This does not introduce an outcome,
retry, write authority or ordinary JSON parameter. The protected payload contract
and positive/negative executable fixtures are in `crates/mijia/src/protected/modern.rs`
and `crates/mijia/src/adapter/tests/modern_accounts/devices.rs`; public schemas are
unchanged. Host/App publication is pending; existing QR reports alone do not
publish physical cards.


ADR-0101 adds explicit typed Host publication after modern QR continuation or
saved-session device refresh/recovery. It reuses the original login directory and
lifetime through protected binding persistence; publication is bounded by the
same QR expiry and foreground generation. Successful authentication is separate
from partial/failed retrieval, and failed publication reports no device IDs without
claiming storage rollback. Existing logical C/App calls remain unchanged; modern
reads still use only protected selected-protocol evidence with no controls/history,
fallback, automatic renewal, complete-inventory time or write-outcome changes.


ADR-0102 exposes modern device publication through API 1.37 opt-in metadata and
explicit modern device refresh/recovery operations. Ordinary account retrieval
selects inventory under protected account leases before I/O: modern if no legacy
slot exists or any binding is modern-selected, otherwise legacy. Missing transports,
expired sessions and request failures never trigger fallback, renewal or replay.
Returned modern roots lose unsupported control/history options; historical absent
bindings retain their selected protocol. This changes account retrieval admission,
not write outcomes, trust markers or execution authority. No complete membership
or physical observation follows from successful publication.

## Modern historical profile selection

ADR-0103 extends explicit historical reads for the exact protected outlet profile.
The saved binding selects the credential slot before I/O. Modern uses the reviewed
api.mijia.tech statistics origin with per-field RC4; legacy retains api.io.mi.com.
This choice does not change current-state routing or the reference continuous codec.
No request selects a raw endpoint or key. Failure never tries the other protocol,
renews credentials or changes write receipts. Account/device leases and foreground
cancellation cover the whole query and result validation; history remains separate
from current state, execution acceptance and physical verification.

## Modern exact property control selection

ADR-0104 admits only the existing 29 exact scalar mappings across five models.
The protected binding selects the legacy or modern session before transmission.
Modern controls use a separately registered fixed-origin transport and the
reference continuous codec, never the modern history profile or legacy token.
Transport registration and local option inspection do not grant command authority.
Exact Core grants, protected family/model/spec/domain checks and independent trusted
approval for elevated outlet settings remain mandatory. Unknown/removed membership
or unavailable transport refuses stale public descriptors before any send.

Each command sends one property once with its complete protected root identity.
A response requires encrypted global success and exactly one matching DID/SIID/PIID
item: zero is accepted, nonzero explicitly rejected. Neither establishes physical
verification. HTTP/global failures, invalid/uncorrelated replies, cancellation and
timeout after the conservative send marker remain possibly sent and produce unknown
receipts. Changing a saved binding to modern does not invalidate a prior legacy
unknown receipt or allow its request ID to send again. No automatic fallback,
renewal, retry, companion property, state read or rollback is added.

Public command/result schemas, outcomes and native API 1.37 are unchanged.
Positive/negative fixtures are the executable protocol, Core and Host cases in
crates/mijia/src/modern/reads/tests/writes.rs,
crates/mijia/src/adapter/tests/modern_accounts/controls.rs and
crates/host/tests/mijia_accounts/modern_host/controls.rs. Existing ordinary schema
fixtures and C ABI checks retain their previous shapes and trust boundary.

## Typed property-log reads

ADR-0105 adds separate typed numeric property-history primitives. They use the
selected session's fixed community history profile, never reference-codec or
cross-protocol fallback. Each query validates the exact pinned numeric field and
bounded window before one read, and checks lifecycle again after parsing. Reported
log values, including negative values, are not live snapshots, daily statistics,
actor attribution or physical write-verification evidence. No cache, receipt,
permission, credential or saved mapping changes. Core/native/App sample-history
admission remains a dependent phase; their existing daily-history contract is
unchanged. Positive/negative protocol fixtures are in
crates/mijia/src/property_history/tests.rs and
crates/mijia/src/modern/reads/tests/property_history.rs, with fixed-origin local
TLS coverage in crates/mijia/src/transport/tests.rs.

## Reported property samples through Core and hosts

ADR-0105 adds explicit `reported_samples` reads on the selected connection. Core
inspects local options, distinguishes `(target, aggregation)`, then independently
validates result identity, domain, exact nullable unit and receipt timing. The
adapter repeats protected exact-model/field and selected-protocol admission and
retains account/device leases across send and post-reply validation. Cancellation,
deadline, drop and late replies release resources without publishing a stale result.

API 1.38 inspection defaults to daily-only for old clients; an explicit sample
opt-in does not grant write authority or trigger networking. Flutter negotiates the
version and suspends foreground state polling while the history dialog is open.
CLI and local MCP preserve the same Core series. Samples never enter snapshots,
receipts or verification; no retry, fallback, pagination, background work or
credential renewal occurs. Daily statistics keep their existing nonnegative values
and conversion; signed samples retain the protected property's source domain.

## Exact hood manual writes

ADR-0106 uses the existing single-property legacy/modern write paths for four
exact hood settings. Each command has one exact device/component/property/value
grant and requires the independently admitted reference-setting family. Preserve
vendor identifiers, Boolean values and original enum codes. No power, light,
ventilation policy or timer write is appended to satisfy another setting.
A correlated acknowledgement is accepted, never verified physical operation.
Possible-send timeout, cancellation or malformed acknowledgement remains unknown
without replay or cross-protocol fallback. Existing receipt/request-ID guards apply.


## Typed non-idempotent Mijia actions

ADR-0107 adds six exact no-argument protocol primitives, with no Core action or
host grant yet. Every admitted attempt uses a params object and one selected
legacy/modern tracked control transport. Complete protected root identity and
exact model/specification/SIID/AIID precede sending; no caller arguments or
unreviewed outputs are admitted. Public property catalogs and saved payloads
remain unchanged. A consumed WriteProgress cannot send again on either transport.

Only a correlated encrypted global-success/result-code-zero acknowledgement
marks accepted. Explicit reviewed pre-execution rejection is separate from code 1,
execution errors, return errors, timeouts and unknown codes, which remain possibly
sent. Cancellation, drop, uncorrelated/malformed outputs and global/HTTP failures
never prove physical rollback or authorize replay. No retry, renewal, fallback,
verification read or state mutation follows. Core request-ID receipt integration
is required before exposing these non-idempotent actions through native/App.
Public schemas are unchanged; positive/negative executable protocol fixtures are
in crates/mijia/src/action/tests.rs and
crates/mijia/src/modern/reads/tests/actions.rs, with actual local TLS matrices in
crates/mijia/src/transport/tests.rs. This evidence is not provider acceptance.


## Protected Core action admission

ADR-0108 now connects ADR-0107's six primitives to typed Core. Empty-object
arguments and non_idempotent/unavailable-verification metadata are re-derived
from the protected exact model/specification and independent action catalog.
The dedicated adapter action policy and Core grant both precede the selected
protocol send; existing property policies cannot transfer authority. Public action
metadata or caller SIID/AIID/in fields cannot choose wire operations. Account
leases and Core device locks cover the whole attempt, including late-result refusal.

The existing request-ID journal is written before possible send. Accepted,
rejected, not-sent and unknown receipts are retained; after cancellation/deadline/
drop an uncertain action is never retried. Authorized same-ID execution returns
its existing receipt after reopen or explicit protocol adoption. Revoked authority
refuses before receipt lookup without changing the stored outcome. Retiring history
retains request IDs and does not enable resend. Only a new explicit request can
represent another action; it must never be created automatically to recover unknown.

Public command/capability schemas retain their shape. The exact no-argument
contract is represented by examples/capability.mijia-action.json and
examples/command.mijia-action.json, with closed-argument counterexamples in
scripts/validate_contracts.py. Protected Core fixtures are in
crates/mijia/src/adapter/tests/modern_accounts/actions.rs. Native/App startup and
inspection adoption remain pending; the existing native/property flows stay intact.


## Native action composition

ADR-0109/API 1.40 supplies independent startup-only exact empty-argument policies
to both Core and the protected Mijia adapter. Inspection opt-in does not enable
execution. A granted reopen followed by explicit account refresh publishes the
admitted actions; model, membership, root and selected-protocol checks still
precede sending. No new result, replay, verification or approval semantics follow.
Existing property authority cannot authorize a toggle. Host/C input counterexamples
and the modern inspect/grant/refresh/execute/unknown journey live in
crates/host/tests/mijia_action_grants.rs,
crates/host/tests/mijia_accounts/modern_host/actions.rs and
scripts/verify_native_contract.py.


## Elevated exact power-toggle actions

ADR-0110 adds five pinned outlet/wall-switch toggles to the typed action path.
Every action needs an independent exact Core and action-family grant, followed by
actual trusted user consent through the immutable approval boundary. The adapter
re-derives elevated risk from protected source evidence and checks the private
ValidatedCommand marker before scope authorization or I/O. Public low-risk
metadata, property grants or confirmed arguments cannot bypass this check.

No state read, inferred next state, companion power/mode setting or retry is added.
Accepted remains unverified; an unknown send cannot receive another approval or
be replayed after reopen or history retirement. Logical parent-bearing entries
remain denied. Native API 1.40 startup/inspection do not admit these new toggles.
Positive/negative fixtures are examples/capability.mijia-power-action.json,
examples/command.mijia-power-action.json, scripts/validate_contracts.py and
crates/mijia/src/adapter/tests/modern_accounts/actions/power.rs. No result or
command schema shape changes; existing grant/approval semantics are retained.

## Native power-toggle consent

ADR-0111/API 1.41 admits exact elevated switch toggles through independent startup
action grants and opt-in local inspection. Granted reopen plus explicit refresh
publishes descriptors; none of these steps is consent. Only the existing immutable
trusted approval bridge can execute each action. The protected adapter checks the
private marker independently. Ordinary plan/execute refuse; unknown receipts
remain unavailable for another preview or replay after reopen/retirement.
Positive and negative boundaries are exercised by the native startup fixture,
Host actions/power.rs journey and actual C ABI validator. Outcomes and public
command/result shapes are unchanged; response inspection admits exact switch-3,
switch-11 and switch-12 components in addition to the existing switch-2.

## Exact maintenance and stop action execution

ADR-0112 adds four typed no-argument actions with existing exact action grants
and immutable trusted approval. The protected adapter re-derives elevated risk,
root/model/specification and empty arguments before the one selected-protocol send.
Camera maintenance and card ejection can interrupt monitoring/recording; hood stop
is a request, not proof of safety or rollback. Accepted remains unverified and
possible-send uncertainty remains unknown without another preview or replay.
Positive/negative fixtures are examples/capability.mijia-maintenance-action.json,
examples/command.mijia-maintenance-action.json, scripts/validate_contracts.py and
the maintenance Core tests. Command/result schema shapes are unchanged. API 1.41
keeps these operations out of startup grants and local action inspection.

## Native maintenance action consent

ADR-0113/API 1.42 exposes four exact maintenance/stop actions with independent
startup grants and explicit local inspection. These steps never replace immutable
trusted approval. Core and the protected adapter independently require consent
before a send; ordinary plan/execute remain denied. Uncertain execution cannot
receive another preview or replay after reopen/retirement. No result semantics,
write retry, state inference or companion operation changes. Positive startup and
negative argument/scope fixtures are checked by schemas, fictional Host journeys
and the rebuilt actual C ABI. Inspection response components add only the three
exact camera/storage/hood component identifiers; existing flags retain their scopes.

## Exact additional Mijia switch and outlet targets

ADR-0114 adds seven independently granted exact setters across three additional
models. Power, power-loss behavior and local-input locking are elevated; indicator
control is low risk. Full protected model/source/root evidence and outlet-family
authority are checked independently of public risk metadata. One absolute target
uses one property request on the selected protocol, without toggle, companion
writes or state inference. Accepted remains unverified and uncertain sends never
retry, renew credentials or switch protocols. Explicit refresh upgrades old
read-only derivations; existing receipts and identities remain intact.


## Typed event occurrence reads

ADR-0115 adds a single bounded historical request on the selected legacy or modern
Mijia community history profile. Exact pinned model/spec/SIID/EIID admission and
positive nonfuture window validation precede sending. Modern subject and protected
mapping checks remain independent. The result contains only correlated reported
timestamps, preserves equal-second rows and leaves completeness unproven. A full
page is not evidence of more data. No payload interpretation, subscription, retry,
renewal, fallback, state-cache update or write verification follows.

This protocol phase does not expose an ordinary request schema or Core/Host/App
method. Later Core integration must independently enforce protected membership,
read policy and account leases. Positive and negative executable fixtures live in
crates/mijia/src/event_history/tests.rs and
crates/mijia/src/modern/reads/tests/event_history.rs; local TLS tests additionally
check fixed origins, encryption headers, redirects, response bounds and cleanup.
Existing command/result schemas and write outcomes are unchanged.


## Protected Core event history

ADR-0116 now supplies typed Core event inspection and reads with default-refusing
Adapter hooks. Local inspection validates a declared event's occurrence_times
projection and rejects duplicate targets or invalid limits. A read selects one
exact connection, locks the device and enforces the positive half-open window
before adapter I/O. Core validates receipt, target, source, count and sorted record
times, then rechecks the full descriptor before publishing. Equal timestamps are
valid reported rows; no current-state or execution evidence is created.

Mijia's typed opt-in independently checks protected account/model/spec/root/listed
membership and the full published component layout, holds the account lease and
uses the selected protocol only. Old scalar descriptors do not silently acquire
event options. Changed bindings, stale/late results and cancellation fail without
fallback, reauthentication or replay. The existing event-history schema and
capability.mijia-event-history example describe the boundary; positive/negative
runtime fixtures are in crates/host/tests/event_history.rs and the Mijia protected
event-history suite. Native API 1.43 and ordinary method schemas remain unchanged.

## Native event occurrence queries (ADR-0117)

API 1.44 delegates inspect_event_history_options/read_event_history through the
ordinary Session read boundary into ADR-0116 Core validation. Startup opt-in only
selects the reviewed adapter family; it does not grant write authority, relax
protected device admission, or initiate network work. Explicit connect/refresh
publishes event descriptors. Session cancellation, background, close and deadline
release pending reads and suppress late output; there is no automatic retry,
renewal, fallback, subscription, receipt or snapshot write. Occurrence timestamps
never satisfy action verification. C/JNI signatures and storage formats are unchanged.

## CLI/MCP event-history dispatch (ADR-0118)

CLI event-history commands and local stdio MCP event_history_options/read tools
call ADR-0116 Core queries directly. Exact selection/window validation, account
and device locks, one selected-protocol request and late-result suppression remain
unchanged. Mijia hosts opt into reviewed events and modern saved-session reads;
legacy/modern selection comes from protected identity, never tool arguments. Reads
cannot initiate login renewal or fallback. EOF, cancellation and deadlines release
pending work without replay, state-cache updates or execution receipts. Eight
read-only MCP tools do not grant any appliance-control authority; the separate
Meiju startup-grant boundary still controls the two additional planning/execution tools.
