# 11. Mijia native integration

Implementation is in progress under [ADR-0070](../adr/0070-mijia-native-cloud.md).
The owner will perform real-account and device acceptance. Do not access saved
pilot credentials or contact appliances as part of development tests.

## Protocol evidence

- MIT `Squachen/micloud`, commit `dc002b87fc09ef36199715cfa10f1eaa2d2d4210`:
  [login and inventory](https://github.com/Squachen/micloud/blob/dc002b87fc09ef36199715cfa10f1eaa2d2d4210/micloud/micloud.py),
  [request encoding](https://github.com/Squachen/micloud/blob/dc002b87fc09ef36199715cfa10f1eaa2d2d4210/micloud/miutils.py).
  HomeVerb independently implements the wire format in Rust; attribution and
  the source license are retained under `third-party/micloud`.
- Current [community MIoT client](https://github.com/al-one/hass-xiaomi-miot/blob/master/custom_components/xiaomi_miot/core/xiaomi_cloud.py)
  was inspected for endpoint and response facts only. No GPL source is included.
- [Xiaomi's HA client](https://github.com/XiaoMi/ha_xiaomi_home/blob/main/custom_components/xiaomi_home/miot/miot_cloud.py)
  demonstrates a distinct registered OAuth flow; its application identity is not
  HomeVerb's authorization and is not reused.

The native password flow requests a login signature, submits the transient
password MD5 digest, then exchanges an allowlisted STS location for a user session.
Cloud requests use per-request random/time nonces, SHA-256 derived nonce keys,
SHA-1 signatures and RC4 with the first 1024 stream bytes discarded. These legacy
algorithms implement vendor compatibility within authenticated TLS; they are not
chosen for new security protocols. No shared developer secret is needed.

## Development phases

The owner selected mainland China and requested all returned devices as named
cards, including unsupported placeholders, with doorbells, door locks and fans
first. Device online hints must not become power or lock state. Fan controls need
actual readable/writable MIoT evidence and independent grants; locks and doorbells
initially expose only reviewed readable properties. Unlocking remains denied.

1. Bounded protocol, protected login/session, inventory and MIoT property decoding,
   with independent fictional vectors and failure/cancellation tests.
2. Protected bindings, exact model/spec evidence, Core adapter and account lifecycle.
3. Versioned native host and Flutter connection/read flows, then reviewed controls.
4. Owner acceptance of real login/challenges, complete regional inventory, same-account
   reconnection, device readings, phone lifecycle and separately authorized writes.

Only executed local checks will be recorded as passing. No model is verified by
source research or synthetic tests. Regional support and challenge handling must
be checked with the owner's account before a supported-account claim.

## Protocol phase evidence

On October 18, 2026, `cargo test -p homeverb-mijia --offline --locked`
passed 16 fictional tests, including independently calculated signing/encryption
vectors, login failure/challenge checks, inventory and property correlation,
verified local TLS, redirect/certificate/body/cookie rejection and pending-socket
cleanup on cancellation/deadline/drop. The initial sandbox run passed 12 cases
and blocked four socket cases; the subsequent ordinary local-socket run passed
all 16. No vendor, account, secure store or appliance was accessed.
`cargo clippy -p homeverb-mijia --all-targets --offline --locked -- -D warnings`
passed. Core registration, complete-home inventory, account persistence, App and
device semantics remain later phases rather than protocol-test conclusions.

Repository checks passed for Markdown/JSON links, 20 schemas, 49 positive and
267 negative fixtures, and English/source hygiene using the existing `.venv`.
System Python lacked the schema dependency, and creating a new temporary venv
could not download packages under restricted networking; the already-provisioned
repository venv supplied the required dependencies without installation.
Workspace Clippy with Rust 1.96 passed. Rust 1.99 workspace Clippy stops on four
existing Meiju `chunks_exact_to_as_chunks` warnings; the new crate passes on 1.99.
The workspace test run stops at 14 existing Meiju LAN cases because macOS lacks
the documented `127.0.0.2` / `127.0.1.1` aliases (`AddrNotAvailable`); these are
blocked, not passing tests. No privileged network configuration was changed.

The Rust 1.96 workspace build and formatting check passed. Existing runtime checks
also passed: 35 CLI/MCP cases with seven documented macOS skips, 34 native methods
with 55 live fictional examples plus trusted bridge suites, 68 Python unit tests,
seven presentation tests and 186 existing commit messages. These regressions do
not make the new Mijia library available through any host yet. No remote push,
publication or real-device operation was performed.
