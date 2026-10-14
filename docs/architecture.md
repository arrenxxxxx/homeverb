# System architecture

## Runtime view

![Architecture overview](diagrams/architecture-overview.svg)

One repository, one control library, three independent hosts. The CLI process, mobile process, and MCP child process each own their own Core instance. The shared box in the diagram represents code structure, not a node that all clients access over HTTP.

```mermaid
flowchart TB
  CLI[CLI: command lifetime] --> C1[In-process Core]
  APP[App: foreground operation] --> C2[In-process Core]
  MCP[MCP: host stdio session] --> C3[In-process Core]
  C1 --> A1[In-process adapters]
  C2 --> A2[In-process adapters]
  C3 --> A3[In-process adapters]
  A1 --> LAN[Appliance LAN interfaces]
  A1 --> CLOUD[Vendor cloud APIs]
  A2 --> LAN
  A2 --> CLOUD
  A3 --> LAN
  A3 --> CLOUD
```

## Layers and dependency direction

| Module | Responsibility | Forbidden dependencies |
|---|---|---|
| CLI / Mobile / MCP | Input, interaction, output, trusted host identity | Vendor wire details; invoking and parsing another entry point |
| Application / Core | Validation, policy, routing, execution, verification | UI frameworks and MCP wire protocol |
| Domain / Schema | Devices, capabilities, commands, results, errors | Network libraries and database drivers |
| Adapter API | Platform contracts and extension points | A specific adapter implementation |
| Adapters | Authentication, pairing, protocols, capability mapping | Flutter UI and CLI presentation |
| Platform services | Local persistence, secret stores, clocks, network permissions | Brand-specific business decisions |

Core depends on interfaces. Hosts inject the adapter registry and platform services during startup. Storage paths, secret access, and the trusted principal come from the host, not arbitrary model-controlled tool arguments.

## Main operation flow

Start -> open local configuration -> resolve account and device -> validate capabilities and policy -> refresh authentication only if needed -> select a route -> execute -> optionally verify -> save the snapshot and outcome -> release resources.

Discovery and control have deadlines and cancellation tokens. Adapters cannot bypass Core by creating periodic workers. Subscriptions belong to explicit, bounded foreground tasks and must not outlive the host.

## Local persistence

SQLite stores configuration, device references, capability caches, timestamped observations, and bounded redacted execution records. Credentials live in the system secret store. Desktop CLI and MCP may share a store only for the same OS user and explicitly selected profile. Phone installations have independent storage and do not automatically synchronize. There is no central database.

Use short transactions, account-refresh locks, and per-device write coordination. Such locks only coordinate the local profile; they cannot prevent a vendor app or another computer from controlling the device concurrently.

## Proposed technology

Rust is the proposed Domain/Core/Adapter/CLI implementation language. Flutter is the owner-selected app framework, using in-process FFI. MCP is another host adapter. `flutter_rust_bridge` is a possible binding mechanism, but packaging, network permissions, cancellation, and secure storage require project-specific prototypes; the library's existence does not make this project runnable. [R4](references.md)

Flutter is selected; the proposed Core toolchain and binding mechanism still require prototype evidence. Validate Android core loading and cancellation, at least one desktop CLI, and restricted credential access early. Existing Python device libraries can inform protocols and test comparisons; they must not become mandatory remote services for the phone app.

## Decisions

[No daemon](adr/0001-no-daemon.md) | [Embedded core](adr/0002-embedded-core.md) | [Capability first](adr/0003-capability-first.md) | [Uncertain writes](adr/0004-uncertain-writes.md) | [Native authentication gate](adr/0005-native-auth.md)

Project and public identifier naming are defined in [ADR-0006](adr/0006-project-name.md).

[ADR-0008](adr/0008-foreground-authorization-sharing.md) permits explicit foreground local-network authorization transfer. The receiving installation subsequently operates independently; no shared control service or automatic synchronization is introduced.
