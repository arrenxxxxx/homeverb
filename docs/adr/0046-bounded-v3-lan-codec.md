# ADR-0046: Bound the Midea V3 LAN wire codec

Status: accepted for synthetic implementation within the additional-LAN scope.

## Evidence and decision

Port the V3 handshake and `8370` wire layout from the MIT `midea-local` revision
`ef28169926914d15f40fe521b9cc3d3c2f553c59`, specifically `security.py` and the
`device.py` authentication call. Retain the [notice](../../third-party/midea-local/NOTICE.md).
Use RustCrypto AES-256-CBC with the protocol's zero IV, SHA-256 wire digests and
constant-time digest comparison. This legacy compatibility format is not suitable
for HomeVerb storage or authorization sharing; it does not replace modern AEAD.

`V3Handshake` accepts only an explicitly supplied 64-byte token and 32-byte key in
native `SecretBytes`. Its 72-byte request is also protected; there are no default
keys, plaintext cache or credential guesses. Consume the exchange when validating
one complete type-1 response. Decrypt its 32-byte challenge, check its SHA-256
digest, and XOR with the supplied key to derive a native-only session key.
Explicit `ERROR` and failed key confirmation return fixed `AUTH_REQUIRED`, not
automatic cloud login or credential replacement. Malformed framing is separate.

Encrypted requests use type 6; only type-3 frames decode as responses. The header
length always includes the 32-byte digest. The pinned encoder adds those bytes
only when padding is needed; preserve the decoder's complete-frame length rule
for aligned payloads too, and cover this boundary with independent vectors.
Random padding comes from the OS. Empty or oversized outgoing payloads are
rejected before encoding. Requests use sequences 0 through 65534, then refuse
further encoding rather than silently wrap and reuse a sequence in one session.

Cap a complete frame at 4096 bytes. `v3_frame_len` examines a six-byte header
without buffering and supports callers splitting fragmented/coalesced TCP data.
A reported length is not validity. Handshake and response decoders require one
exact complete frame and reject unknown types, bad alignment, impossible padding,
oversize, trailing data and failed digests before exposing plaintext. Key,
request, plaintext and padding ownership use zeroizing buffers; AES/CBC cleanup
features are enabled. Native types have redacted Debug and no serialization.

## Trust and integration limits

Key confirmation alone does not bind an account, advertised ID, endpoint or exact
model. A decoded response sequence is not replay protection, freshness, causal
verification or proof of a successful write. Keep protected identity, membership,
permission, model, observation and write-progress checks in their existing layers.
Do not use decode success to authorize another route or retry an uncertain write.

This phase opens no TCP socket, fetches or persists no device credentials, and
adds no native JSON method, route or control. Protected token/key retrieval,
identity reconciliation, bounded TCP session ownership, inner packet/model codecs
and actual device acceptance remain required. The frame limit and credential
lengths are this implementation's admission limits, not universal model claims.

## Verification

Independent Python hashlib/OpenSSL vectors use fictional byte sequences and cover
the handshake, derived session traffic, both frame directions, aligned/unaligned
payloads, every truncation boundary, changed keys and corrupted data. Tests cover
coalesced prefixes, bounded lengths, counter exhaustion, constructor refusal,
redaction and compile-fail serialization. No real credential or appliance is used.
