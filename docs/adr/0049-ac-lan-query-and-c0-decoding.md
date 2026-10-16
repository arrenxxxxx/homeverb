# ADR-0049: Encode AC LAN queries and decode scoped C0 candidates

Status: accepted for synthetic implementation within the additional-LAN scope.

## Source and scope

Use `message.py`, `crc8.py`, `devices/ac/message.py` and the model caveats in
`devices/ac/__init__.py` from the pinned MIT `midea-local` revision
`ef28169926914d15f40fe521b9cc3d3c2f553c59`. The ordinary packet/V3 envelope already
has separate validation. This phase adds the inner standard AC query and C0
candidate mapping only, without registering a model or issuing a request.

`encode_ac_status_query(protocol_version, sequence)` constructs the reviewed
33-byte AA query: family AC, query kind 3, body `41 81 00 ff` followed by the
remaining fixed query bytes, explicit sequence, Dallas/Maxim CRC-8 and AA sum
checksum. Admit sequences 1..253, matching the source's generated values. Never
allocate a process-global counter, wrap or retry automatically. An explicit
protocol byte is wire input, not proof that a model supports that version.

`decode_ac_c0_status(expected_protocol_version, message)` accepts exactly one
complete AA frame (maximum 256 bytes) with the AC family, matching protocol byte,
declared length and sum checksum. Require query kind 3 and C0 body type, and at
least 15 body bytes. Set replies, notifications, alternate body/header formats
and unsupported units remain unadmitted. No nested C0 CRC or sequence-correlation
claim is made: the inspected source does not establish those response semantics.

Return native-only redacted `AcC0Status` candidates: power, raw mode/fan codes,
target Celsius temperature and optional indoor/outdoor Celsius temperatures.
The target is the low temperature nibble plus 16 and its separate half-degree
bit. Ambient 0xff is unavailable, never zero. Follow the source's ambient
half-degree rule and use decimal nibbles only when the body exceeds 20 bytes.
For a nonzero decimal, truncate the signed half-degree integer toward zero and
apply the decimal with the original sign. Reject decimal nibbles above 9 when
the corresponding sensor is available; ignore irrelevant decimals for 0xff.
An explicit Fahrenheit bit returns unsupported rather than an assumed conversion.

## Model and execution boundaries

Do not apply a brand-wide range, enum or capability definition to these native
values. In particular, the source documents stale C0 temperatures for product
22013279 and alternate BB/new-protocol paths. This decoder does not select a
product profile or promise physical compatibility. A future adapter must use
protected model/profile evidence to admit its query and fields, and keep
unproven freshness explicit. Successful parsing is not a permission, correlated
acknowledgement, observation timestamp, state cache update or verified write.

No endpoint, credentials, account, networking, native JSON method, SQLite schema,
session format, control encoder or automatic background work changes. The caller
must first authenticate and reconcile the endpoint and validate its outer packet
against the protected device identity. Existing cloud routes remain unchanged.

## Verification

Independent fictional frames cover query CRC/checksum and sequence boundaries,
C0 on/off, raw codes, half/decimal/negative temperatures, unavailable sensors,
short/extended/max bodies, unsupported units/types/families/versions, malformed
lengths/checksums and all truncations. Compare CRC results against the complete
pinned upstream lookup table and positive field mappings against its unchanged
C0 parser. Exercise the packet-to-C0 decode chain without opening sockets.
Compile-fail and Debug checks keep the native result out of ordinary serialization.
