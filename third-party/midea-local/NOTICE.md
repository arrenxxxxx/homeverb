# Midea Local attribution

Source: https://github.com/midea-lan/midea-local

Pinned revision: `ef28169926914d15f40fe521b9cc3d3c2f553c59`.

Upstream files: `midealocal/discover.py`, `midealocal/security.py` and
`midealocal/packet_builder.py`.

License: MIT; retain the complete upstream [LICENSE.txt](LICENSE.txt) when
distributing the derived portions. HomeVerb's release license remains undecided.

HomeVerb modifications: a bounded Rust decoder for V2 and V3-wrapped discovery
packets, with strict inner framing/checksum/padding validation, native-only
redacted candidates and fictional independent test vectors. No upstream network
loops, credential caches, device logging or automatic discovery are copied.
Public fixed wire constants do not authenticate a device or protect HomeVerb
credentials. LAN session authentication and physical compatibility are not
established by this port.
