# ADR-0054: Confirm a foreground scan for one saved Meiju device

Status: accepted for synthetic implementation within the additional-LAN scope.

## Decision

Connect the existing bounded UDP collector and protected candidate reconciliation
through a typed native Session operation. It searches for one selected saved
device on one explicitly selected network. It does not authenticate a LAN peer,
fetch a token/key, open TCP, read appliance state, add a connection or change cards.

`prepare_meiju_lan_scan` takes exact local device/connection IDs and an owned
`DiscoveryNetwork`. The trusted host must obtain the selected assigned interface
from its platform and handle local-network permission; private address syntax
does not prove interface ownership or consent. Expose native address/prefix
getters for the immutable preview. No ordinary JSON address, endpoint or
`confirmed` parameter is accepted. Preparation validates Core metadata and the
protected mainland account binding/listed membership without network traffic.

Return an opaque, non-cloneable, nonserializable `PendingMeijuLanScan`, containing
the descriptor, connection and network. It belongs to that Session and lifecycle
generation and expires after 60 monotonic seconds. Show its exact device and
network before actual trusted UI consent. Dropping it dismisses the request.
`scan_meiju_lan_candidate` consumes it on every attempt, including host admission
failure; it is not a general command or an elevated appliance-write approval.

Confirmation rechecks the Session, generation, effective descriptor and protected
binding. Hold the account lease through one collector invocation and subsequent
protected reconciliation. Use the original operation budget, capped by preview
expiry; the collector separately caps setup/collection at five seconds and sends
one probe to each of its two protocol ports. No automatic rescan or cloud fallback.
Background/close cancels active work and invalidates previews; cancelled, expired
or dropped work releases leases/sockets. Check lifecycle again before returning.

Return native `LanScanOutcome`: `NotObserved`, `Incomplete`, `Ambiguous` or
`UniqueCandidate`. Only the selected candidate can leave the adapter; unrelated
device hints are dropped. The result is nonserializable/redacted and confers no
route, credential or model authority. Missing evidence is not proof of absence;
limits and conflicting hints cannot yield a chosen endpoint. Existing account
and report validation still applies before and after collection.

Use an injected `LanDiscovery` transport with a production `UdpLanDiscovery`
implementation. Like the cloud transport, injection is a trusted compiled native
boundary for deterministic integration tests, not a downloadable plugin or a
caller-supplied ordinary request. Construction performs no network operation.

## Compatibility and verification

Add fixed native scan-invalid/expired errors and locale entries. Native JSON API
1.14.1, C/JNI exports, CLI/MCP, storage formats, cloud routing and write semantics
are unchanged. The mobile consent/permission/interface-selection bridge remains
pending; this phase does not claim phone or physical-device support.

Exercise the actual Session, adapter, account vault and SQLite with fictional
secrets and a scripted discovery transport. Verify no network during preparation,
one invocation after confirmation, selected-account identity and membership,
limits/conflicts, retained leases, metadata/lifecycle changes, foreign/expired
previews, cancellation/deadline/drop, redaction and unchanged storage. Existing
loopback collector tests prove UDP bounds/closure without household traffic.
No real account, appliance or platform permission is accessed by this phase.
