# Product requirements and scope

Requirements baseline: v0.1.1-docs. The 0.2.0 Mock MVP implements a subset; see [task status](task-breakdown.md). Real-device and mobile release gates below remain unmet.

## Positioning

Provide independent cross-brand appliance-control tools for users, developers, and local AI hosts. Unify access, capabilities, and invocation, without building another central smart-home platform. Run when requested; stop when finished. A vendor cloud is not a backend operated by this project.

## User journeys

| User | Journey | Acceptance criterion |
|---|---|---|
| CLI user | Install binary -> authenticate/pair -> discover -> control | No pre-existing daemon; scriptable JSON; exit after the command |
| Mobile user | Open app without a HomeVerb login -> authorize a platform or receive eligible authorization -> discover -> organize rooms -> control | The app itself controls devices without a computer or NAS |
| Midea/Hualing owner | Connect the existing Midea Meiju account -> automatically retrieve bound devices -> read and control supported appliances | No re-pairing or manual device credentials; direct vendor-cloud operation at home or away, including mobile data |
| AI user | Configure a local executable -> call read or authorized write tools | Host-owned stdio session; no public network listener |

## Required functionality

Multiple platforms and accounts; viable native authentication; LAN/cloud discovery and deduplication; stable local IDs and aliases; dynamic capability descriptions; cached snapshots and on-demand refresh; common actions; explicit route policies; execution verification; structured errors; protected credentials; access control; and redacted diagnostics. CLI, App, and MCP use the same Core API.

The owner-selected Flutter app uses custom local rooms and device cards, with Devices, Platforms, and Settings as its primary destinations. Platform management uses consumer-facing appliance-platform terminology. Explicit nearby authorization sharing belongs in Settings and follows [ADR-0008](adr/0008-foreground-authorization-sharing.md), with no app account, background receiver, or sender relay. See the [screen inventory](mobile-app.md) for the planned functions; sharing eligibility requires separate provider evidence.

The first real integration targets Midea/Hualing air conditioners, dishwashers, and refrigerators already bound in Meiju. Successful provider authorization starts bounded foreground account-device retrieval and automatically adds supported cards; unsupported returned devices remain visible with their limitation. Reauthorization preserves local organization and identity. Daily cloud reads and controls are required, while LAN is an additional route on the same Device. No LAN discovery or home gateway is required for the cloud journey. Exact controls depend on model, region, account permissions, and tested protocol evidence; this is not a promise of complete official-app feature parity. See the [integration plan](development/05-midea.md).

## Out of scope

A replacement HA integration platform, persistent NAS gateways, microservices, deployed Docker services, cross-device cloud sync, historical charts, continuous collection, rule engines, unattended project scheduling, remote HTTP MCP, webhook receiver services, centrally hosted accounts, and a universal first-time Wi-Fi provisioning engine.

Bounded foreground observation and waiting for a result are allowed. Hidden background loops are not. Native device schedules may be considered as future device capabilities, but the project does not promise to execute future schedules itself.

## Admission without a project server

The client must be able to authenticate legitimately, obtain the required authorization, and reach the target itself. Platforms that require confidential-client backend authentication with no native alternative are `blocked`. Embedding a shared secret in a public client is not a workaround. The owner-approved [ADR-0022](adr/0022-meiju-community-cloud-admission.md) narrowly admits the pinned public Meiju community app-compatibility material for the personal MVP; confidential developer credentials remain prohibited.

For LAN-only devices, the current process must reach the device network. Otherwise return `LAN_UNREACHABLE` before sending. Existing user-managed VPNs are not deployed or maintained by this project. Offline reads may use available cache entries only when clearly marked as cached.

## Non-functional targets - not measurements

A normal command has a default total budget of 10 seconds; discovery defaults to 10 seconds; explicit interactive authentication has a separate 180-second budget. Overrides are bounded. The target in a controlled test environment is to stop remaining client work within 2 seconds after cancellation. Protocol-specific exceptions must be documented; infinite retries are never hidden.

No default telemetry or diagnostic uploads. Listing local configuration without an account must not trigger vendor requests. Offline inspection must not contact an update service.

## Release gate

From a clean environment with all project processes stopped, one client must independently control a verified device. Exiting leaves no daemon, startup entry, persistent listener, or periodic task. Test CLI and mobile independence separately. Two different protocol families must pass the same capability calls before claiming that the cross-platform abstraction is demonstrated.
