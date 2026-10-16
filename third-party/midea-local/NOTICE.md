# Midea Local attribution

Source: https://github.com/midea-lan/midea-local

Pinned revision: `ef28169926914d15f40fe521b9cc3d3c2f553c59`.

Upstream files: `midealocal/discover.py`, `midealocal/security.py` and
`midealocal/packet_builder.py`; V3 authentication also references `midealocal/device.py`,
and Meiju v2 credential lookup references `midealocal/cloud.py`.
Standard AC query/C0 mapping also uses `midealocal/message.py`, `midealocal/crc8.py`,
`midealocal/devices/ac/message.py` and the profile caveats in its `__init__.py`.

License: MIT; retain the complete upstream [LICENSE.txt](LICENSE.txt) when
distributing the derived portions. HomeVerb's release license remains undecided.

HomeVerb modifications: a bounded Rust decoder for V2 and V3-wrapped discovery
packets, with strict inner framing/checksum/padding validation, native-only
redacted candidates and fictional independent test vectors. A new operation-owned
collector replaces upstream networking with explicit subnet scope, a fixed total
window, response limits and cancellation. No upstream network loops, credential
caches, device logging or automatic discovery are copied.
The V3 port uses protected handshake/session buffers, bounded complete frames,
fixed errors, strict direction/digest checks and nonwrapping request counters.
It includes digest bytes in declared lengths even for aligned plaintext. An
operation-owned TCP helper replaces persistent/reconnecting upstream transport
with one scoped exchange, bounded exact frame reads and command-send tracking.
It neither fetches credentials nor copies default keys or automatic retries.
A separate account-bound lookup ports the Meiju v2 request shape and six-byte
UDP-ID derivation. It requires fresh unique home membership and strict token/key
selection, returns protected handshake material, and omits upstream fallback,
default keys, credential persistence and automatic method switching.
The ordinary packet port bounds `5a5a` frames, checks the expected six-byte
identity and strict padding before returning protected plaintext, and retains
the source timestamp format without treating it as observation evidence. It
does not copy permissive stream parsing, raw packet logging or status callbacks.
The AC query/C0 port replaces the shared wrapping counter with an explicit query
sequence, implements the equivalent CRC bitwise, validates AA bounds/checksum,
preserves unavailable sensor values and rejects unsupported units/invalid decimals.
It returns native candidates without copying model admission or freshness claims.
Public fixed wire constants do not authenticate a device or protect HomeVerb
credentials. LAN session authentication and physical compatibility are not
established by this port.
