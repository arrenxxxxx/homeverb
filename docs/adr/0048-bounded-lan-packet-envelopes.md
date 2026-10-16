# ADR-0048: Bound the Midea LAN packet envelope

Status: accepted for synthetic implementation within the additional-LAN scope.

## Evidence and decision

Port the ordinary `5a5a` packet layout from the pinned MIT `midea-local` revision
`ef28169926914d15f40fe521b9cc3d3c2f553c59`, `packet_builder.py`, `security.py` and
the framing in `device.py`. The existing V3 transport carries these packets as
encrypted payloads. Reuse the public AES-128-ECB key and salted MD5 material already
attributed for discovery; neither is a secret or peer authentication mechanism.

Expose native pure `encode_lan_packet`, `decode_lan_packet` and `lan_packet_len`
functions. Ordinary packets have a 40-byte header, PKCS#7-padded encrypted body
and 16-byte salted MD5 trailer. Admit only the `01 11` data message type and a
nonempty plaintext body. Heartbeat/ack-only envelopes and other variants remain
unadmitted. Header fields do not distinguish request, response or discovery
semantics; the following inner model parser must validate the payload before use.

Encode the selected nonzero six-byte appliance ID, the source's reversed
two-decimal-digit UTC time fields (centiseconds through century), and the ordinary
request flags. Take the timestamp explicitly; reject years outside 1..9999 and
chrono leap-second representations that Python datetime cannot encode. Do not
derive freshness or causal evidence from this outgoing timestamp.

The decoder requires the exact expected native appliance ID from its trusted
caller. Check the complete length, checksum, six-byte identity and every padding
byte before releasing plaintext through `SecretBytes`. Bytes 26 and 27 are
reserved, not additional appliance-ID bits, matching discovery identity scope.
Do not expose header timestamps, message IDs or metadata as observation evidence.
Malformed or partial packets never return partial plaintext. Fixed errors do not
include addresses, provider IDs, ciphertext or decrypted contents.

Cap the complete envelope at 4040 bytes (3983 plaintext bytes). This is the
largest block-aligned packet that fits inside the existing 4096-byte V3 frame
limit. The six-byte prefix helper reports only a bounded frame length; it neither
allocates a buffer nor claims validity. Exact decoding rejects coalesced suffixes;
a future stream caller must split at the reported boundary within its Operation.

## Integration and verification

No new socket, cloud request, credential access, automatic retry, route, native
JSON method or storage format is introduced. The packet helper does not validate
an endpoint, model, permission, inner command or write acknowledgement. A matching
ID and public checksum are forgeable and do not replace V3 key confirmation or
protected identity reconciliation. Decoding does not mark accepted/verified,
authorize a fallback or change transmission progress. Model message codecs and
physical acceptance remain necessary for a usable LAN route.

Use independent Python hashlib and OpenSSL vectors with fictional identifiers,
timestamps and plaintext, including block boundaries and the maximum size.
Verify exact outgoing bytes, all truncations, coalesced prefixes, changed identity,
checksum failure, valid-checksum invalid padding, unsupported types and malformed
lengths. Check that maximum packets fit the existing V3 encoder and that protected
outputs remain redacted/nonserializable. No real account or appliance is used.
