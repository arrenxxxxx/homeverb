# ADR-0053: Decode bounded standard dishwasher and refrigerator LAN reads

Status: accepted for synthetic implementation within the selected appliance scope.

## Evidence and decision

The pinned `midea-local` revision `ef28169926914d15f40fe521b9cc3d3c2f553c59`
defines standard E1 and CA `MessageQuery` requests in their respective
`midealocal/devices/*/message.py` files. Both are AA query messages with one
zero body-type byte and no additional body. Their device implementations use
the negotiated appliance message version. CA also has a different Toshiba
0x55 path; do not select that path or infer manufacturer/model compatibility.

Implement the standard query encoders and strict query-response decoders. Check
complete AA bounds/length, expected family/version, sum checksum, query kind 3
and body type zero. Refuse setter acknowledgements, notifications, exception
frames and alternative headers/layouts. A malformed reply returns no partial
candidate. No retries, default values for missing bytes or freshness claims.

E1 requires at least the 17-byte general prefix consumed by the source. Return
raw status, mode, additional-option and temperature codes; door-open and
water/salt/rinse-aid shortage bits; and remaining minutes. Preserve the source's
length-dependent extension: include byte 32 as the high time byte only when the
body has at least 34 bytes. Do not invent program names, completion, a start/pause
state, missing optional sensors or a temperature unit from these bytes.

CA requires at least its 25-byte general prefix. Return raw mode flags, the two
main compartment power bits, packed Celsius targets and half-degree actual
temperatures. Refuse the Fahrenheit flag because its alternative fields are
outside this port. Do not clamp reads to ordinary setpoint bounds. Do not parse
flex zones, door/energy/exception/optional extensions or copy the source's default
zeros for absent fields. Additional bytes carry no admitted semantics here.

Add `query_dishwasher_v3` and `query_refrigerator_v3` alongside `query_ac_v3`.
Share the existing bounded sequence: validate inputs, one V3 handshake, appliance
message-version negotiation, then one family-specific query with a new clock
timestamp. Check the expected private target in both ordinary packet replies,
then family/version/kind in each inner message. Cancellation, deadline, failure
or future drop closes the socket. Response counters are not assumed to echo
requests. No background refresh, reconnect or extra protocol query is added.

## Admission and verification

These functions require already-admitted native endpoint/network/credentials.
Return nonserializable, redacted native candidates, not Core Snapshots, public
device cards or evidence of physical support. Standard source defaults and cloud
model mappings do not establish exact-model LAN admission. Meiju still refuses
the connection-link hook; no new JSON, C/JNI, CLI or MCP method is introduced.
API 1.14.1, cloud behavior, grants and storage formats remain unchanged.

Compare independent fictional requests and selected response fields with the
unchanged pinned source classes. Test flag polarity, signed/fractional Celsius,
time-extension boundaries, short/truncated/oversized/foreign/unsupported frames,
and redaction. Exercise both composed sequences with independent nested packet
vectors and loopback peers, including identity/context failures, lifecycle
interruption and socket closure. Retain upstream attribution. Do not contact a
real account or appliance, or claim the owner's models have passed this phase.
