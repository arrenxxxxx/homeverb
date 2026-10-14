# Platform and protocol adapter contract

## Responsibilities

Adapters own viable authentication and pairing, discovery, identity evidence, capability descriptions, state reads, protocol writes, acknowledgement mapping, bounded subscriptions, and cleanup. Core owns common permissions, routing, total budgets, execution state, and log redaction.

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

## Discovery and identity

LAN scans have deadlines and user-authorized network boundaries; never scan the public internet. A URL or IP returned by a discovered device is untrusted input. Validate allowed networks and vendor endpoints to prevent malicious responses from triggering arbitrary requests. Cloud enumeration is account/region scoped, paginated, rate-limited, and deduplicated.

## Plugin packaging

Version one uses compile-time registration and explicit feature switches. Mobile packages contain reviewed adapters. Do not promise runtime code downloads on every mobile platform. A later desktop-specific bounded subprocess mechanism may be evaluated separately, but reusing a Python library must not require a companion service.

## Definition of done

Authentication feasibility evidence -> fictional/redacted fixtures -> contract tests -> one physical device in read-only mode -> owner-authorized safe writes -> verification and fault cases -> compatibility matrix update. Without a real device, use `experimental` or `mock-tested`, not `verified`.

Libraries such as `midea-local` can inform protocol work. Citing one does not establish a port, mobile compatibility, or support for every model. [R5](references.md)
