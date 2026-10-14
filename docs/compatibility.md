# Platform and device compatibility matrix

> This is a design and contract repository. Every row below is a candidate direction, not an implemented support claim. There are currently no physical-device verification records for this project.

| Candidate | Potential route | Evidence still required | Project status |
|---|---|---|---|
| Midea platform/protocol | LAN; vendor-cloud onboarding may be required | Model, vendor app, key acquisition, protocol version, region | planned / unverified |
| Hualing devices | Adapter selected by actual protocol | Verify each model; do not infer from brand | planned / unverified |
| Samsung televisions | Candidate LAN protocol | Pairing, firmware, wake behavior, capabilities, authentication | planned / unverified |
| SmartThings | Vendor cloud API | Backend-free native authorization, region, API permissions and cost | planned / auth gate pending |
| Hisense devices | Candidate account platform/API | Regional platform, app, account, protocol, native authorization | planned / unverified |

Brand names express user interest, not protocol boundaries. `midea-local` and SmartThings CLI provide protocol/API references, but do not prove native authentication, mobile embedding, or model support in this project. [R5 and R6](references.md)

## Status definitions

Foreground authorization sharing is also planned and unverified for every candidate. Login support alone does not establish transferability. Before enabling sharing, record provider-specific evidence for permission scope, device binding, refresh-token behavior across installations, expiry, and revocation under [ADR-0008](adr/0008-foreground-authorization-sharing.md).

| Status | Meaning |
|---|---|
| planned | Candidate requirement only |
| protocol-researched | Traceable protocol evidence exists |
| mock-tested | Fictional/redacted contract fixtures pass |
| experimental | Implementation exists with limited verified paths |
| verified | A specific model and firmware passed physical acceptance |
| blocked | Authentication, networking, licensing, or another hard gate is unmet |
| unsupported | Explicitly outside the current scope |

## Required verified record

Brand, exact model, category, firmware, region, vendor app, account type, transport, adapter version, verification date, reproducible steps, tested reads/writes, verification method, unsupported features, disconnect tests, and duplicate-execution tests. Remove accounts, IPs, MAC addresses, and serial numbers before committing evidence.

## Independent native-client gate

Do not skip authentication feasibility merely because another feature works. Label cloud-only operation `cloud-required`. Separate cloud-dependent onboarding from daily LAN operation. Platform support does not imply every product sold under that brand is supported.
