# ADR-0046: Bound Midea V3 LAN sessions

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

The codec itself opens no TCP socket, fetches or persists no device credentials,
and adds no native JSON method, route or control. Protected token/key retrieval,
identity reconciliation, inner packet/model codecs and actual device acceptance
remain required. The transport follow-up below has synthetic evidence only. The frame limit and credential
lengths are this implementation's admission limits, not universal model claims.

## Verification

Independent Python hashlib/OpenSSL vectors use fictional byte sequences and cover
the handshake, derived session traffic, both frame directions, aligned/unaligned
payloads, every truncation boundary, changed keys and corrupted data. Tests cover
coalesced prefixes, bounded lengths, counter exhaustion, constructor refusal,
redaction and compile-fail serialization. No real credential or appliance is used.

## Follow-up: one operation-owned TCP exchange

`exchange_v3` binds an outbound IPv4 socket to the selected local address and
connects once to a supplied endpoint in that private/link-local subnet. Reject
network/broadcast/self addresses, port zero, invalid outgoing payload sizes and
reused transmission markers before connecting. There is no DNS, proxy, public
network, automatic endpoint selection, reconnect, retry, worker or pooled socket.
The trusted native caller must validate protected identity, current membership,
permissions, exact model and payload before invoking this low-level function.
No entry point or registered adapter invokes it yet.

One inherited Operation owns connection, handshake, one command frame, one response
frame and cleanup. No step resets its deadline. Read exact six-byte headers before
allocating the bounded remainder; reject unexpected kinds and handshake lengths
early. Fragmented reads are completed within the same budget, and coalesced bytes
are not consumed past the selected frame. No connection escapes the call.

Handshake traffic does not transmit an appliance command. Immediately before the
first command write, set `WriteProgress` to `possibly_sent`; preserve that marker
on partial writes, disconnect, malformed responses, cancellation or future drop.
Never mark a response `accepted` or `verified` here. Framing, digest and sequence
validation do not correlate an appliance acknowledgement or establish freshness;
the future adapter must validate inner protocol and model evidence. Do not replay
a command, reconnect or switch routes because this function returned an error.

Loopback-only tests cover fragmented/coalesced frames, exact synthetic handshake
and command bytes, connection refusal, preflight guards, rejected/corrupt/truncated
handshakes, invalid/lost command responses and socket closure before/after command
transmission on cancellation, deadline and dropped futures. Clock tests wait for
actual peer I/O before advancing time. No household endpoint or real credentials
are accessed; TCP integration does not establish physical compatibility.
