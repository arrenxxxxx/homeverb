# ADR-0051: Negotiate the message protocol before a bounded AC LAN read

Status: accepted for synthetic implementation within the additional-LAN scope.

## Evidence and decision

The pinned `midea-local` revision `ef28169926914d15f40fe521b9cc3d3c2f553c59`
implements `MessageQueryAppliance`/`MessageApplianceResponse` in `message.py`.
`device.py::refresh_status` sends and awaits the appliance query before building
status queries because some devices reject a guessed message protocol version.
This is distinct from the outer V2/V3 transport protocol.

Add pure native appliance-query helpers for AC/E1/CA. Encode the standard 30-byte
AA request with protocol byte zero, message kind A0 and 19 zero body bytes.
Decode only a complete AA response of 12..256 bytes with exact length, expected
family, sum checksum and kind A0; return its protocol byte. The source defines
no additional body semantics here. Do not infer model, subtype, identity, status,
freshness or request correlation from that byte. Unsupported families and other
message kinds are refused; malformed responses never trigger a default version.

Add `query_ac_v3(network, endpoint, handshake, target, clock, sequence,
operation)` as a low-level native composition. The trusted caller must separately
admit the account-bound endpoint, model and explicit foreground network access
before invoking it. A unique discovery hint alone does not grant that authority.
The helper accepts already-protected handshake material and the expected private
target; it does not fetch keys, choose endpoints or register a route.

Within one inherited Operation and one TCP connection:

1. Validate inputs and encode the appliance query before opening a socket.
2. Complete one V3 handshake.
3. Send the wrapped AC appliance query and validate the V3 response, ordinary
   envelope target and AA appliance reply before selecting its message version.
4. Build and send one standard AC status query with that version and the explicit
   caller sequence and a new timestamp from the injected clock. Validate its
   response envelope and C0 version/family/kind.
5. Return native `AcC0Status` candidates and close the connection.

No retry, extra query, reconnect, fallback, persistent session, background task
or partial result is allowed. Failure, cancellation, deadline or future drop
closes the socket. Existing `exchange_v3` write transmission tracking remains
unchanged. These two read requests do not produce a write receipt or claim
acceptance. V3 response counters are decoded but not assumed to echo requests;
the source does not establish that correlation rule.

## Boundaries and verification

This composition does not establish a model-specific Core route, a fresh
Snapshot, physical support or the mobile LAN journey. Known C0 temperature
caveats and BB alternatives still require protected model/profile admission.
Account reconciliation and credential lookup remain explicit separate steps.
Native JSON API 1.14.1, C/JNI exports, storage formats and cloud behavior do not
change. Reuse existing fixed errors without exposing endpoints or protocol data.

Use independent fictional AA and nested packet/V3 vectors and a loopback peer.
Verify negotiated nonzero versions, exactly two ordered queries after one
handshake, expected identity at both replies, malformed/unsupported negotiation
stopping before status, invalid C0, preflight refusal, cancellation/deadline/drop
at both query stages, and socket closure without reconnects. Compare appliance
query bytes and protocol extraction with the unchanged pinned source classes.
