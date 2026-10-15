# ADR-0045: Decode bounded Midea LAN discovery candidates

Status: accepted for offline implementation within the existing additional-LAN scope.

## Evidence and scope

Port the discovery envelope and AES-128-ECB/PKCS#7 compatibility format from
`midea-local` revision `ef28169926914d15f40fe521b9cc3d3c2f553c59`:
`midealocal/discover.py`, `security.py` and `packet_builder.py`. Preserve its
[MIT notice](../../third-party/midea-local/NOTICE.md). These public protocol
constants are not user credentials or secret-store cryptography.

Begin with a pure decoder and the fixed discovery request. Accept at most 512
bytes, matching the reviewed UDP receive bound. Recognize the inner V2 `5a5a`
packet and the V3 discovery wrapper with eight leading and sixteen trailing bytes.
Validate the inner declared length, ciphertext block size, salted MD5 trailer,
PKCS#7 padding, nonzero 48-bit appliance ID, advertised TCP port and SSID family
field. Reject truncated or oversized data instead of guessing missing fields.
V1 XML discovery and its extra TCP exchange remain unsupported.

Use the actual datagram source address supplied by the caller, not an address
embedded in the response. Do not retain the serial number, MAC, SSID or derived
display model. Keep the appliance ID and endpoint in a native-only candidate with
redacted Debug output and no serialization implementation. Preserve unknown family
codes without granting capabilities. Fixed errors must not contain packet data.

The checksum uses a public constant and is not authentication. V3 discovery
framing does not establish a V3 authenticated session; the outer trailer has no
verified authentication meaning here. A candidate is untrusted identity evidence,
not a Device, connection, pairing result, permission or physical-support claim.
Do not merge cards by advertised appliance ID, model, name, address or family.

## Integration gates

This phase opens no sockets, reads no account and changes no public native JSON,
C export, storage format or cloud execution behavior. It does not make the LAN
route available in CLI, App or MCP. Before exposing that route, implement bounded
foreground UDP collection, protected account/identity reconciliation, V3 token/key
retrieval and handshake where required, framed TCP operations and exact-model
message codecs. Preserve existing route and uncertain-write rules. Actual LAN
scanning and physical/model acceptance remain separate authorized operations.

## Verification

Use independent Python hashlib and OpenSSL known-answer packets containing only
fictional data. Test V2 and wrapped V3 decoding, AC/E1/CA and unknown families,
all truncation boundaries, oversized frames, checksum/ciphertext corruption,
invalid padding, lengths, IDs, ports and family fields, and redaction. Compile-fail
coverage must prevent accidental candidate serialization. No network or live
appliance is needed for these checks.
