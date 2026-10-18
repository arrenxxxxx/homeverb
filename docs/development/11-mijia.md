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
- Pinned [community MIoT client](https://github.com/al-one/hass-xiaomi-miot/blob/a3a07b8504bacfee114a6052ae864d7dd149bf71/custom_components/xiaomi_miot/core/xiaomi_cloud.py)
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

## Inventory and specification phase

The native library now unions the legacy account device list with accessible
home device pages. Limits are 300 homes, 16 pages per home, 300 entries per page
and 4096 unique devices within one inherited deadline. Duplicate identity with a
different model/specification is refused. Home failures retain positive results
with sanitized issues, while authorization failure and cancellation stop the
operation. Community home-list coverage is not proven globally complete, so the
result cannot remove historical bindings or claim a complete-inventory timestamp.

Explicit public specification lookup uses the
[MIoT index](https://miot-spec.org/miot-spec-v2/instances?status=all) and exact
instance URNs. Index responses are capped at 8 MiB/100000 entries; instance and
account responses remain capped at 1 MiB. No account cookies or encryption headers
go to the public specification origin. There is no startup/background fetch.
Only exact released model matches and unambiguous selected versions are admitted.

Reviewed wire facts were inspected in public specifications for
`zhimi.fan.za4` revision 1, `loock.lock.v1` revision 1 and
`linp.doorbell.g03` revision 1 on October 18. These are protocol examples, not
owner-device identification or support certification. The code projects only
matching standard service/property URNs with actual `read` access:

- Fan power is a Boolean `power.state`. Fan level, mode, horizontal swing and
  angle retain `vendor.xiaomi.mijia.fan` semantics and reported types.
- Lock `status` remains a raw numeric `vendor.xiaomi.mijia.lock.status`; a vendor
  code such as `255` does not become an unlocked/locked Boolean. No lock writes.
- Doorbell volume and readable battery level remain vendor properties. Neither
  implies a doorbell power switch. No camera stream, event history or notification
  subscription is added.
- Identity/serial properties, event-only fields, arbitrary vendor extensions and
  other device families are not mapped. They will remain unsupported placeholders.

Capabilities currently have no actions and all properties remain read-only.
Reported fan write domains are retained internally as evidence only, with type,
range, step and enum checks; future controls still need independent startup grants
and execution validation. Per-service component IDs prevent merging distinct units.

`cargo +1.96.0 test -p homeverb-mijia --offline --locked` passed 28 fictional tests;
`cargo +1.96.0 clippy -p homeverb-mijia --all-targets --offline --locked -- -D warnings`
passed. Added cases cover model/version/refusal, fan/lock/doorbell distinctions,
identity/event exclusion, exact home union, pagination cycles/conflicts, partial
errors, cancellation and actual TLS proof that spec requests omit account cookies.
Core/vault registration and native/App connection are still pending.

Public spec response SHA-256 values at inspection (source data is not vendored):

| Source | SHA-256 |
|---|---|
| Fan `urn:miot-spec-v2:device:fan:0000A005:zhimi-za4:1` | `cab2d8b2de82ca17db5abc7c701fb589fb294253afeb08304d0a064f9a1ca859` |
| Lock `urn:miot-spec-v2:device:lock:0000A038:loock-v1:1` | `e4e25ddb4a2ce037414ebb95e7f0092df71f2914e3f973291af1596a94ea8233` |
| Doorbell `urn:miot-spec-v2:device:doorbell:0000A03B:linp-g03:1` | `aa4bf6dc423e22d764568d2d868b6be3467d089a110b526cf61930b01477e732` |

Each instance is available from
`https://miot-spec.org/miot-spec-v2/instance?type=<exact-URN>`. Tests use independent
fictional specifications; public-model facts do not establish a physical test.
