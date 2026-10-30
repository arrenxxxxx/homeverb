# Mijia QR protocol attribution

The independent HomeVerb Rust Mijia QR authentication implementation uses
protocol evidence from MIT-licensed `ttys026/mit`, commit
`a5d32181d0251ee36dde6edb316e16879d5ac175`, inspected October 30, 2026.
Copyright (c) 2026 Troy Li. Preserve the [MIT license](LICENSE.txt).

Source: [src/mijia_api.rs](https://github.com/ttys026/mit/blob/a5d32181d0251ee36dde6edb316e16879d5ac175/src/mijia_api.rs)
(blob `8965a67d21e8921a64d4d49535717dd153dcc3be`), especially QR bootstrap,
completion and explicit renewal. HomeVerb implements its own protected types,
exact endpoint checks, original-lifetime cancellation, single-use challenge,
strict response/identity validation and bounded transport. It does not adopt
plaintext session files, configurable arbitrary origins, assumed 30-day expiry,
automatic relogin, retries or device/control behavior from the reference.

This protocol is unofficial. Source licensing and anonymous endpoint inspection
do not establish successful account authentication or physical compatibility.
