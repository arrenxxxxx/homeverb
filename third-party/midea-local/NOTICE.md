# Midea Local attribution

Source: https://github.com/midea-lan/midea-local

Pinned revision: `ef28169926914d15f40fe521b9cc3d3c2f553c59`.

Upstream files: `midealocal/discover.py`, `midealocal/security.py` and
`midealocal/packet_builder.py`; V3 authentication also references `midealocal/device.py`.

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
It includes digest bytes in declared lengths even for aligned plaintext. It
neither fetches credentials nor opens TCP connections or copies default keys.
Public fixed wire constants do not authenticate a device or protect HomeVerb
credentials. LAN session authentication and physical compatibility are not
established by this port.
